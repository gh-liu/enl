use actix_web::{web, HttpResponse};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
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
    match insert_subscriber(form.into_inner(), conn.get_ref()).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[tracing::instrument(
    name = "Saving new subscriber detail in the databse.",
    skip(form, conn)
)]
async fn insert_subscriber(form: FormData, conn: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        form.email,
        form.name,
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
