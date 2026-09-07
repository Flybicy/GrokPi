## [0.1.9] - 2026-09-07

### Added
- `grok-pi` now auto-installs a missing Pi host on first run: the official
  pi.dev installer first, then `npm i -g @earendil-works/pi-coding-agent` as
  fallback. Well-known install dirs are added to the process PATH so the
  freshly installed Pi is found without a shell restart. Opt out with
  `PI_GROK_NO_PI_BOOTSTRAP=1` (automatic under `PI_OFFLINE=1`).

- `grok-pi config` launches a loopback WebUI (CC-lite style) for configuring Pi model providers: provider list, API kind/base URL/key, and per-model fields, edited against Pi's `models.json` through the same snapshot/backup/atomic-write transaction as `/pi-models`. `--port` overrides the default port (31415), `--lan` opts into LAN access, `--no-open` skips opening the browser.
- New bundled extension `pi-grok-skill-wiki` (F2 → Agent → **Pi skill wiki**, on by default): a lightweight WikiSkill-style persistent knowledge layer. The pattern catalog is injected into the system prompt each session; the agent records reusable techniques/pitfalls via `skill_wiki_record`, reads them back with `skill_wiki_read` / `skill_wiki_search`, and users can drop quick notes with `/skill-note`. Stored under `~/.grok-pi/skill-wiki` (patterns + index + evolution log + inbox).
- Documented the recommended context-compression path (`docs/usage/context-compression.md`): install `billion-context-pi` for model-driven long-session compression. Verified end-to-end against a local OpenAI-compatible mock on pi 0.85.1: message `<acp>` refs, `acp_status` / `compress` tool execution, protected-zone and min-size guardrails, and coexistence with the bundled grok-pi bridges.

### Fixed

- `mouse.rs` no longer fails to compile on the pinned toolchain: fullscreen opens from the Tasks pane are deferred until after the `&self.tasks.view_button_rects` iteration completes (E0502).

### Install

```bash
# macOS / Linux
curl -fsSL https://github.com/Flybicy/GrokPi/releases/download/v0.1.9/install.sh | GROK_PI_VERSION=v0.1.9 sh
```

```powershell
# Windows
$env:GROK_PI_VERSION='v0.1.9'; irm https://github.com/Flybicy/GrokPi/releases/download/v0.1.9/install.ps1 | iex
```

Release assets: macOS aarch64/x86_64, Linux x86_64/aarch64, Windows x86_64/aarch64.
