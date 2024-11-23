use std::cmp::PartialEq;
use std::sync::mpsc::Sender;
use std::thread;
use esp_idf_hal::gpio::Gpio44;
use log::info;
use smart_leds_trait::{SmartLedsWrite, White};
use serde::{Serialize, Deserialize};
use esp_idf_hal::rmt::CHANNEL0;
use ws2812_esp32_rmt_driver::driver::color::LedPixelColorGrbw32;
use ws2812_esp32_rmt_driver::{LedPixelEsp32Rmt, RGBW8};

const NUMBER_OF_LEDS: usize = 60;

#[derive(Serialize, Deserialize, Debug)]
enum LedState {
    On,
    Off,
}


#[derive(Serialize, Deserialize, Debug)]
pub struct Led {
    led_state: LedState,
    
    color: Color,

    percentage: u8,
}

#[derive(Debug, Clone,Deserialize, Serialize)]
pub enum Color {
    White,
    Red,
    Green,
    Blue,
    Yellow,
}

impl Color {
    pub fn to_rgb(&self) -> (u8, u8, u8) {
        match self {
            Color::White => (255, 255, 255),
            Color::Red => (255, 0, 0),
            Color::Green => (0, 255, 0),
            Color::Blue => (0, 0, 255),
            Color::Yellow => (255, 255, 0),
        }
    }
}

pub struct LedController {
    tx : Sender<Led>,
}

impl PartialEq for LedState {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (LedState::On, LedState::On) => true,
            (LedState::Off, LedState::Off) => true,
            _ => false,
        }
    }
}

impl PartialEq for Color {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Color::White, Color::White) => true,
            (Color::Red, Color::Red) => true,
            (Color::Green, Color::Green) => true,
            (Color::Blue, Color::Blue) => true,
            (Color::Yellow, Color::Yellow) => true,
            _ => false,
        }
    }
}

impl PartialEq for Led {
    fn eq(&self, other: &Self) -> bool {
        self.led_state == other.led_state && self.color == other.color && self.percentage == other.percentage
    }
}


impl LedController {

    pub fn new(led_pin : Gpio44,channel: CHANNEL0) -> LedController {
        let (tx, rx) = std::sync::mpsc::channel::<Led>();
        thread:: spawn( move || {
            let mut led = Led {
                led_state: LedState::Off,
                color: Color::White,
                percentage: 0,
            };
            let mut ws2812 = LedController::init_led(led_pin,channel);
            loop {
                led = match rx.recv() {
                    Ok(new_led) => {
                        if new_led == led {
                            thread::sleep(std::time::Duration::from_secs(1));
                            continue;
                        }
                        let (r,g,b) = new_led.color.to_rgb();
                        match new_led.led_state {
                            LedState::On => {
                                info!("Turning LED on: {:?}", new_led);
                                let pixels = (0..NUMBER_OF_LEDS).map(|i|
                                        if i % ((100/led.percentage) as f32).round() as usize == 0 { RGBW8::new_alpha(r, g, b, White(0)) } else { RGBW8::new_alpha(0, 0, 0, White(0)) }
                                );

                                ws2812.write(pixels).unwrap();
                                thread::sleep(std::time::Duration::from_millis(1000));
                            }
                            LedState::Off => {
                                info!("Turning LED off: {:?}", new_led);
                                let pixels = std::iter::repeat(RGBW8::new_alpha(0, 0, 0, White(0))).take(NUMBER_OF_LEDS);
                                ws2812.write(pixels).unwrap();
                            }
                        }
                        new_led
                    }
                    Err(_) => {
                        info!("Error receiving message");
                        continue;
                    }
                };
            }
        });
        LedController {
            tx,
        }
    }

    fn init_led(led_pin : Gpio44,channel: CHANNEL0) -> LedPixelEsp32Rmt<'static,RGBW8,LedPixelColorGrbw32> {
        LedPixelEsp32Rmt::<RGBW8, LedPixelColorGrbw32>::new(channel, led_pin).unwrap()
    }

    pub fn set_led_state(&mut self, new_led : Led) {
        self.tx.send(new_led).unwrap();
    }


}
