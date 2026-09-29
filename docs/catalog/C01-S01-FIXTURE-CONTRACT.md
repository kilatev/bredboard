# C01 fixture contract: Sections 01 and 02 slices

Status: implemented; manual assembly evidence is still required before any
entry can become `ready`.

The six Level 1 fixtures use the product's 5 V external supply, not the 9 V
label in the source drawings. Connectivity is derived from board holes,
component pins, and explicit wire endpoints. The fixtures use stable IDs and
reserve every pin and wire endpoint hole exactly once.

| Entry | Fixture | Source reference | Controls/readouts | Model decision |
|---|---|---|---|---|
| C01-S01-01 | `c01-s01-01-first-safe-light.json` | `01-level-1-first-current.md`, `01.svg` | LED current; bounded safe light | The source's destructive “smoke” outcome is replaced with a calculated current and the 470 ohm limiter. |
| C01-S01-02 | `c01-s01-02-button-and-switch.json` | `01-level-1-first-current.md`, `02.svg` | B1 button; S1 SPDT; D1/D2 currents | `common` selects `normally_closed` or `normally_open`; each branch has its own 470 ohm resistor. |
| C01-S01-03 | `c01-s01-03-series-and-parallel.json` | `01-level-1-first-current.md`, `03.svg` | D1–D3 and resistor currents | The hole-level fixture keeps the series pair and parallel branch physically separate; R1 is 220 ohms and R2–R4 are 470 ohms. |
| C01-S01-04 | `c01-s01-04-potentiometer-dimmer.json` | `01-level-1-first-current.md`, `04.svg` | RV1 ratio; D1 current | The potentiometer is a two-terminal rheostat with a 330 ohm minimum resistance. |
| C01-S01-05 | `c01-s01-05-reverse-polarity.json` | `01-level-1-first-current.md`, `05.svg` | D0 diode and D1 LED currents | `diode` is a two-pin smooth rectifier model with `anode/cathode`, 0.7 V forward voltage, and a visible cathode band. Reversing the source blocks the branch. |
| C01-S01-06 | `c01-s01-06-smooth-fade.json` | `01-level-1-first-current.md`, `06.svg` | B1 button; C1 voltage; D1 current | The 470 uF capacitor and 470 ohm discharge resistor document a sub-second nominal time constant; the old “couple seconds” claim is not used in the English fixture text. |

Automated evidence for this slice is provided by the core C01 fixture and
polarity tests and the app's embedded-board, menu, control, sprite, and reset
tests. Manual continuity, polarity, measurement, and visual evidence remain
open in the shared assembly protocol.

## Section 02 NPN fixtures

The admitted NPN slice selects a BC547 convention for content and assembly:
the flat face is the reference orientation and the electrical names remain
`base`, `collector`, and `emitter` in the project contract. The source's
“BC547 or 2N3904” alternative is not exposed in the fixture. The three
implemented Section 02 fixtures use the normalized 5 V supply. S02-06 follows
the five-transistor topology drawn in its SVG; the source BOM's three-NPN count
is retained as a catalog discrepancy rather than silently dropping two stages.

| Entry | Fixture | Controls/readouts | Model decision |
|---|---|---|---|
| C01-S02-01 | `c01-s02-01-transistor-key.json` | B1; D1 and Q1 collector current | The source's unlisted motor behavior is excluded; the calculated LED load is the documented switched output. |
| C01-S02-02 | `c01-s02-02-dusk-night-light.json` | R3 ambient light; RV1 threshold; D1/Q1 current | The photoresistor and potentiometer are two-terminal rheostats; dark raises the base drive and bright lowers it. |
| C01-S02-03 | `c01-s02-03-touch-button.json` | TP1 touch-resistance dial; D1 current; Q1/Q2 collector currents | `TP1` is a two-terminal controlled resistance: 1 GΩ dry/open at ratio 0 and 100 kΩ calibrated contact at ratio 1. The two NPNs follow the SVG's Darlington emitter-to-base cascade; the app control is an explicit physical-input substitute, not a scripted LED state. |
| C01-S02-04 | `c01-s02-04-water-sensor.json` | WP1 water-conductivity dial; BZ1 current; Q1 collector current | `WP1` is a two-terminal controlled resistance: 1 GΩ dry at ratio 0 and 1 kΩ wet endpoint at ratio 1. The cup and probes remain a battery-only manual prop. |
| C01-S02-05 | `c01-s02-05-two-transistor-flasher.json` | C1/C2 voltages; D1/D2 currents; Q1/Q2 collector currents | The SVG cross-coupled topology uses 47 kΩ feedback resistors, 47 µF electrolytics, and a 0.5 V initial C2 state to break symmetry. The bounded transient solve sustains alternating calculated LED transitions. |
| C01-S02-06 | `c01-s02-06-transistor-logic.json` | S1/S2; D1 AND, D2 OR, D3 NOT currents | The SVG topology is authoritative for the five NPN stages; buttons feed active-high A/B rails through calculated resistor networks, and the DC-only transistor Jacobian is used without changing the fixed transient oscillator path. |

The speaker entry remains blocked in the ledger until its output-load contract
is accepted. Touch-pad and
water-probe fixtures use explicit controlled resistance and still require the
battery-only manual protocol; no scripted output is used.
The core also now has a mirrored `pnp_transistor` model with the same named
pins and bounded beta/saturation parameters. It is model-tested as a high-side
load; the S02-07 fixture remains blocked until the speaker power/current bound
and oscillator startup are accepted.
