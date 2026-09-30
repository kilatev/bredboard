# Module model

`ComponentKind::Module` is the shared electrical contract for ready-made
catalog modules. It is intentionally pin-level: `Component::pins` maps named
module pins to board holes, and the existing topology compiler derives all
connectivity from those holes and wire endpoints. The core does not maintain a
module-specific netlist, access the filesystem, or depend on Bevy.

Every module declares exactly one `Supply` and one `Ground` role and selects a
bounded calculated behavior:

- `analog_transfer` provides finite input impedance and a bounded output from
  one or more sensed pins;
- `threshold_output` provides a finite-resistance high/low output selected by
  the solved input voltage;
- `open_collector` provides one or more input-controlled sinks, suitable for
  driver modules such as a multi-channel coil driver;
- `regulated_supply` provides a finite-resistance output target limited by the
  input voltage after dropout and by the configured target voltage.

All resistances are finite and validated. Outputs are stamped into the common
MNA/nonlinear solver on every DC or fixed-step transient solve, and calculated
output voltages are exposed in `SolveResult::module_output_voltages`. A module
does not encode a recorded audio clip, radio packet, distance result, motion
scene, or other scripted lesson outcome. Such source-specific behavior still
needs an explicit fixture contract and, where applicable, a separate physical
scope decision.

The model is suitable as the shared contract for the catalog's regulator,
stepper-driver, voice/sensor, ultrasonic, Li-ion charger, and solar-station
follow-up fixtures. Those fixtures are deliberately not part of this change.
