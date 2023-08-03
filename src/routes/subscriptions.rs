use std::fmt::Display;

use actix_web::{web, HttpResponse, ResponseError};
use anyhow::Context;
use chrono::Utc;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use reqwest::StatusCode;
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::{
    domain::{self, NewSubscriber, SubscriberEmail, SubscriberName},
    email_client::EmailClient,
    startup::ApplicationBaseUrl,
};

pub struct StoreTokenError(sqlx::Error);

impl ResponseError for StoreTokenError {}
impl Display for StoreTokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "A database error was encountered while trying to store a subscription token"
        )
    }
}

impl std::fmt::Debug for StoreTokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // write!(f, "{}\nCaused by:\n\t{}", self, self.0)
        error_chain_fmt(self, f)
    }
}

impl std::error::Error for StoreTokenError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.0)
    }
}

#[derive(thiserror::Error)]
pub enum SubscribeError {
    #[error("{0}")]
    ValidationError(String),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
    // #[error("Failed to store the confirmation token for a new subscriber.")]
    // StoreTokenError(#[from] StoreTokenError),
    // #[error("Failed to send a confirmation email.")]
    // SendEmailError(#[from] reqwest::Error),
    // // DatabaseError(sqlx::Error),
    // #[error("Failed to acquire a postgres connection from the pool.")]
    // PoolError(#[source] sqlx::Error),
    // #[error("Failed to insert new subscription in the database.")]
    // InsertSubscriberError(#[source] sqlx::Error),
    // #[error("Failed to commit SQL transcation to sotre a new subscriber.")]
    // TransactionCommitError(#[source] sqlx::Error),
}
impl std::fmt::Debug for SubscribeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl ResponseError for SubscribeError {
    fn status_code(&self) -> StatusCode {
        match self {
            SubscribeError::ValidationError(_) => StatusCode::BAD_REQUEST,
            SubscribeError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<String> for SubscribeError {
    fn from(value: String) -> Self {
        SubscribeError::ValidationError(value)
    }
}

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

impl TryFrom<FormData> for NewSubscriber {
    type Error = String;

    fn try_from(valur: FormData) -> Result<Self, Self::Error> {
        let name = SubscriberName::parse(valur.name)?;
        let email = SubscriberEmail::parse(valur.email)?;
        Ok(NewSubscriber { name, email })
    }
}

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(form,conn,email_client,base_url),
    fields(
        sub_email = %form.email,
        sub_name  = %form.name
    ),
)]
pub async fn subscribe(
    form: web::Form<FormData>,
    conn: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
    base_url: web::Data<ApplicationBaseUrl>,
) -> Result<HttpResponse, SubscribeError> {
    let new_sub = form.0.try_into()?;
    let mut tx = conn
        .begin()
        .await
        .context("Failed to acquire a Postgres connection from the pool")?;
    // .map_err(|e| SubscribeError::UnexpectedError(Box::new(e)))?;
    let sub_id = insert_subscriber(&new_sub, &mut tx)
        .await
        .context("Failed to insert new subscriber in the database.")?;
    // .map_err(|e| SubscribeError::UnexpectedError(Box::new(e)))?;
        // .map_err(|e| SubscribeError::UnexpectedError(Box::new(e)))?;
    let subscription_token = generate_subscription_token();
    store_token(&mut tx, sub_id, &subscription_token)
        .await
        .context("Failed to acquire a Postgres connection from the pool")?;
        // .map_err(|e| SubscribeError::UnexpectedError(Box::new(e)))?;
    tx.commit()
        .await
        .context("Failed to commit SQL transaction to store a new subscriber.")?;
        // .map_err(|e| SubscribeError::UnexpectedError(Box::new(e)))?;
    send_confirmed_email(&email_client, new_sub, &base_url.0, &subscription_token)
        .await
        .context("Failed to send a confirmation email.")?;
        // .map_err(|e| SubscribeError::UnexpectedError(Box::new(e)))?;
    Ok(HttpResponse::InternalServerError().finish())
}

#[tracing::instrument(name = "Store subscription token in the database", skip(conn, token))]
async fn store_token(
    conn: &mut Transaction<'_, Postgres>,
    sub_id: Uuid,
    token: &str,
) -> Result<(), StoreTokenError> {
    sqlx::query!(
        r#"
    INSERT INTO subscription_tokens (subscription_token, subscriber_id)
    VALUES ($1, $2)
        "#,
        token,
        sub_id,
    )
    .execute(conn)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        StoreTokenError(e)
    })?;
    Ok(())
}

async fn send_confirmed_email(
    email_client: &EmailClient,
    new_sub: domain::NewSubscriber,
    base_url: &str,
    subscription_token: &str,
) -> Result<(), reqwest::Error> {
    let confirmation_link = format!(
        "{}/subscriptions/confirm?subscription_token={}",
        base_url, subscription_token
    );
    let plain_body = format!(
        "Welcome to our newsletter!\nVisit {} to confirm your subscription.",
        confirmation_link
    );
    let html_body = format!(
        "Welcome to our newsletter!<br />Click <a href=\"{}\">here</a> to confirm your subscription.",
        confirmation_link
    );
    email_client
        .send_email(&new_sub.email, "Welcome!", &html_body, &plain_body)
        .await
}

#[tracing::instrument(
    name = "Saving new subscriber detail in the databse.",
    skip(new_sub, conn)
)]
async fn insert_subscriber(
    new_sub: &domain::NewSubscriber,
    conn: &mut Transaction<'_, Postgres>,
) -> Result<Uuid, sqlx::Error> {
    let sub_id = Uuid::new_v4();
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at, status)
        VALUES ($1, $2, $3, $4, 'pending_confirmation')
        "#,
        sub_id,
        new_sub.email.as_ref(),
        new_sub.name.as_ref(),
        Utc::now()
    )
    .execute(conn)
    .await
    .map_err(|e| {
        tracing::error!("Fail to eecute query: {:?}", e);
        e
    })?;

    Ok(sub_id)
}

fn generate_subscription_token() -> String {
    let mut rng = thread_rng();
    std::iter::repeat_with(|| rng.sample(Alphanumeric))
        .map(char::from)
        .take(25)
        .collect()
}

pub fn error_chain_fmt(
    e: &impl std::error::Error,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    writeln!(f, "{}\n", e)?;
    let mut current = e.source();
    while let Some(cause) = current {
        writeln!(f, "Caused by:\n\t{}", cause)?;
        current = cause.source();
    }
    Ok(())
}
