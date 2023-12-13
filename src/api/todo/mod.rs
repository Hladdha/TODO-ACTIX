use std::sync::Arc;

use actix_web::{web, HttpResponse};

use crate::database::DatabaseManager;
use serde::{Deserialize, Serialize};

use super::*;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.route("/", web::get().to(get_todo))
        .route("/add", web::post().to(add_to_todo));
}

#[derive(Deserialize, Debug, Serialize)]
pub struct UpdateTodo {
    pub task: String,
}

async fn get_todo(req: HttpRequest, db_mgr: web::Data<Arc<DatabaseManager>>) -> HttpResponse {
    let session_token = get_session_token(&req);

    println!("{:?}", session_token);

    let user_id = match session_token {
        Some(session_token) => db_mgr
            .users
            .get_session_token(session_token)
            .await
            .map(|u| u.id),
        None => None,
    };

    if let Some(user_id) = user_id {
        HttpResponse::Ok().json(
        db_mgr.todo.get_user_todo(user_id).await
    )}else {
        ApiResponse::from(ApiError::InternalServerError)
    }
}

async fn add_to_todo(
    req: HttpRequest, 
    db_mgr: web::Data<Arc<DatabaseManager>>,
    payload: web::Json<UpdateTodo>
) -> HttpResponse {
    let session_token = get_session_token(&req);
    let user_id = match session_token {
        Some(session_token) => db_mgr
            .users
            .get_session_token(session_token)
            .await
            .map(|u| u.id),
        None => None,
    };
    db_mgr.todo.add_to_todo(user_id.unwrap(), payload.0.task).await;

    HttpResponse::Ok().json(
        db_mgr.todo.get_user_todo(user_id.unwrap()).await
    )
}