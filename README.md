# bredboard

An educational 2D breadboard simulator in Rust and Bevy, with a standalone electrical simulation core.

**Status: T01 workspace scaffold.** The app displays a startup screen; circuit models, lessons, and file workflows belong to later tasks.

The MVP targets Linux and validates the same application in Chromium and Firefox through WebAssembly. It teaches beginners with three guided experiments: an LED and resistor, capacitor charging/discharging, and a transistor switch. Custom assemblies will be importable through documented JSON.

## Project documents

- [Product and technical plan](docs/PLAN.md)
- [Task index and Codex Goal workflow](docs/TASKS.md)
- [Post-MVP roadmap](docs/roadmap/POST-MVP.md)
- [Contributor guide](CONTRIBUTING.md)
- [Agent instructions](AGENTS.md)

English is the canonical language for documentation, UI, lessons, diagnostics, schemas, and commit messages. Application strings live in `crates/app/src/text.rs`, separate from application logic.

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

The web build uses the version of `wasm-bindgen` pinned by `Cargo.lock`. Install its matching CLI and package the compiled WASM:

```sh
cargo install wasm-bindgen-cli --version 0.2.128 --locked
wasm-bindgen --target web --out-name bredboard_app --out-dir web/pkg target/wasm32-unknown-unknown/debug/bredboard-app.wasm
python3 -m http.server 8000 --directory web
```

Open `http://127.0.0.1:8000/` in Chrome for the T01 browser smoke check. The page uses WebGL2 and must be served over HTTP; opening the HTML file directly will not load the WASM module. Re-run the WASM build and `wasm-bindgen` command after app changes.

## Licensing

The project is open source under the [MIT License](LICENSE), selected in the initial GitHub commit. Release preparation includes dependency/asset license and attribution review.

[Dependency and embedded font origins, licenses, and notices](docs/DEPENDENCIES.md) are recorded for this workspace.

Repository: [kilatev/bredboard](https://github.com/kilatev/bredboard). The local checkout uses colocated Git/Jujutsu with `origin` set to `git@github.com:kilatev/bredboard.git`. Configure contributor identity at repository scope in both Git and Jujutsu; do not change global identity settings for this project.
