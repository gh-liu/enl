use std::net::TcpListener;

fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("fail to bind random port");
    let port = listener.local_addr().unwrap().port();
    let server = enl::run_server(listener).expect("fail to bind address");
    let _ = tokio::spawn(server);

    format!("http://127.0.0.1:{}", port)
}

#[tokio::test]
async fn health_check_ok() {
    let address = spawn_app();

    let client = reqwest::Client::new();

    let resp = client
        .get(&format!("{}/health_check", &address))
        .send()
        .await
        .expect("fail to execute request");

    assert!(resp.status().is_success());
    assert_eq!(Some(0), resp.content_length());
}
