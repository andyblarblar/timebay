use crate::dist_sensor::DistanceSensor;

pub struct ApplicationContext<T>
where
    T: DistanceSensor,
{
    sensor: T,
    zero: u32,
}

impl<T: DistanceSensor> ApplicationContext<T> {
    /// Creates a new context. Sensor should already be connected and ready.
    pub fn new(sensor: T, default_zero: u32) -> Self {
        Self {
            sensor,
            zero: default_zero,
        }
    }

    /// Zeros the sensor. This is stored internally, and also returned.
    pub async fn zero(&mut self) -> u32{
        let mut avg = 0;

        for _ in 1..=10 {
            avg += self.sensor.get_reading().await.dist;
        }

        let zero = avg / 10;
        self.zero = zero;

        zero
    }
}
