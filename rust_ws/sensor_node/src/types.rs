use async_trait::async_trait;
use rand::{thread_rng, RngCore};
use std::cmp::max;

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

/// A sensor capable of reading distance values.
#[async_trait]
pub trait DistanceSensor {
    /// Gets a distance reading.
    async fn get_reading(&mut self) -> DistanceReading;
}

/// Reads random distance values between two bounds.
pub struct MockDistanceReader {
    max: u32,
    min: u32,
}

impl MockDistanceReader {
    /// Creates a mock sensor that reads random values between min and max.
    pub fn new(min: u32, max: u32) -> Self {
        Self { max, min }
    }
}

#[async_trait]
impl DistanceSensor for MockDistanceReader {
    async fn get_reading(&mut self) -> DistanceReading {
        DistanceReading::new(max(thread_rng().next_u32() % self.max, self.min))
    }
}