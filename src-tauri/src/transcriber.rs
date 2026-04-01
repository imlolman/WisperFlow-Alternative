use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};
use std::sync::atomic::{AtomicU32, Ordering};

pub struct Transcriber {
    ctx: WhisperContext,
    call_count: AtomicU32,
}

unsafe impl Send for Transcriber {}
unsafe impl Sync for Transcriber {}

impl Transcriber {
    pub fn load(model_path: &str) -> anyhow::Result<Self> {
        let params = WhisperContextParameters::default();
        let ctx = WhisperContext::new_with_params(model_path, params)
            .map_err(|e| anyhow::anyhow!("Failed to load whisper model: {:?}", e))?;
        Ok(Self { ctx, call_count: AtomicU32::new(0) })
    }

    /// Reload context to prevent any internal state accumulation
    pub fn reload(&mut self, model_path: &str) -> anyhow::Result<()> {
        let params = WhisperContextParameters::default();
        let ctx = WhisperContext::new_with_params(model_path, params)
            .map_err(|e| anyhow::anyhow!("Failed to reload whisper model: {:?}", e))?;
        self.ctx = ctx;
        self.call_count.store(0, Ordering::SeqCst);
        Ok(())
    }

    pub fn should_reload(&self) -> bool {
        self.call_count.load(Ordering::Relaxed) >= 15
    }

    pub fn transcribe(&self, audio: &[f32]) -> Option<String> {
        self.call_count.fetch_add(1, Ordering::Relaxed);

        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
        params.set_language(Some("en"));
        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);
        // Allow multiple segments for longer audio
        params.set_single_segment(false);
        params.set_no_context(true);
        params.set_suppress_blank(true);
        params.set_suppress_nst(true);
        params.set_no_speech_thold(0.6);
        params.set_temperature(0.0);
        params.set_temperature_inc(0.0);
        params.set_initial_prompt("This is a voice dictation for typing text.");

        let mut state = match self.ctx.create_state() {
            Ok(s) => s,
            Err(e) => {
                eprintln!("[openbolo] create_state failed: {:?}", e);
                return None;
            }
        };
        if let Err(e) = state.full(params, audio) {
            eprintln!("[openbolo] state.full() failed: {:?}", e);
            return None;
        }

        let n = state.full_n_segments();
        if n == 0 {
            return None;
        }

        let mut text = String::new();
        for i in 0..n {
            if let Some(seg) = state.get_segment(i) {
                if seg.no_speech_probability() > 0.6 {
                    continue;
                }
                match seg.to_str_lossy() {
                    Ok(s) => {
                        let s = s.trim();
                        if !is_hallucination(s) {
                            if !text.is_empty() && !s.is_empty() {
                                text.push(' ');
                            }
                            text.push_str(s);
                        }
                    }
                    Err(_) => {}
                }
            }
        }

        // Drop state explicitly before returning to free whisper memory
        drop(state);

        let text = text.trim().to_string();
        if text.is_empty() || text.len() <= 1 {
            None
        } else {
            Some(text)
        }
    }
}

fn is_hallucination(text: &str) -> bool {
    let t = text.to_lowercase();

    if t.contains('♪') {
        return true;
    }

    let exact = [
        "(upbeat music)", "(dramatic music)", "(soft music)", "(music)",
        "(music playing)", "[music]", "[music playing]", "(silence)",
        "(blank audio)", "(no audio)", "(applause)", "(laughing)",
        "(laughter)", "thank you for watching", "thanks for watching",
        "thank you.", "thank you", "thanks.",
        "subscribe", "like and subscribe", "please subscribe",
        "see you in the next", "you",
    ];
    for p in &exact {
        if t == *p {
            return true;
        }
    }

    // Initial prompt echoed back by whisper on silence
    let prompt_hallucinations = [
        "this is a voice dictation for typing text.",
        "this is a voice dictation for typing text",
        "voice dictation for typing text",
    ];
    for p in &prompt_hallucinations {
        if t == *p || t.contains(*p) {
            return true;
        }
    }

    // Any parenthetical/bracketed description
    if (t.starts_with('(') && t.ends_with(')')) || (t.starts_with('[') && t.ends_with(']')) {
        return true;
    }

    // Repeated short patterns (e.g., "... ... ..." or similar)
    if t.len() > 3 && t.chars().filter(|c| !c.is_whitespace() && *c != '.').count() == 0 {
        return true;
    }

    false
}

pub fn model_exists() -> bool {
    crate::config::model_path().exists()
}

pub async fn download_model<F>(on_progress: F) -> anyhow::Result<()>
where
    F: Fn(u64, u64) + Send + 'static,
{
    use futures_util::StreamExt;

    let url = "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.en.bin";
    let dest = crate::config::model_path();

    if let Some(parent) = dest.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    let client = reqwest::Client::new();
    let resp = client.get(url).send().await?;
    let total = resp.content_length().unwrap_or(0);
    let mut stream = resp.bytes_stream();

    let tmp = dest.with_extension("bin.tmp");
    let mut file = tokio::fs::File::create(&tmp).await?;
    let mut downloaded: u64 = 0;

    use tokio::io::AsyncWriteExt;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        file.write_all(&chunk).await?;
        downloaded += chunk.len() as u64;
        on_progress(downloaded, total);
    }
    file.flush().await?;
    drop(file);

    tokio::fs::rename(&tmp, &dest).await?;
    Ok(())
}
