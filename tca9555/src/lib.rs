// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, you can obtain one at https://mozilla.org/MPL/2.0/.

#![cfg_attr(not(test), no_std)]
#![deny(missing_docs)]

//! Driver for the TCA9555/TCA9535 16-port I/O expander.
//!
//! This crate currently only has byte-oriented operation implemented.
//! Individual control of pins will be the subject of a future release.
//!
//! ## Example
//! ```no_run
//! use embedded_hal::blocking::i2c::WriteRead;
//! use tca9555::{Tca9555, DeviceAddr};
//! fn read_ports<E>(i2c: impl WriteRead<Error = E>) -> Result<(), E> {
//!     let mut tca = Tca9555::new(i2c, DeviceAddr::default());
//!     let port0: u8 = tca.read_port_0()?;
//!     let port1: u8 = tca.read_port_1()?;
//!     let all_inputs: u16 = tca.read_all()?;
//!     Ok(())
//! }
//! ```

use embedded_hal::blocking::i2c::{Write, WriteRead};

mod command {
    pub const READ_PORT_0: u8 = 0x00;
    pub const READ_PORT_1: u8 = 0x01;
    pub const WRITE_PORT_0: u8 = 0x02;
    pub const WRITE_PORT_1: u8 = 0x03;
    pub const POLARITY_INVERT_PORT_0: u8 = 0x04;
    pub const POLARITY_INVERT_PORT_1: u8 = 0x05;
    pub const CONFIGURATION_PORT_0: u8 = 0x06;
    pub const CONFIGURATION_PORT_1: u8 = 0x07;
}

use command::*;

/// Represents the address of a connected TCA9555
#[derive(Copy, Clone, Debug)]
pub enum DeviceAddr {
    /// Default address when all address pins are connected to GND (0x20)
    Default,
    /// Set an alternative address with the values of the (A0, A1, A2)
    /// pins
    Alternative(bool, bool, bool),
}

impl Default for DeviceAddr {
    fn default() -> Self {
        Self::Default
    }
}

impl DeviceAddr {
    const DEFAULT: u8 = 0x20;

    /// Get the raw address
    pub fn addr(self) -> u8 {
        match self {
            DeviceAddr::Default => Self::DEFAULT,
            DeviceAddr::Alternative(a0, a1, a2) => {
                Self::DEFAULT | a0 as u8 | ((a1 as u8) << 1) | ((a2 as u8) << 2)
            }
        }
    }
}

/// Type alias for TCA9555. Both chips implement the same I2C commands
/// so in theory are interchangeable here. Use with a TCA9535 has not
/// been tested.
pub type Tca9535<I2C> = Tca9555<I2C>;

/// TCA9555 device
pub struct Tca9555<I2C> {
    address: DeviceAddr,
    i2c: I2C,
}

impl<I2C> Tca9555<I2C> {
    /// Create a TCA9555 device with the given address
    pub fn new(i2c: I2C, address: DeviceAddr) -> Self {
        Self { i2c, address }
    }
}

impl<I2C, E> Tca9555<I2C>
where
    I2C: WriteRead<Error = E>,
{
    /// Read input port 0 in full, returning a u8. Reads the current logic
    /// level of the pins, regardless of whether they have been configured
    /// as inputs or outputs
    pub fn read_port_0(&mut self) -> Result<u8, E> {
        let mut value = [0];
        self.i2c
            .write_read(self.address.addr(), &[READ_PORT_0], &mut value)
            .and(Ok(value[0]))
    }

    /// Read input port 1 in full, returning a u8. Reads the current logic
    /// level of the pins, regardless of whether they have been configured
    /// as inputs or outputs
    pub fn read_port_1(&mut self) -> Result<u8, E> {
        let mut value = [0];
        self.i2c
            .write_read(self.address.addr(), &[READ_PORT_1], &mut value)
            .and(Ok(value[0]))
    }

    /// Read both input ports in turn, combining their values into a u16.
    /// Reads the current logic level of the pins, regardless of whether
    /// they have been configured as inputs or outputs
    pub fn read_all(&mut self) -> Result<u16, E> {
        let port0 = self.read_port_0()?;
        let port1 = self.read_port_1()?;
        Ok(u16::from_be_bytes([port1, port0]))
    }
}

impl<I2C, E> Tca9555<I2C>
where
    I2C: Write<Error = E>,
{
    /// Write the given byte to port 0. Has no effect on pins which have
    /// been configured as inputs.
    pub fn write_port_0(&mut self, value: u8) -> Result<(), E> {
        self.i2c.write(self.address.addr(), &[WRITE_PORT_0, value])
    }

    /// Set the port 0 direction register. Bits set to 0 are in output
    /// mode, while bits set to 1 are in input mode.
    pub fn set_port_0_direction(&mut self, dir_mask: u8) -> Result<(), E> {
        self.i2c
            .write(self.address.addr(), &[CONFIGURATION_PORT_0, dir_mask])
    }

    /// Set the port 0 polarity inversion register. Bits set to 1 have
    /// their polarity inverted
    pub fn set_port_0_polarity_invert(
        &mut self,
        polarity_mask: u8,
    ) -> Result<(), E> {
        self.i2c.write(
            self.address.addr(),
            &[POLARITY_INVERT_PORT_0, polarity_mask],
        )
    }

    /// Write the given byte to port 1. Has no effect on pins which have
    /// been configured as inputs.
    pub fn write_port_1(&mut self, value: u8) -> Result<(), E> {
        self.i2c.write(self.address.addr(), &[WRITE_PORT_1, value])
    }

    /// Set the port 1 direction register. Bits set to 0 are in output
    /// mode, while bits set to 1 are in input mode.
    pub fn set_port_1_direction(&mut self, dir_mask: u8) -> Result<(), E> {
        self.i2c
            .write(self.address.addr(), &[CONFIGURATION_PORT_1, dir_mask])
    }

    /// Set the port 1 polarity inversion register. Bits set to 1 have
    /// their polarity inverted
    pub fn set_port_1_polarity_invert(
        &mut self,
        polarity_mask: u8,
    ) -> Result<(), E> {
        self.i2c.write(
            self.address.addr(),
            &[POLARITY_INVERT_PORT_1, polarity_mask],
        )
    }

    /// Write the given u16 across all 16 output pins
    pub fn write_all(&mut self, value: u16) -> Result<(), E> {
        let [port1, port0] = value.to_be_bytes();
        self.write_port_0(port0)?;
        self.write_port_1(port1)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn address_set() {
        assert_eq!(DeviceAddr::DEFAULT, 0x20);
        assert_eq!(DeviceAddr::Alternative(false, false, false).addr(), 0x20);
        assert_eq!(DeviceAddr::Alternative(true, false, false).addr(), 0x21);
        assert_eq!(DeviceAddr::Alternative(false, true, true).addr(), 0x26);
    }
}
