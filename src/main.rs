mod api;
mod database;

use std::sync::Arc;

use actix::Actor;
use actix_cors::Cors;
use actix_web::{middleware, web, App, HttpServer};

use api::users::user_mgr::UserManager;
use database::DatabaseManager;


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let db_mgr = Arc::new(DatabaseManager::new().await);
    let user_mgr_addr = UserManager::new(db_mgr.clone()).start();

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .data(db_mgr.clone())
            .data(user_mgr_addr.clone())
            .service(
                web::scope("/api")
                    .wrap(
                        Cors::default()
                            .allowed_origin("localhost")
                            .allowed_methods(vec!["GET", "POST", "DELETE"])
                            .allow_any_header()
                            .max_age(3600),
                    )
                    .configure(api::config),
            )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}


// .service(get_todo)
//             .service(get_todo_by_id)
//             .service(add_todo_by_id)
//             .service(delete_todo_by_id)