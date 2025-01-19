#include <BLEDevice.h>
#include <BLEUtils.h>
#include <BLEServer.h>
#include <DHT.h>
#include <SPI.h>
#include <Wire.h>
#include <Adafruit_GFX.h>
#include <Adafruit_SSD1306.h>

#define DEVICE_NAME "Temp Sensor TEST"
#define TEMP_SERVICE_UUID "962c847a-3359-4c4d-a6c9-65db376e7a9f"
#define TEMP_CHAR_UUID "a38bad93-d032-4d97-825e-08bfda98af2f"
#define HUMI_CHAR_UUID "63449677-e46a-4db7-b699-fa7a0d17f9f9"

#define SCREEN_WIDTH 128
#define SCREEN_HEIGHT 64
#define OLED_RESET -1
Adafruit_SSD1306 display(SCREEN_WIDTH, SCREEN_HEIGHT, &Wire, OLED_RESET);

#define I2C_SDA 22
#define I2C_SCL 23

#define BUTTON_PIN 21

DHT dht(16, DHT11);

BLECharacteristic* tempChar;
BLECharacteristic* humiChar;

bool deviceConnected = false;
bool guardDeviceConnected = false;

int n = 0;
float h = 0.0;
float t = 0.0;

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
  Wire.begin(I2C_SDA, I2C_SCL);
  // init serial monitor
  Serial.begin(9600);
  Serial.println(F("Temp Sensor"));

  pinMode(BUTTON_PIN, INPUT);

  if(!display.begin(SSD1306_SWITCHCAPVCC, 0x3C)) { 
    Serial.println(F("SSD1306 allocation failed"));
    for(;;); // Don't proceed, loop forever
  }

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
  Serial.println(F("BLE setup finished!"));

  display.display();
  delay(2000);
  display.clearDisplay();

  dht.begin();
}

void loop() {
  if (n == 20) {
    h = dht.readHumidity();
    humiChar->setValue(h);
    t = dht.readTemperature();
    tempChar->setValue(t);

    Serial.print(F("Humidity: "));
    Serial.print(h);
    Serial.print(F("% - Temperature: "));
    Serial.print(t);
    Serial.println("ºC");
    n = 0;
  } else {
    n++;
  }

  int buttonState = digitalRead(BUTTON_PIN);
  if (buttonState == HIGH) {
    display.clearDisplay();
    display.setTextColor(WHITE);
    display.setTextSize(1);
    display.setCursor(0, 0);
    display.println(F("Temperature"));
    display.setTextSize(2);
    display.print(t);
    display.println(F(" C"));
    display.setTextSize(1);
    display.println(F("Humidity"));
    display.setTextSize(2);
    display.print(h);
    display.println(F(" %"));
    display.display();
    delay(1);
  } else {
    display.clearDisplay();
    display.display();
  }


  if(!deviceConnected && guardDeviceConnected) {
    delay(500);
    BLEDevice::startAdvertising();
    guardDeviceConnected = false;
  }
  delay(100);
}
