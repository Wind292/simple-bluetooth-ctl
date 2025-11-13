use std::cmp::{self, Ordering};

use btleplug::platform::{Adapter, Peripheral};
use btleplug::{platform::Manager};
use btleplug::api::{Central, Manager as _, ScanFilter};
use btleplug::api::Peripheral as APIPeripheral;

#[derive(Clone, Debug)]
pub struct Device {
    pub peripheral: Peripheral,
    pub connection_strength: Option<i16>,
    pub display_name: Option<String>,
    pub adress: String,
    pub index: Option<usize>
}

impl Device {
    pub fn new(peripheral: Peripheral,connection_strength: Option<i16>,display_name: Option<String>,adress: String,index: Option<usize>) -> Self {
        Device { peripheral, connection_strength, display_name, adress, index }
    }
}


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
    pub async fn get_scanned_devices(&mut self) -> Vec<Device> { // Peripheral, signal strength, display name, mac adress
        let devices  = self.adapter.clone().unwrap().peripherals().await.unwrap();
        let mut device_signal_list: Vec<Device> = Vec::new();

        for device in devices {
            let properties = device.properties().await.unwrap_or_default().unwrap_or_default();

            let adress = properties.address.to_string();
            let signal = properties.rssi;
            let name = properties.local_name;
            device_signal_list.push(Device::new(device, signal, name, adress, None));
        }

        device_signal_list.sort_by(|d1, d2| {sort_devices_check((d1.connection_strength, d1.display_name.clone(), d1.adress.clone()), (d2.connection_strength, d2.display_name.clone(), d2.adress.clone()))});
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
    (mut rssi1, display_name1, adress1): (Option<i16>, Option<String>, String),
    (mut rssi2, display_name2, adress2): (Option<i16>, Option<String>, String)
) -> cmp::Ordering {
    if rssi1.is_some() && rssi2.is_some() {
        if rssi1.unwrap()/25 != rssi2.unwrap()/25 {
            return (rssi2.unwrap()).cmp(&rssi1.unwrap())
        }
    }


    match (display_name1, display_name2) {
        (Some(d1), Some(d2)) => return d1.cmp(&d2).then(adress1.cmp(&adress2)),
        (None, Some(_)) => return cmp::Ordering::Greater,
        (Some(_), None) => return cmp::Ordering::Less,
        (None, None) => {adress1.cmp(&adress2)}
    }
}


