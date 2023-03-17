use crate::dist_sensor::{DistanceSensor, MockDistanceReader};
use tf_luna::TfLuna;

/// Connects to the distance sensor
pub async fn create_sensor() -> impl DistanceSensor {
    #[cfg(feature = "no_sensor")]
    {
        MockDistanceReader::new(10, 12000)
    }
    #[cfg(not(feature = "no_sensor"))]
    {
        TfLuna::new().expect("Could not connect to TFLuna!")
    }
}
