extern crate alloc;
use embassy_time::Delay;
use embedded_aht20::{Aht20, DEFAULT_I2C_ADDRESS};
use embedded_hal_async::i2c::I2c;

use ens160::{Ens160, Status as Ens160Status};

use crate::measurement::{AirQualityData, Measurement};
use defmt::Format;

#[derive(Format)]
pub enum Error {
    ThSensorInit,
    ThSensorMeasure,
    AqSensorReset,
    AqSensorOperational,
    AqSensorSetTemp,
    AqSensorSetHum,
    AqSensorTvoc,
    AqSensorEco2,
    AqSensorAqi,
    AqSensorStatus,
}

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
    pub async fn new(i2c_th: I2C, i2c_aq: I2C) -> Result<Self, Error> {
        let th_sensor = Aht20::new(i2c_th, DEFAULT_I2C_ADDRESS, Delay)
            .await
            .map_err(|_| Error::ThSensorInit)?;

        let mut aq_sensor = Ens160::new(i2c_aq, 0x53);
        aq_sensor.reset().await.map_err(|_| Error::AqSensorReset)?;
        embassy_time::Timer::after_millis(250).await;
        aq_sensor
            .operational()
            .await
            .map_err(|_| Error::AqSensorOperational)?;
        embassy_time::Timer::after_millis(50).await;

        Ok(Self {
            th_sensor,
            aq_sensor,
        })
    }

    pub async fn measure(&mut self) -> Result<Measurement, Error> {
        let th_measurement = self
            .th_sensor
            .measure()
            .await
            .map_err(|_| Error::ThSensorMeasure)?;

        let temperature_celsius = th_measurement.temperature.celsius();
        let relative_humidity_percent = th_measurement.relative_humidity;

        let status = self
            .aq_sensor
            .status()
            .await
            .map_err(|_| Error::AqSensorStatus)?;
        let ens160_status: Ens160Status = status;

        let air_quality = if ens160_status.data_is_ready() {
            self.aq_sensor
                .set_temp((temperature_celsius * 100.0) as i16)
                .await
                .map_err(|_| Error::AqSensorSetTemp)?;
            self.aq_sensor
                .set_hum((relative_humidity_percent * 100.0) as u16)
                .await
                .map_err(|_| Error::AqSensorSetHum)?;

            let tvoc_ppb = self
                .aq_sensor
                .tvoc()
                .await
                .map_err(|_| Error::AqSensorTvoc)?;
            let eco2_ppm = self
                .aq_sensor
                .eco2()
                .await
                .map_err(|_| Error::AqSensorEco2)?;
            let air_quality_index = self
                .aq_sensor
                .air_quality_index()
                .await
                .map_err(|_| Error::AqSensorAqi)?;

            Some(AirQualityData {
                air_quality_index,
                eco2_ppm: *eco2_ppm,
                tvoc_ppb,
            })
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
