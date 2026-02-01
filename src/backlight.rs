//! Backlight controller
use embassy_nrf::{Peri, gpio::{Level, Output, OutputDrive}, peripherals};

/// Backlight controller
pub struct BacklightController {
    low: Output<'static>,
    medium: Output<'static>,
    high: Output<'static>,
    brightness_level: InternalBrightnessLevel,
    enabled: bool,
}

impl BacklightController {
    /// Create a new backlight controller instance
    pub fn new(
        backlight_low_pin: Peri<'static, peripherals::P0_14>,
        backlight_med_pin: Peri<'static, peripherals::P0_22>,
        backlight_high_pin: Peri<'static, peripherals::P0_23>,
    ) -> Self {
        Self {
            low: Output::new(backlight_low_pin, Level::High, OutputDrive::Standard),
            medium: Output::new(backlight_med_pin, Level::High, OutputDrive::Standard),
            high: Output::new(backlight_high_pin, Level::High, OutputDrive::Standard),
            brightness_level: InternalBrightnessLevel::Medium,
            enabled: false,
        }
    }

    /// Get the current brightness level
    pub fn brightness(&self) -> BrightnessLevel {
        if !self.enabled {
            BrightnessLevel::Off
        } else {
            match self.brightness_level {
                InternalBrightnessLevel::Low => BrightnessLevel::Low,
                InternalBrightnessLevel::Medium => BrightnessLevel::Medium,
                InternalBrightnessLevel::High => BrightnessLevel::High,
            }
        }
    }

    fn internal_set_brightness(&mut self, level: InternalBrightnessLevel) {
        self.low.set_high();
        self.medium.set_high();
        self.high.set_high();
        self.brightness_level = level;
        match level {
            InternalBrightnessLevel::Low => self.low.set_low(),
            InternalBrightnessLevel::Medium => self.medium.set_low(),
            InternalBrightnessLevel::High => self.high.set_low(),
        }
    }

    /// Set the brightness level
    pub fn set_brightness(&mut self, level: BrightnessLevel) {
        match level {
            BrightnessLevel::Off => self.disable(),
            BrightnessLevel::Low => self.internal_set_brightness(InternalBrightnessLevel::Low),
            BrightnessLevel::Medium => self.internal_set_brightness(InternalBrightnessLevel::Medium),
            BrightnessLevel::High => self.internal_set_brightness(InternalBrightnessLevel::High),
        }
    }

    /// Disable the backlight
    pub fn disable(&mut self) {
        self.low.set_high();
        self.medium.set_high();
        self.high.set_high();
        self.enabled = false;
    }

    /// Enable the backlight at the last set brightness level
    pub fn enable(&mut self) {
        self.internal_set_brightness(self.brightness_level);
        self.enabled = true;
    }

    /// Toggle the backlight
    pub fn toggle(&mut self) {
        match self.enabled {
            true => self.disable(),
            false => self.enable(),
        }
    }
}

/// Brightness levels for the backlight
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BrightnessLevel {
    /// Backlight off
    Off,
    /// Low brightness
    Low,
    /// Medium brightness
    Medium,
    /// High brightness
    High,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum InternalBrightnessLevel {
    Low,
    Medium,
    High,
}