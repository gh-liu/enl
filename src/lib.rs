pub mod configuration;
pub mod routes;
pub mod startup;
pub mod telemetry;

// #[actix_web::get("/health_check")]
// async fn health_check() -> impl Responder {
//     HttpResponse::Ok().finish()
// }

// #[cfg(test)]
// mod tests {
//     use crate::health_check;
//     use actix_web::{
//         http::{self, header::ContentType},
//         test,
//     };
//     #[actix_web::test]
//     async fn health_check_ok() {
//         let req = test::TestRequest::default()
//             .insert_header(ContentType::plaintext())
//             .to_http_request();
//         let resp = health_check(req).await;
//         assert_eq!(resp.status(), http::StatusCode::OK);
//     }
// }
