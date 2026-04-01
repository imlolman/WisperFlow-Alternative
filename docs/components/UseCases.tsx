const cases = [
  {
    title: "Email & docs",
    body: "Draft messages, Google Docs, and Notion without touching the keyboard, ideal for long-form thoughts.",
  },
  {
    title: "Developers",
    body: "Speak commit messages, comments, and README snippets; paste into the terminal or IDE when you’re done.",
  },
  {
    title: "Accessibility",
    body: "Reduce typing load for RSI, carpal tunnel, or motor difficulty: local inference, no cloud dependency.",
  },
  {
    title: "Privacy-first work",
    body: "Legal, health, or finance contexts where sending audio to a third party is a non-starter.",
  },
  {
    title: "Meeting notes",
    body: "Capture rough transcripts in your own editor; everything stays on disk you control.",
  },
  {
    title: "Creative writing",
    body: "Brainstorm dialogue and prose in a natural flow, then edit the text that lands in your doc.",
  },
  {
    title: "Support & sales",
    body: "Dictate replies in Zendesk, Intercom, or email, faster than typing repetitive explanations.",
  },
  {
    title: "Students & researchers",
    body: "Outline essays and lab notes hands-free; pair with your favorite citation manager or editor.",
  },
  {
    title: "Slack & Discord",
    body: "Rapid-fire channel updates from the menubar without context-switching to a separate dictation app.",
  },
  {
    title: "Multilingual input",
    body: "Lean on Whisper’s language models locally, great when you mix languages or need a second pass in text.",
  },
  {
    title: "Social & content",
    body: "Turn spoken ideas into posts for LinkedIn, X, or newsletters, then tighten wording before publishing.",
  },
  {
    title: "Coding with voice",
    body: "Dictate variable names, docstrings, and SQL fragments; you stay in flow while the cursor stays put.",
  },
];

export function UseCases() {
  return (
    <section className="mx-auto max-w-6xl px-6 py-20 md:py-28" id="use-cases">
      <div className="mx-auto max-w-2xl text-center">
        <h2 className="text-3xl font-semibold tracking-tight text-white md:text-4xl">
          Built for real workflows
        </h2>
        <p className="mt-4 text-lg text-zinc-400">
          Swap in screenshots per use case when you’re ready. These are the stories OpenBolo shines in.
        </p>
      </div>
      <ul className="mt-14 grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
        {cases.map((item) => (
          <li
            key={item.title}
            className="group rounded-2xl border border-white/[0.08] bg-white/[0.02] p-6 transition-colors hover:border-sky-500/30 hover:bg-white/[0.04]"
          >
            <div className="mb-4 aspect-video overflow-hidden rounded-xl border border-dashed border-white/10 bg-black/30 transition-colors group-hover:border-sky-500/25">
              <div className="flex h-full items-center justify-center px-3 text-center text-xs font-medium text-zinc-600">
                Screenshot placeholder
              </div>
            </div>
            <h3 className="text-base font-semibold text-white">{item.title}</h3>
            <p className="mt-2 text-sm leading-relaxed text-zinc-400">{item.body}</p>
          </li>
        ))}
      </ul>
    </section>
  );
}
