#![allow(unused_must_use)]

use std::default::Default;
use std::{env, io};

use actix_cors::Cors;
use actix_web::dev::Service;
use actix_web::web;
use actix_web::{http, App, HttpServer};
use futures::FutureExt;

use utoipa::{
    openapi::security::{ApiKey, ApiKeyValue, SecurityScheme},
    Modify, OpenApi,
};
use utoipa_swagger_ui::SwaggerUi;

mod config;
mod api;
mod constants;
mod error;
mod models;
mod services;
mod utils;
mod schema;
mod middleware;

#[actix_rt::main]
async fn main() -> io::Result<()> {
    dotenv::dotenv().expect("Failed to read .env file");
    env::set_var("RUST_LOG", "actix_web=debug");
    env_logger::init();

    let app_host = env::var("APP_HOST").expect("APP_HOST not found.");
    let app_port = env::var("APP_PORT").expect("APP_PORT not found.");
    let app_url = format!("{}:{}", &app_host, &app_port);
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL not found.");

    let pool = config::db::init_db_pool(&db_url);
    config::db::run_migration(&mut pool.get().unwrap());

    #[derive(OpenApi)]
    #[openapi(
        info(title = "Actix-JWT",
            description = "Actix + JWT + Swagger",
            version = "1.0.0",
            contact(
                name = "Support",
                email = "info@example.com"
            )),
        paths(
            api::account_controller::signup,
        ),
        components(
            schemas(
                models::user::UserDTO,
            )
        ),
        tags(
            (name = "users", description = "User endpoints."),
        ),
        modifiers(&SecurityAddon)
    )]
    struct ApiDoc;

    struct SecurityAddon;

    impl Modify for SecurityAddon {
        fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
            let components = openapi.components.as_mut().unwrap();
            components.add_security_scheme(
                "api_key",
                SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("Authorization"))),
            )
        }
    }

    let openapi = ApiDoc::openapi();


    HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::default() // allowed_origin return access-control-allow-origin: * by default
                    .allowed_origin("http://127.0.0.1:3000")
                    .allowed_origin("http://localhost:3000")
                    .allowed_origin("http://localhost:8000")    // For Swagger
                    .send_wildcard()
                    .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
                    .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
                    .allowed_header(http::header::CONTENT_TYPE)
                    .max_age(3600),
            )
            .app_data(web::Data::new(pool.clone()))
            .wrap(actix_web::middleware::Logger::default())
            .wrap(crate::middleware::auth_middleware::Authentication) // Comment this line if you want to integrate with yew-address-book-frontend
            // .wrap_fn(|req, srv| srv.call(req).map(|res| res))
            .configure(config::app::config_services)
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-docs/openapi.json", openapi.clone()),
            )
    })
    .bind(&app_url)?
    .run()
    .await
}
