use std::net::TcpListener;

// #[actix_web::main]
#[tokio::main]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:0").expect("fail to bind random port");
    enl::run_server(listener)?.await
}
