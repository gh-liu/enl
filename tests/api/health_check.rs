use crate::helper::spawn_app;
use chrono::Utc;
use uuid::Uuid;

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
        INSERT INTO subscriptions (id, email, name, subscribed_at, status)
        VALUES ($1, $2, $3, $4, 'confirmed')
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
