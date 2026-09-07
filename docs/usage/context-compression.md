# Context compression with billion-context-pi

grok-pi delegates context management to Pi. For long sessions we recommend the
[`billion-context-pi`](https://github.com/ranxianglei/billion-context-pi)
extension (MIT), which replaces Pi's built-in truncation with model-driven
compression: the model decides *when* and *what* to compress via a `compress`
tool, ranges become labeled, searchable blocks (T1→T3 distillation tiers), and
`acp_status` / `search_context` / `decompress` inspect or restore them.

## Install

```bash
pi install npm:billion-context-pi
```

grok-pi automatically picks it up on the next start (global Pi extension
discovery requires no grok-pi flags).

## Compatibility notes (verified 2026-09-07, pi 0.85.1)

Tested end-to-end against a local OpenAI-compatible mock server:

- The extension loads under grok-pi's extension model; every message gets the
  invisible `<acp>` ref tags and the `acp_status` / `compress` tools register
  and execute correctly through the real tool-call path.
- It coexists with our bundled bridges (`pi-grok-skill-wiki`, todo, etc.) without
  interference.
- Guardrails behave: compression of the protected recent window is rejected
  (`"entirely within the protected zone"`), and sub-threshold ranges are
  rejected with `min 5000 chars` guidance.

Rules that matter:

- **Only one `context`-event extension at a time.** billion-context-pi cancels
  Pi's built-in auto-compaction and takes over. Do not install a second
  compression extension.
- Extension handler ordering in Pi is load-order dependent; billion-context
  documents itself as robust to handlers *before* it. If context behavior looks
  wrong, check installed extensions with `pi list`.
- If you route models through the `billion-context` proxy (`bili <client>`), the
  in-process extension detects it and steps aside automatically.

## Tuning

Context window derives from the active model's `contextWindow` — which the
`grok-pi config` WebUI writes into `models.json`. If compression kicks in too
early/late, verify the window value there first.
