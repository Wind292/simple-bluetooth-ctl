use btleplug::platform::{Adapter, Peripheral};
use btleplug::{platform::Manager};
use btleplug::api::{Central, Manager as _, ScanFilter};
use btleplug::api::Peripheral as APIPeripheral;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::time::Instant;
pub struct Instance {
    manager: Option<Manager>,
    adapter: Option<Adapter>,
    preffered_adapter_name: Option<String>,
}


impl Instance {
    // Creates an uninitalized `Instance`
    pub async fn new() -> Self {
        Instance{ 
            manager: None,
            adapter: None,
            preffered_adapter_name: None
        }
    } 
    // Inits the `Instance`
    pub async fn init(&mut self) {
        // init the Manager first because most other things depend on it
        self.manager = Some(Manager::new().await.unwrap());
        
        self.preffered_adapter_name = Some("hci0 (usb:v1D6Bp0246d0554)".to_string()); // IN FILE
        self.update_adapters().await;
    }

    pub async fn start_scan(&mut self) {
        self.adapter.as_mut().unwrap().start_scan(ScanFilter::default()).await.unwrap();
    }


    // Takes about 3ms which is plenty fast
    pub async fn get_scanned_devices(&mut self) -> Vec<(Peripheral, Option<i16>)> {
        let mut devices  = self.adapter.clone().unwrap().peripherals().await.unwrap();
        let mut device_signal_list: Vec<(Peripheral, Option<i16>)> = Vec::new();
        for device in devices {
            let signal = device.properties().await.unwrap().unwrap().rssi;
            device_signal_list.push((device, signal));
        }
        device_signal_list.sort_by(|d1, d2| {d1.1.cmp(&d2.1)});
        device_signal_list
    }

    async fn update_adapters(&mut self){
        let adapters = self.manager.as_mut().unwrap().adapters().await.unwrap();

        for adapter in adapters.clone(){
            if adapter.adapter_info().await.unwrap() == *self.preffered_adapter_name.as_ref().unwrap() {
                self.adapter = Some(adapter.clone());
                break
            }
        }
    }
}
