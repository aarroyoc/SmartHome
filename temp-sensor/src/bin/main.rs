#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::delay::Delay;
use esp_hal::gpio::{Io, Level, Output, Input, Pull, OutputOpenDrain};
use esp_hal::prelude::*;
use esp_hal::timer::timg::TimerGroup;
use log::info;
use embedded_dht_rs::dht11::Dht11;
extern crate alloc;

#[entry]
fn main() -> ! {
    let peripherals = esp_hal::init({
        let mut config = esp_hal::Config::default();
        config.cpu_clock = CpuClock::max();
        config
    });

    esp_println::logger::init_logger_from_env();

    esp_alloc::heap_allocator!(72 * 1024);


    // let timg0 = TimerGroup::new(peripherals.TIMG0);
    // let init = esp_wifi::init(
    // 	timg0.timer0,
    // 	esp_hal::rng::Rng::new(peripherals.RNG),
    // 	peripherals.RADIO_CLK,
    // )
    // .unwrap();

    // let delay = Delay::new();
    // loop {
    //     info!("Hello world!");
    //     delay.delay(500.millis());
    // }
    let delay = Delay::new();    
    let mut led = Output::new(peripherals.GPIO13, Level::Low);
    let button = Input::new(peripherals.GPIO12, Pull::Down);
    let od_for_dht11 = OutputOpenDrain::new(peripherals.GPIO14, Level::High, Pull::None);
    let mut dht11 = Dht11::new(od_for_dht11, delay);

    loop {
	
	// led.toggle();
	delay.delay_millis(2500u32);
	info!("Temp sensor v1.0");
	if button.is_high() {
	    led.set_high();
	} else {
	    led.set_low();
	}

	match dht11.read() {
	    Ok(sensor_reading) => info!(
		"DHT11 Sensor - Temperature {} ºC, humidity: {} %",
		sensor_reading.temperature,
		sensor_reading.humidity
	    ),
	    Err(error) => info!("An error occurred while reading the data: {:?}", error)
	}
    }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/v0.22.0/examples/src/bin
}
