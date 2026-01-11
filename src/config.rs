use serde::{Serialize, Deserialize};

#[derive(Clone, Deserialize, Serialize, Debug, Default)]
pub struct Config {
    pub input: InputConfig
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct InputConfig {
    pub exchange_name: String,
}

impl Default for InputConfig {
    fn default() -> Self {
        InputConfig { 
            exchange_name: "akari_events".into(),
        }
    }
}

