use actix_web::{HttpServer, App, web, HttpResponse, guard};
use crate::repositories::init_repositories;
use sqlx::PgPool;
use crate::constants::DATABASE_URL;
use crate::actors::token_refresher::TokenRefresher;
use actix::Actor;
use actix_files::NamedFile;

mod extractors;
mod models;
mod services;
mod constants;
mod repositories;
mod actors;
mod guards;

async fn web_index() -> std::io::Result<NamedFile> {
    NamedFile::open("web/dist/index.html")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let pool = PgPool::connect(DATABASE_URL).await.expect("Could not connect to database");
    let _refresh_actor = TokenRefresher::new(pool.clone()).start();

    HttpServer::new(move || {
        App::new()
            .data(pool.clone())
            .service(web::scope("/api/v1")
                .configure(init_repositories)
                .default_service(web::route().to(|| HttpResponse::NotFound()))
            )
            .service(actix_files::Files::new("/", "web/dist").index_file("index.html"))
            .default_service(
                web::resource("")
                    .route(web::get().to(web_index))
                    .route(
                        web::route()
                            .guard(guard::Not(guard::Get()))
                            .to(|| HttpResponse::MethodNotAllowed()),
                    )
            )
    })
        .bind("127.0.0.1:8082")?
        .run()
        .await
}
