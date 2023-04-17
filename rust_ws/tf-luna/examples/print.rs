use std::path::PathBuf;
use tf_luna::TfLuna;

#[tokio::main]
async fn main() {
    println!("init...");
    let mut sensor = TfLuna::new(PathBuf::from("/dev/ttyUSB0")).unwrap();

    println!("Connected!");
    let mut err_cnt = 0;
    loop {
        let reading = sensor.read().await;

        if let Err(err) = reading {
            println!("Errored with: {}", err);
            err_cnt += 1;
        } else {
            println!("Reading: {:?} Err count: {}", reading, err_cnt);
        }
    }
}
