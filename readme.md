# Rust Esp32 LED Controller
This is the code for the esp32s3 microcontroller that is connected to the LED strip (ws2812). It is a simple mqtt client that listens to the topic `led` and sets the color of the LED strip according to the message it receives.
The messages can be sent with the `led-cli` tool [here](https://github.com/leoseg/LED-CLI-Controller).


The client is written in Rust and uses the `esp-idf` framework, for configuration a 'cfg.toml' file is used like the 'cfg.toml.example' file.
See [here](https://docs.esp-rs.org/book/installation/riscv-and-xtensa.html) and [here](https://docs.esp-rs.org/book/installation/std-requirements.html) for further requirements.
