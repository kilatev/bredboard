# C01 detailed readiness audit

Status: audited; implementation not started.

C01 covers source sections 01 (Level 1) and 02 (Transistors), all 13
schemes. Every source Markdown record, BOM, SVG path, and SVG label set was
reviewed. The 13 SVGs were rendered with `rsvg-convert` for a contact-sheet
review; the machine-readable labels were also checked against the BOMs.

No C01 scheme is `ready`: that disposition requires a validated project
fixture, automated evidence, and the required manual breadboard evidence. The
current rows distinguish schemes that can enter fixture work after the shared
contract is accepted from schemes blocked by a missing model, a physical prop,
or a source-design decision.

## Shared C01 decisions

- Normalize the fixture supply to the product's documented 5 V external source
  unless an owner-approved 9 V fixture contract is recorded. The source BOMs
  say 9 V, while the product board presentation currently describes 5 V.
- Select one exact transistor part and package for each fixture. “BC547 (or
  2N3904)” is not a pin contract: the package orientation and C/B/E or E/B/C
  mapping must be explicit.
- Keep the current two-terminal rheostat contract for the potentiometer and
  photoresistor. A third potentiometer wiper is not a hidden connection; the
  fixture must document the two used terminals.
- Preserve source polarity in the fixture: LED anode/cathode, diode
  anode/cathode, electrolytic positive/negative, and supply positive/negative.
- Use stable component IDs and hole IDs; derive all connectivity from board
  contacts, pins, and wires. No schematic label becomes a second netlist.
- Treat touch pads and water probes as physical-prop decisions, not as
  scripted electrical outcomes. If admitted, define a safe controlled input
  and record it as manual evidence.

## Per-scheme audit

| Audit key / source | Component mapping | Pin or connection ambiguities | Required electrical models | Sprite/art needs | Fixture risks | Manual assembly risks | Current disposition / next action |
|---|---|---|---|---|---|---|---|
| `C01-S01-01` — [01.svg](../../breadboard-circuits/spec/svg/01-level-1-first-current/01.svg), “Светодиод с резистором” | `dc_voltage_source ×1`; `resistor(470 Ω) ×1`; `led(red) ×1` | Supply polarity; LED anode/cathode; exact source voltage. | Existing source, resistor, and LED models; bounded overload diagnostic instead of a destructive “smoke” outcome. | Existing source/resistor/LED art is sufficient. | 9 V source label conflicts with the product's 5 V fixture convention; placement must make polarity readable. | Clip polarity, LED flat/long-lead orientation, and resistor lead placement. | `blocked_component` for fixture work; correct the destructive narrative and choose fixture voltage before admission. |
| `C01-S01-02` — [02.svg](../../breadboard-circuits/spec/svg/01-level-1-first-current/02.svg), “Кнопка и выключатель” | Source; momentary button; SPDT changeover switch; `resistor(470 Ω) ×2`; red and green LEDs. | 6×6 button side-pair orientation; SPDT common/NC/NO lugs; both LED polarities. | Existing source, resistor, LED, button, and changeover models. | Existing art covers all listed kinds. | Two independent control paths must have distinct stable IDs and no shared hole; source voltage still needs normalization. | Button straddles the board gap; SPDT lug order and rail continuity must be marked. | `blocked_component` only because no fixture exists; admit after the canonical placement pattern and source-voltage decision. |
| `C01-S01-03` — [03.svg](../../breadboard-circuits/spec/svg/01-level-1-first-current/03.svg), “Последовательно и параллельно” | Source; red LED `×3`; `resistor(220 Ω) ×1`; `resistor(470 Ω) ×3`. | LED polarity in the series branch and each parallel branch; SVG labels omit one resistor unit in the series annotation and need hole-level mapping. | Existing source, resistor, and LED models; current balance/readout must cover multiple branches. | Existing art is sufficient. | Series and parallel examples must be separate readable subassemblies; avoid a shared node that makes the lesson ambiguous. | Three parallel branches need one current-limiting resistor per branch; inspect rail breaks and duplicate lead holes. | `blocked_component` for fixture work; admit only after the branch topology is represented and checked. |
| `C01-S01-04` — [04.svg](../../breadboard-circuits/spec/svg/01-level-1-first-current/04.svg), “Диммер на потенциометре” | Source; `potentiometer(10 kΩ) ×1`; `resistor(330 Ω) ×1`; red LED. | The source must state which two pot terminals are used; do not imply a modeled wiper or voltage divider. | Existing potentiometer, source, resistor, and LED models; ratio action must be deterministic. | Existing potentiometer and other art is sufficient. | Current model is a two-terminal rheostat; the source drawing's control interpretation must match it. | Trimmer orientation and minimum-resistance setting can overdrive the LED; set a safe series floor. | `blocked_component` for fixture work; admit after two-terminal content contract and minimum current limit are written. |
| `C01-S01-05` — [05.svg](../../breadboard-circuits/spec/svg/01-level-1-first-current/05.svg), “Защита от переполюсовки” | Source; `diode(1N4007) ×1`; `resistor(470 Ω) ×1`; green LED. | Diode and LED polarity; source reversal behavior; whether the diode is series protection or a shunt path. | New rectifier-diode model and pin contract; existing source, resistor, and LED models. | New diode sprite/art is required with a visible cathode band. | Reversed source must produce a bounded no-light diagnostic, not an invalid or destructive state. | Diode band and LED cathode must be independently visible; check the battery clip before power. | `blocked_component`; create the diode model/art contract before C01 fixture work. |
| `C01-S01-06` — [06.svg](../../breadboard-circuits/spec/svg/01-level-1-first-current/06.svg), “Плавное угасание” | Source; button; `resistor(100 Ω) ×1`; `resistor(470 Ω) ×1`; electrolytic `470 µF ×1`; red LED. | Button orientation; capacitor positive/negative; charge and discharge paths; initial capacitor voltage. | Existing source, button, resistor, capacitor, and LED models; fixed-step transient behavior. | Existing art is sufficient. | `470 µF × 470 Ω ≈ 0.22 s` nominal time constant, inconsistent with “a couple seconds” unless the observable threshold is documented; fixture must set initial state. | Electrolytic stripe/polarity, button gap placement, and discharge path must be checked disconnected first. | `blocked_design`; correct the timing claim and document the initial condition before fixture admission. |
| `C01-S02-01` — [01.svg](../../breadboard-circuits/spec/svg/02-level-2-transistors/01.svg), “Транзисторный ключ” | Source; NPN `×1`; button; `resistor(10 kΩ) ×1`; `resistor(470 Ω) ×1`; red LED. | BC547 versus 2N3904 pinout; NPN emitter/base/collector; source text mentions a motor that is absent from the BOM. | Existing NPN, source, resistor, LED, and button models after one exact part is selected. | Existing NPN and other art is sufficient. | Do not expose an unlisted motor behavior; the base drive and LED load need a calculated fixture readout. | TO-92 flat-face orientation and C/B/E map must be printed in the assembly record. | `blocked_design`; reconcile text/BOM and select the exact NPN package before implementation. |
| `C01-S02-02` — [02.svg](../../breadboard-circuits/spec/svg/02-level-2-transistors/02.svg), “Сумеречный ночник” | Source; NPN; GL5528 photoresistor; `potentiometer(10 kΩ)`; `resistor(10 kΩ) ×1`; `resistor(470 Ω) ×1`; white LED. | NPN pinout; LDR ratio meaning (dark/light); pot two-terminal use; threshold polarity. | Existing NPN, photoresistor, potentiometer, source, resistor, and LED models. | Existing art is sufficient. | Darkness/light control must map monotonically to the documented behavior; source 9 V label still needs normalization. | LDR placement and pot setting must be recorded; LED polarity and transistor package orientation. | `blocked_component` for fixture work; admit after control/readout ranges and pin map are fixed. |
| `C01-S02-03` — [03.svg](../../breadboard-circuits/spec/svg/02-level-2-transistors/03.svg), “Сенсорная кнопка” | Source; NPN `×2`; `resistor(1 kΩ) ×1`; `resistor(470 Ω) ×1`; red LED; two contact pads. | NPN pinouts; contact-pad polarity and resistance are undefined; touch is not a current component contract. | Existing NPN/source/resistor/LED models plus a defined physical touch input or controlled resistance; no scripted outcome. | Existing art covers electrical parts; contact-pad art/prop is unspecified. | Must decide whether physical touch is in current product scope and how the input maps to a deterministic control. | Battery-only operation, current limit, clean pads, and a reproducible touch procedure; no USB connection. | `physical_scope`; define and approve a safe physical-prop contract before any fixture. |
| `C01-S02-04` — [04.svg](../../breadboard-circuits/spec/svg/02-level-2-transistors/04.svg), “Датчик воды” | Source; NPN; `resistor(1 kΩ) ×1`; active buzzer; two probe wires; water cup. | Probe spacing, water conductivity, and buzzer polarity/load are unspecified. | Existing NPN/source/resistor/buzzer models plus a controlled water/probe input; no hidden conductivity script. | Existing electrical art; cup/probe prop is unspecified. | Water sensing is off-board and not represented by the current board model; isolate the prop and define expected resistance range. | Battery-only, spill protection, dry-board handling, probe depth, and current limit are mandatory. | `physical_scope`; scope as a prop experiment or explicitly exclude before fixture work. |
| `C01-S02-05` — [05.svg](../../breadboard-circuits/spec/svg/02-level-2-transistors/05.svg), “Мигалка на двух транзисторах” | Source; NPN `×2`; `resistor(47 kΩ) ×2`; `resistor(470 Ω) ×2`; electrolytic `47 µF ×2`; red LED `×2`. | Both NPN pinouts and both capacitor polarities; startup symmetry and initial capacitor state. | Existing NPN, capacitor, resistor, LED, and source models; deterministic transient initialization is required. | Existing art is sufficient. | Cross-coupled oscillator must start deterministically and stop with a bounded diagnostic; placement must keep feedback wires legible. | Match each capacitor stripe to the source drawing; verify both transistor flat faces and LED polarities without power. | `blocked_component` for fixture work; admit after initial-state and oscillator-range evidence is defined. |
| `C01-S02-06` — [06.svg](../../breadboard-circuits/spec/svg/02-level-2-transistors/06.svg), “Логика на кнопках и транзисторах” | Source; NPN `×3`; buttons `×2`; `resistor(10 kΩ) ×6`; `resistor(470 Ω) ×3`; green LED `×3`. | NPN pinouts; A/B button mapping; exact AND/OR/NOT node mapping and LED active level. | Existing NPN, button, resistor, LED, and source models; deterministic control ordering and truth-table readout. | Existing art is sufficient. | Three subcircuits need readable grouping and no cross-branch hole reuse; all inputs need defined released states. | Button gap orientation, pull-up/down resistor placement, and transistor pin labels must be explicit. | `blocked_component` for fixture work; admit after topology, truth-table, and placement evidence. |
| `C01-S02-07` — [07.svg](../../breadboard-circuits/spec/svg/02-level-2-transistors/07.svg), “Простая сирена” | Source; NPN `×1`; PNP `×1`; `resistor(10 kΩ) ×1`; `potentiometer(100 kΩ) ×1`; ceramic `47 nF ×1`; speaker `8 Ω / 0.5 W ×1`. | PNP E/B/C pinout and model; NPN/PNP bias; speaker polarity and allowable drive; pot two-terminal use. | New PNP model and pin contract; existing source, NPN, potentiometer, capacitor, and speaker models; bounded audio-load behavior. | New PNP sprite can reuse the approved TO-92 shape only after a runtime kind exists; existing speaker art is sufficient. | A 9 V source and 8 Ω load can exceed 0.5 W without a checked amplitude/current bound; oscillator startup and pot range need limits. | Exact transistor package orientation, speaker protection, pot minimum setting, and no prolonged overload. | `blocked_component`; define PNP model/pins and speaker current/power bounds before implementation. |

## C01 readiness summary

| Group | Schemes | Gate | Evidence state |
|---|---|---|---|
| Existing electrical kinds, fixture work pending | S01-01 through S01-04, S02-02, S02-05, S02-06 | 5 V/9 V decision, canonical pins, hole-level fixtures, solver/readout checks | Not started; no scheme is ready. |
| Source-design correction required | S01-06, S02-01 | Correct timing claim; reconcile missing motor and exact transistor choice | Blocked. |
| New component model/art required | S01-05, S02-07 | Diode and PNP contracts, sprites, model tests, safe bounds | Blocked. |
| Physical-prop scope required | S02-03, S02-04 | Touch/water input, battery-only safety, reproducible manual procedure | Blocked. |

## C01 entry criteria

Before implementation begins, the C01 card must have accepted evidence for:

1. the shared assembly protocol and reference board/rail convention;
2. exact supply voltage and current limits for every admitted fixture;
3. canonical pin maps for LED, diode, NPN, PNP, button, SPDT, capacitor,
   potentiometer, photoresistor, buzzer, and speaker;
4. a decision for the touch and water entries;
5. a corrected timing/content decision for S01-06 and a corrected BOM/text
   decision for S02-01;
6. the diode and PNP model task dependencies, including property/regression
   checks and sprite requirements.

No C01 implementation, menu registration, solver change, sprite, or UI change
was made by this audit.
