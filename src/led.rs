extern crate alloc;
use esp_hal::gpio::{AnyPin, DriveMode};
use esp_hal::ledc::{
    LSGlobalClkSource, Ledc, LowSpeed,
    channel::{self, Channel, ChannelIFace},
    timer::{self, TimerIFace},
};
use esp_hal::peripherals::LEDC;
use esp_hal::time::Rate;
use palette::{FromColor, Hsv, Srgb};

use crate::mk_static;
use defmt::Format;

#[derive(Format)]
pub enum Error {
    LedcInit,
}

pub struct Led {
    red_channel: &'static mut Channel<'static, LowSpeed>,
    green_channel: &'static mut Channel<'static, LowSpeed>,
    blue_channel: &'static mut Channel<'static, LowSpeed>,
}

impl Led {
    pub fn new(
        ledc: LEDC<'static>,
        red_pin: AnyPin<'static>,
        green_pin: AnyPin<'static>,
        blue_pin: AnyPin<'static>,
    ) -> Result<Self, Error> {
        let mut ledc = Ledc::new(ledc);
        ledc.set_global_slow_clock(LSGlobalClkSource::APBClk);

        let mut lstimer0 = ledc.timer::<LowSpeed>(timer::Number::Timer0);
        lstimer0
            .configure(timer::config::Config {
                duty: timer::config::Duty::Duty5Bit,
                clock_source: timer::LSClockSource::APBClk,
                frequency: Rate::from_khz(24),
            })
            .map_err(|_| Error::LedcInit)?;

        let timer0 = mk_static!(timer::Timer<'static, LowSpeed>, lstimer0);

        let red_channel = mk_static!(
            Channel<'static, LowSpeed>,
            ledc.channel(channel::Number::Channel0, red_pin)
        );
        red_channel
            .configure(channel::config::Config {
                timer: timer0,
                duty_pct: 0,
                drive_mode: DriveMode::PushPull,
            })
            .map_err(|_| Error::LedcInit)?;

        let green_channel = mk_static!(
            Channel<'static, LowSpeed>,
            ledc.channel(channel::Number::Channel1, green_pin)
        );
        green_channel
            .configure(channel::config::Config {
                timer: timer0,
                duty_pct: 0,
                drive_mode: DriveMode::PushPull,
            })
            .map_err(|_| Error::LedcInit)?;

        let blue_channel = mk_static!(
            Channel<'static, LowSpeed>,
            ledc.channel(channel::Number::Channel2, blue_pin)
        );
        blue_channel
            .configure(channel::config::Config {
                timer: timer0,
                duty_pct: 100,
                drive_mode: DriveMode::PushPull,
            })
            .map_err(|_| Error::LedcInit)?;

        Ok(Self {
            red_channel,
            green_channel,
            blue_channel,
        })
    }

    pub async fn set_color(&mut self, h: f32, s: f32, v: f32) {
        let rgb = Srgb::from_color(Hsv::new(h, s, v));
        let r = (rgb.red * 100.0) as u8;
        let g = (rgb.green * 40.0) as u8;
        let b = (rgb.blue * 50.0) as u8;

        self.red_channel.set_duty(r).unwrap();
        self.green_channel.set_duty(g).unwrap();
        self.blue_channel.set_duty(b).unwrap();
    }
}
