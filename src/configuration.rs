use config::{Config, ConfigError, File, FileFormat};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct AppSettings {
    pub host: String,
    pub port: u16,
}

impl AppSettings {
    pub fn address(&self) -> String {
        format!("{}:{}", &self.host, self.port)
    }
}

#[derive(Deserialize, Debug)]
pub struct AppConfig {
    pub app_settings: AppSettings,
}

impl AppConfig {
    pub fn build(env: String) -> Result<Self, ConfigError> {
        let mut config = Config::builder();
        config = match env.try_into().unwrap() {
            Environment::Production => {
                config.add_source(File::new("config/config.prod.yml", FileFormat::Yaml))
            }
            Environment::Development => {
                config.add_source(File::new("config/config.dev.yml", FileFormat::Yaml))
            }
            Environment::Test => {
                config.add_source(File::new("config/config.dev.yml", FileFormat::Yaml))
            }
        };

        Ok(config.build()?.try_deserialize()?)
    }
}

pub enum Environment {
    Production,
    Development,
    Test,
}

impl TryFrom<String> for Environment {
    type Error = &'static str;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "PRODUCTION" => Ok(Environment::Production),
            "DEVELOPMENT" => Ok(Environment::Development),
            "TEST" => Ok(Environment::Test),
            "" => Err("environment variable missing"),
            _ => Err("environment variable should be either PRODCUTION/TEST/DEVELOPMENT"),
        }
    }
}
