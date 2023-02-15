# About

`pallet-cairo` is a pallet that provides a way to execute Cairo programs in a Substrate environment.

# Configuration

The pallet is configurable through the `Config` trait. The following associated types are defined:

- `RuntimeEvent`: Because this pallet emits events, it depends on the runtime's definition of an event.
- `Randomness`: The pallet uses a random number generator to generate random numbers. This trait provides the source of randomness.
- `MaxSierraProgramLength`: The maximum length of a Sierra program in bytes.
- `MaxCairoAssemblyProgramLength`: The maximum length of a Cairo assembly program in bytes.

# License

This project is licensed under the **MIT license**.

See [LICENSE](LICENSE) for more information.