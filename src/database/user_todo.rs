use serde::{Deserialize, Serialize};
use crate::api::users::user::UserId;
use futures::future::OptionFuture;
use mongodb::{
    bson::{doc,self},
    Collection, Database,
};

#[derive(Clone, Serialize, Default, PartialEq, Debug, Eq, Deserialize)]
pub struct Todo {
    list: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct TodoStorage {
    user_id: UserId,
    todo: Todo,
}

impl TodoStorage {
    async fn to_todo_list(
        self,
    ) -> Todo {
        Todo {
            list: self.todo.list,
        }
    }
}

pub struct UserTodo {
    todo: Collection<TodoStorage>,
}

impl UserTodo {
    pub fn new(db: &Database) -> Self {
        UserTodo  {
            todo: db.collection_with_type("users_todo"),
        }
    }

    pub async fn get_user_todo(&self, user_id: UserId) -> Option<Todo> {
        let user: OptionFuture<_> = self
            .todo
            .find_one(doc! {"user_id": user_id.to_string()}, None)
            .await
            .ok()
            .flatten()
            .map(|user| user.to_todo_list())
            .into();
        user.await
    }

    pub async fn add_to_todo(&self, user_id: UserId, task: String) -> bool {
        let mut todo = self.get_user_todo(user_id).await;

        if let Some(mut value) =  todo {
            value.list.push(task);
            todo = Some(value);
        }

        self.todo
            .update_one(
                doc! { "user_id": user_id.to_string()},
                doc! { "$set": bson::to_document(&todo).unwrap() },
                None,
            )
            .await
            .is_ok()
    }
}

