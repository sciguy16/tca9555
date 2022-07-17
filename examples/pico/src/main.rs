// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, you can obtain one at https://mozilla.org/MPL/2.0/.

#![cfg_attr(not(test), no_std)]
#![no_main]

use defmt::info;
use defmt_rtt as _;
use embedded_hal::digital::v2::ToggleableOutputPin;
use embedded_time::duration::*;
use embedded_time::rate::Extensions;
use panic_halt as _;
use rp_pico::entry;
use rp_pico::hal;
use rp_pico::hal::pac;
use rp_pico::hal::Clock;
use tca9555::Tca9555;

#[entry]
fn main() -> ! {
    info!("Start boot");

    // Grab our singleton objects
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();

    // Set up the watchdog driver - needed by the clock setup code
    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);

    // Configure the clocks
    //
    // The default is to generate a 125 MHz system clock
    let clocks = hal::clocks::init_clocks_and_plls(
        rp_pico::XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    // The single-cycle I/O block controls our GPIO pins
    let sio = hal::Sio::new(pac.SIO);

    let pins = rp_pico::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    info!("Init I2C");

    let mut delay = cortex_m::delay::Delay::new(
        core.SYST,
        clocks.system_clock.freq().integer(),
    );

    let mut led = pins.led.into_push_pull_output();

    let sda_pin = pins.gpio4.into_mode::<hal::gpio::FunctionI2C>();
    let scl_pin = pins.gpio5.into_mode::<hal::gpio::FunctionI2C>();

    let i2c = hal::I2C::i2c0(
        pac.I2C0,
        sda_pin,
        scl_pin,
        400.kHz(),
        &mut pac.RESETS,
        clocks.peripheral_clock,
    );

    let mut tca = Tca9555::new(i2c, tca9555::DeviceAddr::default());

    loop {
        let port0 = tca.read_all().unwrap();
        info!("port 0 is: {:04x}", port0);
        led.toggle().unwrap();

        delay.delay_ms(500);
    }
}
