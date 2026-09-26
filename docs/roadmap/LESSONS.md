# Fixed exercise expansion — Phase 2 specification

This is the dedicated specification required by
[the post-MVP roadmap](POST-MVP.md) before implementing any part of "AI-authored
lessons" Phase 2. It scopes a smaller, concrete slice: extend the fixed exercise
list from the MVP's 3 circuits to 13, without free assembly, without a lesson
scripting engine, and without user-facing file controls. Every new exercise is
still a fixed, built-in fixture selected from a menu, exactly like the existing
LED, RC, and transistor circuits; only the number of menu entries and the
component catalog grow.

Do not reopen MVP scope (T00–T14) or T15–T17 while executing this phase. Do not
implement free assembly, drag-and-drop editing, undo/redo, or user-facing project
files as part of this phase; those remain Phase 1.

## Source and translation

The ten exercise designs originate from an owner-authored mockup (owner-only
link, not needed to implement or review):
<https://claude.ai/artifact/Xa645F8b1DtjcEB3rF3pDi>. The mockup is in Russian
and is not authoritative; per `AGENTS.md`, all UI text, lesson text, and code
must ship in English. The table below is the canonical English exercise list;
task cards must not re-translate from the mockup.

## New components required

Three new `ComponentKind` values are needed. Sprite designs for two of them
already exist as "future parts" in
[the sprite design reference](../design/sprites/README.md)
(`future-trimmer-potentiometer.txt`, `future-photoresistor.txt`); the third
(buzzer) needs a new design following the same style rules.

| New kind | Electrical role | Sprite status |
| --- | --- | --- |
| `potentiometer` | Two-terminal variable resistor (wired as a rheostat: only the wiper and one end are connected in the exercises below); resistance set by a continuous control input, not a project parameter, so it can be turned live like a button is pressed | Designed (T17 Part B) |
| `photoresistor` | Two-terminal variable resistor whose resistance is driven by a continuous "ambient light" control input instead of a project parameter | Designed (T17 Part B) |
| `buzzer` | Fixed-resistance two-terminal load with a current-derived "sounding" presentation state (visual only; the app has no audio system) | Not designed yet — needs a new entry in the sprite reference |

Both variable resistors share one new mechanism: a continuous, user-driven
control input (0.0–1.0) analogous to `ControlState` for switches, but a ratio
instead of a discrete position. Implement this once, generically, rather than
duplicating it per kind — the same way three-pin placement was implemented
once for the transistor and changeover switch in T17.

## Exercise list

Difficulty follows the existing informal tiers (easy/medium/hard) used only in
menu presentation, not in the electrical model.

| # | English title | New parts needed | Existing parts reused | Difficulty |
| --- | --- | --- | --- | --- |
| E1 | First Light | — | `dc_voltage_source`, `resistor`, `led` | Easy |
| E2 | Push-Button Switch | — | + `momentary_button` | Easy |
| E3 | Two LEDs in Series | — | two `led`, one `resistor` | Easy |
| E4 | Two LEDs in Parallel | — | two independent `resistor`+`led` branches | Medium |
| E5 | Brightness Dial | `potentiometer` | `resistor`, `led` | Medium |
| E6 | Light-Reactive LED | `photoresistor` | `resistor`, `led` | Medium |
| E7 | Buzzer Doorbell | `buzzer` | `momentary_button` | Easy |
| E8 | Transistor Switch | — | `momentary_button`, `resistor` x2, `led`, `npn_transistor` | Hard |
| E9 | Logical AND | — | two `momentary_button` in series, `resistor`, `led` | Medium |
| E10 | Smooth Fade | — | `momentary_button`, `capacitor`, `resistor`, `led` | Hard |

E1–E4, E8 closely resemble the existing MVP circuits in wiring shape but are
separate, simpler fixed exercises with their own titles, tasks, and hints; do
not merge them with the existing three benches or change the existing three
benches' behavior.

Each exercise needs, in English: a title, a one-paragraph explanation of the
circuit's behavior, a parts list, a short player task/goal, and the connection
list. Source content for these (translated and adapted from the mockup, not
copied verbatim where the mockup states a check specific to the mockup's own
UI) lives with whichever task card adds the exercise; this spec does not
duplicate the full copy to avoid drift between two authored copies.

## Task sequence

1. [T19 — Variable-resistor components](../tasks/T19-variable-resistor-components.md):
   `potentiometer` and `photoresistor` kinds, the shared continuous control
   input, solver stamps, and sprites.
2. [T20 — Buzzer component](../tasks/T20-buzzer-component.md): `buzzer` kind,
   sprite design and implementation.
3. [T21 — Scrollable exercise menu](../tasks/T21-scrollable-exercise-menu.md):
   generalize the fixed 3-circuit menu to a scrollable list of N fixed
   circuits. Land this before either exercise batch so both can register
   through the same list.
4. [T22 — Basic new exercises](../tasks/T22-fixed-exercises-basic.md): E1, E2,
   E3, E4, E7, E8, E9, E10 (needs T20 and T21).
5. [T23 — Variable-resistor exercises](../tasks/T23-fixed-exercises-variable.md):
   E5, E6 (needs T19 and T21).

T22 and T23 can be done in either order once their dependencies land; neither
depends on the other.

## Explicitly out of scope for this phase

- Free placement, wire editing, or a schematic view.
- A lesson-step/hint/completion-condition engine; each exercise is a fixed
  fixture with static text, like the existing three circuits.
- Audio output for the buzzer; it is a visual-only presentation state.
- A true three-terminal potentiometer voltage divider; the exercises here only
  use it as a two-terminal rheostat, matching the mockup's own wiring.
- WASM browser interaction testing (unchanged MVP policy).
