# C01-S01 fixture contract

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
