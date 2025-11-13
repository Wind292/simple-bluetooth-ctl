use std::cmp;

use btleplug::platform::{Adapter, Peripheral};
use btleplug::{platform::Manager};
use btleplug::api::{Central, Manager as _, ScanFilter};
use btleplug::api::Peripheral as APIPeripheral;

#[derive(Clone, Debug)]
pub struct Instance {
    manager: Option<Manager>,
    adapter: Option<Adapter>,
    preffered_adapter_name: Option<String>,
}


impl Instance {
    // Creates an uninitalized `Instance`
    pub fn new() -> Self {
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
    pub async fn get_scanned_devices(&mut self) -> Vec<(Peripheral, Option<i16>, Option<String>, String)> { // Peripheral, signal strength, display name, mac adress
        let devices  = self.adapter.clone().unwrap().peripherals().await.unwrap();
        let mut device_signal_list: Vec<(Peripheral, Option<i16>, Option<String>, String)> = Vec::new();

        for device in devices {
            let properties = device.properties().await.unwrap_or_default().unwrap_or_default();

            let adress = properties.address.to_string();
            let signal = properties.rssi;
            let name = properties.local_name;
            device_signal_list.push((device, signal, name, adress));
        }

        device_signal_list.sort_by(|d1, d2| {sort_devices_check((d1.1, d1.2.clone(), d1.3.clone()), (d2.1, d2.2.clone(), d1.3.clone()))});
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


fn sort_devices_check(
    (rssi1, display_name1, adress1): (Option<i16>, Option<String>, String),
    (rssi2, display_name2, adress2): (Option<i16>, Option<String>, String)
) -> cmp::Ordering {
    
    if display_name1.is_some() && display_name2.is_some() {return display_name1.cmp(&display_name2);} 

    return adress1.cmp(&adress2);
  

}