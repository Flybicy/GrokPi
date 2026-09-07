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

## 本机构建环境（Windows, 可复用）
- 无 MSVC。用 rustup GNU toolchain：`1.94.0-x86_64-pc-windows-gnu`（rust-toolchain.toml 锁 1.94.0）；已另装 nightly-msvc 仅供语法检查。
- MinGW 工具链（免管理员, msys2 包组）: `C:\mingw-rust\msys\mingw64\bin`（gcc 16.2 + binutils 2.47 + crt/headers/winpthreads/zlib/zstd/gettext/libiconv）。镜像源: mirrors.tuna.tsinghua.edu.cn/msys2（curl 需加 `--ssl-no-revoke`）。
- protoc: `C:\protoc\bin` (33.0)。
- 检查命令: `set PATH=%USERPROFILE%\.cargo\bin;C:\mingw-rust\msys\mingw64\bin;C:\protoc\bin;%PATH% && cargo +1.94.0-x86_64-pc-windows-gnu check --target x86_64-pc-windows-gnu -p xai-grok-pager-bin --bin grok-pi`
- rustup 自更新在此网络下报错无害；rust-mingw 自带 dlltool 有 CreateProcess bug ，绝对不要用 self-contained 目录里的，有 msys binutils 就够。
- 用 stable 1.98 会在上游 mouse.rs 报 E0502（borrowck 更严）；以锁定的 1.94 为准。

## Current state (2026-09-07, done)
- Flybicy/GrokPi 已重建为 Dwsy fork；已推送 main：5e36e9e0 (webui) + 893d7be2 (mouse.rs 借用修复)。
- cargo check（1.94.0-gnu）+ 完整 build 通过；`grok-pi config --no-open` 实测：GET 页面/配置、PUT 保存、备份生成、Host 校验 403 全部正常。
- 修复上游 HEAD 编译错误：mouse.rs 两处 E0502，改为先收集后打开（deferred_fullscreen）。可向上游提 PR。
- 已建周循环 heartbeat 自动化（id: grokpi）检查 Dwsy 上游新提交并评估与本 fork 冲突。

## Open items & next step
1. 可选：把 mouse.rs 的 E0502 修复 PR 给上游 Dwsy/grok-pi-tui。
2. Skill 增强已落地（轻量版 WikiSkill）：内置扩展 extensions/pi-grok-skill-wiki（F2 pi_skill_wiki 默认开）= patterns/ wiki 页 + index/log/inbox，before_agent_start 注入目录，skill_wiki_record/read/search 工具，/skill-note /skill-wiki 命令。已用 pi-coding-agent 0.84.3 类型 typecheck + mock harness 行为测试（target/skill-wiki-test，未提交）。未做：真实 LLM 端到端跑通（需 pi + 供应商 key）。
3. 上下文压缩（billion-context-pi 路线）未开工：用户定为 skill 完成后再做；可能接单点 = 文档/WebUI 引导 `pi install npm:billion-context-pi` + 兼容性踩坑记录。


## 2026-09-07 收尾
- pi bootstrap 自愈落地并实测：删净 pi 后启动 grok-pi -> 自动走 pi.dev 安装器(失败自动回退 npm) -> pi 0.85.1 装好 -> TUI 正常进入。装完后 augment_path_with_pi_dirs 会把常见安装目录补进进程 PATH。
- commit 14fd9f2a 已推送 main。修复了我那边一个破折号编码乱码。
- 用户机器上 pi 已由 bootstrap 重新装回(npm 全局)，无需手动处理。
