# CONTEXT.md — grok-pi (Flybicy fork)

## Goal & constraints
- Fork/continuation of Dwsy/grok-pi-tui (Grok TUI frontend + Pi agent backend).
- Direction (user): keep Grok's TUI design as frontend, open-source Pi as backend; periodically sync both upstreams (xai/grok-build TUI + badlogic pi via Dwsy/grok-pi-tui).
- 最小改动、贴合现有 repo 风格；多会话任务维护本文件。

## Decisions
- 供应商配置改为 WebUI 优先：`grok-pi config` 子命令（cclite configui 风格），编辑 Pi `models.json`，复用 `xai-grok-pager::pi_model_config` 的快照/备份/原子写事务；loopback-only + Host/Origin 校验（防 DNS rebinding），默认端口 31415，`--lan`/`--no-open`/`--port`。
- 推送目标：github.com/Flybicy/GrokPi（注意：该 fork 目前是从 xai 源码 fork 的，历史不同源；推送需要用户删除重 fork Dwsy/grok-pi-tui，或批准强推覆盖）。

## Current state
- 本地工作副本：E:\DSHProject\grok-pi-tui（upstream clone，origin=Dwsy/grok-pi-tui）。
- 已新增：`crates/codegen/xai-grok-pager-bin/src/bin/grok_pi/config_web.rs`（loopback HTTP server + API）、`config_web.html`（单页编辑器）、`grok-pi config` 子命令接入 cli.rs/grok-pi.rs；README/README.zh-CN/CHANGELOG 已改。
- 编译验证：本机刚装 rustup；`cargo check -p xai-grok-pager-bin --bin grok-pi` 运行中（首次需下 1.92 工具链）。
- 未推送：等 fork 问题确认。

## Key artifacts
- WebUI server: crates/codegen/xai-grok-pager-bin/src/bin/grok_pi/config_web.rs
- WebUI page:   crates/codegen/xai-grok-pager-bin/src/bin/grok_pi/config_web.html
- models.json 事务层: crates/codegen/xai-grok-pager/src/pi_model_config.rs

## Open items & next step
1. 编译验证：本机无 MSVC linker，已尝试（VS Build Tools 静默安装被 UAC 挡住）。语法级检查已过；完整 cargo check 待 linker 可用。
2. 确认 fork 处置（重 fork Dwsy/grok-pi-tui 或批准强推 Flybicy/GrokPi main）后提交推送。
3. 后续增强设计已落盘 docs/proposals/context-and-skill-evolution.md（billion-context-pi 压缩 + WikiSkill 三层 skill 演化 + 上游同步节奏）。
