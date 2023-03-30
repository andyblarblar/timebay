use crate::dist_sensor::{DistanceSensor, MockDistanceReader};
use std::path::PathBuf;
use tf_luna::TfLuna;

/// Connects to the distance sensor
pub async fn create_sensor() -> impl DistanceSensor {
    #[cfg(feature = "no_sensor")]
    {
        MockDistanceReader::new(10, 12000)
    }
    #[cfg(not(feature = "no_sensor"))]
    {
        // Attempt to connect if connected to either the UART adapter or on a Le Potato
        let res = TfLuna::new(PathBuf::from("/dev/ttyUSB0"));

        if let Ok(sensor) = res {
            sensor
        } else {
            TfLuna::new(PathBuf::from("/dev/ttyAML6")).expect("No avialible tf-lunas!")
        }
    }
}
