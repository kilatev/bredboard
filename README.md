# bredboard

An educational 2D breadboard simulator planned in Rust and Bevy, with a standalone electrical simulation core.

**Status: planning complete; application implementation has not started.** There is no runnable application or Cargo workspace yet. Start with task T01; do not interpret this README as a claim that planned features already exist.

The MVP targets Linux and validates the same application in Chromium and Firefox through WebAssembly. It teaches beginners with three guided experiments: an LED and resistor, capacitor charging/discharging, and a transistor switch. Custom assemblies will be importable through documented JSON.

## Project documents

- [Product and technical plan](docs/PLAN.md)
- [Task index and Codex Goal workflow](docs/TASKS.md)
- [Post-MVP roadmap](docs/roadmap/POST-MVP.md)
- [Contributor guide](CONTRIBUTING.md)
- [Agent instructions](AGENTS.md)

English is the canonical language for documentation, UI, lessons, diagnostics, schemas, and commit messages. User-facing text will be separated from logic for future localization.

## Development and licensing

Build and verification commands will be established by T01 and documented here. Until then, only documentation checks apply.

The project is open source under the [MIT License](LICENSE), selected in the initial GitHub commit. Release preparation includes dependency/asset license and attribution review.

Repository: [kilatev/bredboard](https://github.com/kilatev/bredboard). The local checkout uses colocated Git/Jujutsu with `origin` set to `git@github.com:kilatev/bredboard.git`. Configure contributor identity at repository scope in both Git and Jujutsu; do not change global identity settings for this project.
