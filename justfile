# arduino-cli core install esp32:esp32
flash-ulisp-esp32:
	arduino-cli compile --fqbn esp32:esp32:uPesy_wroom ulisp-esp32
	arduino-cli upload -p /dev/ttyUSB0 --fqbn esp32:esp32:uPesy_wroom ulisp-esp32
shell-ulisp-esp32:
	minicom -D /dev/ttyUSB0 -b 9600
	
