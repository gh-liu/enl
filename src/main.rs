use std::net::TcpListener;

use enl::configuration::get_configuration;
use enl::startup::run_server;
use enl::telemetry::{get_subscriber, init_subscriber};

use secrecy::ExposeSecret;
use sqlx::PgPool;

// #[actix_web::main]
#[tokio::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("enl".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("fail to get configuration");
    let listener = TcpListener::bind(format!("127.0.0.1:{}", configuration.application_port))
        .expect("fail to bind random port");

    let db_pool = PgPool::connect(configuration.database.connection_string().expose_secret())
        .await
        .expect("fail to connect postgres");

    run_server(listener, db_pool)?.await
}
