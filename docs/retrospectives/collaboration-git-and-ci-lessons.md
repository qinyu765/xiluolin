# 并发协作、Git、PR 与 CI 经验复盘

## 1. 背景

本轮开发不仅涉及代码复杂度，还出现了多 Agent/多任务同时操作同一仓库的问题。

如果不记录这部分，后续只看最终代码会误以为过程是线性的：

```text
修改 → 测试 → 提交
```

真实过程包含：

- 工作流规则在开发中途变化；
- 同一目录被另一个 Codex 任务写入；
- HEAD 和分支在两次命令之间改变；
- 未提交改动与新任务交叉；
- 使用 worktree 隔离；
- 堆叠 PR 不触发 CI；
- rebase 到新 main 时出现文档冲突；
- 最终代码进入 main，但部分 PR 仍显示 Open。

这些问题对任何使用 AI Agent 并行开发的仓库都具有参考价值。

## 2. 第一次并发异常：意外出现 T030 改动

### 2.1 现象

准备从 `main` 切换到 `dev` 时，Git 拒绝：

```text
local changes would be overwritten by checkout
```

但当前任务此前没有修改这些文件。

`git status` 显示：

- `credentials.rs`；
- `recording_file_security.rs`；
- `data.rs`；
- `pipeline.rs`；
- T030 文档；
- Cargo 依赖；
- 多个敏感日志修改。

### 2.2 更异常的信号

两次检查之间：

- HEAD 从 `7546fcc` 变成 `54d460e`；
- `origin/main` 同步更新；
- 新测试文件出现；
- README 和方案文档 mtime 继续变化。

说明另一个 Agent/进程正在同一工作树中提交 T029 并实施 T030。

### 2.3 错误处理方式

如果直接执行：

```bash
git reset --hard
git stash
git switch
git add -A
```

可能导致：

- 覆盖另一个任务；
- 把两个任务提交到同一个 commit；
- stash 对方未完成内容；
- 丢失未跟踪文件；
- 产生难以解释的历史。

### 2.4 实际处理

1. 停止写入；
2. 只读检查 diff、HEAD、暂存区、mtime；
3. 轮询数秒确认是否仍在变化；
4. 识别 T029 已提交、T030 在途；
5. 接续 T030，而不是从头重复实现；
6. 保留对方已有代码，重点做审查、边界补强和文档收口。

## 3. 第二次并发异常：工作区被切换到另一分支

### 3.1 现象

实施 T031 时再次检查状态，发现：

```text
## chore/open-source-positioning
```

同时 50 多个文档、License、README、Agent 规则和社区文件发生变化。

这说明另一个任务把同一工作区切换到了开源治理分支。

### 3.2 风险

T031 的代码改动已经混在该工作区：

- CaptureSession 代码；
- open-source 文档；
- tracker；
- indicator rename；
- README。

继续 `git add -A` 会把两个逻辑完全不同的任务提交在一起。

## 4. 补丁备份

停止写入后，先备份 T031：

```text
/tmp/xiluolin-t031-backup/
  ├─ t031.patch
  ├─ t031-code.patch
  ├─ capture_session.rs
  ├─ focus_capture.rs
  └─ task document
```

关键原则：

- 只提取明确属于当前任务的文件；
- 不对未知改动执行 restore/reset；
- 新文件单独复制，因为普通 `git diff` 不包含未跟踪文件；
- staged rename 需要使用 `git diff HEAD`，不能只看工作树 diff。

## 5. 使用 git worktree 隔离

创建独立工作树：

```bash
git worktree add -b feat/capture-session-delivery ../xiluolin-t031 main
```

然后：

- 只应用 T031 source patch；
- 复制未跟踪的新模块；
- 在独立目录运行 build/test；
- 原工作区继续留给开源治理任务。

### 5.1 为什么 worktree 比 stash 更合适

stash 的问题：

- stash 以整个工作区为中心；
- 很难区分两个 Agent 的文件归属；
- 未跟踪文件需要额外参数；
- 恢复时容易冲突；
- 仍然只能有一个当前分支。

worktree 的优势：

- 每个任务独立分支和工作目录；
- 共享 Git object database；
- 构建、状态和提交互不干扰；
- 适合 Agent 并行。

### 5.2 清理重复副本

确认 T031 已安全迁移后，再精确 restore 原工作区中的 T031 文件，并删除未跟踪副本。

不能直接 `git clean -fd`，因为可能删除对方任务的新文件。

## 6. 工作流规则中途变化

仓库最初历史规则是：

```text
dev 日常开发 → PR 到 main
```

随后变成：

```text
main 直接开发 → 提交前等待用户批准
```

开源治理后又统一为：

```text
main 稳定基线
短生命周期分支
PR 到 main
验证通过后自动提交和推送
用户明确要求 dry-run 时才停止
```

### 经验

Agent 每次开始 Git 操作前都应该重新读取当前 `AGENTS.md`，不能只依赖对话早期读到的规则。

规则文件也应该只有一个事实源。本项目最终使用：

```text
CLAUDE.md -> AGENTS.md
.claude -> .agents
```

避免两套规则长期漂移。

## 7. 删除“每次提交都等待批准”

用户明确要求后，以下约束被删除：

- 提交前停下等待批准；
- 推送前停下等待批准；
- 创建 PR 前停下等待批准。

新的规则：

- 验证后先汇报范围和结果；
- 直接提交、推送、创建或更新 PR；
- 用户明确说 review-only、dry-run 或不要发布时才停止；
- 工作区存在归属不明的改动时仍必须停止。

“无需批准”不等于“无需安全检查”。

## 8. 堆叠 PR

功能按依赖拆分：

```text
#23 CaptureSession
  ↓
#24 Readiness
  ↓
#25 Capture retention
  ↓
#26 Reprocessing
  ↓
#27 Local ASR
```

优点：

- 每个 PR 主题明确；
- 后续任务可以在前一任务未合并时继续；
- review diff 较小；
- 单个提交可独立理解。

缺点：

- base branch 不是 main；
- main 变化后需要逐层 rebase；
- GitHub UI 和 CI 行为更复杂；
- 下层 PR 合并前无法直接合并上层。

## 9. 堆叠 PR 没有触发 CI

### 9.1 现象

PR #24–#27 以功能分支为 base 时，GitHub 没有生成可见 CI check。

尝试：

- Draft → Ready；
- Close → Reopen；

仍没有运行。

### 9.2 临时验证方案

把最终 PR #27 的 base 临时改为 `main`，触发包含全部 T031–T035 的累计 Windows CI。

验证通过后再恢复 base 为 #26 分支。

优点：

- 最终累计代码在 Windows 验证；
- 不需要给每个 PR增加空提交。

缺点：

- PR diff 临时变大；
- GitHub 状态可能让 reviewer 困惑；
- 不能替代每层独立 CI。

### 9.3 更好的长期方案

- workflow 增加 `workflow_dispatch`；
- 提供可手动传入 ref 的 CI；
- 使用 merge queue；
- 每个功能 PR 统一以 main 为 base，但通过依赖说明管理；
- 或采用专门支持 stacked PR 的工具。

## 10. main 更新后的堆叠 rebase

开源治理 PR 合入 main 后，T031 分支被同步 main，后续分支仍基于旧 T031 commit。

需要逐层 rebase：

```text
T032 onto new T031
T033 onto new T032
T034 onto new T033
T035 onto new T034
```

### 10.1 备份

rebase 前创建本地 backup 分支：

```text
backup/input-readiness-pre-sync
backup/capture-retention-pre-sync
backup/capture-reprocessing-pre-sync
backup/local-asr-pre-sync
```

### 10.2 冲突模式

主要冲突在：

- README；
- solution-design；
- task tracker。

原因是开源治理重新组织文档，而功能提交也更新相同段落。

### 10.3 解决原则

保留新的文档结构，同时插入功能事实：

- 使用“持续开发”而不是旧“MVP 开发拆分”；
- 保留双语入口、贡献说明和 License；
- 将 Readiness、Retention、Reprocessing、Local ASR 放入新结构；
- 不简单选择 ours/theirs 整文件覆盖。

### 10.4 推送

rebase 改写历史后使用：

```bash
git push --force-with-lease
```

不能使用裸 `--force`，因为它会覆盖远端其他新提交。

## 11. PR 状态与 main 事实不一致

最终观察到：

- #22、#23 显示 Merged；
- #24–#27 仍显示 Open；
- 但 `main` 的最终同步提交已经包含 T032–T035 代码。

可能原因：

- 代码通过 merge/sync commit 直接进入 main；
- GitHub 无法根据非相同 commit hash 自动标记堆叠 PR merged；
- rebase/merge 改写了提交身份。

### 风险

- reviewer 误以为功能尚未上线；
- 后续可能重复合并；
- PR diff 与 main 接近或为空；
- 任务状态难以维护。

### 建议

1. 对每个 Open PR 执行：

```bash
git diff origin/main...origin/feature
```

2. 如果功能已经完整进入 main：

- 在 PR 留言对应 main commit；
- 关闭 PR；
- 删除远端分支。

3. 不要仅根据 PR 状态判断功能是否已落地，应检查 main tree。

## 12. CI 时间和缓存

加入 whisper.cpp 后 Windows CI 接近 10 分钟。

原因：

- native C/C++ build；
- cargo check 编译一次；
- cargo test 可能再次构建测试产物；
- 未使用 Rust target cache。

建议：

- `Swatinem/rust-cache` 或等价缓存；
- 缓存 whisper-rs build 输出；
- 检查 check/test 是否可以减少重复构建；
- 前端和 Rust job 分开；
- 设置 concurrency 取消旧 run。

## 13. Agent 并发开发规则建议

### 开始任务前

- `git status --short --branch`；
- `git log -3`；
- 确认当前 worktree；
- 确认是否存在其他 Codex/IDE 操作；
- 读取最新 AGENTS.md。

### 发现未知改动时

禁止：

- reset；
- clean；
- stash 全部；
- add -A；
- checkout 覆盖。

应该：

1. 查看 diff；
2. 查看 staged diff；
3. 查看 mtime；
4. 轮询是否仍变化；
5. 备份当前任务；
6. 使用 worktree 隔离；
7. 只清理明确属于自己的副本。

### 提交前

- 明确文件归属；
- 显式 `git add path...`；
- 避免在混合工作区 `git add -A`；
- 检查 commit stat；
- 检查没有模型、录音、Key、target、dist。

## 14. 本轮可复用经验

1. **未知改动先调查，不要假定是用户或自己产生的。**
2. **HEAD 在两次命令间变化，是强烈的并发信号。**
3. **mtime 轮询可以判断另一个进程是否仍在写入。**
4. **worktree 是多 Agent 并行的首选隔离方式。**
5. **补丁备份要考虑 staged rename 和 untracked 文件。**
6. **rebase 前创建 backup ref。**
7. **强推只用 `--force-with-lease`。**
8. **最终必须对账 main、PR、tracker 和文档状态。**
9. **CI 成功是提交级证据，不是所有桌面交互的证据。**
10. **免批准发布仍需范围检查和工作区所有权判断。**

## 15. 相关命令

```bash
# 查看工作树
 git status --short --branch
 git worktree list

# 同时看暂存与未暂存
 git diff --stat
 git diff --cached --stat
 git diff HEAD --stat

# 创建隔离 worktree
 git worktree add -b feat/example ../repo-example main

# 安全强推 rebase 分支
 git push --force-with-lease origin feat/example

# 检查远端 main
 git ls-remote origin refs/heads/main

# 检查 PR
 gh pr view <number> --json state,mergedAt,baseRefName,headRefName,statusCheckRollup
```
