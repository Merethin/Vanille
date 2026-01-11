use serde::{Serialize, Deserialize};

#[derive(Clone, Deserialize, Serialize, Debug, Default)]
pub struct Config {
    pub input: InputConfig,
    pub database: DBConfig
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct InputConfig {
    pub url: String,
    pub exchange_name: String,
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct DBConfig {
    pub url: String,
}

impl Default for InputConfig {
    fn default() -> Self {
        InputConfig { 
            url: "amqp://guest:guest@0.0.0.0:5672".into(),
            exchange_name: "akari_events".into(),
        }
    }
}

impl Default for DBConfig {
    fn default() -> Self {
        DBConfig { 
            url: "postgres://postgres@127.0.0.1".into(),
        }
    }
}
