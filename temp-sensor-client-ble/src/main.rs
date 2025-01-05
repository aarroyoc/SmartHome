use bluez_async::{DeviceInfo, BluetoothSession};
use std::time::Duration;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::io::Read;
use tokio::{fs, time};
use uuid::{uuid, Uuid};
use serde::Deserialize;
use axum::Router;
use axum::routing::get;
use axum::extract::State;

const TEMP_SERVICE_UUID: Uuid = uuid!("962c847a-3359-4c4d-a6c9-65db376e7a9f");
const TEMP_CHAR_UUID: Uuid = uuid!("a38bad93-d032-4d97-825e-08bfda98af2f");
const HUMI_CHAR_UUID: Uuid = uuid!("63449677-e46a-4db7-b699-fa7a0d17f9f9");
const SCAN_DURATION: Duration = Duration::from_secs(10);
const LOOP_WAIT_DURATION: Duration = Duration::from_secs(30);

#[derive(Deserialize)]
struct Config {
    devices: Vec<String>
}

struct SensorData {
    temperature: f32,
    humidity: f32,
}

type AppState = Arc<Mutex<HashMap<String, SensorData>>>;

async fn manage_sensor(session: BluetoothSession, device: DeviceInfo, sensor_data: AppState) -> Result<(), eyre::Report> {
    session.connect(&device.id).await?;

    let alias = device.alias.unwrap();
    loop {
	let service = session
	    .get_service_by_uuid(&device.id, TEMP_SERVICE_UUID)
	    .await
	    .unwrap();

	let temp_characteristic = session
	    .get_characteristic_by_uuid(&service.id, TEMP_CHAR_UUID)
	    .await
	    .unwrap();

	let humi_characteristic = session
	    .get_characteristic_by_uuid(&service.id, HUMI_CHAR_UUID)
	    .await
	    .unwrap();

	let mut temp_bytes: &[u8] = &session.read_characteristic_value(&temp_characteristic.id).await?;
	let mut humi_bytes: &[u8] = &session.read_characteristic_value(&humi_characteristic.id).await?;	
	let mut temp_bytes_buffer = [0; 4];
	let mut humi_bytes_buffer = [0; 4];
	match (temp_bytes.read_exact(&mut temp_bytes_buffer), humi_bytes.read_exact(&mut humi_bytes_buffer)) {
	    (Ok(()), Ok(())) => {
		let temperature = f32::from_le_bytes(temp_bytes_buffer);
		let humidity = f32::from_le_bytes(humi_bytes_buffer);
		println!("{} - {}ºC - {}%", alias, temperature, humidity);
		{
		    let mut sensor_data = sensor_data.lock().unwrap();
		    sensor_data.insert(alias.clone(), SensorData {
			temperature,
			humidity,
		    });
		}	
	    }
	    _ => {}
	}

	time::sleep(LOOP_WAIT_DURATION).await;
    }
}

#[tokio::main]
async fn main() -> Result<(), eyre::Report> {
    pretty_env_logger::init();

    let config_file_str = fs::read_to_string("config.toml").await?;
    let config: Config = toml::from_str(&config_file_str).expect("Could not parse config.toml file");
	

    let (_, session) = BluetoothSession::new().await?;

    println!("Start Bluetooth discovery");
    session.start_discovery().await?;
    time::sleep(SCAN_DURATION).await;
    session.stop_discovery().await?;
    println!("Ended Bluetooth discovery");

    let devices = session.get_devices().await?;

    let mut handles = vec![];
    let sensor_data = Arc::new(Mutex::new(HashMap::new()));
    for device in devices {
	let alias = device.clone().alias.unwrap_or("".into());
	if config.devices.contains(&alias) {
	    println!("Managing device: {}", alias);	    
	    let session0 = session.clone();
	    let sensor_data = sensor_data.clone();
	    handles.push(tokio::task::spawn(async move {
		manage_sensor(session0, device, sensor_data).await.expect("Error while managing device");
	    }));
	}
    }
    // futures::future::join_all(handles).await;
    let app =  Router::new()
	.route("/metrics", get(metrics))
	.with_state(sensor_data);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:6765").await?;
    println!("Listening on {}/metrics", listener.local_addr().unwrap());
    axum::serve(listener, app).await?;

    Ok(())
}

async fn metrics(State(state): State<AppState>) -> String {
    let sensor_data = state.lock().unwrap();

    let mut output = String::from("");
    for (device, data) in sensor_data.iter() {
	output.push_str(&format!("smart_home_temperature{{device=\"{}\"}} {}\n", device, data.temperature));
	output.push_str(&format!("smart_home_humidity{{device=\"{}\"}} {}\n", device, data.humidity));
    }
    output
}
