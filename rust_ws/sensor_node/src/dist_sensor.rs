//! Abstractions over the distance sensor

use async_trait::async_trait;
use rand::distributions::Distribution;
use rand::{thread_rng, RngCore};
use std::cmp::max;
use std::io;
use std::time::Duration;
use thiserror::Error;

/// A sensor capable of reading distance values.
#[async_trait]
pub trait DistanceSensor {
    /// Gets a distance reading.
    async fn get_reading(&mut self) -> Result<DistanceReading, SensorError>;
}

/// Errors from interacting with the sensor
#[derive(Error, Debug)]
pub enum SensorError {
    #[error(transparent)]
    IOError(#[from] io::Error),
}

/// A distance reading
pub struct DistanceReading {
    /// Distance reading in mm
    pub dist: u32,
}

impl DistanceReading {
    pub fn new(dist: u32) -> Self {
        Self { dist }
    }
}

/// Reads random distance values between two bounds.
pub struct MockDistanceReader {
    max: u32,
    min: u32,
    pdf: rand::distributions::Bernoulli,
}

impl MockDistanceReader {
    /// Creates a mock sensor that reads random values between min and max.
    pub fn new(min: u32, max: u32) -> Self {
        Self {
            max,
            min,
            pdf: rand::distributions::Bernoulli::new(0.3).unwrap(),
        }
    }
}

#[async_trait]
impl DistanceSensor for MockDistanceReader {
    async fn get_reading(&mut self) -> Result<DistanceReading, SensorError> {
        tokio::time::sleep(Duration::from_millis(10)).await;
        Ok(DistanceReading::new(max(
            if self.pdf.sample(&mut thread_rng()) {
                4000
            } else {
                12000
            },
            self.min,
        )))
    }
}
