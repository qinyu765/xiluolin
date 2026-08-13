# macOS Permission Identity Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 为 XiLuoLin 增加不泄露个人证书信息的稳定本机签名入口，并清理当前 Mac 上冲突的应用注册与隐私授权。

**Architecture:** 仓库提供两个明确分离的 macOS 构建入口：通用入口显式使用 ad-hoc 签名，个人体验入口要求调用者通过 `APPLE_SIGNING_IDENTITY` 注入非 ad-hoc 身份。权限请求代码保持不变，本机通过重新安装稳定签名包、精准注销 XiLuoLin 注册和重置两项 TCC 服务恢复一致状态。

**Tech Stack:** Tauri 2、pnpm、POSIX shell、Node.js test runner、macOS codesign/LaunchServices/tccutil、Computer Use

## Global Constraints

- 不使用公司开发者身份。
- 不提交个人证书信息、私钥、邮箱或本机证书哈希。
- 不修改暗色模式。
- 不删除源码、worktree、应用数据、历史、模型或录音。
- 免费 Apple Development 身份仅支持本机开发体验，不承诺 Developer ID 公证或跨 Mac 分发。
- `/Applications/XiLuoLin.app` 是唯一体验入口；构建产物可以保留但不主动注册到 LaunchServices。

---

### Task 1: 稳定签名构建入口

**Files:**
- Create: `scripts/build-macos-signed.sh`
- Create: `scripts/build-macos-signed.test.mjs`
- Modify: `package.json`
- Modify: `src-tauri/tauri.conf.json`

**Interfaces:**
- Consumes: 调用环境中的 `APPLE_SIGNING_IDENTITY`。
- Produces: `pnpm tauri:build:macos:arm64:signed`；缺少身份或身份为 `-` 时以状态码 2 退出，否则构建 arm64 app 与 DMG。

- [ ] **Step 1: 写构建入口的失败测试**

使用 Node.js test runner 启动真实 shell 脚本，覆盖三种可观察行为：未提供身份时退出 2、身份为 `-` 时退出 2、提供身份时将身份和两个最低系统版本变量传给 PATH 中的伪 `pnpm`，并传递 `tauri build --target aarch64-apple-darwin --bundles app,dmg` 参数。

- [ ] **Step 2: 运行测试并确认失败**

Run: `node --test scripts/build-macos-signed.test.mjs`

Expected: FAIL，因为 `scripts/build-macos-signed.sh` 尚不存在。

- [ ] **Step 3: 实现最小 shell 构建入口**

脚本使用 `set -eu`，拒绝空值和 `-`，设置 `MACOSX_DEPLOYMENT_TARGET=13.0`、`CMAKE_OSX_DEPLOYMENT_TARGET=13.0`，最后 `exec pnpm tauri build --target aarch64-apple-darwin --bundles app,dmg`。

- [ ] **Step 4: 连接 package scripts 与 Tauri 配置**

从 `tauri.conf.json` 删除全局 `signingIdentity: "-"`；让 `tauri:build:macos:arm64` 在命令级显式设置 `APPLE_SIGNING_IDENTITY=-`，增加 `tauri:build:macos:arm64:signed` 调用新脚本，并把新 Node 测试加入 `pnpm test`。

- [ ] **Step 5: 运行测试并确认通过**

Run: `node --test scripts/build-macos-signed.test.mjs && pnpm test`

Expected: PASS，三个签名入口测试和现有测试全部通过。

- [ ] **Step 6: 提交构建入口**

```bash
git add scripts/build-macos-signed.sh scripts/build-macos-signed.test.mjs package.json src-tauri/tauri.conf.json
git commit -m "build: 增加稳定 macOS 开发签名入口"
```

### Task 2: 本机签名与权限说明

**Files:**
- Modify: `docs/macos-build.md`

**Interfaces:**
- Consumes: Task 1 的两个构建命令。
- Produces: 不包含个人身份值的本机签名、权限重置和换机限制说明。

- [ ] **Step 1: 更新构建说明**

说明通用 ad-hoc 构建与稳定本机签名构建的区别；给出仅含占位符的 `APPLE_SIGNING_IDENTITY="Apple Development: <name> (<team-id>)" pnpm tauri:build:macos:arm64:signed` 示例；说明证书/私钥必须存在于当前 Mac 钥匙串、系统设置登录账号无关、证书变化或换 Mac 后需要重新授权。

- [ ] **Step 2: 更新验证与限制说明**

增加 `codesign -d -r -` 检查：稳定签名 DR 应包含 Apple 签名锚点、开发证书和 identifier，而不是仅绑定具体 `cdhash`；保留免费账号不可公证和不适合跨 Mac 分发的限制。

- [ ] **Step 3: 检查并提交文档**

Run: `git diff --check`

Expected: 无输出，状态码 0。

```bash
git add docs/macos-build.md
git commit -m "docs: 补充 macOS 本机签名与权限说明"
```

### Task 3: 本机重签、清理与权限验证

**Files:**
- Modify: `/Applications/XiLuoLin.app`（本机安装，不进入 Git）
- Modify: macOS LaunchServices 与 TCC 中 `com.xiluolin.desktop` 的记录（本机状态，不进入 Git）

**Interfaces:**
- Consumes: Task 1 的稳定签名构建入口和当前钥匙串中有效的个人 Apple Development 身份。
- Produces: 唯一安装入口、稳定 DR、重新授权后的麦克风与辅助功能状态。

- [ ] **Step 1: 构建并验证稳定签名产物**

在当前 shell 临时设置所选个人身份并运行 `pnpm tauri:build:macos:arm64:signed`。对生成的 `.app` 执行 `codesign --verify --deep --strict --verbose=2` 和 `codesign -d -r -`，确认 TeamIdentifier 与 DR 来自个人身份且不记录到仓库。

- [ ] **Step 2: 退出应用并安装唯一体验副本**

退出当前运行的 XiLuoLin，把旧 `/Applications/XiLuoLin.app` 移到废纸篓中的带时间戳备份，再用 `ditto` 安装刚构建的 `.app`。不触碰 Application Support、Keychain、历史、模型或录音。

- [ ] **Step 3: 精准清理 XiLuoLin 注册**

推出 `/Volumes/XiLuoLin`；仅对 LaunchServices 中 bundle identifier 为 `com.xiluolin.desktop` 且路径不是 `/Applications/XiLuoLin.app` 的条目执行 `lsregister -u`。不运行全局 `lsregister -kill`，不删除任何 worktree 或构建产物。

- [ ] **Step 4: 重置两项授权**

运行 `tccutil reset Microphone com.xiluolin.desktop` 与 `tccutil reset Accessibility com.xiluolin.desktop`，然后从 `/Applications/XiLuoLin.app` 启动应用。

- [ ] **Step 5: 重新授权并检查就绪状态**

在设置页请求麦克风，接受与用途描述一致的系统提示；请求辅助功能并在系统设置中启用唯一的 XiLuoLin 条目。返回应用点击“重新检查”，确认麦克风和自动粘贴均显示已就绪。

- [ ] **Step 6: 执行功能烟测**

使用一次短录音确认录音状态可启动和停止；在空白测试文本框中验证自动粘贴，确认失败时仍保留剪贴板降级。测试内容不包含敏感信息。

### Task 4: 全量验证与 PR 更新

**Files:**
- Verify only: repository and Draft PR #36

**Interfaces:**
- Consumes: Tasks 1–3 的提交和本机验证结果。
- Produces: 通过验证并推送到 `feat/voice-platform-architecture` 的变更。

- [ ] **Step 1: 执行仓库验证**

Run: `pnpm check`

Expected: 前端格式、lint、测试、构建、bindings 检查、Rust fmt/check/test 全部通过。

- [ ] **Step 2: 执行差异与签名验证**

Run: `git diff --check && git status --short && codesign --verify --deep --strict --verbose=2 /Applications/XiLuoLin.app`

Expected: 无未提交改动，签名验证成功。

- [ ] **Step 3: 推送并核对 Draft PR**

Run: `git push origin feat/voice-platform-architecture`

Expected: Draft PR #36 自动包含新提交，base=`main`、head=`feat/voice-platform-architecture`，不请求 reviewer。
