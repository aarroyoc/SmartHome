#include <BLEDevice.h>
#include <BLEUtils.h>
#include <BLEServer.h>
#include "DHT.h"

#define DEVICE_NAME "Temp Sensor TEST"
#define TEMP_SERVICE_UUID "962c847a-3359-4c4d-a6c9-65db376e7a9f"
#define TEMP_CHAR_UUID "a38bad93-d032-4d97-825e-08bfda98af2f"
#define HUMI_CHAR_UUID "63449677-e46a-4db7-b699-fa7a0d17f9f9"

DHT dht(16, DHT11);

BLECharacteristic* tempChar;
BLECharacteristic* humiChar;

bool deviceConnected = false;
bool guardDeviceConnected = false;

class ServerCallbacks: public BLEServerCallbacks {
  void onConnect(BLEServer* server) {
    deviceConnected = true;
    guardDeviceConnected = true;
  };

  void onDisconnect(BLEServer* server) {
    deviceConnected = false;
  }
};

void setup() {
  // init serial monitor
  Serial.begin(9600);
  Serial.println("Temp Sensor");

  BLEDevice::init(DEVICE_NAME);
  BLEServer* server = BLEDevice::createServer();
  BLEService* service = server->createService(TEMP_SERVICE_UUID);
  tempChar = service->createCharacteristic(
    TEMP_CHAR_UUID,
    BLECharacteristic::PROPERTY_READ
  );
  humiChar = service->createCharacteristic(
    HUMI_CHAR_UUID,
    BLECharacteristic::PROPERTY_READ
  );
  service->start();
  BLEAdvertising* advertising = BLEDevice::getAdvertising();
  advertising->addServiceUUID(TEMP_SERVICE_UUID);
  advertising->setScanResponse(true);
  BLEDevice::startAdvertising();
  Serial.println("BLE setup finished!");

  dht.begin();
}

void loop() {
  float h = dht.readHumidity();
  humiChar->setValue(h);
  float t = dht.readTemperature();
  tempChar->setValue(t);

  Serial.print("Humidity: ");
  Serial.print(h);
  Serial.print("% - Temperature: ");
  Serial.print(t);
  Serial.println("ºC");

  if(!deviceConnected && guardDeviceConnected) {
    delay(500);
    BLEDevice::startAdvertising();
    guardDeviceConnected = false;
  }
  delay(2000);
}
