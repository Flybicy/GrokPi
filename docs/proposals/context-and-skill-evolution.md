# Enhancement Roadmap: Context Compression & WikiSkill-style Skill Evolution

Status: proposal (not implemented). Captures the design references and the
intended integration shape for grok-pi (Grok Pager frontend + Pi agent backend).

## 1. Prompt-driven context compression (reference: billion-context-pi)

Reference: https://github.com/ranxianglei/billion-context-pi

billion-context-pi is a Pi extension (in-process) that replaces Pi's built-in
auto-compaction with model-driven compression:

- Intercepts Pi's `context` event (fires before every LLM call) with an
  8-stage pipeline: assign refs → sync blocks → prune → filter → hide calls →
  recommend → nudge → emergency truncate.
- Every message gets an invisible `<acp>` ref (`m00001`, ...) that the model
  uses to name compression ranges.
- Exposes tools to the model: `compress`, `decompress`, `search_context`,
  `acp_status`, plus `acp_delegate` / `acp_delegate_wait` / `acp_delegate_cancel`
  for clean-context subagent delegation.
- Compressed ranges become labeled blocks (T1 → T2 → T3 distillation tiers),
  searchable without re-expansion; protected tools, user messages, and the
  recent working set are never compressed.
- Claimed effect: a single session sustains ~1e10–6e10 cumulative tokens;
  live context stays bounded (~150–200K).

Integration notes for grok-pi:

- It is a stock Pi extension (`pi install npm:billion-context-pi`); grok-pi's
  extension bridge (`PI_GROK_REMOTE_TUI`) already loads such extensions. The
  main work items are compatibility checks with grok-pi's bundled bridges and
  the native compactionsurface (`/compact`), plus deciding whether to vendor
  or recommend it.
- Only one context-compression extension may be active at a time (Pi has no
  handler ordering/priority for the `context` event).
- grok-pi native surfaces to reconcile with: `/compact`, session tree views,
  cost/token display. Decide whether compressed-block metadata is surfaced in
  the Pager (e.g. fold markers) or stays Pi-internal.

## 2. WikiSkill-style skill evolution (reference: Google Research paper)

Reference: "WikiSkill: Compiling Agent Experience into Persistent Knowledge
for Skill Evolution" (Google Research / Virginia Tech, 2026; local copy
`E:\Deskbk\tasks\test1\WikiSkill Compiling Agent Experience into Persistent
Knowledge for Skill Evolution.pdf`).

Core idea: co-evolve skills with a persistent knowledge base, three layers:

- `raw/` — immutable execution traces (write-once).
- `wiki/` — persistent, compounding knowledge: pattern pages under
  `wiki/patterns/`, `index.md` catalog, `logs.md` evolution log,
  `skill-impact.md` tracker (records which proposals were accepted/rejected,
  so rejected interventions are never re-proposed).
- `skills/` — the active skill set (`SKILL.md` + `PURPOSE.md` mapping each
  skill back to the wiki patterns that motivated it).

Evolutionary loop per iteration:

1. Inference Agent runs tasks with active skills (wiki access restricted
   during rollouts — ablation shows wiki leakage hurts skill development).
2. Wiki Maintainer does root-cause analysis on failing traces, extracts
   successful strategies, updates pattern pages + logs (patch-based edits).
3. Skill Proposer (ReAct-style, `read_file` tool) reads the wiki index,
   skill-impact tracker, and sampled traces; produces exactly one atomic
   proposal (new skill or patch-based edit to one existing skill).
4. Gating & rollback: the candidate skill set is evaluated on a validation
   split; only improvements are kept, otherwise rolled back (recorded in
   `skill-impact.md`).

Key findings relevant to grok-pi: persistent wiki accumulation is critical
(ablation), evolved skills transfer across model families, and skill updates
should be conditional/reversible rather than append-only.

Integration sketch for grok-pi (Pi's skills ecosystem already exists under
`~/.grok-pi` / `.grok-pi` resources):

- Add an opt-in "skill evolution" mode driven by grok-pi's existing skill
  discovery: a maintainer/proposer pair run as Pi subagents (leveraging
  Subagents V2 teams), writing into e.g. `~/.grok-pi/skill-evo/{raw,wiki,skills}`.
- Session exports (`/export` / recap) can seed `raw/` traces; a scheduled or
  `/skill-evolve` command runs one evolution iteration.
- Gating needs an evaluation harness — reuse project task definition or a
  small replay set. This is the largest open design question.

## 3. Upstream sync cadence

- Grok TUI upstream: xAI monorepo syncs (see `SOURCE_REV`).
- Pi upstream: `@earendil-works/pi-coding-agent` releases (grok-pi pins a
  minimum Pi version via `pi_version.rs`).
- Periodic sync = track Dwsy/grok-pi-tui (which tracks both) and rebase this
  fork's local feature commits; the drip-feed interval should follow upstream
  release tags rather than daily churn.
