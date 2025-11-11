mod bluetooth;
use std::time::Duration;

use btleplug::api::Peripheral;
use tokio;

#[tokio::main]
async fn main() {
    println!("Hello, world!");

    let mut inst = bluetooth::Instance::new().await;

    inst.init().await;

    inst.start_scan().await;

    loop {
        let devices = inst.get_scanned_devices().await;
        std::thread::sleep(Duration::from_secs(2));
        for dev in devices {
            println!("{:?}", dev.0.properties().await.unwrap().unwrap().local_name)
        }
        println!("------_BREAK_----");
    }
}
