extern crate alloc;
use embassy_time::Delay;
use embedded_aht20::{Aht20, DEFAULT_I2C_ADDRESS};
use embedded_hal_async::i2c::I2c;

use ens160::{Ens160, Status as Ens160Status};

use crate::measurement::{AirQualityData, Measurement};
pub struct Sensor<I2C>
where
    I2C: I2c,
{
    th_sensor: Aht20<I2C, embassy_time::Delay>,
    aq_sensor: Ens160<I2C>,
}

impl<I2C> Sensor<I2C>
where
    I2C: I2c,
{
    pub async fn new(i2c_th: I2C, i2c_aq: I2C) -> Result<Self, ()> {
        let th_sensor = Aht20::new(i2c_th, DEFAULT_I2C_ADDRESS, Delay)
            .await
            .unwrap();

        let mut aq_sensor = Ens160::new(i2c_aq, 0x53);
        aq_sensor.reset().await.unwrap();
        embassy_time::Timer::after_millis(250).await;
        aq_sensor.operational().await.unwrap();
        embassy_time::Timer::after_millis(50).await;

        Ok(Self {
            th_sensor,
            aq_sensor,
        })
    }

    pub async fn measure(&mut self) -> Result<Measurement, ()> {
        let th_measurement = self.th_sensor.measure().await.unwrap();

        let temperature_celsius = th_measurement.temperature.celsius();
        let relative_humidity_percent = th_measurement.relative_humidity;

        let air_quality = if let Ok(status) = self.aq_sensor.status().await {
            let ens160_status: Ens160Status = status;
            if ens160_status.data_is_ready() {
                self.aq_sensor
                    .set_temp((temperature_celsius * 100.0) as i16)
                    .await
                    .unwrap();
                self.aq_sensor
                    .set_hum((relative_humidity_percent * 100.0) as u16)
                    .await
                    .unwrap();

                let tvoc_ppb = self.aq_sensor.tvoc().await.unwrap();
                let eco2_ppm = self.aq_sensor.eco2().await.unwrap();
                let air_quality_index = self.aq_sensor.air_quality_index().await.unwrap();

                Some(AirQualityData {
                    air_quality_index,
                    eco2_ppm: *eco2_ppm,
                    tvoc_ppb,
                })
            } else {
                None
            }
        } else {
            None
        };

        Ok(Measurement {
            temperature_celsius,
            relative_humidity_percent,
            air_quality,
        })
    }
}
