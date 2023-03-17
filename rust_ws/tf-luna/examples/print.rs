use tf_luna::TfLuna;

#[tokio::main]
async fn main() {
    println!("init...");
    let mut sensor = TfLuna::new().unwrap();

    println!("Connected!");
    loop {
        let reading = sensor.read().await.unwrap();

        println!("Reading: {:?}", reading);
    }
}
