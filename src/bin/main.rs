#![no_std]
#![no_main]
#![feature(try_blocks)]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use co2zy::{led::Led, sensor::Sensor, ui::Ui};
use defmt::{info, warn};
use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;
use embassy_executor::Spawner;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_time::Timer;
use esp_hal::clock::CpuClock;
use esp_hal::i2c::master::{Config as I2cConfig, I2c};
use esp_hal::time::Rate;
use esp_hal::timer::timg::TimerGroup;
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

    let i2c_bus = try {
        I2c::new(
            peripherals.I2C0,
            I2cConfig::default().with_frequency(Rate::from_khz(400)),
        )?
        .with_scl(peripherals.GPIO18)
        .with_sda(peripherals.GPIO19)
        .into_async()
    }
    .unwrap_or_else(|e| defmt::panic!("Failed to initialize I2C bus: {}", e));

    let i2c_bus = Mutex::<NoopRawMutex, _>::new(i2c_bus);

    info!("Initializing sensors...");
    let mut sensor = Sensor::new(I2cDevice::new(&i2c_bus), I2cDevice::new(&i2c_bus))
        .await
        .unwrap_or_else(|e| defmt::panic!("Failed to initialize sensors: {}", e));

    info!("Initializing UI...");
    let mut ui = Ui::new(I2cDevice::new(&i2c_bus))
        .await
        .unwrap_or_else(|e| defmt::panic!("Failed to initialize UI: {}", e));

    info!("Initializing LED...");
    let mut led = Led::new(peripherals.RMT, peripherals.GPIO8.into())
        .unwrap_or_else(|e| defmt::panic!("Failed to initialize LED: {}", e));

    loop {
        let measurement = match sensor.measure().await {
            Ok(m) => m,
            Err(e) => {
                warn!("Failed to measure sensors: {}", e);
                Timer::after_secs(5).await;
                continue;
            }
        };

        let color = measurement.get_air_quality_color();
        led.set_color(color, 1.0, 0.7).await;

        ui.render(&measurement)
            .await
            .unwrap_or_else(|e| warn!("Failed to render UI: {}", e));

        Timer::after_secs(5).await;
    }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.0.0/examples/src/bin
}
