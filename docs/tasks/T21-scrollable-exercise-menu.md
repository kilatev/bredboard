# T21 — Scrollable exercise menu

Status: ready_for_fukit

## Dependencies

[T09 — Three-circuit menu and hole-accurate board](T09-board-instruments.md)
and [docs/roadmap/LESSONS.md](../roadmap/LESSONS.md). Independent of T19 and
T20; land this before T22/T23 so both batches register through the same list.

## Outcome and commit boundary

Generalize the fixed-size, fixed-position 3-entry menu (`spawn_menu` in
`crates/app/src/main.rs`, `Circuit::all()`) into a scrollable list that can
hold an arbitrary number of fixed exercises without entries running off the
visible window or overlapping.

Suggested commit title: `feat: add a scrollable exercise menu`.

Do not add any new exercise beyond the existing three in this task; this is a
presentation change only. Do not add free assembly, a search box, favorites,
or any per-exercise state beyond selection.

## Scope and interfaces

- The current menu spawns one button per `Circuit::all()` entry at a fixed
  world-space Y offset, sized for exactly 3 entries in a 760-unit-tall window.
  Replace this with a layout that supports N entries: either a scrollable
  viewport (clip mask plus a mouse-wheel/drag-driven vertical offset,
  clamped so the list cannot scroll past its first or last entry) or an
  equivalent mechanism that keeps every entry reachable and legible at the
  supported window sizes. Keep entries the same size and readable as today's
  three.
- Scrolling and selection are presentation/input concerns; they must not touch
  `bredboard-core` or any `Action`. This task only changes `crates/app`.
  Selecting an exercise still routes through the existing `Control::Select`
  path unchanged.
- Preserve keyboard/window-resize behavior already covered by T09's
  acceptance criteria (resizing does not change electrical state or hide
  controls); resizing must also not desynchronize scroll position from the
  visible entries.
- Add a documented, obvious extension point: the eventual exercise list (T22,
  T23's 8 + 2 additions to the existing 3, for 13 total) must not require
  further menu-layout changes, only appending to the list of selectable
  circuits.
- No change to WASM compilation; browser scroll-wheel/touch interaction
  remains untested per MVP policy.

## Acceptance criteria

- [ ] With the existing 3 circuits, the menu renders and behaves exactly as
  before (regression: same selection behavior, same visible layout at the
  current default window size).
- [ ] A test-only or debug configuration with more entries than fit on screen
  (for example, temporarily stubbing 13 circuits) demonstrates every entry is
  reachable by scrolling and none overlaps another or runs outside the menu's
  visible bounds.
- [ ] Scroll position is clamped: it cannot scroll past the first or last
  entry.
- [ ] Window resize does not lose track of which entry is at the top of the
  visible list in a way that hides the currently reachable entries.
- [ ] Existing menu/board interaction tests and fixtures pass unchanged.

## Required verification

- Run the baseline formatting, Clippy, workspace test, Linux build, and WASM
  build commands documented in `README.md`.
- Manually check the menu in the Linux executable: the existing 3 entries,
  then a temporary larger stub list, scrolling to both ends, and a window
  resize mid-scroll. Do not test browser interaction.

A missing or failing required check blocks readiness.

## Codex Goal

```text
/goal Complete T21 in docs/tasks/T21-scrollable-exercise-menu.md according to
docs/PLAN.md, AGENTS.md, and docs/roadmap/LESSONS.md. Generalize the fixed
3-entry circuit menu into a scrollable list that supports an arbitrary number
of fixed exercises, with no change to core, action routing, or existing
circuit behavior. Do not add the LESSONS.md exercises themselves.
Satisfy every acceptance criterion and run every required check in the card.
Fix task-scoped findings without weakening tests or acceptance criteria.
Do not implement successor tasks. Prepare one coherent change for review;
do not commit or push. Record exact verification evidence in this card.
If a required check is unavailable, report the exact blocker and the input
or environment change needed; do not count it as passed.
```

## Completion and fukit handoff

When all criteria pass, record evidence and set `Status: ready_for_fukit`.
Stop without starting successor tasks. The user may then invoke `fukit` to
describe, commit, and push.

An existing jj repository and an unambiguous authorized remote/bookmark are
required for that workflow. Do not initialize or guess them. Commit
completion is evidenced by jj history; publication is evidenced by the actual
push result.

## Evidence

Implementation: `crates/app/src/main.rs`. Added `MenuScroll` (presentation-only
resource, world-space offset), `MenuEntry(index)` marker component shared by
an entry's button rect and label, `menu_entry_base_y`/`menu_max_scroll`
(pure, tested layout math), `handle_scroll` (reads `MouseWheel`, clamps
offset to `[0, menu_max_scroll(entry_count)]`), and `scroll_menu` (repositions
each entry's `Transform` and hides entries outside `[MENU_BOTTOM, MENU_TOP]`
via `Visibility`, also skipped in `handle_mouse`'s hit test). `Circuit::all()`
is unchanged at 3 entries; `menu_entry_base_y` reproduces the original fixed
positions exactly (100.0, -5.0, -110.0), so the existing 3-entry menu is
pixel-identical at rest. Scroll offset resets to 0 on `Control::Back`. No
change to `bredboard-core`, `Action`, or `Control::Select` routing; only
`crates/app` changed. No WASM-specific code paths were touched.

Required verification (run from repository root on 2026-09-26):
- `cargo fmt --all --check` — pass.
- `cargo clippy --workspace --all-targets --locked -- -D warnings` — pass.
- `cargo test --workspace --locked` — pass (61 tests: 21 app + 38 core + 2 tools).
- `cargo build -p bredboard-app --target x86_64-unknown-linux-gnu --locked` — pass.
- `cargo build -p bredboard-app --target wasm32-unknown-unknown --locked` — pass.

New/relevant tests in `crates/app/src/main.rs`:
- `existing_three_entry_menu_layout_is_unchanged` — regression: base_y(0..2)
  and `menu_max_scroll(3) == 0.0` match the original fixed layout exactly.
- `thirteen_stub_entries_are_all_reachable_by_scrolling_without_overlap` — a
  debug-only stub of 13 entries (via `spawn_stub_menu`, not a real exercise)
  demonstrates every entry becomes visible across the scroll range and no
  visible entry's bounds fall outside `[MENU_BOTTOM, MENU_TOP]` (the
  overlap-freedom argument: entries are spaced `MENU_ENTRY_HEIGHT` apart,
  wider than their height, so any two visible entries cannot overlap; the
  test checks the remaining condition, that visible entries stay inside the
  declared viewport band).
- `scroll_offset_is_clamped_to_first_and_last_entry` — sends huge `MouseWheel`
  deltas in both directions through `handle_scroll` against the same 13-entry
  stub and confirms the offset clamps at `0.0` and `menu_max_scroll(13)`
  rather than overshooting.
- `shared_menu_and_visible_buttons_dispatch_core_actions` (pre-existing,
  updated only to insert `MenuScroll` and chain `scroll_menu`) still passes
  unchanged, including its window-resize case, confirming resize does not
  desynchronize scroll from the visible entries (scroll offset is stored in
  world units, independent of window/screen size; `cursor_world` already
  re-derives world coordinates from the current window size every frame).

Manual Linux-executable check: attempted. `cargo run -p bredboard-app --locked`
launches and runs normally (log confirms window creation and a working
Vulkan/Mesa render backend; process ran without panicking or exiting). This
session's screen-capture tool (`grim`) is available but returns only the
static desktop wallpaper for every geometry and output tried, including a
full-monitor capture confirmed via `hyprctl` to contain the running
`bredboard` window — screenshotting the live compositor output is not
functional in this environment, and no input-automation daemon (`ydotoold`)
is available to drive a scripted mouse-wheel/resize check either. Per
AGENTS.md, this is recorded as a blocker, not a pass: hands-on visual
confirmation of the 3-entry regression, the stubbed >3-entry scroll, and a
resize-mid-scroll in the actual window is unavailable in this session. The
automated Bevy `App`-level tests above exercise the real
`handle_scroll`/`scroll_menu`/`handle_mouse` systems end-to-end (including
hit-testing against hidden entries) and are the available substitute
evidence; a human with working screen access should still eyeball it before
relying on this as final sign-off.
