use std::thread;
use std::time::Duration;
use embedded_svc::wifi::{ClientConfiguration, Configuration};
use esp_idf_hal::modem::Modem;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::nvs;
use esp_idf_svc::wifi::EspWifi;

pub struct Wifi {

}

pub struct WifiConfig {
    pub is_enabled: bool,
    pub ssid: String,
    pub pass: String,
}

impl Clone for WifiConfig {
    fn clone(&self) -> Self {
        Self {
            is_enabled: self.is_enabled,
            ssid: self.ssid.clone(),
            pass: self.pass.clone()
        }
    }
}

impl Wifi {
    pub fn create(config: WifiConfig, modem: Modem) -> Self {
        if !config.is_enabled {
            return Self { };
        }

        let sysloop = EspSystemEventLoop::take().unwrap();
        let nvs_partition = nvs::EspDefaultNvsPartition::take().unwrap();
        let mut wifi_driver = EspWifi::new(modem, sysloop.clone(), Some(nvs_partition)).unwrap();

        wifi_driver
            .set_configuration(&Configuration::Client(ClientConfiguration {
                ssid: config.ssid.as_str().into(),
                password: config.pass.as_str().into(),
                ..Default::default()
            }))
            .unwrap();

        wifi_driver.start().unwrap();
        wifi_driver.connect().unwrap();

        println!("after wifi connect");

        while wifi_driver.sta_netif().get_ip_info().unwrap().ip.is_unspecified() {
            thread::sleep(Duration::from_millis(2000));
        }

        println!("ip acquired");

        let ip_info = wifi_driver.sta_netif().get_ip_info().unwrap();
        println!("ip: {}", ip_info.ip);
        println!("dns: {:?}", ip_info.dns);

        Self {

        }
    }
}