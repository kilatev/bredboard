# T31 — Build-first beginner learning flow

Status: ready_for_fukit

## Owner request

Editing is a baseline product capability, not an optional future mode. A child
can follow an electrical schematic, invent a virtual breadboard layout without
owning hardware, or inspect a ready layout and reproduce it physically. Keep
help immediately accessible and avoid requiring theory or patience before a
first successful build. Scope is bredboard only; thegame is a UX reference.

## Bounded delivery

- A prominent beginner entry and three introductory lessons: LED, RC and NPN.
- Original electrical SVG diagrams, displayed offline in the shared app using
  checked-in PNG renderings. Clearly distinguish schematic and physical layout.
- Editable learner board, independent read-only ready board, and schematic tabs.
  Switching views preserves the learner's Project, simulation and undo history.
- Optional short assembly guidance, exact highlighted contacts, one-step assisted
  placement and ordinary undo. No mandatory quiz, timer or hidden answer gate.
- Calculated readings remain authoritative. Alternative working layouts are
  allowed; exact-hole guidance describes one example, not the only solution.
- Hardware replication notes: power off while wiring, LED polarity, capacitor
  polarity, transistor pinout varies by part, and beginner-safe 5 V supply.
- Preserve existing library entries and free assembly. No accounts, lesson DSL,
  save format changes or rewrite of the full catalog in this task.

## Product direction

The first lesson can reveal the ready board immediately. Later lessons begin
with the schematic and an empty learner board, while the ready view and optional
guide remain available. Future catalog lessons should progressively remove
initial placement assistance, without withholding the answer from a child who
needs it. Existing catalog work remains available as a circuit library; it is
not a prerequisite for the introductory path.

## Acceptance

- Run formatting, Clippy, workspace tests and Linux/WASM builds.
- Seeded property coverage for preview independence and assisted assembly/undo.
- Real native interaction covers start, diagram, building, next-step help,
  calculated behavior, undo, and returning from preview without losing work.
- Record commands, outcomes and assets/attribution here before marking ready.

## Implementation notes

`learning.rs` supplies three authored introductory paths over the existing
assembly editor. The immutable reference Project is independent of the learner
Project and SimulationState. Preview rendering requests a calculated step on a
temporary reference bench; view commands never replace the learner's state.
Keyboard and pointer editing are disabled outside MY BOARD. Switching back
preserves the editor's selection, pending gesture, typed input and Undo.

The optional guide compares placed parts/wires with one reference recipe. It
selects the correct next tool after manual or assisted placement and uses each
lesson's actual nominal values (for example, RC R1 is 1k, not the free palette's
330 ohm default). Assisted steps use the same atomic validation/recalculation and
Undo path as manual edits. An occupied hole or duplicate ID rejects the step
without overwriting the learner's different layout. Guide matching is only
assistance; it does not grade electrical success or block other arrangements.

Incomplete guided assemblies still expose actual core diagnostics, with a
neutral STILL BUILDING prefix instead of implying that the child failed.
Completed or unguided invalid circuits retain the ordinary failure diagnostic.
No scripted readings, separate editable netlist, new dependencies, core source
changes or persistence/schema changes were introduced.

SVG and PNG assets are original project artwork under the repository MIT
license. Their electrical connections and values follow the three existing
fixtures; the repository's broader reference catalog is not silently substituted
for the implemented models. Screenshots and instructions live in
[the beginner preview](../design/learning/README.md).

## Verification commands and environment

Rust 1.95.0 / Bevy 0.19.1 on Linux x86_64, using the pinned lockfile:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo build -p bredboard-app --target x86_64-unknown-linux-gnu --locked
cargo build -p bredboard-app --target wasm32-unknown-unknown --locked
git diff --check
```

Managed-environment builds reused a shared Cargo target directory, with
`CARGO_BUILD_JOBS=2`, `CARGO_PROFILE_DEV_DEBUG=0` and
`CARGO_PROFILE_TEST_DEBUG=0`. Native dependency metadata and software Vulkan
were workspace-local, as in T30. No system package installation or project
dependency change was required. Browser runtime remains deferred by PLAN;
WASM evidence is compilation only.

Seven added tests cover all three calculated circuits and image decoding,
automatic tool advancement during manual guidance, RC-specific manual nominal
values, a different working placement, actual Bevy pointer/keyboard interaction,
and two seeded properties (32 cases each, seed `20261025`) for preview
independence and assisted edits/Undo.

Native verification used the real app on Xorg :98 with Mesa llvmpipe/Vulkan,
XTest pointer/key events and screenshots. Opened the primary beginner entry,
viewed diagrams and ready examples, manually placed the first resistor,
assisted the remaining LED steps, operated B1, ran/paused, and inspected D1:
**8.576 mA**, anode **2.17 V**, cathode **0.00 V**. Opening the ready view and
returning retained the lit circuit and calculated readings; Undo removed the
last structural step. The RC guide placed its actual **1000 ohm** R1 and showed
an unfinished-circuit diagnostic without inventing readings. Confirmed the NPN
ready view includes all component contacts and seven wires without clipping.

The first capture of a newly started software-rendered window preceded its
first paint; it was not treated as successful visual evidence. Published
screenshots were taken after rendering completed. The environment has no audio
device (ALSA warnings); audio was outside this task. A preliminary Clippy pass
caught a large enum; guide steps now borrow the immutable reference instead of
cloning full components during inspection. An earlier test run was interrupted
while making that change and is not counted as a pass.

## Final results

- Formatting and `git diff --check`: passed.
- Workspace/all-targets Clippy with `-D warnings`: passed.
- Full workspace tests: **240 passed** (82 app, 155 core, 3 tools; no doctests).
  App tests took 314.32 s, core 121.62 s and tools 2.28 s in the recorded run.
- Final Linux x86_64 and WASM builds: passed after all source changes.
- Real native input/rendering verification: passed as described above; preview
  screenshots were inspected after the actual window rendered.
- Ready status records completed verification. Commit/publication is reported
  separately after the actual GitHub operation; it is not implied by this card.

## Remaining limits

The new build-first path covers three introductory circuits, not the full
catalog. The existing library and seven-kind free-assembly palette remain
available. No lesson DSL, persistent progress, automatic grading, file-saving
UI or localization was added. Guidance follows the example contacts, while
independent layouts remain supported by the normal solver/editor.

## Two-level native demonstration

[90-second recorded demo](../design/learning/two-levels-demo.mp4): First Light
(0:02) and RC charge/discharge (0:34). Recorded directly from the native window
after first-paint warmup. Shows schematic/reference/learner views, manual first
placements, assisted subsequent steps, actual 8.576 mA LED operation and C1
voltage rising/falling when S1 changes. English captions occupy an added strip
below the app viewport; there is no audio. No app source was changed to record
the demonstration.
