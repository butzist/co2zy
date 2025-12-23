extern crate alloc;
use esp_hal::rmt::{PulseCode, Rmt};
use esp_hal::time::Rate;
use esp_hal::{gpio::AnyPin, peripherals::RMT};
use esp_hal_smartled::{SmartLedsAdapterAsync, smart_led_buffer};
use palette::{FromColor, Hsv, Srgb};
use smart_leds::{RGB8, SmartLedsWriteAsync};

use crate::mk_static;
use defmt::Format;

#[derive(Format)]
pub enum Error {
    RmtInit,
}

pub struct Led<'a> {
    led: SmartLedsAdapterAsync<'a, 25>,
}

impl<'a> Led<'a> {
    pub fn new(rmt: RMT<'a>, gpio: AnyPin<'a>) -> Result<Self, Error> {
        let rmt_buffer = mk_static!([PulseCode; 25], smart_led_buffer!(1));

        let async_rmt = Rmt::new(rmt, Rate::from_mhz(80))
            .map_err(|_| Error::RmtInit)?
            .into_async();
        let led = SmartLedsAdapterAsync::new(async_rmt.channel0, gpio, rmt_buffer);

        Ok(Self { led })
    }

    pub async fn set_color(&mut self, h: f32, s: f32, v: f32) {
        let rgb = Srgb::from_color(Hsv::new(h, s, v));
        let (r, g, b) = (
            (rgb.red * 255.0) as u8,
            (rgb.green * 255.0) as u8,
            (rgb.blue * 255.0) as u8,
        );
        let data = [RGB8::new(r, g, b); 1];
        let _ = self.led.write(data.iter().cloned()).await;
    }
}
