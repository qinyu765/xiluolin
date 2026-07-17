# XiLuoLin

[简体中文](README.md) | **English**

XiLuoLin is an open-source AI voice input assistant for office work, writing, and programming. It turns short voice recordings into ready-to-use text and reduces the effort spent typing, editing, and polishing through persona-based rewriting, hotwords, local history, and desktop text delivery.

XiLuoLin is independently initiated and actively maintained. Community participation through Issues, Discussions, and Pull Requests is welcome.

## Product Direction

XiLuoLin focuses on the complete workflow from speaking an idea to getting text that can be used immediately:

- **Voice capture**: microphone recording, short-audio processing, global shortcuts, and recording status feedback.
- **Speech recognition**: configurable ASR Providers that convert audio into raw text.
- **Persona-based rewriting**: restructure tone and content with built-in personas such as Prompt Engineer, Task Collaborator, Idea Organizer, and Formal Message Assistant, or with custom personas.
- **Hotword dictionary**: manage project names, personal names, and technical terms to reduce correction work caused by recognition errors.
- **Desktop delivery**: clipboard and automatic paste output with fallback behavior when a preferred method is unavailable.
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
- Clipboard delivery, automatic paste, and error feedback
- Home, persona, hotword, and settings pages
- TypeScript, frontend build, Rust formatting, compilation, and test checks

Current priorities:

- Verify microphone, shortcut, credential-store, and cross-application delivery behavior across operating systems
- Improve the home-page voice entry point and recording-state experience
- Establish reliable versioning, installers, releases, and compatibility documentation
- Improve Provider configuration, failure recovery, and automated testing
- Continue improving contributor documentation, Issue management, and technical decision records

## Documentation

The detailed product and engineering documents are currently maintained in Chinese:

- [Documentation index](docs/README.md)
- [Product requirements](docs/requirements-analysis.md)
- [Technical design](docs/solution-design.md)
- [Usage and verification guide](docs/usage-guide.md)
- [Troubleshooting guide](docs/troubleshooting.md)
- [Project roadmap](docs/roadmap.md)
- [Contribution guide](CONTRIBUTING.md)
- [Security policy](SECURITY.md)
- [Code of Conduct](CODE_OF_CONDUCT.md)

## Technology Stack

- Desktop framework: Tauri v2
- Frontend: React 19, TypeScript, and Vite
- UI: Tailwind CSS, shadcn/ui, and Radix UI
- Local storage: SQLite, Tauri Store, and the operating-system credential store
- Audio: cpal and hound
- External services: configurable ASR and text-processing Providers

## Requirements

- Node.js 20+
- pnpm 10+
- Rust stable toolchain
- Windows: Microsoft Visual Studio C++ Build Tools and WebView2 Runtime
- macOS / Windows: microphone permission; automatic text delivery may also require accessibility or input-monitoring permission

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

GitHub Actions runs quality checks for pushes to `main` and Pull Requests targeting `main`. Changes involving recording, global shortcuts, credentials, or text delivery still require manual verification in a desktop environment.

## Configuration and Usage

1. Start the application and open **Settings**.
2. Configure Zhipu GLM-ASR-2512 or another supported ASR service.
3. Configure the OpenAI Responses API or a compatible text-processing service.
4. Select the microphone, shortcuts, and output method.
5. Select a built-in persona or create a custom persona.
6. Add project names, personal names, and technical terms that require more accurate recognition.
7. Place the cursor in a target application and use a global shortcut to start voice input.

See the [usage and verification guide](docs/usage-guide.md) for detailed setup, validation paths, and failure scenarios.

## Privacy and Security

- Audio is sent only to the ASR Provider explicitly configured by the user.
- Raw recognized text is sent only to the text-processing Provider explicitly configured by the user.
- API keys are stored in Windows Credential Manager, macOS Keychain, or another operating-system-native credential store.
- History, personas, hotwords, and statistics are stored in local SQLite by default and are not uploaded to a XiLuoLin server.
- Temporary recordings created by the application are removed after either successful or failed processing. User-selected external audio files are never deleted by this cleanup logic.
- Logs must not contain API keys, complete user text, or complete recording paths.

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
