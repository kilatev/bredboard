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

## Extended exercise list (E11–E30)

A second owner-authored mockup (owner-only link, not needed to implement or
review): <https://claude.ai/artifact/9kKBB71Mty3D57GwJZckKm>, in Russian,
levels 11–30, extends the exercise list from 10 to 30. Unlike E1–E10, every
one of these 20 uses component kinds already in the catalog after T17/T19/T20
(`dc_voltage_source`, `resistor`, `led`, `momentary_button`, `changeover_switch`,
`capacitor`, `npn_transistor`, `potentiometer`, `photoresistor`, `buzzer`); no
new `ComponentKind` is needed for this slice. The board's two independent rail
pairs (`TP+`/`TP-` and `BP+`/`BP-`, both wired to a common negative in E28)
already support two coexisting `dc_voltage_source` components, so E28 (dual
battery) needs no new board or persistence support either.

| # | English title | Notable mechanism | Difficulty |
| --- | --- | --- | --- |
| E11 | Two-Way Switch | `changeover_switch` selects one of two LED branches | Easy |
| E12 | Mixed Series/Parallel Wiring | two independent branches, one with LEDs in series | Easy |
| E13 | Capacitor Against Bounce | capacitor across a button-driven LED branch | Easy |
| E14 | Buttons OR | two buttons in parallel | Medium |
| E15 | Transistor Inverter | button pulls the base low, closing the transistor (logical NOT) | Medium |
| E16 | Automatic Night Light | photoresistor pulls the base low in light (inverse of E6) | Medium |
| E17 | Light Alarm | photoresistor between `+` and base; light opens the transistor, buzzer sounds | Medium |
| E18 | Volume Control | potentiometer in series with the buzzer, current-limiting | Medium |
| E19 | Three Independent Branches | three resistor/LED branches with different resistor values for equal brightness | Medium |
| E20 | Turn-On Delay | large-resistor RC charge into the base, delayed transistor turn-on | Hard |
| E21 | Adjustable Night-Light Threshold | photoresistor + potentiometer divider sets the trigger level | Hard |
| E22 | Mixed Logic | `(A AND B) OR C` from three buttons | Hard |
| E23 | Light and Sound Together | one transistor drives an LED and a buzzer in parallel | Hard |
| E24 | Capacitor Charge and Discharge | switch routes current into a charge branch or a discharge branch | Hard |
| E25 | Transistor AND | two transistors in series, each gated by its own button | Hard |
| E26 | Transistor OR | two transistors with tied collectors, each gated by its own button | Hard |
| E27 | Shared Brightness Control | one potentiometer node feeds two transistor bases together | Hard |
| E28 | Power Source Selector | switch picks between a 5 V and a 9 V source, each with its own resistor | Hard |
| E29 | Sensitivity Detector | Darlington pair amplifies a tiny base current through a 1 MΩ resistor | Hard |
| E30 | Two-Transistor Flasher | astable multivibrator: cross-coupled capacitors, no button | Hard |

Full parts lists, explanations, player tasks, and connection lists for E11–E30
are specified in their own task cards (T24–T27), not duplicated here, per the
same rule as E1–E10.

E30 is a genuine free-running oscillator, unlike every other fixed exercise
(including E1–E10), which are static or button/dial-driven. A perfectly
symmetric numeric simulation of a symmetric astable multivibrator can settle
at (or never leave) the unstable symmetric equilibrium instead of oscillating,
the way a real circuit's component tolerances break the tie. T27 (which adds
E30) must specify and verify a concrete, deterministic way to break that
symmetry (for example, a tiny asymmetry in initial capacitor voltages or
resistor values, or an explicit documented initial condition) and must not
claim E30 works from a single lucky run; the acceptance criteria require a
regression test that starts from the documented initial state and confirms
sustained oscillation over many transient steps.

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
6. [T24 — Fixed exercises E11–E15](../tasks/T24-fixed-exercises-e11-e15.md).
7. [T25 — Fixed exercises E16–E20](../tasks/T25-fixed-exercises-e16-e20.md).
8. [T26 — Fixed exercises E21–E25](../tasks/T26-fixed-exercises-e21-e25.md).
9. [T27 — Fixed exercises E26–E30](../tasks/T27-fixed-exercises-e26-e30.md).

T22 and T23 can be done in either order once their dependencies land; neither
depends on the other. T24–T27 each depend only on T21 (the scrollable menu)
and this document, and are independent of each other and of T22/T23; execute
them in numeric order only because each specifies its fixtures' `id` numbers
relative to the previous card's, not because of any functional dependency.

## Explicitly out of scope for this phase

- Free placement, wire editing, or a schematic view.
- A lesson-step/hint/completion-condition engine; each exercise is a fixed
  fixture with static text, like the existing three circuits.
- Audio output for the buzzer; it is a visual-only presentation state.
- A true three-terminal potentiometer voltage divider; the exercises here only
  use it as a two-terminal rheostat, matching the mockup's own wiring.
- WASM browser interaction testing (unchanged MVP policy).
