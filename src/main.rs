use actix_web::{get, post, web, App, HttpResponse, HttpServer};

use serde::Deserialize;
use serde::Serialize;

use std::sync::Mutex;
use std::vec;

struct AppStateWithCounter {
    counter: Mutex<i32>,
}

struct AllTodoStorage {
    todos:  Mutex<Vec<Todo>>,
}

#[derive(Clone, Serialize, PartialEq, Debug)]
struct Task {
    id: i32,
    task: String,
}

#[derive(Clone, Serialize, Default, PartialEq, Debug)]
struct Todo {
    todo_id: i32,
    list: Vec<Task>,
}

#[derive(Serialize)]
enum ResponseStatus {
    #[serde(rename = "success")]
    Success,
    #[serde(rename = "error")]
    Error,
}

#[derive(Serialize)]
struct ResponseSuccess {
    status: ResponseStatus,
    data: Todo,
}

#[derive(Serialize)]
struct ResponseError {
    status: ResponseStatus,
    message: String,
}

#[derive(Deserialize, Debug)]
struct TodoUpdate {
    title: Option<String>,
}

#[derive(Deserialize, Debug)]
struct TodoDelete {
    id: Option<i32>,
}

async fn get_dummy_data(data: web::Data<AppStateWithCounter>) -> Todo {
    let mut counter = data.counter.lock().unwrap();
    *counter += 1;
    let list : Todo = Todo{
        todo_id: counter.abs(),
        list: vec![Task {
            id: 1,
            task: String::from("First Task")}
        ],
    };

    list
}

#[get("/todos")]
async fn get_todo(data: web::Data<AppStateWithCounter>, todo_lists: web::Data<AllTodoStorage>) -> HttpResponse {
    let list = get_dummy_data(data).await;
    let mut todos = todo_lists.todos.lock().unwrap();
    let mut new_todo = vec![list.clone()];
    for todo in todos.iter() {
        new_todo.push(todo.clone());
    }

    *todos = new_todo;
    
    HttpResponse::Ok().json(ResponseSuccess {
        status: ResponseStatus::Success,
        data: list,
    })
}

#[get("/todo/{id}")]
async fn get_todo_by_id(path: web::Path<i32>, data: web::Data<AllTodoStorage>) -> HttpResponse {
    let todos = data.todos.lock().unwrap();

    for todo in todos.iter() {
        if todo.todo_id == path.abs() {
            return HttpResponse::Ok().json(ResponseSuccess {
                status: ResponseStatus::Success,
                data: todo.clone(),
            })
        }
    }
    HttpResponse::Ok().json(ResponseError {
        status: ResponseStatus::Error,
        message: String::from("No Todo for given ID create Your Todo!")
    })
}


#[post("/todo/add/{id}")]
async fn add_todo_by_id(path: web::Path<i32>, payload: web::Json<TodoUpdate>, data: web::Data<AllTodoStorage>) -> HttpResponse {
    let mut todos = data.todos.lock().unwrap();
    let mut list: Todo =  Todo::default();
    let mut new_todo = vec![];

    for todo in todos.iter() {
        if todo.todo_id == path.abs() {
            list = todo.clone();
        }else {
            new_todo.push(todo.clone());
        }
    }

    if list == Todo::default() {
        return HttpResponse::Ok().json(ResponseError {
            status: ResponseStatus::Error,
            message: String::from("No Todo for given ID create Your Todo!")
        })
    }

    let mut task = String::new();
    if let Some(title) = &payload.title {
        task = title.to_string();
    }

    let mut tasks = list.list;
    let last_index = tasks.last().unwrap().id;
    let new_task = Task{
        id: last_index+1,
        task: task
    };
    tasks.push(new_task);

    let todo = Todo{
        todo_id: path.abs(),
        list: tasks,
    };

    new_todo.push(todo.clone());
    *todos = new_todo;

    return HttpResponse::Ok().json(ResponseSuccess {
        status: ResponseStatus::Success,
        data: todo,
    })
}


#[get("/todo/delete/{id}")]
async fn delete_todo_by_id(path: web::Path<i32>, data: web::Data<AllTodoStorage>, payload: web::Json<TodoDelete>) -> HttpResponse {
    let mut todos = data.todos.lock().unwrap();
    let mut list: Todo =  Todo::default();
    let mut new_todo = vec![];

    for todo in todos.iter() {
        if todo.todo_id == path.abs() {
            list = todo.clone();
        }else {
            new_todo.push(todo.clone());
        }
    }

    if list == Todo::default() {
        return HttpResponse::Ok().json(ResponseError {
            status: ResponseStatus::Error,
            message: String::from("No Todo for given ID create Your Todo!")
        })
    }

    let mut tasks = list.list;
    let mut i =0;
    for task in tasks.to_vec() {
        if Some(task.id) == payload.id {
            tasks.remove(i);
        }
        i+=1;
    }

    let todo = Todo{
        todo_id: path.abs(),
        list: tasks,
    };

    new_todo.push(todo.clone());
    *todos = new_todo;

    HttpResponse::Ok().json(ResponseSuccess {
        status: ResponseStatus::Success,
        data: todo,
    })
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let counter = web::Data::new(AppStateWithCounter {
        counter: Mutex::new(0),
    });

    let todos =  web::Data::new(AllTodoStorage {
        todos: Mutex::new(Vec::new()),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(counter.clone())
            .app_data(todos.clone())
            .service(get_todo)
            .service(get_todo_by_id)
            .service(add_todo_by_id)
            .service(delete_todo_by_id)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}