use std::sync::mpsc::Sender;
use esp_idf_hal::gpio::{ InputPin, InterruptType,  PinDriver, Pull};
use crate::led::Led;

pub fn configure_button_turn_off<C:InputPin>(button_pin:C, tx: Sender<Led>) {
    let mut button = PinDriver::input(button_pin).unwrap();
    // Configure button pin with internal pull up
    button.set_pull(Pull::Up).unwrap();
    // Configure button pin to detect interrupts on a positive edge
    button.set_interrupt_type(InterruptType::PosEdge).unwrap();
    // Attach the ISR to the button interrupt
    unsafe { button.subscribe(move||{
        tx.send(Led { ..Default::default()}).unwrap();
    }).unwrap() }
    // Enable interrupts
    button.enable_interrupt().unwrap();
}

