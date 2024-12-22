use std::thread;
use std::time::Duration;

use embedded_dht_rs::dht11::Dht11;
use esp_idf_svc::hal::gpio::PinDriver;
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::hal::delay::Delay;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::wifi::*;
use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::mqtt::client::*;

#[toml_cfg::toml_config]
pub struct Config {
    #[default("")]
    wifi_ssid: &'static str,
    #[default("")]
    wifi_psk: &'static str,
    #[default("")]
    mqtt_broker_url: &'static str,
}

const UUID: &'static str = "ace37263-4817-44fb-b182-9eb5d201af5f";


fn main() -> anyhow::Result<()> {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("Hello, world!");

    let peripherals = Peripherals::take()?;
    let sysloop = EspSystemEventLoop::take()?;

    let app_config = CONFIG;

    let mut esp_wifi = EspWifi::new(peripherals.modem, sysloop.clone(), None)?;
    let mut wifi = BlockingWifi::wrap(&mut esp_wifi, sysloop)?;
    wifi.set_configuration(&Configuration::Client(ClientConfiguration::default()))?;
    wifi.start()?;
    let ap_infos = wifi.scan()?;
    log::info!("AP Infos: {:?}", ap_infos);
    let ours = ap_infos.into_iter().find(|a| a.ssid == app_config.wifi_ssid);
    let channel = if let Some(ours) = ours {
	Some(ours.channel)
    } else {
	None
    };
    wifi.set_configuration(&Configuration::Client(ClientConfiguration {
	ssid: app_config.wifi_ssid.try_into().expect("Could not parse the given SSID"),
	password: app_config.wifi_psk.try_into().expect("Could not parse the given password"),
	channel,
	auth_method: AuthMethod::WPA2Personal,
	..Default::default()}))?;
    log::info!("Trying to connect to {}", app_config.wifi_ssid);
    wifi.connect()?;
    wifi.wait_netif_up()?;
    let ip_info = wifi.wifi().sta_netif().get_ip_info()?;
    log::info!("WiFi DHCP info: {:?}", ip_info);

    let mqtt_config = MqttClientConfiguration::default();
    let mut client = EspMqttClient::new_cb(&app_config.mqtt_broker_url, &mqtt_config, move |_message_event| {

    })?;

    let delay: Delay = Default::default();
    let mut led = PinDriver::output(peripherals.pins.gpio17)?;
    let od_for_dht11 = PinDriver::input_output_od(peripherals.pins.gpio16)?;
    let mut dht11 = Dht11::new(od_for_dht11, delay);

    loop {
	led.set_high().unwrap();
	match dht11.read() {
	    Ok(sensor_reading) => {
		log::info!(
		    "DHT 11 Sensor - Temperature {} ºC - Humidity {} %",
		    sensor_reading.temperature,
		    sensor_reading.humidity);
		let payload = format!("{}", sensor_reading.temperature);
		log::info!("Publishing to topic: {}/temperature", UUID);
		client.enqueue(&format!("{}/temperature", UUID), QoS::AtLeastOnce, false, payload.as_bytes());
	    },
	    Err(err) => log::error!("An error occurred: {:?}", err)
	}
	led.set_low().unwrap();
	delay.delay_ms(750);
    }
    Ok(())
}
