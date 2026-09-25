# bredboard

An educational 2D breadboard simulator in Rust and Bevy, with a standalone electrical simulation core.

**Status: MVP complete.** T00–T14 and their accepted dependencies are complete, including the three-circuit Linux app, integrated verification, and the verified Linux release artifact. Work after this boundary is post-MVP and must not reopen MVP scope.

One Bevy app contains the menu, board, controls, and readouts for Linux and WASM. Linux is the MVP interaction target. The same app compiles to WASM, but browser runtime testing is deferred. The MVP has three fixed circuits and no user-facing file workflow or assembly editor. The board remains fixed in the MVP; readable automatic placement is tracked as post-MVP T18.

## Project documents

- [Product and technical plan](docs/PLAN.md)
- [Task index and Codex Goal workflow](docs/TASKS.md)
- [Post-MVP roadmap](docs/roadmap/POST-MVP.md)
- [Contributor guide](CONTRIBUTING.md)
- [Agent instructions](AGENTS.md)

English is the canonical language for documentation, UI, diagnostics, schemas, and commit messages. The window title lives in `crates/app/src/text.rs`; the shared menu and board view live in `crates/app/src/main.rs`.

## Build and verification

Install Rust through [rustup](https://rustup.rs/). `rust-toolchain.toml` pins Rust 1.95.0, rustfmt, Clippy, and `wasm32-unknown-unknown`. The Linux app needs the [Bevy Linux system dependencies](https://bevy.org/learn/quick-start/getting-started/setup/) for its window and renderer. Run these commands from the repository root:

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo build -p bredboard-app --target x86_64-unknown-linux-gnu --locked
cargo build -p bredboard-app --target wasm32-unknown-unknown --locked
```

Run the Linux app with `cargo run -p bredboard-app --locked`. The core can be checked alone with `cargo test -p bredboard-core --locked`; inspect its dependencies with `cargo tree -p bredboard-core --locked`.

Select one of three circuits in the menu. Each bench has Run/Pause, Reset, and its switch or press/release control. Hover over a hole to read its exact ID and any plugged component pin or wire. The shown 5 V source connects to the rail holes. The board remains fixed; component placement and file controls are future work.

The core uses exact 100 microsecond electrical steps. The app advances 1,000 such steps per wall-clock second, so the RC change is easy to watch; solver results never depend on rendering frame rate.

Generate the Project JSON Schema with `cargo run -p bredboard-tools --locked -- schema`; validate a project with `cargo run -p bredboard-tools --locked -- validate path/to/project.json`. See [the project format guide](docs/PROJECT-FORMAT.md) for the board contact model and authoring example.

Run the resistive DC solver headlessly with `cargo run -p bredboard-tools --locked -- solve path/to/project.json`.

Advance a project by an exact number of 100-microsecond steps with `cargo run -p bredboard-tools --locked -- simulate path/to/project.json 1000`.

Save/validate simulation snapshots and replay action logs with the tools described in [the persistence guide](docs/PERSISTENCE.md).

Run the native MVP transcript and diagnostic verification with `cargo run -p bredboard-tools --locked -- verify-native`. It records the target, Rust version, core versions, and fixed property seed/case count; it exits nonzero on any trace or replay mismatch.

The WASM command above is a compile-only compatibility check. The `web/` page is a canvas and startup wrapper for the same Bevy app; it has no separate product controls. Do not use browser interaction as MVP acceptance evidence. Runtime browser verification belongs to a later deployment phase.

## Licensing

The project is open source under the [MIT License](LICENSE), selected in the initial GitHub commit. Release preparation includes dependency/asset license and attribution review.

[Dependency and embedded font origins, licenses, and notices](docs/DEPENDENCIES.md) are recorded for this workspace.

Repository: [kilatev/bredboard](https://github.com/kilatev/bredboard). The local checkout uses colocated Git/Jujutsu with `origin` set to `git@github.com:kilatev/bredboard.git`. Configure contributor identity at repository scope in both Git and Jujutsu; do not change global identity settings for this project.
