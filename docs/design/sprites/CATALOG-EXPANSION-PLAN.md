# Design expansion plan for the circuit catalog

This lists what the sprite/design reference is missing to eventually draw
the parts named in `breadboard-circuits/spec/catalog.json` (see
[docs/roadmap/CIRCUIT-CATALOG.md](../../roadmap/CIRCUIT-CATALOG.md) for the
implementation-side phase plan this mirrors). **This is a plan, not a
design pass**: no artboard is edited and no `.txt` golden reference exists
yet for anything below. Follow the same "design only" discipline
[T17](../../tasks/T17-more-component-sprites.md) used for its Part B future
parts — each family gets its own reviewable design pass, added to
`docs/design/sprites/README.md` as a new table, only when its implementation
phase is actually requested.

## Current state

The design canvas ("8-bit Breadboard Components",
<https://claude.ai/artifact/HM3Apo7m4PoRJLLGChX7eu>) has four artboards:
`Main.dc.html` (sprite sheet, ~340 KB already), `Board.dc.html` (interactive
board demo), `T17.dc.html` (historical T17 sprites), and `Circuits.dc.html`
("Parts for the 10 circuits"). `docs/design/sprites/README.md` documents 10
implemented kinds and, in "Future parts," 8 designed-but-unimplemented ones
(rectifier/signal diode, ceramic capacitor, PNP transistor, slide/toggle
switch, green/yellow/blue LED). None of that covers anything below.

## New sprite families needed, by category

Grouped the way the gap analysis in `CIRCUIT-CATALOG.md` groups them, since
each family becomes one design pass:

1. **IC body (new shape language).** A generic DIP package — flat black
   body, pin-1 notch/dot, a silkscreened part label — parameterized by pin
   count (8/14/16-pin covers nearly everything: 555/556, 74HC-, CD40-,
   LM358/393/339, LM3914). Design one generator, not one sprite per chip
   number; the printed label is what tells chips apart, the same way the
   resistor's colour bands already encode a value rather than needing a
   sprite per resistance. Needed before Phases B/C/D in the catalog roadmap
   can draw anything.

2. **Relay.** SPDT/DPDT electromechanical body: coil block plus armature,
   two states (de-energized/energized) showing the armature or contact
   throw move, the same visual idea as the existing `changeover_switch`
   slider but driven by coil current instead of a player click. Feeds
   catalog Phase E and I.

3. **Motion/actuator family.** DC motor, TT gearmotor, SG90 servo,
   28BYJ-48 stepper, solenoid, vibration motor, small fan, small pump —
   one "spinning/moving output" visual language (e.g. a rotating mark or
   shaft indicator) shared across the family, plus a distinct silhouette
   per part so they read apart on a crowded board. Feeds catalog Phase E.

4. **Display family.** 7-segment digit (common-cathode and common-anode
   variants, one visual state per lit-segment combination is a rendering
   concern, not per-sprite), 10-LED bargraph, 8×8 LED matrix, RGB LED
   (three independent channels on one body). Feeds catalog Phase D.

5. **Ready-made module family (biggest gap, new body language).** Small
   PCB modules with a pin header row rather than a DIP body: PIR
   HC-SR501, IR receiver TSOP38238, HC-SR04 ultrasonic, MQ-2 gas sensor,
   ISD1820 voice module, 433 MHz TX/RX pair, PT2262/PT2272 encoder/decoder,
   ULN2003/ULN2803 driver board, TP4056 charge module, KY-038 sound
   sensor. Needs its own green/blue PCB-rectangle-plus-header look, with
   one distinguishing icon per module (antenna, lens, trimmer pot) rather
   than a body shape per module. Feeds catalog Phase F.

6. **Remaining discretes**, each small and mostly independent: zener diode
   (banded body variant of the existing rectifier-diode design), Schottky
   diode, JFET, phototransistor, IR LED, photodiode, optocoupler PC817
   (LED+photodetector pair inside one mini-package), NTC thermistor, Hall
   sensor (analog SS49E / digital A3144), crystal can (two-pin, both the
   32.768 kHz and 3.2768 MHz values), piezo disc, electret microphone,
   laser module (reuses the module-family look from item 5), solar panel,
   supercapacitor/ionistor, 18650 cell + holder, power transistor packages
   larger than TO-92 (TIP120/BD139/BD140, TO-220 body), DIP switch bank,
   rotary/wafer switch, reed switch + magnet.

## Board/system-level gaps (not sprite bodies)

- **More off-board power sources.** Only the single 9 V block exists today;
  the catalog needs 3×AA/4×AA/2×AA battery holders, an 18650 holder, and AC
  adapter variants, each as its own off-board block like
  `dc_voltage_source`.
- **A second breadboard side by side.** Several circuits explicitly call
  for an additional 830-point board; `Board.dc.html` only shows one today.
- **Labeled-net convention.** The source schematics mark same-labeled points
  ("RST" etc.) as connected without drawing a wire between them; nothing in
  the current board view has an equivalent, and dense systems (sections
  05/09/19) will be unreadable as pure point-to-point wires. Needs its own
  design decision, not just new sprites.
- **Schematic/reference view.** `docs/PLAN.md` explicitly excludes a
  schematic view from MVP scope, but every one of the 212 circuits ships a
  pre-drawn schematic. Record whether/how those get surfaced (e.g. a
  reference panel) as an open product question before any catalog phase
  assumes it — do not silently reintroduce a schematic editor.
- **"Find the bug" presentation.** Section 20 needs a way to show a built
  board with a marked faulty component, or a built/correct pair side by
  side. No design exists for this yet; it is a new screen, not a new part.
- **Non-breadboard props.** Section 18 and parts of 08/13/19 (lemon, water
  cup, foil, pencil, donor toy/printer/CD drive) have no board
  representation at all. This is the same open scope question flagged in
  `CIRCUIT-CATALOG.md` Phase L — resolve whether these are in scope before
  designing stand-ins for them.

## Suggested sequencing and canvas layout

Do one family per design pass, in the same order as the implementation
phases in `CIRCUIT-CATALOG.md` (A → M), so a design pass is never ahead of
a requested implementation phase. Add each family as its own new artboard
on the existing canvas (e.g. `ICs.dc.html`, `Modules.dc.html`,
`Motors.dc.html`, `Displays.dc.html`) rather than growing `Main.dc.html`
further — it is already the largest file on the canvas (~340 KB) and mixes
every implemented kind today.

## What this document does not do

It records gaps and a suggested order; it does not add any table to
`docs/design/sprites/README.md`, publish any artboard, or generate any
golden reference. That work starts only when a specific family's
implementation phase from `CIRCUIT-CATALOG.md` is actually requested.
