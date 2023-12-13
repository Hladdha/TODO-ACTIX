pub mod session_token;
pub mod user;
pub mod user_mgr;

use actix::Addr;
use actix_web::*;
use HttpResponse as HR;
use super::*;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.route("/register", web::post().to(register))
        .route("/login", web::post().to(login))
        .route("/logout", web::post().to(logout));
}

async fn register(
    user_mgr: web::Data<Addr<user_mgr::UserManager>>,
    payload: web::Form<user_mgr::UserAuth>,
) -> HttpResponse {
    match user_mgr
        .send(user_mgr::msg::Register(payload.into_inner()))
        .await
    {
        Ok(Ok(session_token)) => HR::Ok().cookie(
            actix_web::cookie::Cookie::build("", session_token.to_string())
                .http_only(true)
                .path("/api")
                .finish(),
        ).json(ApiResponse::with_content(
            "Registration successful.",
            session_token,
        )),
        Ok(Err(api_err)) => ApiResponse::from(api_err),
        Err(_) => ApiResponse::from(ApiError::InternalServerError),
    }
}

async fn login(
    user_mgr: web::Data<Addr<user_mgr::UserManager>>,
    payload: web::Form<user_mgr::UserAuth>,
) -> HttpResponse {
    match user_mgr
        .send(user_mgr::msg::Login(payload.into_inner()))
        .await
    {
        Ok(Ok(session_token)) => HR::Ok().cookie(
            actix_web::cookie::Cookie::build("", session_token.to_string())
                .http_only(true)
                .path("/api")
                .finish(),
        ).json(ApiResponse::with_content(
            "Login successful.",
            session_token,
        )),
        Ok(Err(api_err)) => ApiResponse::from(api_err),
        Err(_) => ApiResponse::from(ApiError::InternalServerError),
    }
}

async fn logout(
    req: HttpRequest,
    user_mgr: web::Data<Addr<user_mgr::UserManager>>,
) -> HttpResponse {
    match get_session_token(&req) {
        Some(session_token) => match user_mgr.send(user_mgr::msg::Logout(session_token)).await {
            Ok(Ok(_)) => HR::Ok().json(ApiResponse::new("Logout successful.")),
            Ok(Err(api_err)) => ApiResponse::from(api_err),
            _ => ApiResponse::from(ApiError::InternalServerError),
        },
        None => ApiResponse::from(ApiError::MissingSessionToken),
    }
}



