use bma425::BMA425;
use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;
use embassy_nrf::gpio::{Input, Pull};
use embassy_nrf::peripherals::{TWISPI0, TWISPI1};
use embassy_nrf::twim::{self, Twim};
use embassy_nrf::{bind_interrupts, saadc, spim};
use embassy_nrf::config::Config;
use embassy_sync::mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_time::Duration;
use static_cell::StaticCell;

use crate::battery::BatteryController;
use crate::backlight::BacklightController;
pub use crate::backlight::BrightnessLevel;
use crate::button::Button;
use crate::display::DisplayController;
use crate::vibrator::Vibrator;

static I2C_BUFFER: StaticCell<[u8; 256]> = StaticCell::new();
static I2C_BUS: StaticCell<Mutex<CriticalSectionRawMutex, Twim<'static>>> = StaticCell::new();

bind_interrupts!(
    pub(crate) struct Irqs {
        TWISPI0 => spim::InterruptHandler<TWISPI0>;
        TWISPI1 => twim::InterruptHandler<TWISPI1>;
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
    /// Accelerometer
    pub accelerometer: BMA425<I2cDevice<'static, CriticalSectionRawMutex, Twim<'static>>, Input<'static>>,
    /// Backlight
    pub backlight: BacklightController,
    /// Battery
    pub battery: BatteryController,
    /// Button
    pub button: Button,
    /// Display TODO: Improve this later
    pub display: DisplayController,
    /// Vibrator
    pub vibrator: Vibrator,
}

impl PineTime {
    /// Create a new instance based on HAL configuration
    pub async fn new(config: Config) -> Self {
        let p = embassy_nrf::init(config);
        let mut twim_config = twim::Config::default();
        twim_config.frequency = twim::Frequency::K400;
        let i2c_buffer = I2C_BUFFER.init([0; 256]);
        let i2c = Twim::new(p.TWISPI1, Irqs, p.P0_06, p.P0_07, twim_config, i2c_buffer);
        let i2c_bus = I2C_BUS.init(Mutex::new(i2c));

        Self {
            accelerometer: BMA425::new(
                I2cDevice::new(&*i2c_bus),
                Input::new(p.P0_08, Pull::Down),
            ),
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
                Duration::from_millis(100)
            ),
            display: DisplayController::new(
                p.TWISPI0,
                Irqs,
                p.P0_02,
                p.P0_03,
                p.P0_04,
                p.P0_18,
                p.P0_25,
                p.P0_26,
            ).await,
            vibrator: Vibrator::new(p.P0_16),
        }
    }
}