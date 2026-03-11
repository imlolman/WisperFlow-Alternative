"""Whisper model loading and transcription."""

import warnings

import numpy as np
import torch
import whisper

from .config import SAMPLE_RATE


def get_device() -> str:
    if torch.backends.mps.is_available():
        return "mps"
    if torch.cuda.is_available():
        return "cuda"
    return "cpu"


def load_model(device: str):
    with warnings.catch_warnings():
        warnings.simplefilter("ignore")
        return whisper.load_model("base.en", device=device)


def transcribe(model, audio: np.ndarray, device: str) -> str | None:
    """Transcribe audio array. Returns text or None if no speech detected."""
    with warnings.catch_warnings():
        warnings.simplefilter("ignore")
        use_fp16 = device != "cpu"

        if len(audio) / SAMPLE_RATE > 28:
            res = model.transcribe(audio, language="en", fp16=use_fp16)
            text = res["text"].strip()
        else:
            padded = whisper.pad_or_trim(audio)
            mel = whisper.log_mel_spectrogram(
                padded, n_mels=model.dims.n_mels
            ).to(device)
            opts = whisper.DecodingOptions(language="en", fp16=use_fp16)
            res = whisper.decode(model, mel, opts)
            if res.no_speech_prob > 0.6:
                return None
            text = res.text.strip()

    return text if text and len(text) > 1 else None
