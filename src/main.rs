use std::net::TcpListener;

use enl::configuration::get_configuration;
use enl::startup::run_server;
use sqlx::PgPool;

// #[actix_web::main]
#[tokio::main]
async fn main() -> std::io::Result<()> {
    let configuration = get_configuration().expect("fail to get configuration");
    let listener = TcpListener::bind(format!("127.0.0.1:{}", configuration.application_port))
        .expect("fail to bind random port");

    let connetcion_str = configuration.database.connection_string();
    let db_pool = PgPool::connect(&connetcion_str)
        .await
        .expect("fail to connect postgres");

    run_server(listener, db_pool)?.await
}
