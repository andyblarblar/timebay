use crate::dist_sensor::{DistanceSensor, SensorError};

pub struct ApplicationContext<T>
where
    T: DistanceSensor,
{
    sensor: T,
    /// Current sensor zero, mm
    zero: u32,
    /// Delta off of zero to count as a trigger, mm
    threshold: u32,
}

impl<T: DistanceSensor> ApplicationContext<T> {
    /// Creates a new context. Sensor should already be connected and ready.
    ///
    /// # Args
    /// `threshold` - The difference off zero to consider a trigger in mm
    pub fn new(sensor: T, default_zero: u32, threshold: u32) -> Self {
        Self {
            sensor,
            zero: default_zero,
            threshold,
        }
    }

    /// Zeros the sensor. New zero is stored internally, and also returned.
    pub async fn zero(&mut self) -> Result<u32, SensorError> {
        let mut avg = 0;

        for _ in 1..=100 {
            avg += self.sensor.get_reading().await?.dist;
        }

        let zero = avg / 10;
        self.zero = zero;

        log::debug!("Set zero to {}mm", zero);

        Ok(zero)
    }

    /// Spins until the sensor gets a reading that it considers to be a vehicle passing.
    pub async fn wait_for_trigger(&mut self) -> Result<(), SensorError> {
        loop {
            let reading = self.sensor.get_reading().await?;

            if should_trigger(self.zero, self.threshold, reading.dist) {
                log::debug!("Triggered!");
                return Ok(());
            }
        }
    }
}

/// Checks if a reading counts as a trigger of the car passing
fn should_trigger(zero: u32, threshold: u32, reading: u32) -> bool {
    let closer_than_zero = (zero as i64 - reading as i64) > 0;
    closer_than_zero && (zero - reading) >= threshold
}

#[cfg(test)]
mod test {
    use crate::application::should_trigger;

    #[test]
    fn should_trigger_works() {
        // Reading greater than zero should not trigger
        assert!(!should_trigger(8, 1, 9));

        // Reading exactly the trigger should trigger
        assert!(should_trigger(8, 1, 7));

        // Reading less than trigger should trigger
        assert!(should_trigger(8, 1, 1));
    }
}
