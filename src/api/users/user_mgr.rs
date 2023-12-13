use crate::api::{users::user::*, ApiError};
use crate::database::DatabaseManager;

use actix::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub struct UserManager {
    db: Arc<DatabaseManager>,
}
impl UserManager {
    pub fn new(db: Arc<DatabaseManager>) -> UserManager {
        UserManager {
            db
        }
    }
}

impl Actor for UserManager {
    type Context = Context<Self>;
}

#[derive(Deserialize, Debug, Serialize)]
pub struct UserAuth {
    pub username: String,
    pub password: String,
}

pub mod msg {
    use super::*;
    use crate::api::users::session_token::SessionToken;

    pub struct Register(pub UserAuth);
    impl Message for Register {
        type Result = Result<SessionToken, ApiError>;
    }
    impl Handler<Register> for UserManager {
        type Result = ResponseActFuture<Self, Result<SessionToken, ApiError>>;

        fn handle(&mut self, msg: Register, _ctx: &mut Self::Context) -> Self::Result {
            let auth = msg.0;
            let db = self.db.clone();

            Box::pin(
                async move {
                    let username_is_in_use = db
                        .users
                        .get_username(&auth.username)
                        .await
                        .is_some();

                    if !BackendUserMe::check_password(&auth.password) {
                        Err(ApiError::PasswordInsufficient)
                    } else if username_is_in_use {
                        Err(ApiError::UsernameInUse)
                    } else {
                        let mut user =
                            BackendUserMe::new(auth.username.clone(), auth.password.clone());
                        while db.users.get_id(&user.id).await.is_some() {
                            user.gen_new_id();
                        }
                        db.users.insert(user.clone()).await;
                        db.users
                            .create_session_token(auth)
                            .await
                            .ok_or(ApiError::IncorrectCredentials)
                    }
                }
                .into_actor(self),
            )
            //.map(|res, _, _| res)
            //.boxed_local(ctx)
        }
    }
    pub struct Login(pub UserAuth);
    impl Message for Login {
        type Result = Result<SessionToken, ApiError>;
    }
    impl Handler<Login> for UserManager {
        type Result = ResponseActFuture<Self, Result<SessionToken, ApiError>>;

        fn handle(&mut self, msg: Login, _ctx: &mut Self::Context) -> Self::Result {
            let db = self.db.clone();
            Box::pin(
                async move {
                    db.users
                        .create_session_token(msg.0)
                        .await
                        .ok_or(ApiError::IncorrectCredentials)
                }
                .into_actor(self),
            )
        }
    }

    pub struct Logout(pub SessionToken);
    impl Message for Logout {
        type Result = Result<(), ApiError>;
    }
    impl Handler<Logout> for UserManager {
        type Result = ResponseActFuture<Self, Result<(), ApiError>>;

        fn handle(&mut self, msg: Logout, _ctx: &mut Self::Context) -> Self::Result {
            let db = self.db.clone();
            Box::pin(
                async move {
                    db.users
                        .remove_session_token(msg.0)
                        .await
                        .map_err(|_| ApiError::InternalServerError)
                }
                .into_actor(self),
            )
        }
    }
}