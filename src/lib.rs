use std::net::TcpListener;

use actix_web::dev::Server;
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer};

// #[actix_web::get("/health_check")]
// async fn health_check() -> impl Responder {
//     HttpResponse::Ok().finish()
// }

async fn health_check(_req: HttpRequest) -> HttpResponse {
    HttpResponse::Ok().finish()
}

pub fn run_server(listener: TcpListener) -> Result<Server, std::io::Error> {
    // HttpServer::new(|| App::new().service(health_check))
    let server = HttpServer::new(|| App::new().route("/health_check", web::get().to(health_check)))
        .listen(listener)?
        .run();

    Ok(server)
}

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
