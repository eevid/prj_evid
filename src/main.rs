use serde::{Deserialize, Serialize};

use actix_web::{
    middleware, web, App, HttpRequest, HttpResponse, HttpServer, Responder, Result,
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .wrap(middleware::Logger::default())
            .configure(app_config)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

fn app_config(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("")
            .service(web::resource("/").route(web::get().to(index)))
            .service(web::resource("/register").route(web::post().to(text_register))),
    );
}

async fn index() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(include_str!("../static/index.html")))
}

#[derive(Serialize, Deserialize)]
pub struct RequetParams {
    some_text: String,
}

async fn text_register(req: HttpRequest, params: web::Form<RequetParams>) -> impl Responder {
    println!("リクエスト内容: {:?}", req);

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(format!("あなたが入力した文字は {} です。", params.some_text))
}

#[cfg(test)]
mod tests {
    use super::*;

    use actix_web::body::{Body, ResponseBody};
    use actix_web::dev::{HttpResponseBuilder, Service, ServiceResponse};
    use actix_web::http::{header::CONTENT_TYPE, HeaderValue, StatusCode};
    use actix_web::test::{self, TestRequest};
    use actix_web::web::Form;

    trait BodyTest {
        fn as_str(&self) -> &str;
    }

    impl BodyTest for ResponseBody<Body> {
        fn as_str(&self) -> &str {
            match self {
                ResponseBody::Body(ref b) => match b {
                    Body::Bytes(ref by) => std::str::from_utf8(&by).unwrap(),
                    _ => panic!(),
                },
                ResponseBody::Other(ref b) => match b {
                    Body::Bytes(ref by) => std::str::from_utf8(&by).unwrap(),
                    _ => panic!(),
                },
            }
        }
    }

    #[actix_rt::test]
    async fn text_register_unit_test() {
        let req = TestRequest::default().to_http_request();
        let params = Form(RequetParams {
            some_text: "日本語".to_string(),
        });
        let result = text_register(req.clone(), params).await;
        let resp = match result.respond_to(&req).await {
            Ok(t) => t,
            Err(_) => {
                HttpResponseBuilder::new(StatusCode::INTERNAL_SERVER_ERROR).finish()
            }
        };

        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(
            resp.headers().get(CONTENT_TYPE).unwrap(),
            HeaderValue::from_static("text/html; charset=utf-8")
        );
        assert_eq!(resp.body().as_str(), "あなたが入力した文字は 日本語 です。");
    }

    #[actix_rt::test]
    async fn text_register_integration_test() {
        let mut app = test::init_service(App::new().configure(app_config)).await;
        let req = test::TestRequest::post()
            .uri("/register")
            .set_form(&RequetParams {
                some_text: "日本語だよ".to_string(),
            })
            .to_request();
        let resp: ServiceResponse = app.call(req).await.unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(
            resp.headers().get(CONTENT_TYPE).unwrap(),
            HeaderValue::from_static("text/html; charset=utf-8")
        );
        assert_eq!(resp.response().body().as_str(), "あなたが入力した文字は 日本語だよ です。");
    }
}