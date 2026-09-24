# T01 — Runnable Rust and Bevy workspace

Status: ready_for_fukit

## Dependencies

[T00 — English project documentation and task cards](T00-project-docs.md). Its acceptance criteria must be satisfied before starting this task.

## Outcome and commit boundary

Create core, app, and tools crates with pinned compatible toolchain/dependencies and a minimal Bevy application that opens on Linux and in browsers.

Suggested commit title: `build: add a Linux and web Bevy workspace`.

The deliverable must stand on its own at this milestone. Unfinished successor tasks do not prevent this task from being accepted. Do not include unrelated refactors or publication steps.

## Scope and interfaces

Establish crate boundaries, a small core-to-app interface, Cargo.lock, and documented build/check commands. No circuit models or lesson behavior.

Follow the shared architecture, language, numerical, and persistence contracts in [the plan](../PLAN.md). Repository initialization, remote selection, and publication are not implicit parts of this task.

## Acceptance criteria

- [x] Core can be built and tested without Bevy/windowing dependencies.
- [x] The minimal app launches on Linux and in Chrome through WASM.
- [x] README records exact commands for formatting checks, Clippy, workspace tests, Linux builds, WASM builds, and serving the web artifact.
- [x] Dependency origins/licenses are recorded; user-visible text is separate from application logic.

## Required verification

- Run the newly documented formatting, Clippy, test, Linux-build, and WASM-build commands.
- Launch the application on Linux and in Chrome and record versions and observed result.
- Inspect the core dependency graph for forbidden presentation dependencies.

The owner directed that Chrome alone is sufficient for T01 browser verification. “Baseline checks” means the actual project commands established in T01 and documented in README, not invented commands or an assumed passing suite. Record commands and evidence below. Required browser/UI checks cannot be replaced by successful compilation. A missing or failing required check blocks readiness.

## Codex Goal

```text
/goal Complete T01 in docs/tasks/T01-workspace.md according to docs/PLAN.md
and AGENTS.md. Deliver this outcome: Create core, app, and tools crates with pinned compatible toolchain/dependencies and a minimal Bevy application that opens on Linux and in browsers.
Satisfy every acceptance criterion and run every required check in the card.
Fix task-scoped findings without weakening tests or acceptance criteria.
Do not implement successor tasks. Prepare one coherent change for review;
do not commit or push. Record exact verification evidence in this card.
If a required check is unavailable, report the exact blocker and the input
or environment change needed; do not count it as passed.
```

## Completion and fukit handoff

When all criteria pass, record evidence and set `Status: ready_for_fukit`. Stop without starting the next task. The user may then invoke `fukit` to describe, commit, and push.

An existing jj repository and an unambiguous authorized remote/bookmark are required for that workflow. Do not initialize or guess them. Commit completion is evidenced by jj history; publication is evidenced by the actual push result, not a checkbox pre-written in this change.

## Evidence

Verified on 2026-09-24 with Rust 1.95.0 (`rustc 1.95.0 (59807616e 2026-04-14)`, Cargo 1.95.0). Cargo commands used `mise exec rust@1.95.0 --` because Rust was installed through mise in this environment; the README records the equivalent commands after Rust is on PATH.

- `cargo fmt --all --check` — passed.
- `cargo clippy --workspace --all-targets --locked -- -D warnings` — passed.
- `cargo test --workspace --locked` — passed; all three crates built and their current zero-test harnesses ran successfully.
- `cargo build -p bredboard-app --target x86_64-unknown-linux-gnu --locked` — passed.
- `cargo build -p bredboard-app --target wasm32-unknown-unknown --locked` — passed.
- `wasm-bindgen --target web --out-name bredboard_app --out-dir web/pkg target/wasm32-unknown-unknown/debug/bredboard-app.wasm` using wasm-bindgen 0.2.128 — passed and generated the browser module and WASM artifact.
- `python3 -m http.server 8000 --directory web` — served the page and both generated assets successfully (HTTP 200).
- Linux launch: ran `target/x86_64-unknown-linux-gnu/debug/bredboard-app` on Omarchy 4.0.4, Linux 7.2.5-3-omarchy, Intel Iris Xe using Mesa 26.2.2; the Bevy window opened and visibly rendered “bredboard”, “Workspace ready”, and core version 0.1.0.
- Browser launch: Chrome 154.0.8037.57 opened `http://127.0.0.1:8000/`; the Bevy canvas rendered the same startup screen and core version.
- `cargo tree -p bredboard-core --locked --offline` — output contained only `bredboard-core v0.1.0`, confirming no Bevy or windowing dependencies in the core graph.
- Dependency audit: Cargo metadata reported 510 locked registry packages, all from crates.io and each with a declared license expression. [The inventory and embedded Fira Mono attribution/notice requirements](../DEPENDENCIES.md) record the source and license data. Rust UI strings live in `crates/app/src/text.rs`; web document labels are in `web/index.html`.
- `git diff --check` — passed.

No circuit models or lesson behavior were added.
