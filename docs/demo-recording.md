# README 使用预览录制与交付

本文档约定 README 的 macOS 使用预览如何录制、脱敏、压缩并上传到 GitHub Release。它只展示当前已经稳定的核心闭环，不把实验性实时字幕或未验证的长音频能力录进主视频。

## 目标视频

- 平台：macOS 13+ Apple Silicon。
- 内容：设置 Provider 和快捷键 → 在目标输入框中按快捷键录一段短语音 → 等待识别、整理并自动粘贴。
- 时长：建议 20–45 秒；实际语音保持在 5–15 秒，避免把 28 秒自动停止拍成“功能演示”。
- 画面：只保留应用窗口、录音悬浮窗和一个无敏感信息的目标输入框；不要展示 API Key、私人文本、完整文件路径或其他通知。
- 音频：使用 H.264 MP4；尽量控制在 GitHub 免费计划的 10 MB 视频附件限制内，并保留清晰的悬浮窗状态。
- 命名：`xiluolin-usage-macos-v0.2.0.mp4`；可选海报为 `xiluolin-usage-macos-v0.2.0.png`。

GitHub 的视频附件说明见[官方文档](https://docs.github.com/en/get-started/writing-on-github/working-with-advanced-formatting/attaching-files)。README 只使用 Release 资产 URL，避免依赖个人网盘或第三方播放器。

## 推荐脚本

1. 打开 XiLuoLin 设置页，展示已选择的 ASR、文本 Provider、麦克风和快捷键；API Key 输入框必须隐藏或避开镜头。
2. 切换到一个空白的 TextEdit 文档或其他无敏感信息的输入框，将光标放好。
3. 按住配置的快捷键，口述一段 5–15 秒的示例：

   > 帮我把今天关于快捷键可靠性和本地隐私保护的讨论整理成三项开发任务。

4. 松开快捷键，短暂展示悬浮窗依次进入“识别、整理、输入、完成”；不要等待或强调实验性实时预览。
5. 展示整理后的文本已经进入目标输入框，最后停留在完成状态或历史记录页面即可。

为了避免视频绑定到某个云服务，优先使用本地 Whisper +“原文听写”录制稳定核心闭环；如果要展示人格整理，则使用已验证的 Provider，并确保画面中不出现凭据或私人数据。两种录法都必须明确：这是短语音输入，不是会议或长音频转写。

## 录制与质检

建议使用 QuickTime Player 的“新建屏幕录制”或 macOS 自带录屏工具，录完后导出 H.264 MP4。交付前检查：

- 视频可以从头播放，画面没有黑屏、系统通知或悬浮窗遮挡关键状态。
- 没有 API Key、访问令牌、私人文本、私人应用名称或完整本地路径。
- 录音实际没有超过 28 秒；视频没有暗示支持会议或长音频。
- 视频展示的是自动粘贴成功路径；失败兜底只在使用指南中说明为剪贴板手动粘贴。
- 文件大小、编码和海报都适合 GitHub Release；不把临时录屏文件提交到 Git。

## 上传到 v0.2.0 Release

确认本机 GitHub CLI 已登录并拥有该仓库 Release 的上传权限后，在视频所在目录执行：

```bash
gh release upload v0.2.0 \
  xiluolin-usage-macos-v0.2.0.mp4 \
  xiluolin-usage-macos-v0.2.0.png \
  --clobber
```

如果没有海报，可以只上传 MP4。上传后检查资产名称和下载地址：

```bash
gh release view v0.2.0 --json assets,url
```

上传完成后，把 `README.md` 和 `README.en.md` 顶部演示 GIF 之后的 HTML 注释解除注释，并保留普通下载链接作为 GitHub 不渲染 `<video>` 时的兜底。若视频尚未上传，不要解除注释，避免 README 出现失效播放器。

## 使用预览 GIF

README 顶部内嵌两个 GIF 资产，分别展示通用模式和翻译人格的输入效果。GIF 不提交到 Git，与视频一样以 Release 资产 URL 引用：

- 命名：`xiluolin-usage-macos-v0.2.0-01.gif`（通用模式）、`xiluolin-usage-macos-v0.2.0-02.gif`（翻译人格）。
- 上传：`gh release upload v0.2.0 <file>...`；替换时用同名资产覆盖（`--clobber`），README 无需改动。

## README 片段

上传资产后使用以下片段，中文和英文 README 只替换文案，不改变文件名和 Release URL：

```html
<video controls preload="metadata" poster="https://github.com/qinyu765/xiluolin/releases/download/v0.2.0/xiluolin-usage-macos-v0.2.0.png" width="960">
  <source src="https://github.com/qinyu765/xiluolin/releases/download/v0.2.0/xiluolin-usage-macos-v0.2.0.mp4" type="video/mp4">
  <a href="https://github.com/qinyu765/xiluolin/releases/download/v0.2.0/xiluolin-usage-macos-v0.2.0.mp4">打开使用预览视频</a>
</video>
```
