use crate::types::{DistanceSensor, MockDistanceReader};
use std::sync::Arc;

/// Connects to the distance sensor
pub async fn create_sensor() -> Arc<dyn DistanceSensor> {
    Arc::new(MockDistanceReader::new(10, 12000)) //TODO replace with real sensor
}