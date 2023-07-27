use actix_web::{web, HttpResponse};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::{self, NewSubscriber, SubscriberEmail, SubscriberName};

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

impl TryFrom<FormData> for NewSubscriber {
    type Error = String;

    fn try_from(value: FormData) -> Result<Self, Self::Error> {
        let name = SubscriberName::parse(value.name)?;
        let email = SubscriberEmail::parse(value.email)?;
        Ok(NewSubscriber { name, email })
    }
}

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(form,conn),
    fields(
        sub_email = %form.email,
        sub_name  = %form.name
    ),
)]
pub async fn subscribe(form: web::Form<FormData>, conn: web::Data<PgPool>) -> HttpResponse {
    let new_sub = match form.0.try_into() {
        Ok(sub) => sub,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };

    match insert_subscriber(&new_sub, conn.get_ref()).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[tracing::instrument(
    name = "Saving new subscriber detail in the databse.",
    skip(new_sub, conn)
)]
async fn insert_subscriber(
    new_sub: &domain::NewSubscriber,
    conn: &PgPool,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
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

    Ok(())
}
