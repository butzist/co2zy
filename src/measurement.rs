extern crate alloc;
use ens160::AirQualityIndex;

pub struct AirQualityData {
    pub air_quality_index: AirQualityIndex,
    pub eco2_ppm: u16,
    pub tvoc_ppb: u16,
}

pub struct Measurement {
    pub temperature_celsius: f32,
    pub relative_humidity_percent: f32,
    pub air_quality: Option<AirQualityData>,
}

impl Measurement {
    pub fn get_air_quality_color(&self) -> f32 {
        match self.air_quality {
            Some(ref air_data) => match air_data.air_quality_index {
                AirQualityIndex::Excellent => 120.0,
                AirQualityIndex::Good => 90.0,
                AirQualityIndex::Moderate => 45.0,
                AirQualityIndex::Poor => 15.0,
                AirQualityIndex::Unhealthy => 0.0,
            },
            None => 180.0, // Blue/cyan for unknown air quality
        }
    }
}
