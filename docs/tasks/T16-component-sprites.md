# T16 — 8-bit component sprites

Status: ready_for_fukit

## Dependencies

[T15 — Breadboard-scale capacitor and transistor ranges](T15-capacitor-npn-ranges.md). Its acceptance criteria must be satisfied before starting this task.

## Outcome and commit boundary

Replace the flat rectangle bodies of resistors, LEDs, and momentary buttons with recognizable 8-bit pixel-art sprites, following the approved design in [the sprite design reference](../design/sprites/README.md). Provide an extendable sprite layer so further component kinds can be added one module at a time.

Suggested commit title: `feat: draw resistors, LEDs, and buttons as 8-bit sprites`.

The deliverable must stand on its own at this milestone. Do not include unrelated refactors or publication steps.

## Scope and interfaces

- Sprites are generated in code in `crates/app/src/sprites/`; no image asset files. Each kind implements a `PartArt` trait and is registered in one place; kinds without art keep the plain fallback drawing.
- Bodies are drawn unrotated and placed by quarter turns along two axis pins. Leads are drawn per placement from each pin's hole to the body.
- The board uses a square 16-unit hole pitch (8 art pixels at 2 units each), so sprites stay pixel-exact. Hole IDs, contact groups, and connectivity are unchanged.
- LED brightness remains a presentation mapping of the calculated current (off, dim, lit). Button appearance follows the core control state. Stale readings show the LED off.
- Capacitor, NPN transistor, changeover switch, and voltage source sprites are out of scope.

Follow the shared architecture, language, numerical, and persistence contracts in [the plan](../PLAN.md). The core is not changed.

## Acceptance criteria

- [x] Resistor, LED, and button bodies match the golden references in `docs/design/sprites/` pixel for pixel.
- [x] Resistor bands are computed from `resistance` with the standard four-band code; a fixed-seed property test decodes bands back to within two-digit rounding across 1 Ω to 10 MΩ.
- [x] Leads end in the pin holes for arbitrary hole pairs and orientations (fixed-seed property test), and all embedded fixtures compose.
- [x] LED sprite state follows calculated current monotonically; the embedded LED bench shows the lit sprite when pressed and running, and the off sprite when readings are stale. The button sprite follows its control state.
- [x] Adding a new component sprite requires one new module plus one registry arm; the steps are documented in the design reference.
- [x] Existing app behavior tests (menu, controls, unique holes, stale warning, frame pacing) still pass.

## Required verification

- Run the baseline checks documented in `README.md`.
- Run the golden and property sprite tests.
- Check the LED and transistor benches in the Linux executable with the LED off and lit and the button released and pressed. Do not test browser interaction.

## Codex Goal

```text
/goal Complete T16 in docs/tasks/T16-component-sprites.md according to docs/PLAN.md
and AGENTS.md. Draw resistors, LEDs, and buttons as 8-bit sprites per
docs/design/sprites/README.md with an extendable per-kind sprite layer.
Satisfy every acceptance criterion and run every required check in the card.
Fix task-scoped findings without weakening tests or acceptance criteria.
Do not implement successor tasks. Prepare one coherent change for review;
do not commit or push. Record exact verification evidence in this card.
If a required check is unavailable, report the exact blocker and the input
or environment change needed; do not count it as passed.
```

## Completion and fukit handoff

When all criteria pass, record evidence and set `Status: ready_for_fukit`. Stop without starting the next task. The user may then invoke `fukit` to describe, commit, and push.

An existing jj repository and an unambiguous authorized remote/bookmark are required for that workflow. Do not initialize or guess them. Commit completion is evidenced by jj history; publication is evidenced by the actual push result.

## Evidence

Implementation and verification evidence on 2026-09-25:

- Baseline checks passed: `PATH=/home/vetalik/.cargo/bin:/usr/bin:/bin cargo fmt --all --check`; `PATH=/home/vetalik/.cargo/bin:/usr/bin:/bin cargo clippy --workspace --all-targets --locked -- -D warnings`; `PATH=/home/vetalik/.cargo/bin:/usr/bin:/bin cargo test --workspace --locked` (16 app tests and 37 core tests passed).
- Locked builds passed: `PATH=/home/vetalik/.cargo/bin:/usr/bin:/bin cargo build -p bredboard-app --target x86_64-unknown-linux-gnu --locked` and `PATH=/home/vetalik/.cargo/bin:/usr/bin:/bin cargo build -p bredboard-app --target wasm32-unknown-unknown --locked`. No browser interaction test was run.
- Adding Proptest as an app dev-dependency changed one line of `Cargo.lock` (the app's dependency list; no new packages). The lockfile was updated with `cargo test -p bredboard-app --offline`; all checks above then ran with `--locked`.
- Golden references were generated with `BREDBOARD_BLESS_SPRITES=1 cargo test -p bredboard-app --offline body_sprites` and compared against the approved design prototype's output: all six body sprites (resistor 330 Ω, LED off/dim/lit, button released/pressed) matched pixel for pixel.
- Property tests with fixed seeds: `bands_decode_to_the_value_within_two_digit_rounding` (`0xB4ED_5B17`, 256 cases), `placements_cover_every_pin_hole` (`0x5B17_E500`, 96 cases), `brightness_state_never_drops_as_current_rises` (`0x1ED5_7A7E`, 128 cases). No failing cases were found, so no regressions were retained.
- `part_sprites_follow_core_led_current_and_button_state` spawns the LED bench, checks initial off/released sprites, then pressed and running (lit, pressed), then stale (off).
- Linux UI check (Omarchy, Hyprland, Intel Iris Xe, Mesa 26.2.2, Vulkan): `target/debug/bredboard-app` rendered the LED bench with the banded resistor, off LED, and compact released button. Synthetic key input could not reach the window, so a temporary local-only startup patch (reverted, not part of this change) opened the LED and transistor benches pressed and running; screenshots showed the lit LED with its halo and the pressed button. The click path itself is unchanged and covered by `shared_menu_and_visible_buttons_dispatch_core_actions`; an owner click-through is still recommended.
- Revision 2026-09-25 (owner feedback: resistor and LED bodies hid where legs enter): resistor body reduced from 20 x 10 to 14 x 8 px with 1 px bands, LED dome from 16 to 12 px (sprite 24 to 20 px). Goldens were re-blessed and again matched the revised design prototype pixel for pixel. All baseline checks and both locked target builds above were re-run after the revision with the same results (16 app, 37 core tests). A Linux screenshot of the revised sprites was not captured: the window manager would not focus the app window for scripted capture, so the owner checked the revised sizes in the Linux executable and approved them on 2026-09-25.
- Known presentation limitation: in `fixtures/projects/transistor-bench.json`, R1 (A5–A8) and R2 (A6–A9) share column A, so their bodies overlap on screen. The fixture layout is electrically valid and was not changed by this task.
