# Project format and breadboard contacts

Projects are JSON objects at `format_version: 1`. Generate the JSON Schema with
`bredboard-tools schema`; validate a file with `bredboard-tools validate FILE`.
The validator checks the generated Draft 2020-12 schema, references, catalog
parameters, limits, and derived connectivity. Connectivity is derived and is
never serialized as a second editable netlist.

The supported board is `half_size_solderless`: rows 1–30 each contain a left
five-hole strip A–E and a separate right five-hole strip F–J. Within one row,
holes in each five-hole strip are electrically common; rows are isolated. The
center gap separates E from F. Four power rails, `TP+`, `TP-`, `BP+`, and `BP-`,
are each continuous over rows 1–30. Rail holes use IDs such as `TP+:1` and
`BP-:30`. Wires connect only their `from` and `to` endpoints; crossing segments
have no connection unless they share an endpoint hole.

The shared app draws `TP+`/`TP-` as the left pair of rails and `BP+`/`BP-` as the right pair. The four rails remain independent unless a fixture explicitly connects them. The three built-in fixtures in `fixtures/projects/` use unique holes for every lead and wire endpoint; no two physical plugs occupy one hole.

Each component has a stable string `id`, a catalog `kind`, a map from pin name
to board hole, and a map of numeric parameters. Pin names are: voltage source
(`negative`, `positive`); resistor (`a`, `b`); LED (`anode`, `cathode`);
capacitor (`negative`, `positive`); NPN (`base`, `collector`, `emitter`);
momentary button (`a`, `b`); changeover switch (`common`, `normally_closed`,
`normally_open`). Parameters and inclusive ranges are:

| Kind | Parameter | Unit | Range |
| --- | --- | --- | --- |
| `dc_voltage_source` | `voltage` | V | 0 to 12 |
| `resistor` | `resistance` | ohm | 1 to 10,000,000 |
| `led` | `forward_voltage` | V | 0 to 10 |
| `led` | `series_resistance` | ohm | 1 to 10,000,000 |
| `capacitor` | `capacitance` | F | 1e-12 to 1000 |
| `npn_transistor` | `beta` | dimensionless | 1 to 1000 |
| `npn_transistor` | `saturation_current` | A | 1e-18 to 1 |

The current solver supports every listed catalog kind. It uses calculated
electrical models with the documented limits below; the presence of a model
does not mean it matches every physical component sold under that name.

The voltage and resistance limits above are intentionally sized for battery or
USB-powered hobby breadboards. They replace the unpublished draft limits; the
project format version is unchanged. Valid LED/resistor assemblies in these
ranges converge within the solver's fixed 80-iteration bound.

Projects may include `initial_conditions`. `capacitor_voltages` maps capacitor
IDs to initial volts and defaults to 0 V when an entry is absent. `controls`
maps button IDs to `button_pressed`/`button_released` and changeover switch IDs
to `switch_normally_closed`/`switch_normally_open`; omitted controls default to
released and normally-closed respectively. References and finite capacitor
voltages are validated before simulation.

Example authoring input (also stored as `fixtures/projects/valid-resistor.json`):

```json
{
  "format_version": 1,
  "title": "One resistor across the center gap",
  "board": { "model": "half_size_solderless" },
  "components": [{
    "id": "R1", "kind": "resistor",
    "pins": { "a": "E5", "b": "F5" },
    "parameters": { "resistance": 1000.0 }
  }],
  "wires": [
    { "id": "W1", "from": "TP+:1", "to": "E5" },
    { "id": "W2", "from": "F5", "to": "TP-:30" }
  ]
}
```

## Resistive DC solver (T03)

The headless solver supports all listed catalog kinds. A button is released
(open) by default; a changeover switch connects common to normally-closed by
default. Pass explicit control states to `solve_dc` to change them. Each
connected network needs at least one voltage source; otherwise it is diagnosed
as floating. Nonzero ideal voltage sources shorted by board connectivity,
contradictory source loops, and singular ideal-source arrangements fail
explicitly.

The solver uses `f64` MNA, deterministic component/node ordering, and partial
pivot Gaussian elimination. Ideal-source consistency uses a `1e-9 V` tolerance;
matrix pivots below `1e-12` are treated as singular. Resistors use the
documented 1–10,000,000 ohm range and voltage sources 0–12 V. The divider and
parallel reference circuits are in `fixtures/projects/`. Reported resistor
current is positive from pin A to pin B; source current is positive from its
positive pin to negative pin (so a delivering source has negative current).
Switch current is positive from its first pin to its selected pin.

## MVP LED and transistor approximations

The LED uses a smooth forward curve with voltage `V` from anode to cathode, configured `forward_voltage = Vf`, and `series_resistance = Rs`:

`I = 0.05/Rs * ln(1 + exp((V - Vf)/0.05)) + 1e-9*V` amperes. The implementation evaluates the exponential safely and includes a 1 nS leakage path so open and reverse-biased circuits remain solvable. This is a teaching approximation, not a fitted part datasheet. The UI maps 0–10 mA monotonically to LED brightness. The `led-bench.json` fixture gives 8.576 mA with B1 pressed, close to the simple `(5-2)/(330+20) = 8.57 mA` estimate; released current rounds to 0 mA.

The NPN uses a calculated base-emitter junction with effective threshold `0.026*ln(0.001/saturation_current)` volts and the same smooth diode form with 100 ohm slope. Collector-emitter conductance is `clamp(beta*max(base_current, 0)/0.2, 1 nS, 1 S)`; collector current is conductance times collector-emitter voltage. It is a base-controlled switching approximation, not Ebers–Moll or a prediction of exact transistor curves. In `transistor-bench.json`, pressing B1 gives 0.427 mA through the base-feed resistor, 8.461 mA through the LED and collector, and about 0.040 V collector-emitter. Released LED current is under 1 µA. Nonlinear solves are bounded to 80 iterations and report `nonconvergence` without advancing time if they fail.
