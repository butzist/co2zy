#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use alloc::format;
use defmt::info;
use embassy_embedded_hal::shared_bus;
use embassy_executor::Spawner;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_time::Timer;
use embedded_aht20::{Aht20, DEFAULT_I2C_ADDRESS};
use embedded_graphics::mono_font::{MonoTextStyleBuilder, iso_8859_1::FONT_6X10};
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;
use embedded_graphics::text::{Baseline, Text};
use ens160::{AirQualityIndex, Ens160};
use esp_hal::clock::CpuClock;
use esp_hal::i2c::master::{Config as I2cConfig, I2c};
use esp_hal::rmt::Rmt;
use esp_hal::time::Rate;
use esp_hal::timer::timg::TimerGroup;
use esp_hal_smartled::{SmartLedsAdapterAsync, smart_led_buffer};
use palette::{FromColor, Hsv, Srgb};
use shared_bus::asynch::i2c::I2cDevice;
use smart_leds::{RGB8, SmartLedsWriteAsync};
use ssd1306::{
    I2CDisplayInterface, Ssd1306Async, mode::DisplayConfigAsync, rotation::DisplayRotation,
    size::DisplaySize128x64,
};
use {esp_backtrace as _, esp_println as _};

extern crate alloc;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[esp_rtos::main]
async fn main(_spawner: Spawner) -> ! {
    // generator version: 1.0.1

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 65536);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let sw_interrupt =
        esp_hal::interrupt::software::SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    esp_rtos::start(timg0.timer0, sw_interrupt.software_interrupt0);

    info!("Embassy initialized!");

    let i2c_bus = I2c::new(
        peripherals.I2C0,
        I2cConfig::default().with_frequency(Rate::from_khz(400)),
    )
    .unwrap()
    .with_scl(peripherals.GPIO18)
    .with_sda(peripherals.GPIO19)
    .into_async();
    let i2c_bus: Mutex<NoopRawMutex, _> = Mutex::new(i2c_bus);

    info!("Initializing HT sensor...");
    let mut ht_sensor = Aht20::new(
        I2cDevice::new(&i2c_bus),
        DEFAULT_I2C_ADDRESS,
        embassy_time::Delay,
    )
    .await
    .unwrap();

    info!("Initializing AQ sensor...");
    let mut aq_sensor = Ens160::new(I2cDevice::new(&i2c_bus), 0x53);
    aq_sensor.reset().await.unwrap();
    Timer::after_millis(250).await;
    aq_sensor.operational().await.unwrap();
    Timer::after_millis(50).await;

    info!("Initializing display...");
    let interface = I2CDisplayInterface::new(I2cDevice::new(&i2c_bus));
    let mut display = Ssd1306Async::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();

    display.init().await.unwrap();

    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();

    let rmt = Rmt::new(peripherals.RMT, Rate::from_mhz(80u32))
        .unwrap()
        .into_async();
    let mut rmt_buffer = smart_led_buffer!(1);
    let mut led = SmartLedsAdapterAsync::new(rmt.channel0, peripherals.GPIO8, &mut rmt_buffer);
    let mut set_led = async |h: f32, s: f32, v: f32| {
        let rgb = Srgb::from_color(Hsv::new(h, s, v));
        let (r, g, b) = (
            (rgb.red * 255.0) as u8,
            (rgb.green * 255.0) as u8,
            (rgb.blue * 255.0) as u8,
        );
        let data = [RGB8::new(r, g, b); 1];
        led.write(data.iter().cloned()).await.unwrap();
    };

    let mut show_tvoc = false;
    loop {
        info!("Measuring temp/hum...");
        let measurement = ht_sensor.measure().await.unwrap();
        info!(
            "Temperature: {} °C, Relative humidity: {} %",
            measurement.temperature.celsius(),
            measurement.relative_humidity
        );

        display.clear_buffer();

        if let Ok(status) = aq_sensor.status().await
            && status.data_is_ready()
        {
            aq_sensor
                .set_temp((measurement.temperature.celsius() * 100.0) as i16)
                .await
                .unwrap();
            aq_sensor
                .set_hum((measurement.relative_humidity * 100.0) as u16)
                .await
                .unwrap();

            let tvoc = aq_sensor.tvoc().await.unwrap();
            let eco2 = aq_sensor.eco2().await.unwrap();
            let air_quality_index = aq_sensor.air_quality_index().await.unwrap();

            let hue = match air_quality_index {
                AirQualityIndex::Excellent => 120.0,
                AirQualityIndex::Good => 90.0,
                AirQualityIndex::Moderate => 45.0,
                AirQualityIndex::Poor => 15.0,
                AirQualityIndex::Unhealthy => 0.0,
            };
            set_led(hue, 1.0, 0.7).await;

            Text::with_baseline(
                &format!("Air Quality: {:?}", air_quality_index),
                Point::zero(),
                text_style,
                Baseline::Top,
            )
            .draw(&mut display)
            .unwrap();

            if !show_tvoc {
                Text::with_baseline(
                    &format!("eCO2: {} ppm", *eco2),
                    Point::new(0, 48),
                    text_style,
                    Baseline::Top,
                )
                .draw(&mut display)
                .unwrap();

                show_tvoc = true;
            } else {
                Text::with_baseline(
                    &format!("TVOC: {} ppb", tvoc),
                    Point::new(0, 48),
                    text_style,
                    Baseline::Top,
                )
                .draw(&mut display)
                .unwrap();

                show_tvoc = false;
            }
        } else {
            Text::with_baseline(
                "Computing Air Quality...",
                Point::zero(),
                text_style,
                Baseline::Top,
            )
            .draw(&mut display)
            .unwrap();
        }

        Text::with_baseline(
            &format!("Temperature: {:.1} °C", measurement.temperature.celsius()),
            Point::new(0, 16),
            text_style,
            Baseline::Top,
        )
        .draw(&mut display)
        .unwrap();

        Text::with_baseline(
            &format!("Rel. Humidity: {:.1} %", measurement.relative_humidity),
            Point::new(0, 32),
            text_style,
            Baseline::Top,
        )
        .draw(&mut display)
        .unwrap();

        info!("Flushing display...");
        display.flush().await.unwrap();

        Timer::after_secs(5).await;
    }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.0.0/examples/src/bin
}
