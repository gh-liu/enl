use std::net::TcpListener;

use enl::configuration::get_configuration;
use enl::startup::run_server;
use enl::telemetry::{get_subscriber, init_subscriber};

use sqlx::postgres::PgPoolOptions;

// #[actix_web::main]
#[tokio::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("enl".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("fail to get configuration");
    let listener = TcpListener::bind(format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    ))
    .expect("fail to bind random port");

    let db_pool = PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(configuration.database.with_db());

    run_server(listener, db_pool)?.await
}
