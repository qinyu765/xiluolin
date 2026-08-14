# XiLuoLin

[简体中文](README.md) | **English**

XiLuoLin is an open-source AI voice input assistant for office work, writing, and programming. It turns short voice recordings into ready-to-use text and reduces the effort spent typing, editing, and polishing through persona-based rewriting, hotwords, local history, and desktop text delivery.

XiLuoLin is independently initiated and actively maintained. Community participation through Issues, Discussions, and Pull Requests is welcome.

## Stable Release Downloads

`v0.1.0` is the first stable release:

- macOS 13+ on Apple Silicon: an ad-hoc-signed, non-notarized DMG that requires manual approval on first launch.
- Windows 10/11 x64: an unsigned NSIS installer that may trigger Microsoft Defender SmartScreen.
- These packages may still trigger operating-system warnings because macOS is not notarized and Windows is not code-signed.
- Intel Macs, Windows ARM64, Linux, app stores, and in-app updates are out of scope for this release.

Download only from [GitHub Releases](https://github.com/qinyu765/xiluolin/releases) and verify the file against `SHA256SUMS.txt` from the same release. See the [macOS installation guide](docs/macos-build.md), [Windows installation guide](docs/windows-build.md), and [changelog](CHANGELOG.md) before installing.

## Product Direction

XiLuoLin focuses on the complete workflow from speaking an idea to getting text that can be used immediately:

- **Voice capture**: microphone recording, short-audio processing, global shortcuts, and recording status feedback.
- **Speech recognition**: Zhipu GLM-ASR-2512, OpenAI Whisper, and an offline local Whisper model; macOS can explicitly enable hold-Fn recording with short-tap cancellation.
- **Experimental live captions**: disabled by default; an explicitly downloaded bilingual Zipformer mixed-quantization candidate (about 199.3 MB) provides incremental overlay text without replacing final ASR. Preview failures do not affect final recognition, history, or delivery.
- **Persona-based rewriting**: use polished built-in or custom personas, or select Verbatim Dictation to preserve the raw ASR wording while only normalizing whitespace.
- **Hotword dictionary**: enabled hotwords globally bias ASR; Zhipu receives up to 100 native hotwords, OpenAI and local Whisper use soft prompts, and similar technical terms may compete.
- **Desktop delivery**: clipboard and automatic paste output with a recoverable result window when a preferred method is unavailable.
- **Local data**: store history, personas, hotwords, settings, and personal productivity statistics locally.
- **Open extensibility**: maintain clear Provider and business-module boundaries for additional cloud services, local models, and cross-platform integrations.

## Current Status

XiLuoLin is under active development. The core modules are available, while reliability, cross-platform verification, usability, packaging, and release workflows continue to evolve.

Major capabilities already implemented:

- Tauri v2, React, and TypeScript desktop application foundation
- SQLite local data layer and operating-system credential storage
- Built-in personas, custom personas, and default persona selection
- Hotword dictionary, input history, and statistics
- Zhipu GLM-ASR-2512 Provider
- OpenAI Responses API text-rewriting Provider
- Recording, global shortcuts, a recording indicator, and short-audio processing
- An experimental React live-transcript overlay with explicit, checksummed local model downloads; the candidate model remains No-Go for production redistribution until its training-data license chain is auditable
- Clipboard delivery, automatic paste, and error feedback
- Home, persona, hotword, and settings pages
- TypeScript, frontend build, Rust formatting, compilation, and test checks

Current priorities:

- Verify microphone, shortcut, credential-store, and cross-application delivery behavior across operating systems
- Improve the home-page voice entry point and recording-state experience
- Validate the stable installers and continue improving release and compatibility documentation
- Improve Provider configuration, failure recovery, and automated testing
- Continue improving contributor documentation, Issue management, and technical decision records

## Documentation

The detailed product and engineering documents are currently maintained in Chinese:

- [Documentation index](docs/README.md)
- [Product requirements](docs/requirements-analysis.md)
- [Technical design](docs/solution-design.md)
- [Usage and verification guide](docs/usage-guide.md)
- [Troubleshooting guide](docs/troubleshooting.md)
- [ASR quality evaluation and desktop acceptance](docs/asr-quality-evaluation.md)
- [macOS Apple Silicon build and installation](docs/macos-build.md)
- [Windows x64 build and installation](docs/windows-build.md)
- [Changelog](CHANGELOG.md)
- [Project roadmap](docs/roadmap.md)
- [Contribution guide](CONTRIBUTING.md)
- [Security policy](SECURITY.md)
- [Code of Conduct](CODE_OF_CONDUCT.md)

## Technology Stack

- Desktop framework: Tauri v2
- Frontend: React 19, TypeScript, and Vite
- UI: Tailwind CSS, shadcn/ui, and Radix UI
- Local storage: SQLite, Tauri Store, and the operating-system credential store
- Audio: cpal and hound; statically linked Apache-2.0 sherpa-onnx for optional local live captions
- External services: configurable ASR and text-processing Providers

## Requirements

- Node.js 20+
- pnpm 10+
- Rust stable toolchain
- Windows: Microsoft Visual Studio C++ Build Tools and WebView2 Runtime
- macOS 13+: microphone permission; cross-app automatic paste also requires Accessibility permission
- Windows: microphone permission; cross-integrity-level input can be restricted by the system

## Local Development

```bash
git clone https://github.com/qinyu765/xiluolin.git
cd xiluolin
pnpm install --frozen-lockfile
pnpm check
pnpm tauri dev
```

Common commands:

| Command | Purpose |
|---|---|
| `pnpm dev` | Start the frontend development server |
| `pnpm typecheck` | Run TypeScript type checking |
| `pnpm build` | Type-check and build the frontend |
| `pnpm check:rust` | Run Rust formatting, compilation, and tests |
| `pnpm check` | Run the complete frontend and Rust quality checks |
| `pnpm tauri dev` | Start the desktop application in development mode |
| `pnpm release:check` | Verify frontend, Cargo, Tauri, and optional release-tag versions |
| `pnpm eval:asr` | Compute CER, hotword recall, punctuation F1, and latency for a private benchmark |
| `pnpm tauri:build:macos:arm64` | Build the macOS 13+ Apple Silicon app and DMG |
| `pnpm tauri:build:windows:x64` | Build the Windows 10/11 x64 NSIS installer on Windows |

GitHub Actions runs frontend, Windows/macOS Rust, dependency-security, and secret checks for pushes to `main` and Pull Requests targeting `main`; release Pull Requests also build the macOS DMG and Windows NSIS installer. Changes involving recording, global shortcuts, credentials, or text delivery still require manual verification in a desktop environment.

## Configuration and Usage

1. Start the application and open **Settings**.
2. Configure Zhipu GLM-ASR-2512 or another supported ASR service.
3. Configure the OpenAI Responses API or a compatible text-processing service.
4. Optionally download the verified bilingual Zipformer model under **Settings → Model configuration** to enable live overlay captions.
5. Select the microphone, shortcuts, and output method. On macOS, hold-Fn recording can be enabled after Accessibility permission is granted.
6. Select Verbatim Dictation when no text-model rewriting is wanted, or use a polished built-in/custom persona.
7. Add project names, personal names, and technical terms that require more accurate recognition.
8. Place the cursor in a target application and use a global shortcut to start voice input.

Settings are saved automatically: switches, selects, and shortcuts save immediately; text and API key fields save about 600 ms after typing stops and flush on blur. If a save fails, retry it from the Settings status indicator.

See the [usage and verification guide](docs/usage-guide.md) for detailed setup, validation paths, and failure scenarios.

## Privacy and Security

- Audio is sent only to the ASR Provider explicitly configured by the user.
- Raw recognized text is sent only to the text-processing Provider explicitly configured by the user.
- API keys are stored in Windows Credential Manager, macOS Keychain, or another operating-system-native credential store.
- History, personas, hotwords, and statistics are stored in local SQLite by default and are not uploaded to a XiLuoLin server.
- Temporary recordings created by the application are removed after either successful or failed processing. User-selected external audio files are never deleted by this cleanup logic.
- Logs must not contain API keys, complete user text, or complete recording paths.
- Live captions run locally. Preview text is never used as a final-result fallback and is not written to logs.

Before using a third-party Provider, review its privacy policy, data-retention rules, and terms of service. Report security concerns according to [SECURITY.md](SECURITY.md).

## Contributing

Contributions are welcome in many forms:

- Bug reports, reproducible cases, and cross-platform compatibility feedback
- Product proposals, interaction improvements, and accessibility work
- Provider, recording, shortcut, output, and local-storage improvements
- Tests, documentation, translations, and release-process improvements

Before contributing, read [CONTRIBUTING.md](CONTRIBUTING.md) and [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md). For larger features or architectural changes, open an Issue first, then submit a focused Pull Request from a short-lived branch to `main`.

When a change affects project positioning, capabilities, setup, usage, privacy, compatibility, or contribution instructions, update both [README.md](README.md) and [README.en.md](README.en.md) in the same Pull Request.

## Project Governance

- `main` is the stable development baseline, and changes are merged through branches and Pull Requests.
- The roadmap communicates direction and does not guarantee fixed delivery dates.
- Proposals are evaluated according to user value, reliability, privacy risk, maintenance cost, and architectural consistency.
- Historical development records are kept under `docs/dev/`. References to competitions, demos, MVPs, or older workflows describe their original context and do not define the project's current direction.

## License

XiLuoLin is open source under the [MIT License](LICENSE).
