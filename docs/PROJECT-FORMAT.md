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

Each component has a stable string `id`, a catalog `kind`, a map from pin name
to board hole, and a map of numeric parameters. Pin names are: voltage source
(`negative`, `positive`); resistor (`a`, `b`); LED (`anode`, `cathode`);
capacitor (`negative`, `positive`); NPN (`base`, `collector`, `emitter`);
momentary button (`a`, `b`); changeover switch (`common`, `normally_closed`,
`normally_open`). Parameters and inclusive ranges are:

| Kind | Parameter | Unit | Range |
| --- | --- | --- | --- |
| `dc_voltage_source` | `voltage` | V | -1000 to 1000 |
| `resistor` | `resistance` | ohm | 0.001 to 1,000,000,000 |
| `led` | `forward_voltage` | V | 0 to 10 |
| `led` | `series_resistance` | ohm | 0.001 to 1,000,000,000 |
| `capacitor` | `capacitance` | F | 1e-12 to 1000 |
| `npn_transistor` | `beta` | dimensionless | 1 to 1000 |
| `npn_transistor` | `saturation_current` | A | 1e-18 to 1 |

The catalog is metadata only at this milestone; catalog presence does not imply
that a model can be simulated. Button and switch initial states and solver
models are introduced by later tasks.

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
