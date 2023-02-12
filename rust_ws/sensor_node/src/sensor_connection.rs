use crate::dist_sensor::{DistanceSensor, MockDistanceReader};

/// Connects to the distance sensor
pub async fn create_sensor() -> impl DistanceSensor {
    MockDistanceReader::new(10, 12000) //TODO replace with real sensor
}
