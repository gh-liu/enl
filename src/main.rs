use enl::configuration::{self, get_configuration};
use enl::startup::Application;
use enl::telemetry::{get_subscriber, init_subscriber};

// #[actix_web::main]
#[tokio::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("enl".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("fail to get configuration");
    let server = Application::build(configuration).await?;
    server.run_until_stopped().await?;
    Ok(())
}
