use std::net::TcpListener;

use sqlx::Connection;
use sqlx::Executor;
use sqlx::PgConnection;
use sqlx::PgPool;

use chrono::Utc;
use once_cell::sync::Lazy;
use uuid::Uuid;

use enl::configuration::{get_configuration, DatabaseSettings};
use enl::startup::run_server;
use enl::telemetry::{get_subscriber, init_subscriber};

static TRACING: Lazy<()> = Lazy::new(|| {
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber("test".into(), "debug".into(), std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber("test".into(), "debug".into(), std::io::sink);
        init_subscriber(subscriber);
    }
});

struct TestApp {
    address: String,
    db_pool: PgPool,
}

async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    let listener = TcpListener::bind("127.0.0.1:0").expect("fail to bind random port");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    let mut configuration = get_configuration().expect("fail to get configuration");
    configuration.database.database_name = Uuid::new_v4().to_string();

    let pool = configure_database(&configuration.database).await;
    let server = run_server(listener, pool.clone()).expect("fail to bind address");
    let _ = tokio::spawn(server);

    TestApp {
        address,
        db_pool: pool,
    }
}

#[tokio::test]
async fn health_check_ok() {
    let app = spawn_app().await;

    let client = reqwest::Client::new();

    let resp = client
        .get(&format!("{}/health_check", &app.address))
        .send()
        .await
        .expect("fail to execute request");

    assert!(resp.status().is_success());
    assert_eq!(Some(0), resp.content_length());

    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        "liughcs@gmail.com".to_string(),
        "allen liu".to_string(),
        Utc::now()
    )
    .execute(&app.db_pool)
    .await
    .expect("fail to insert some data");

    let saved = sqlx::query!("SELECT name, email FROM subscriptions")
        .fetch_one(&app.db_pool)
        .await
        .expect("fail to fetch subscriptions");

    assert_eq!(saved.name, "allen liu");
    assert_eq!(saved.email, "liughcs@gmail.com");
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    let app = spawn_app().await;

    let client = reqwest::Client::new();

    let test_cases = vec![
        ("name=allen%20liu", "missing the email"),
        ("email=liughcs%40.com", "missing the name"),
        ("", "missing both the email and email"),
    ];

    for (body, msg) in test_cases {
        let resp = client
            .post(format!("{}/subscriptions", &app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("fail to execute request.");

        assert_eq!(
            resp.status().as_u16(),
            400,
            "the api did not fail with 400 Bad Request when the payload was {}",
            msg
        );
    }
}

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    let app = spawn_app().await;

    let client = reqwest::Client::new();

    let test_cases = vec![("name=allen&email=liughcs%40gmail.com", "empty name")];

    for (body, desc) in test_cases {
        let resp = client
            .post(format!("{}/subscriptions", &app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("fail to execute request.");

        assert_eq!(
            resp.status().as_u16(),
            200,
            "The API did not return a 200 OK when the payload was {}.",
            desc
        );
    }
}

async fn configure_database(config: &DatabaseSettings) -> PgPool {
    // Create database
    let mut connection = PgConnection::connect_with(&config.without_db())
        .await
        .expect("Failed to connect to Postgres");
    connection
        .execute(&*format!(r#"CREATE DATABASE "{}";"#, config.database_name))
        .await
        .expect("Failed to create database.");

    // Migrate database
    let connection_pool = PgPool::connect_with(config.with_db())
        .await
        .expect("Failed to connect to Postgres.");
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");

    connection_pool
}
