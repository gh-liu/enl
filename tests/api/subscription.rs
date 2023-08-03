use crate::helper::spawn_app;
use wiremock::{
    matchers::{method, path},
    Mock, ResponseTemplate,
};

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    let app = spawn_app().await;

    // let client = reqwest::Client::new();

    let test_cases = vec![
        ("name=allen%20liu", "missing the email"),
        ("email=liughcs%40.com", "missing the name"),
        ("", "missing both the email and email"),
    ];

    for (body, msg) in test_cases {
        let resp = app.post_subscriptions(body.into()).await;
        // let resp = client
        //     .post(format!("{}/subscriptions", &app.address))
        //     .header("Content-Type", "application/x-www-form-urlencoded")
        //     .body(body)
        //     .send()
        //     .await
        //     .expect("fail to execute request.");

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

    let body = "name=allen&email=liughcs%40gmail.com";
    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    let resp = app.post_subscriptions(body.into()).await;

    assert_eq!(resp.status().as_u16(), 200,);
}

#[tokio::test]
async fn subscribe_persists_the_new_subscriber() {
    let app = spawn_app().await;

    let body = "name=allen&email=liughcs%40gmail.com";
    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    let resp = app.post_subscriptions(body.into()).await;

    assert_eq!(resp.status().as_u16(), 200,);

    let saved = sqlx::query!("SELECT email, name, status FROM subscriptions")
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription");

    assert_eq!(saved.name, "allen");
    assert_eq!(saved.email, "liughcs@gmail.com");
    assert_eq!(saved.status, "pending_confirmation");
}

#[tokio::test]
async fn subscribe_sends_a_confirmation_email_for_valid_data() {
    let app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    Mock::given(method("post"))
        .and(path("/email"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    app.post_subscriptions(body.into()).await;
}

#[tokio::test]
async fn subscribe_sends_a_confirmation_email_with_a_link() {
    let app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    Mock::given(method("post"))
        .and(path("/email"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    app.post_subscriptions(body.into()).await;

    let email_req = &app.email_server.received_requests().await.unwrap()[0];
    let links = app.get_confirmation_links(&email_req);

    assert_eq!(links.html, links.plain_text);
}

#[tokio::test]
async fn subscribe_fails_if_there_is_a_fatal_database_error() {
    let app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    let result = sqlx::query!("ALTER TABLE subscription_tokens DROP COLUMN subscription_token;",)
        .execute(&app.db_pool)
        .await
        .unwrap();

    let resp = app.post_subscriptions(body.into()).await;

    assert_eq!(resp.status().as_u16(), 500);
}
