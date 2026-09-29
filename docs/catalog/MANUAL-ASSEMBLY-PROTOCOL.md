# Shared manual breadboard assembly protocol

This protocol is the common manual-assembly gate for the catalog execution
plan. It is acceptance evidence independent of Rust tests, SVG rendering, or
app fixture loading. It applies to C01 and is reused by later batches unless a
card records a stricter, entry-specific rule.

## Reference hardware and electrical limits

- Use one documented 830-tie-point solderless breadboard for ordinary
  one-board schemes: rows 1–30, A–E and F–J contact strips, center gap, and
  two visually distinct power rails on each side.
- Treat the reference rails as continuous only after checking the board. If a
  physical board has split rails, install and record explicit rail jumpers;
  never rely on an unlabeled visual assumption.
- Use a regulated 5.0 V supply for the product baseline unless the task card
  explicitly approves another source in the source specification's 5–12 V
  range. No mains, inverter, or unapproved USB connection enters the circuit.
- Use a current-limited supply appropriate to the fixture. Record the limit;
  stop immediately on an unexpected short, heating, smell, or visible damage.
- Disconnect power before every placement, polarity, continuity, rail, and
  pin-number check. Do not power a deliberately faulty or reversed assembly
  longer than the minimum observation needed by its card.
- Skin-contact experiments are battery-only and require the card's explicit
  current limit. Do not connect a person to a USB-powered, mains-referenced,
  or computer-connected circuit. Class-2 laser experiments require the beam
  to be kept away from eyes and reflective surfaces.

## Canonical translation rules

- Give every part a stable fixture ID (`V1`, `R1`, `D1`, `Q1`, `S1`, and so
  on), and use the same ID in the content record, hole layout, readouts, and
  assembly notes.
- Use the exact purchased part and package named by the fixture. A source
  alternative such as “BC547 or 2N3904” must be resolved to one part before
  the hole map is published.
- Record pins by electrical name, not by drawing position: source
  `positive/negative`; resistor and two-terminal variable parts `a/b`; LED or
  diode `anode/cathode`; polarized capacitor `positive/negative`; NPN/PNP
  `emitter/base/collector`; SPDT `common/normally_closed/normally_open`;
  ICs by manufacturer pin number with notch or dot orientation.
- For the current potentiometer and photoresistor contract, use only the two
  documented rheostat terminals. Do not silently connect or serialize a third
  wiper pin.
- A breadboard contact group joins holes only as documented by the board
  model. A wire crossing without an endpoint does not connect. Mark every
  rail jumper and every cross-gap connection in the hole-level layout.
- Keep every lead and wire endpoint on a distinct intended hole. Duplicate
  occupancy, an unmarked rail break, or a schematic-only net label is a
  buildability finding.

## Assembly and observation sequence

1. Inventory the exact BOM, board count, supply voltage/current limit, and
   required tools. Confirm that every nonstandard prop is present.
2. With power disconnected, identify the board orientation, rail polarity,
   rail breaks, center gap, and all required rail jumpers.
3. Place polarized and pin-sensitive parts using the canonical pin map. Check
   package notch/flat face, LED/diode bands, capacitor stripe, and connector
   polarity against the fixture record.
4. Place resistors, controls, and props. Route jumpers exactly between the
   recorded hole IDs; keep signal and power paths visually separable for a
   novice.
5. Perform continuity checks for the positive rail, negative rail, each
   intended series path, each intended shared node, and the absence of a
   positive-to-negative short. Record measured values and meter mode.
6. Compare every occupied hole with the fixture layout. Resolve unoccupied,
   duplicate, or ambiguous pins before power is connected.
7. Apply the approved low-voltage source. Observe startup for the minimum
   safe interval, then record supply voltage/current and any unexpected
   heating, noise, oscillation, or instability.
8. Exercise every specified button, switch, sensor, control ratio, and prop
   state. Record the physical output and measured voltage/current at the
   control points named by the task card.
9. Compare those observations with the documented content behavior and app
   readouts. A mismatch is evidence to record, not a reason to edit the
   observation away.
10. Disconnect power, return the board to a safe state, and record deviations,
    missing parts, unsafe behavior, ambiguous instructions, and any repair.

## Evidence record

Use one record per representative assembly with the stable audit key in the
filename, for example `C01-S01-02-YYYY-MM-DD`. The record contains:

- source section/file, SVG path, fixture/project ID, board count, and exact
  part substitutions (if any);
- supply type, measured voltage, current limit, and meter/tool identifiers;
- one overview photo or equivalent visual record with the board orientation
  and rails visible;
- close-up records for pin-sensitive packages, polarized parts, rail
  jumpers, and any unusual prop;
- continuity/short result before power, startup observation, every control
  state, and measured comparison with the fixture readout;
- deviations, missing or ambiguous BOM items, unsafe observations, and the
  disposition of each finding (`source_defect`, `implementation_blocker`,
  `safety_blocker`, or `manual_assembly_finding`).

The ledger records the evidence path and result. `not run`, an unmeasured
claim, or an SVG screenshot without a physical record does not satisfy the
manual gate.

## C01-specific risks

- Resolve the source 9 V labels against the product's 5 V fixture baseline.
- Select exact BC547/2N3904 and BC557/2N3906 packages before placement.
- S02-03 touch pads use the fixture's explicit contact-resistance endpoints;
  S02-04 water probes use the explicit dry/wet conductivity endpoints. Keep
  both battery-only and outside the ordinary powered-board path until the
  physical safety and input measurements are recorded.
- Do not reproduce S01-01's “LED burns with smoke” behavior on a real board.
- Treat the S02-07 8 Ω speaker as a power-limited load; stop if the measured
  output can exceed its stated 0.5 W rating.
