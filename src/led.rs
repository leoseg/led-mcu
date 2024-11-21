use serde::{Serialize, Deserialize, Deserializer};


#[derive(Serialize, Deserialize, Debug)]
enum LedState {
    On,
    Off,
}


#[derive(Serialize, Deserialize, Debug)]
pub struct Led {
    led_state: LedState,
    
    color: String,

    percentage: u8,
}

impl Led {
    
    pub fn set_led_state(&mut self) {
        println!("Setting LED state to {:?}", self.led_state);
    }
}
