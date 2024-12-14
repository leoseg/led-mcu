use std::cmp::PartialEq;
use std::sync::mpsc::Sender;
use std::thread;
use esp_idf_hal::gpio::{AnyOutputPin};
use log::info;
use smart_leds_trait::{SmartLedsWrite};
use serde::{Serialize, Deserialize};
use esp_idf_hal::rmt::{CHANNEL0};
use smart_leds::RGB8;
use ws2812_esp32_rmt_driver::driver::color::{LedPixelColorGrb24};
use ws2812_esp32_rmt_driver::{LedPixelEsp32Rmt, Ws2812Esp32Rmt};

const NUMBER_OF_LEDS: usize = 60;

#[derive(Serialize, Deserialize, Debug)]
enum LedState {
    On,
    Rotate,
    Off,
}

/// Struct to represent a Led Setting
#[derive(Serialize, Deserialize, Debug)]
pub struct Led {
    led_state: LedState,
    
    color: Color,

    percentage: u8,

    speed: u8,
}

#[derive(Debug, Clone,Deserialize, Serialize)]
pub enum Color {
    White,
    Red,
    Green,
    Blue,
    Purple,
    Yellow,
}

impl Color {
    pub fn to_rgb(&self) -> (u8, u8, u8) {
        match self {
            Color::White => (255, 255, 255),
            Color::Red => (255, 0, 0),
            Color::Green => (0, 255, 0),
            Color::Blue => (0, 0, 255),
            Color::Purple => (255, 0, 255),
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
            (LedState::Rotate, LedState::Rotate) => true,
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
            (Color::Purple, Color::Purple) => true,
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

    /// Function to initialize Led Controller and start a thread to listen for messages to update Led state
    /// led_pin: AnyOutputPin - GPIO Pin to which the LED is connected
    /// channel: CHANNEL0 - RMT Channel to use for LED is Channel 0
    pub fn new<C:esp_idf_hal::rmt::RmtChannel>(led_pin : AnyOutputPin, channel: C) -> LedController {
        let (tx, rx) = std::sync::mpsc::channel::<Led>();
        thread:: spawn( move || {
            let mut led = Led {
                led_state: LedState::Off,
                color: Color::White,
                percentage: 1,
                speed: 0,
            };
            let mut pixels = vec![RGB8::default(); NUMBER_OF_LEDS];
            let mut ws2812 = LedController::init_led(led_pin,channel);
            ws2812.write(std::iter::repeat(RGB8::default()).take(NUMBER_OF_LEDS)).unwrap();
            loop {
                led = match rx.recv() {
                    Ok(new_led) => {
                        let (r,g,b) = new_led.color.to_rgb();
                        match new_led.led_state {
                            // If led already was on just continue and sleep for 1 second
                            LedState::On => {
                                if(new_led != led) {
                                    info!("Turning LED on: {:?}", new_led);
                                    let step = (100 / new_led.percentage) as usize;
                                    pixels = (0..NUMBER_OF_LEDS)
                                        .map(|i| {
                                            if i % step == 0 {
                                                RGB8::new(r, g, b)
                                            } else {
                                                RGB8::default()
                                            }
                                        })
                                        .collect();
                                    ws2812.write(&pixels).unwrap();
                                }
                                thread::sleep(std::time::Duration::from_millis(1000));
                            }
                            LedState::Rotate => {
                                if(led.led_state != LedState::Rotate || led.percentage != new_led.percentage){
                                    info!("Rotating LED: {:?}", new_led);
                                    let step = (100 / new_led.percentage) as usize;
                                    pixels= (0..NUMBER_OF_LEDS)
                                        .map(|i| {
                                            if i % step == 0 {
                                                RGB8::new(r, g, b)
                                            } else {
                                                RGB8::default()
                                            }
                                    })
                                    .collect();
                                    ws2812.write(&pixels).unwrap();
                                    thread::sleep(std::time::Duration::from_millis(10000/new_led.speed as u64));
                                }else{
                                    pixels.as_mut_slice().rotate_right(1);
                                    // Get all Pixels not set to default (Turned off) and set to given color
                                    let default_rgb = RGB8::default();
                                    pixels.iter_mut().for_each(|pixel| {
                                        if *pixel != default_rgb {
                                            *pixel = new_led.color.to_rgb().into();
                                        }
                                    });
                                    thread::sleep(std::time::Duration::from_millis(10000/new_led.speed as u64));
                                }
                            }
                            LedState::Off => {
                                info!("Turning LED off: {:?}", new_led);
                                ws2812.write(std::iter::repeat(RGB8::default()).take(NUMBER_OF_LEDS)).unwrap();
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

    fn init_led<C: esp_idf_hal::rmt::RmtChannel>(led_pin: AnyOutputPin, channel: C) -> LedPixelEsp32Rmt<'static,RGB8, LedPixelColorGrb24> {
        Ws2812Esp32Rmt::new(channel,led_pin).unwrap()
    }

    pub fn set_led_state(&mut self, new_led : Led) {
        self.tx.send(new_led).unwrap();
    }

}
