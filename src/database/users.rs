use futures::future::OptionFuture;
use mongodb::{
    bson::doc,
    Collection, Database,
};
use serde::{Deserialize, Serialize};

use crate::
    api::users::{
        session_token::SessionToken,
        user::{BackendUserMe, HashedPassword, UserId},
        user_mgr::UserAuth,
    };

pub struct UserCollection {
    collection: Collection<DbUser>,
}

impl UserCollection {
    pub fn new(db: &Database) -> Self {
        UserCollection {
            collection: db.collection_with_type("users"),
        }
    }

    async fn get_auth(
        &self,
        auth: UserAuth,
    ) -> Option<BackendUserMe> {
        if let Some(user) = self.get_username(&auth.username).await {
            if user.password.matches(&auth.password) {
                return Some(user);
            }
        }
        None
    }

    pub async fn get_id(
        &self,
        id: &UserId,
    ) -> Option<BackendUserMe> {
        let user: OptionFuture<_> = self
            .collection
            .find_one(doc! {"_id": id.to_string()}, None)
            .await
            .ok()
            .flatten()
            .map(|user| user.to_backend_user())
            .into();
        user.await
    }


    pub async fn get_username(
        &self,
        username: &str,
    ) -> Option<BackendUserMe> {
        let user: OptionFuture<_> = self
            .collection
            .find_one(doc! { "username": username }, None)
            .await
            .ok()
            .flatten()
            .map(|user| user.to_backend_user())
            .into();
        user.await
    }

    pub async fn get_session_token(
        &self,
        session_token: SessionToken,
    ) -> Option<BackendUserMe> {
        let user: OptionFuture<_> = self
            .collection
            .find_one(doc! {"session_tokens": session_token.to_string() }, None)
            .await
            .ok()
            .flatten()
            .map(|user| user.to_backend_user())
            .into();
        user.await
    }

    pub async fn create_session_token(
        &self,
        auth: UserAuth,
    ) -> Option<SessionToken> {
        if let Some(user) = self.get_auth(auth).await {
            let session_token = SessionToken::new();
            return self
                .collection
                .update_one(
                    doc! {"username": user.username},
                    doc! { "$push": { "session_tokens": session_token.to_string() } },
                    None,
                )
                .await
                .map(|_| session_token)
                .ok();
        }
        None
    }

    pub async fn remove_session_token(&self, session_token: SessionToken) -> Result<(), ()> {
        let session_token_str = session_token.to_string();
        self.collection
            .update_one(
                doc! { "session_tokens": &session_token_str },
                doc! { "$pull": { "session_tokens": session_token_str } },
                None,
            )
            .await
            .map(|_| ())
            .map_err(|_| ())
    }

    pub async fn insert(&self, user: BackendUserMe) -> bool {
        self.collection
            .insert_one(DbUser::from_backend_user(user), None)
            .await
            .is_ok()
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct DbUser {
    #[serde(rename = "_id")]
    pub id: UserId,
    pub username: String,
    pub password: HashedPassword,

    #[serde(default)]
    pub email: Option<String>,

    #[serde(skip)]
    #[allow(dead_code)]
    pub session_tokens: Vec<SessionToken>,
}

impl DbUser {
    fn from_backend_user(user: BackendUserMe) -> Self {
        DbUser {
            id: user.id,
            username: user.username,
            password: user.password,
            email: user.email,
            session_tokens: vec![],
        }
    }
    async fn to_backend_user(
        self,
    ) -> BackendUserMe {
        BackendUserMe {
            id: self.id,
            username: self.username,
            password: self.password,
            email: self.email,
        }
    }
}