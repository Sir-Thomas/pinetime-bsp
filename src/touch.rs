//! Touch controller module

use cst816s::{CST816S, TouchEvent};
use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;
use embassy_nrf::{gpio::{Input, Output}, twim::Twim};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_time::Delay;

/// Touch Controller
pub struct TouchController {
    controller: CST816S<I2cDevice<'static, NoopRawMutex, Twim<'static>>, Input<'static>, Output<'static>>
}

impl TouchController {
    /// Create a new TouchController
    pub async fn new(
        i2c_bus: I2cDevice<'static, NoopRawMutex, Twim<'static>>,
        interrupt_pin: Input<'static>,
        reset_pin: Output<'static>,
    ) -> Self {
        let mut controller = CST816S::new(
            i2c_bus,
            interrupt_pin,
            reset_pin
        );
        controller.init(Delay).await.unwrap();
        Self {
            controller,
        }
    }

    /// Wait for a touch event
    pub async fn wait_for_touch(&mut self) -> TouchEvent {
        self.controller.wait_for_touch().await.unwrap()
    }
}