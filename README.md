# tca9555

Embedded-HAL driver crate for the TCA9555/TCA9535 16 port I/O expander.

This crate currently only implements byte-oriented operations - pin
operations will be part of a future update.

Read operations have been tested. Write operations have been implemented
but not tested.

## Examples
There is an example application provided for the Raspberry Pi Pico,
developed for the Pimoroni "PIM551" keypad module.

```rust
let mut tca = Tca9555::new(i2c, tca9555::DeviceAddr::default());
let inputs: u16 = tca.read_all().unwrap();
```

# License
This crate is distributed under the terms of the Mozilla Public License
Version 2.0.
