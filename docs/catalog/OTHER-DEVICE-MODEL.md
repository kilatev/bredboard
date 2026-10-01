# Other-device electrical contract

`ComponentKind::Other` is the shared core contract for a catalog part whose
electrical behavior is understood but does not yet justify a dedicated
component kind. It is not a fixture result table and it does not create a
second netlist.

Every instance declares named pins and their roles in `other_device`. The
instance's `Component::pins` map still connects those pins to board holes;
topology is compiled from holes, pins, and wire endpoints exactly as for every
other component. A behavior may only refer to pins declared by the contract.

The supported calculated behaviors are:

- `resistive`: a two-terminal finite impedance;
- `voltage_source`: a bounded Thevenin source with non-zero internal
  resistance;
- `linear_transfer`: one finite-resistance output driven by named input
  voltages relative to a reference, with explicit gain, offset, and output
  bounds; and
- `transformer`: a four-terminal finite-resistance primary/secondary transfer
  with an explicit turns ratio; and
- `ring_modulator`: a bounded multiplicative transfer from named signal and
  carrier inputs to an output relative to a reference; and
- `voltage_controlled_resistance`: an electrically isolated two-terminal
  output whose resistance is interpolated from a named control voltage and
  clamped to its declared range.

The `bjt_test_socket` behavior is the explicit device-under-test contract for
three-pin transistor tester fixtures. It declares base, collector, and emitter
pins, a socket polarity, a fixture-selected subject polarity, and a subject
state (`working`, `open`, or `shorted`). A polarity mismatch is electrically
treated as an open subject. A matching working subject reuses the calculated
NPN/PNP model (`beta` and `saturation_current`); failure states use bounded
three-terminal resistive paths. This is a fixture-time test-state mapping, not
a scripted LED result or a runtime swappable-part editor.

The transformer contract is intentionally a bounded voltage-transfer
approximation: it includes finite primary and secondary loading but does not
model magnetic flux, inductance, phase, or saturation. The ring-modulator
contract calculates the product of normalized fixed-step electrical signal and
carrier voltages; it does not record or synthesize a microphone waveform by
wall-clock time. These contracts cover the shared electrical boundary needed
by the catalog's
remaining connector, source, sensor, optical, audio, actuator-load, and module
records. They do not claim to model a physical prop, audio waveform, optical
alignment, motor mechanics, charging protocol, thermal inertia, or firmware.
Those remain fixture-specific gaps and must stay in the ledger until a later
task defines them. A fixture must choose explicit pin roles and numerical
parameters from its source review; it must not use `Other` to hide an
unsupported or scripted outcome.

`SolveResult::other_terminal_currents` reports calculated current into each
declared pin. Linear-transfer output voltages are reported separately in
`SolveResult::other_output_voltages`. All values are derived from the solved
node voltages on fixed solver steps and are deterministic for the same project
and controls.
