# C01 detailed readiness audit

Status: audited; Section 01 and the admitted Section 02 slice are implemented,
with manual acceptance and the remaining source/design decisions still open.

C01 covers source sections 01 (Level 1) and 02 (Transistors), all 13
schemes. Every source Markdown record, BOM, SVG path, and SVG label set was
reviewed. The 13 SVGs were rendered with `rsvg-convert` for a contact-sheet
review; the machine-readable labels were also checked against the BOMs.

Eleven C01 schemes now have validated project fixtures and automated solver
evidence. They are not `ready`: that disposition also requires the required
manual breadboard evidence. The remaining rows distinguish schemes blocked by
a missing physical prop, a source-design mismatch, or a runtime/design risk.

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
- Treat touch pads and water probes as physical-prop inputs, not scripted
  outcomes. The admitted fixture contract uses a two-terminal controlled
  resistance: ratio 0 is dry/open and ratio 1 is the documented contact or
  water-conductivity endpoint. The app exposes that ratio explicitly, while
  the physical assembly remains battery-only and requires the manual protocol.

## Per-scheme audit

| Audit key / source | Component mapping | Pin or connection ambiguities | Required electrical models | Sprite/art needs | Fixture risks | Manual assembly risks | Current disposition / next action |
|---|---|---|---|---|---|---|---|
| `C01-S01-01` — [01.svg](../../breadboard-circuits/spec/svg/01-level-1-first-current/01.svg), “Светодиод с резистором” | `dc_voltage_source ×1`; `resistor(470 Ω) ×1`; `led(red) ×1` | Supply polarity; LED anode/cathode; exact source voltage. | Existing source, resistor, and LED models; bounded overload diagnostic instead of a destructive “smoke” outcome. | Existing source/resistor/LED art is sufficient. | 9 V source label conflicts with the product's 5 V fixture convention; placement must make polarity readable. | Clip polarity, LED flat/long-lead orientation, and resistor lead placement. | `blocked_component` for fixture work; correct the destructive narrative and choose fixture voltage before admission. |
| `C01-S01-02` — [02.svg](../../breadboard-circuits/spec/svg/01-level-1-first-current/02.svg), “Кнопка и выключатель” | Source; momentary button; SPDT changeover switch; `resistor(470 Ω) ×2`; red and green LEDs. | 6×6 button side-pair orientation; SPDT common/NC/NO lugs; both LED polarities. | Existing source, resistor, LED, button, and changeover models. | Existing art covers all listed kinds. | Two independent control paths must have distinct stable IDs and no shared hole; source voltage still needs normalization. | Button straddles the board gap; SPDT lug order and rail continuity must be marked. | `blocked_component` only because no fixture exists; admit after the canonical placement pattern and source-voltage decision. |
| `C01-S01-03` — [03.svg](../../breadboard-circuits/spec/svg/01-level-1-first-current/03.svg), “Последовательно и параллельно” | Source; red LED `×3`; `resistor(220 Ω) ×1`; `resistor(470 Ω) ×3`. | LED polarity in the series branch and each parallel branch; SVG labels omit one resistor unit in the series annotation and need hole-level mapping. | Existing source, resistor, and LED models; current balance/readout must cover multiple branches. | Existing art is sufficient. | Series and parallel examples must be separate readable subassemblies; avoid a shared node that makes the lesson ambiguous. | Three parallel branches need one current-limiting resistor per branch; inspect rail breaks and duplicate lead holes. | `blocked_component` for fixture work; admit only after the branch topology is represented and checked. |
| `C01-S01-04` — [04.svg](../../breadboard-circuits/spec/svg/01-level-1-first-current/04.svg), “Диммер на потенциометре” | Source; `potentiometer(10 kΩ) ×1`; `resistor(330 Ω) ×1`; red LED. | The source must state which two pot terminals are used; do not imply a modeled wiper or voltage divider. | Existing potentiometer, source, resistor, and LED models; ratio action must be deterministic. | Existing potentiometer and other art is sufficient. | Current model is a two-terminal rheostat; the source drawing's control interpretation must match it. | Trimmer orientation and minimum-resistance setting can overdrive the LED; set a safe series floor. | `blocked_component` for fixture work; admit after two-terminal content contract and minimum current limit are written. |
| `C01-S01-05` — [05.svg](../../breadboard-circuits/spec/svg/01-level-1-first-current/05.svg), “Защита от переполюсовки” | Source; `diode(1N4007) ×1`; `resistor(470 Ω) ×1`; green LED. | Diode and LED polarity; source reversal behavior; whether the diode is series protection or a shunt path. | Calculated smooth two-pin diode with `anode/cathode`; existing source, resistor, and LED models. | Diode sprite has a visible cathode band. | Reversed source produces a bounded no-light diagnostic. | Diode band and LED cathode must be independently visible; check the battery clip before power. | `blocked_component` pending manual polarity and measurement evidence; fixture and model are implemented. |
| `C01-S01-06` — [06.svg](../../breadboard-circuits/spec/svg/01-level-1-first-current/06.svg), “Плавное угасание” | Source; button; `resistor(100 Ω) ×1`; `resistor(470 Ω) ×1`; electrolytic `470 µF ×1`; red LED. | Button orientation; capacitor positive/negative; charge and discharge paths; initial capacitor voltage. | Existing source, button, resistor, capacitor, and LED models; fixed-step transient behavior. | Existing art is sufficient. | `470 µF × 470 Ω ≈ 0.22 s` nominal time constant, inconsistent with “a couple seconds” unless the observable threshold is documented; fixture must set initial state. | Electrolytic stripe/polarity, button gap placement, and discharge path must be checked disconnected first. | `blocked_design`; correct the timing claim and document the initial condition before fixture admission. |
| `C01-S02-01` — [01.svg](../../breadboard-circuits/spec/svg/02-level-2-transistors/01.svg), “Транзисторный ключ” | Source; NPN `×1`; button; `resistor(10 kΩ) ×1`; `resistor(470 Ω) ×1`; red LED. | BC547 versus 2N3904 pinout; NPN emitter/base/collector; source text mentions a motor that is absent from the BOM. | Existing NPN, source, resistor, LED, and button models after one exact part is selected. | Existing NPN and other art is sufficient. | Do not expose an unlisted motor behavior; the base drive and LED load need a calculated fixture readout. | TO-92 flat-face orientation and C/B/E map must be printed in the assembly record. | `blocked_design`; reconcile text/BOM and select the exact NPN package before implementation. |
| `C01-S02-02` — [02.svg](../../breadboard-circuits/spec/svg/02-level-2-transistors/02.svg), “Сумеречный ночник” | Source; NPN; GL5528 photoresistor; `potentiometer(10 kΩ)`; `resistor(10 kΩ) ×1`; `resistor(470 Ω) ×1`; white LED. | NPN pinout; LDR ratio meaning (dark/light); pot two-terminal use; threshold polarity. | Existing NPN, photoresistor, potentiometer, source, resistor, and LED models. | Existing art is sufficient. | Darkness/light control must map monotonically to the documented behavior; source 9 V label still needs normalization. | LDR placement and pot setting must be recorded; LED polarity and transistor package orientation. | `blocked_component` for fixture work; admit after control/readout ranges and pin map are fixed. |
| `C01-S02-03` — [03.svg](../../breadboard-circuits/spec/svg/02-level-2-transistors/03.svg), “Сенсорная кнопка” | Source; NPN `×2`; `resistor(1 kΩ) ×1`; `resistor(470 Ω) ×1`; red LED; two contact pads. | NPN pinouts; contact-pad polarity and resistance are undefined in the source. | New calculated `touch_pad` controlled resistance plus existing NPN/source/resistor/LED models; the two NPNs use the SVG's Darlington emitter-to-base cascade. | New gold contact-pad sprite; existing electrical art is sufficient. | The product surface exposes the contact resistance explicitly; the fixture uses a calibrated 100 kΩ contact endpoint and 1 GΩ dry endpoint. | Battery-only operation, current limit, clean pads, and a reproducible touch procedure; no USB connection. | `fixture_ready`; `TP1` is an explicit physical-prop control and still requires manual safety evidence. |
| `C01-S02-04` — [04.svg](../../breadboard-circuits/spec/svg/02-level-2-transistors/04.svg), “Датчик воды” | Source; NPN; `resistor(1 kΩ) ×1`; active buzzer; two probe wires; water cup. | Probe spacing, water conductivity, and buzzer polarity/load are unspecified in the source. | New calculated `water_probe` controlled resistance plus existing NPN/source/resistor/buzzer models; no hidden conductivity script. | New blue probe sprite; cup and water remain manual props. | The product surface exposes probe conductivity explicitly; the fixture uses 1 kΩ wet and 1 GΩ dry endpoints. | Battery-only, spill protection, dry-board handling, probe depth, and current limit remain mandatory. | `fixture_ready`; `WP1` is an explicit physical-prop control and still requires manual safety evidence. |
| `C01-S02-05` — [05.svg](../../breadboard-circuits/spec/svg/02-level-2-transistors/05.svg), “Мигалка на двух транзисторах” | Source; NPN `×2`; `resistor(47 kΩ) ×2`; `resistor(470 Ω) ×2`; electrolytic `47 µF ×2`; red LED `×2`. | Both NPN pinouts and both capacitor polarities; startup symmetry and initial capacitor state. | Existing NPN, capacitor, resistor, LED, and source models; deterministic transient initialization is required. | Existing art is sufficient. | The admitted fixture uses the source's 47 kΩ/47 µF values, 5 V normalized supply, and 0.5 V C2 startup offset; the solver's bounded 200-iteration nonlinear solve sustains calculated polarity transitions. | Match each capacitor stripe to the source drawing; verify both transistor flat faces and LED polarities without power. | `fixture_ready`; structural validation and a 60,000-step alternating LED regression passed; manual battery-only evidence remains required. |
| `C01-S02-06` — [06.svg](../../breadboard-circuits/spec/svg/02-level-2-transistors/06.svg), “Логика на кнопках и транзисторах” | Source; NPN `×3` in the catalog BOM, but five NPN symbols appear in the SVG; buttons `×2`; `resistor(10 kΩ) ×6`; `resistor(470 Ω) ×3`; green LED `×3`. | NPN pinouts; A/B button mapping; exact AND/OR/NOT node mapping and LED active level; source SVG/BOM transistor-count mismatch is retained. | Existing NPN, button, resistor, LED, and source models; DC transistor Jacobian is used for the static truth table while transient behavior remains unchanged. | Existing art is sufficient. | Three subcircuits use separate hole-level nets and active-high button rails; all four combinations have calculated readouts. | Button gap orientation, pull-up/down resistor placement, and transistor pin labels must be explicit. | `blocked_component` pending manual evidence; fixture and calculated truth-table regression are implemented using the SVG's five-NPN topology. |
| `C01-S02-07` — [07.svg](../../breadboard-circuits/spec/svg/02-level-2-transistors/07.svg), “Простая сирена” | Source; NPN `×1`; PNP `×1`; `resistor(10 kΩ) ×1`; `potentiometer(100 kΩ) ×1`; ceramic `47 nF ×1`; speaker `8 Ω / 0.5 W ×1`. | PNP E/B/C pinout and model; NPN/PNP bias; speaker polarity and allowable drive; pot two-terminal use. | Calculated PNP model now exists with the same named pins and bounded parameters; speaker-load and oscillator-startup contracts remain open. | PNP reuses the approved TO-92 art; existing speaker art is sufficient. | A 9 V source and 8 Ω load can exceed 0.5 W without a checked amplitude/current bound; oscillator startup and pot range need limits. | Exact transistor package orientation, speaker protection, pot minimum setting, and no prolonged overload. | `blocked_component`; a source-topology candidate converged but settled at 0.556 A / 2.47 W into 8 Ω and did not sustain a calculated oscillation, so no fixture was retained. |

## C01 readiness summary

| Group | Schemes | Gate | Evidence state |
|---|---|---|---|
| Implemented fixture slice, manual acceptance pending | S01-01 through S01-06, S02-01 through S02-06 | Fixture validation, calculated readouts, manual continuity/polarity/control evidence | Ten fixtures and menu entries exist; none is release-ready without manual evidence. |
| Source/design or numerical correction required | S02-07 | Speaker bound and startup contract | Blocked. |
| Implemented physical-prop fixture slice | S02-03, S02-04 | Touch/water input contracts are explicit; battery-only safety and reproducible manual procedure remain | Fixture and calculated control tests passed; manual evidence pending. |

## C01 entry criteria

Before C01 can be marked ready, the card must have accepted evidence for:

1. the shared assembly protocol and reference board/rail convention;
2. exact supply voltage and current limits for every admitted fixture;
3. canonical pin maps for LED, diode, NPN, PNP, button, SPDT, capacitor,
   potentiometer, photoresistor, buzzer, and speaker;
4. the explicit touch-pad and water-probe resistance contract, followed by
   battery-only manual evidence;
5. a corrected timing/content decision for S01-06 and a corrected BOM/text
   decision for S02-01;
6. the diode and PNP model task dependencies, including property/regression
   checks and sprite requirements.

The implementation evidence is recorded in the C01 task card and catalog
ledger. Manual acceptance and the unresolved S02 source/design decisions
remain open.
