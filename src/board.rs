use bma425::BMA425;
use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;
use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice;
use embassy_nrf::gpio::{Input, Level, Output, OutputDrive, Pull};
use embassy_nrf::peripherals::{self, TWISPI0, TWISPI1};
use embassy_nrf::spim::Spim;
use embassy_nrf::twim::{self, Twim};
use embassy_nrf::{Peri, bind_interrupts, saadc, spim, spis};
use embassy_nrf::config::Config;
use embassy_sync::mutex::Mutex;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_time::{Delay, Duration};
use static_cell::StaticCell;

use crate::battery::BatteryController;
use crate::backlight::BacklightController;
pub use crate::backlight::BrightnessLevel;
use crate::button::Button;
use crate::display::DisplayController;
use crate::vibrator::Vibrator;

static I2C_BUFFER: StaticCell<[u8; 256]> = StaticCell::new();
static I2C_BUS: StaticCell<Mutex<NoopRawMutex, Twim<'static>>> = StaticCell::new();
static SPI_BUS: StaticCell<Mutex<NoopRawMutex, Spim<'static>>> = StaticCell::new();

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
    pub accelerometer: BMA425<I2cDevice<'static, NoopRawMutex, Twim<'static>>, Input<'static>>,
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
        let i2c_bus = Self::initialize_i2c_bus(
            p.TWISPI1,
            p.P0_06,
            p.P0_07,
        );
        let spi_bus = Self::initialize_spi_bus(
            p.TWISPI0,
            p.P0_02,
            p.P0_03,
            p.P0_04,
        );

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
                SpiDevice::new(spi_bus, Output::new(p.P0_25, Level::High, OutputDrive::Standard)),
                p.P0_18,
                p.P0_26,
            ).await,
            vibrator: Vibrator::new(p.P0_16),
        }
    }

    fn initialize_i2c_bus(
        twispi1: Peri<'static, peripherals::TWISPI1>,
        p0_06: Peri<'static, peripherals::P0_06>,
        p0_07: Peri<'static, peripherals::P0_07>
    ) -> &'static Mutex<NoopRawMutex, Twim<'static>> {
        let mut twim_config = twim::Config::default();
        twim_config.frequency = twim::Frequency::K400;
        let i2c_buffer = I2C_BUFFER.init([0; 256]);
        let i2c = Twim::new(twispi1, Irqs, p0_06, p0_07, twim_config, i2c_buffer);
        I2C_BUS.init(Mutex::new(i2c))
    }

    fn initialize_spi_bus(
        twispi0: Peri<'static, peripherals::TWISPI0>,
        sck_pin: Peri<'static, peripherals::P0_02>,
        mosi_pin: Peri<'static, peripherals::P0_03>,
        miso_pin: Peri<'static, peripherals::P0_04>,
    ) -> &'static Mutex<NoopRawMutex, Spim<'static>> {
        let mut spim_config = spim::Config::default();
        spim_config.frequency = spim::Frequency::M8;
        spim_config.mode = spis::MODE_3;
        let spim = spim::Spim::new(twispi0, Irqs, sck_pin, miso_pin, mosi_pin, spim_config);
        let spi_bus = Mutex::new(spim);
        SPI_BUS.init(spi_bus)
    }
}