#include <WiFi.h>
#include <PubSubClient.h>
#include "DHT.h"

const char WIFI_SSID[] = "Centuria-Dorada";
const char WIFI_PSK[] = "XXX";
const char MQTT_HOST[] = "broker.mqtt-dashboard.com";
#define MQTT_PORT 1883

DHT dht(16, DHT11);
WiFiClient espClient;
PubSubClient client(espClient);
#define MSG_BUFFER_SIZE 50
char msg[MSG_BUFFER_SIZE];

void setup() {
  // init serial monitor
  Serial.begin(9600);
  Serial.println("Temp Sensor");

  // init wifi
  WiFi.mode(WIFI_STA);
  WiFi.begin(WIFI_SSID, WIFI_PSK);

  while (WiFi.status() != WL_CONNECTED) {
    delay(500);
    Serial.println("Connecting to WiFi...");
  }

  // init MQTT
  client.setServer(MQTT_HOST, MQTT_PORT);

  dht.begin();
}

void reconnect() {
  // Loop until we're reconnected
  while (!client.connected()) {
    Serial.print("Attempting MQTT connection...");
    // Create a random client ID
    String clientId = "ace37263-4817-44fb-b182-9eb5d201af5f";
    // Attempt to connect
    if (client.connect(clientId.c_str())) {
      Serial.println("connected");
      // Once connected, publish an announcement...
      // client.publish("temperature", "hello world");
      // ... and resubscribe
      // client.subscribe("inTopic");
    } else {
      Serial.print("failed, rc=");
      Serial.print(client.state());
      Serial.println(" try again in 5 seconds");
      // Wait 5 seconds before retrying
      delay(5000);
    }
  }
}

void loop() {
  delay(2000);
  if (!client.connected()) {
    reconnect();
  }
  client.loop();

  float h = dht.readHumidity();
  float t = dht.readTemperature();
  
  snprintf(msg, MSG_BUFFER_SIZE, "%f - %f", h, t);
  client.publish("temperature", msg);

  Serial.print("Humidity: ");
  Serial.print(h);
  Serial.print("% - Temperature: ");
  Serial.print(t);
  Serial.println("ºC");
}
