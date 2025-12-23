extern crate alloc;
use crate::measurement::Measurement;
use alloc::format;
use embedded_graphics::mono_font::{MonoTextStyle, MonoTextStyleBuilder, iso_8859_1::FONT_6X10};
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;
use embedded_graphics::text::{Baseline, Text};
use embedded_hal_async::i2c::I2c;
use ssd1306::{
    I2CDisplayInterface, Ssd1306Async, mode::DisplayConfigAsync, rotation::DisplayRotation,
    size::DisplaySize128x64,
};

pub struct Ui<I2C>
where
    I2C: I2c,
{
    display: Ssd1306Async<
        display_interface_i2c::I2CInterface<I2C>,
        DisplaySize128x64,
        ssd1306::mode::BufferedGraphicsModeAsync<DisplaySize128x64>,
    >,
    text_style: MonoTextStyle<'static, BinaryColor>,
    show_tvoc: bool,
}

impl<I2C> Ui<I2C>
where
    I2C: I2c,
{
    pub async fn new(i2c: I2C) -> Result<Self, ()> {
        let interface = I2CDisplayInterface::new(i2c);
        let mut display = Ssd1306Async::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
            .into_buffered_graphics_mode();

        display.init().await.unwrap();

        let text_style = MonoTextStyleBuilder::new()
            .font(&FONT_6X10)
            .text_color(BinaryColor::On)
            .build();

        Ok(Self {
            display,
            text_style,
            show_tvoc: false,
        })
    }

    fn write_line(&mut self, line_num: i32, text: &str) -> Result<(), ()> {
        let y = line_num * 16;
        Text::with_baseline(text, Point::new(0, y), self.text_style, Baseline::Top)
            .draw(&mut self.display)
            .unwrap();
        Ok(())
    }

    pub async fn render(&mut self, measurement: &Measurement) -> Result<(), ()> {
        self.display.clear_buffer();

        if let Some(ref air_data) = measurement.air_quality {
            self.write_line(0, &format!("Air Quality: {:?}", air_data.air_quality_index))?;

            if !self.show_tvoc {
                self.write_line(3, &format!("eCO2: {} ppm", air_data.eco2_ppm))?;
                self.show_tvoc = true;
            } else {
                self.write_line(3, &format!("TVOC: {} ppb", air_data.tvoc_ppb))?;
                self.show_tvoc = false;
            }
        } else {
            self.write_line(0, "Computing Air Quality...")?;
        }

        self.write_line(
            1,
            &format!("Temperature: {:.1} °C", measurement.temperature_celsius),
        )?;
        self.write_line(
            2,
            &format!(
                "Rel. Humidity: {:.1} %",
                measurement.relative_humidity_percent
            ),
        )?;

        self.display.flush().await.unwrap();
        Ok(())
    }
}
