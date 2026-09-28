# Coarse triage of all 20 catalog sections

This is the C00 section-level review. It identifies missing capability, safety
assumptions, physical dependencies, and source-design risks before any circuit
implementation. It is deliberately not a solver proof or a manual assembly
pass. Per-entry coordinates and negative evidence are in the
[full ledger](CATALOG-AUDIT-LEDGER.md); C01 receives a detailed audit in
[C01-READINESS-AUDIT.md](C01-READINESS-AUDIT.md).

## Method and evidence

The audit used the source README, all 20 section Markdown files,
`catalog.json`, `catalog.schema.json`, and all referenced SVGs. For each
section it checked:

- the declared section count against the JSON entry count;
- source Markdown headings, embedded SVG paths, and BOM rows against the
  corresponding JSON records;
- that each referenced section Markdown file and SVG exists;
- BOM terms for unsupported component families, package/pin ambiguity,
  physical props, multi-board systems, stored energy, motors, skin contact,
  laser use, and forbidden mains/programmable assumptions;
- source notes that change the assembly or safety interpretation.

The scan found no missing section Markdown files, missing SVGs, heading/image
coordinate mismatches, or per-circuit BOM mismatches. “No missing file” does
not mean “design correct”: the section findings below remain open until the
per-scheme pipeline and manual gate pass.

## Section triage

Disposition counts are the coarse baseline assigned to ledger rows. The
execution-plan meanings are used: `blocked_component` means a model, package,
fixture capability, or required art/contract is missing; `blocked_design`
means a source or safety decision must be corrected; `physical_scope` means a
physical prop, actuator, or multi-board experience is not represented by the
current board fixture; `out_of_scope` means the source is a different mode or
explicitly excluded pending a scope decision.

| Section | Source file | Entries / SVGs present | Coarse design, safety, and buildability findings | Baseline dispositions |
|---|---|---:|---|---|
| S01 | [01-level-1-first-current.md](../../breadboard-circuits/spec/01-level-1-first-current.md) | 6 / 6 | Basic discrete source records are complete; audit found the sacrificial LED wording and the short RC fade claim as design findings. Potentiometer/diode contracts are not all represented by the current catalog fixture set. | blocked_component 5, blocked_design 1 |
| S02 | [02-level-2-transistors.md](../../breadboard-circuits/spec/02-level-2-transistors.md) | 7 / 7 | Basic transistor references are complete; BC547/2N3904 is an unresolved pin-choice notation. Touch and water entries introduce off-board inputs; the PNP/speaker siren needs a PNP contract and output safety review. | blocked_component 4, blocked_design 1, physical_scope 2 |
| S03 | [03-level-3-first-ics.md](../../breadboard-circuits/spec/03-level-3-first-ics.md) | 7 / 7 | IC references are complete, but every entry needs an IC pin/package contract; 555, comparator, Schmitt, and D-flip-flop models are not current catalog capability. Anti-bounce has an explicit two-variant source note. | blocked_component 7 |
| S04 | [04-level-4-counting-display.md](../../breadboard-circuits/spec/04-level-4-counting-display.md) | 9 / 9 | Dense counting/display section; requires digital event/state models, display pin contracts, debounce, and current limiting. Source notes identify CD4017 output-current limitations and anti-bounce requirements. | blocked_component 9 |
| S05 | [05-level-5-systems.md](../../breadboard-circuits/spec/05-level-5-systems.md) | 8 / 8 | System section includes multi-board scope and an explicitly approximate computer. Review required for power budgets, reset/clock states, display mapping, and the SRAM/diode-matrix architecture before any implementation card. | blocked_component 8 |
| S06 | [06-sound.md](../../breadboard-circuits/spec/06-sound.md) | 13 / 13 | Sound section depends on timer/op-amp/audio models and has a source note mentioning a transformer; no mains is allowed. Oscillator startup, virtual ground, loudspeaker loading, and audio presentation need separate decisions. | blocked_component 13 |
| S07 | [07-home-tools.md](../../breadboard-circuits/spec/07-home-tools.md) | 15 / 15 | Home-tool section mixes measurement instruments, modules, and a class-2 laser. Laser safety and source-powered instruments need explicit boundaries; the section cannot be treated as ordinary low-voltage fixtures. | blocked_component 12, physical_scope 3 |
| S08 | [08-motors.md](../../breadboard-circuits/spec/08-motors.md) | 19 / 19 | Motor section is mostly outside the current electrical/physical board model. Each entry needs motor/driver/flyback/current and motion-prop contracts; source notes include a deliberately unsafe no-diode teaching variant. | blocked_component 2, physical_scope 17 |
| S09 | [09-hardware-logic-modules.md](../../breadboard-circuits/spec/09-hardware-logic-modules.md) | 19 / 19 | Hardware-logic/modules section needs IC/module contracts, digital timing, and several physical or sensor props. MQ-2 warm-up, HC-SR04 timing, and the large ping-pong system are explicit readiness risks. | blocked_component 12, physical_scope 7 |
| S10 | [10-power-energy.md](../../breadboard-circuits/spec/10-power-energy.md) | 9 / 9 | Power/energy section contains the only direct 220 V/AC warning and several stored-energy/current risks. The rectifier must remain low-voltage adapter-only; regulators, charging, boost, solar, and motor loads need separate safety review. | blocked_component 6, blocked_design 1, physical_scope 2 |
| S11 | [11-op-amps.md](../../breadboard-circuits/spec/11-op-amps.md) | 8 / 8 | Op-amp section needs op-amp, instrumentation, oscillator, and analog reference models. Virtual-ground stability, diode amplitude control, and signal-source assumptions must be documented before fixtures. | blocked_component 8 |
| S12 | [12-digital-analog.md](../../breadboard-circuits/spec/12-digital-analog.md) | 7 / 7 | Digital/analog section needs ADC/DAC/display contracts and a decision on diode-ROM representation. ICL7107 explicitly requires a split supply; no programmable shortcut is allowed. | blocked_component 7 |
| S13 | [13-biosignals-science.md](../../breadboard-circuits/spec/13-biosignals-science.md) | 7 / 7 | Biosignal/science section requires battery-only skin-contact rules, high-impedance sensor contracts, and physical electrode evidence. The source notes explicitly prohibit mains/USB for muscle sensing. | blocked_component 4, physical_scope 3 |
| S14 | [14-light-effects.md](../../breadboard-circuits/spec/14-light-effects.md) | 8 / 8 | Light/effects section needs RGB/display/optical and motor presentation; the POV and 3×3×3 cube are physical assemblies. Strobe and laser-like light risks need explicit learner warnings. | blocked_component 6, physical_scope 2 |
| S15 | [15-relay-logic.md](../../breadboard-circuits/spec/15-relay-logic.md) | 7 / 7 | Relay section requires coil/contact/flyback models and physical relay behavior. The self-oscillating buzzer intentionally omits a flyback diode, so the source note must be preserved as a safety/design exception. | physical_scope 7 |
| S16 | [16-analog-computer.md](../../breadboard-circuits/spec/16-analog-computer.md) | 6 / 6 | Analog-computer section needs op-amp/multiplier/analog-switch models and careful supply rails. The CD4066 entry has an explicit ±7.5 V requirement that conflicts with the simple single-rail fixture baseline. | blocked_component 5, physical_scope 1 |
| S17 | [17-light-communication.md](../../breadboard-circuits/spec/17-light-communication.md) | 7 / 7 | Optical communication section needs light-coupling models, timing, alignment, and class-2 laser controls. The dual-beam number transfer is a multi-part communication fixture, not a direct wire. | blocked_component 6, physical_scope 1 |
| S18 | [18-hacks.md](../../breadboard-circuits/spec/18-hacks.md) | 16 / 16 | Hacks mix simple experiments with donor hardware, physical materials, and nonstandard construction. Each entry needs an explicit representation decision; do not replace physical meaning with a scripted visual. | blocked_component 11, physical_scope 5 |
| S19 | [19-world-tasks.md](../../breadboard-circuits/spec/19-world-tasks.md) | 22 / 22 | World-task section is dominated by multi-board systems, motors, sensors, actuators, and physical props. BOM quantities and power budgets are often estimates; accept only after decomposition into buildable subassemblies. | blocked_component 4, physical_scope 18 |
| S20 | [20-find-the-bug.md](../../breadboard-circuits/spec/20-find-the-bug.md) | 12 / 12 | Find-the-bug is a distinct fault-pair game mode. SVGs and notes identify intended faults, but it needs a fault representation/diagnostic interaction contract and must not be exposed as ordinary fixtures. | out_of_scope 12 |

## Cross-section component inventory

The first inventory is intentionally grouped by electrical behavior rather
than by every source spelling. Existing current contracts are: DC source,
resistor, LED, capacitor, NPN transistor, momentary button, SPDT changeover
switch, potentiometer, photoresistor, buzzer, and speaker. The following
families require new or separately documented work before their BOM items can
be accepted as fixtures:

| Family | Representative source items | Readiness consequence |
|---|---|---|
| Rectifier/signal diode | 1N4007, 1N4148, diode matrices | Diode pins, polarity, model, sprite, and flyback/OR use need a bounded contract. |
| PNP and other discrete transistors | BC557/2N3906, TIP120, MOSFET, JFET | Pin conventions and device equations/limits are not interchangeable with NPN. |
| Timer, comparator, op-amp, logic, counter, memory ICs | NE555/556, LM393/358/3914, 74HC*, CD*, TL*, ICL7107 | Each package needs pin numbering, unused-input handling, supply limits, and a model/state contract. |
| Displays and indicators | 7-segment, bargraph, matrices, RGB | Multi-pin mapping and current limits need a presentation/electrical contract. |
| Motion and electromechanical | motors, servos, steppers, relays, solenoids, pumps | Requires electrical load/driver/flyback plus deterministic motion/prop scope. |
| Ready-made modules and sensors | PIR, FC-51, MQ-2, HC-SR04, 433 MHz, TP4056, ISD1820 | Requires a pin-level black-box contract; no hidden firmware or scripted outcome. |
| Physical learning props | water, touch pads, electrodes, models, donor hardware, laser alignment | Requires a safe, reproducible manual protocol and explicit product scope. |

## Findings that must not be lost

- The source README says all schematics were checked by eye but must still be
  checked against a solver and a real board; the ledger preserves that as an
  open gate.
- The source allows 5–12 V batteries/adapters but the product plan's
  displayed external supply is 5 V. C01 must choose and document the fixture
  voltage rather than silently inheriting 9 V labels.
- Section 10's rectifier note is low-voltage-adapter-only; no 220 V source may
  enter the breadboard.
- Skin electrodes in section 13 remain battery-only, and class-2 laser notes
  require learner warnings and physical handling rules.
- Section 20 is not ordinary content: it is a fault-pair diagnostic mode and
  remains `out_of_scope` until its interaction contract exists.
- No entry is accepted solely because an SVG renders or a BOM parses.

## Baseline decision

The corpus is internally reconciled and fully indexed. The implementation
initiative remains open with the ledger dispositions above; the next bounded
work item is C01 after the shared C00b protocol and the C01 blockers are
accepted by the owner.
