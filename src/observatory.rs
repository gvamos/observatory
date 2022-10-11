use std::borrow::Cow;

use rocket::State;
use rocket::tokio::sync::Mutex;
use rocket::serde::json::{Json, Value, json};
use rocket::serde::{Serialize, Deserialize};

// The type to represent the ID of a message.
type Id = usize;

// We're going to store all of the messages here. No need for a DB.
type ReportList = Mutex<Vec<String>>;
type Reports<'r> = &'r State<ReportList>;

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct Report<'r> {
    id: Option<Id>,
    message: Cow<'r, str>
}

#[post("/observatory", format = "json", data = "<message>")]
async fn new(message: Json<Report<'_>>, list: Reports<'_>) -> Value {
    let mut list = list.lock().await;
    let id = list.len();
    list.push(message.message.to_string());
    json!({ "status": "ok", "id": id })
}

#[put("/observatory/<id>", format = "json", data = "<message>")]
async fn update(id: Id, message: Json<Report<'_>>, list: Reports<'_>) -> Option<Value> {
    match list.lock().await.get_mut(id) {
        Some(existing) => {
            *existing = message.message.to_string();
            Some(json!({ "status": "ok" }))
        }
        None => None
    }
}

#[get("/observatory/<id>", format = "json")]
async fn get(id: Id, list: Reports<'_>) -> Option<Json<Report<'_>>> {
    let list = list.lock().await;

    Some(Json(Report {
        id: Some(id),
        message: list.get(id)?.to_string().into(),
    }))
}

#[catch(404)]
fn not_found() -> Value {
    json!({
        "status": "error",
        "reason": "Resource was not found."
    })
}

pub fn stage() -> rocket::fairing::AdHoc {
    rocket::fairing::AdHoc::on_ignite("OBSERVATORY", |rocket| async {
        rocket.mount("/observatory", routes![new, update, get])
            .register("/observatory", catchers![not_found])
            .manage(ReportList::new(vec![]))
    })
}
