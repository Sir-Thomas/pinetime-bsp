use embassy_nrf::{bind_interrupts, saadc};
use embassy_nrf::config::Config;
use embassy_time::Duration;

use crate::battery::BatteryController;
use crate::backlight::BacklightController;
pub use crate::backlight::BrightnessLevel;
use crate::button::Button;
use crate::vibrator::Vibrator;

bind_interrupts!(
    /// Interrupts used by the PineTime board
    pub struct Irqs {
        // TWISPI0 => spim::InterruptHandler<peripherals::TWISPI0>;
        // TWISPI1 => twim::InterruptHandler<peripherals::TWISPI1>;
        SAADC => saadc::InterruptHandler;
        // RNG => rng::InterruptHandler<peripherals::RNG>;
        // EGU0_SWI0 => mpsl::LowPrioInterruptHandler;
        // CLOCK_POWER => mpsl::ClockInterruptHandler;
        // RADIO => mpsl::HighPrioInterruptHandler;
        // TIMER0 => mpsl::HighPrioInterruptHandler;
        // RTC0 => mpsl::HighPrioInterruptHandler;
    }
);

/// Represents all the peripherals and pins available for the PineTime.
pub struct PineTime {
    /// Backlight
    pub backlight: BacklightController,
    /// Battery
    pub battery: BatteryController,
    /// Button
    pub button: Button,
    /// Vibrator
    pub vibrator: Vibrator,
}

impl Default for PineTime {
    fn default() -> Self {
        Self::new(Default::default())
    }
}

impl PineTime {
    /// Create a new instance based on HAL configuration
    pub fn new(config: Config) -> Self {
        let p = embassy_nrf::init(config);

        Self {
            battery: BatteryController::new(
                p.P0_31,
                p.SAADC,
                Irqs,
                p.P0_12,
            ),
            backlight: BacklightController::new(
                p.P0_14,
                p.P0_22,
                p.P0_23,
            ),
            button: Button::new(
                p.P0_15,
                p.P0_13,
                Duration::from_millis(50)
            ),
            vibrator: Vibrator::new(p.P0_16),
        }
    }
}