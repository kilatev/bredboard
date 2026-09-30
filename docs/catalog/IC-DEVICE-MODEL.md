# Shared IC/device model contract

Status: implemented in `bredboard-core`; no catalog fixture is admitted by
this contract alone.

`ComponentKind::IcDevice` represents a bounded pin-level IC or module. The
component's `pins` map remains the only physical connectivity source: it maps
named device pins to board holes, and wires/board contacts derive the compiled
nodes. The optional `Component::ic_device` value supplies roles and the
calculated transfer law; it does not contain node IDs or a serialized netlist.

Every device declares exactly one `supply` and one `ground` pin, names each
referenced input/output pin, and stays within the 32-pin/8-input limits. The
core validates finite values, output bounds, and finite input/output
resistances before a project is accepted.

The shared transfer laws are:

- `linear`: a finite-input-impedance, finite-output-resistance transfer with
  bounded gain and offset, suitable for educational analog IC/module blocks;
- `comparator`: a voltage-difference threshold with bounded high/low output;
- `logic`: a calculated multi-input Boolean operation with a supply-midpoint
  threshold and finite output resistance.

The solver evaluates these laws from the voltages on the derived topology at
each fixed step and exposes calculated output-pin voltages in
`SolveResult::ic_device_output_voltages`. No catalog ID, fixture name, wall
clock, renderer state, or scripted result participates in the calculation.

The contract closes only the shared electrical IC/device model gap. Individual
catalog entries may still require a source-specific timing/state model, an
optical/audio/sensor/actuator module, safety review, physical prop, layout, or
fixture/menu work; those remain in the ledger disposition and notes.
