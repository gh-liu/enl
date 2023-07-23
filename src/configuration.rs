#[derive(serde::Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application_port: u16,
}

#[derive(serde::Deserialize)]
pub struct DatabaseSettings {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub database_name: String,
}

impl DatabaseSettings {
    pub fn connection_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.user, self.password, self.host, self.port, self.database_name,
        )
    }

    pub fn connection_string_without_db(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}",
            self.user, self.password, self.host, self.port,
        )
    }
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    let cwd = std::env::current_dir().expect("fail to get current dir");
    let config_fir = cwd.join("configuration");

    let settings = config::Config::builder()
        .add_source(config::File::from(config_fir.join("base.yaml")))
        .build()?;

    settings.try_deserialize::<Settings>()
}
