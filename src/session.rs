use std::borrow::Cow;
use std::time::{SystemTime, UNIX_EPOCH};

use rocket::State;
use rocket::tokio::sync::Mutex;
use rocket::serde::json::{Json, Value, json, serde_json};
use rocket::serde::{Serialize, Deserialize};

use chrono::prelude::{DateTime, Utc};

#[allow(dead_code)]

// The type to represent the ID of a message.
type Id = usize;

// We're going to store all of the messages here. No need for a DB.
type MessageList = Mutex<Vec<String>>;
type Messages<'r> = &'r State<MessageList>;

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct Message<'r> {
    id: Option<Id>,
    message: Cow<'r, str>
}


/*
 * Observatory main page
 */
#[get("/")]
fn index() -> String {
    format!("Hello, observatory")
}

/*
 * Sessions
 */
#[get("/")]
fn session_index() -> String {
    // format!("Hello, sessions")
    json!({
        "status": "ok",
        "content": "session list goes here."
    }).to_string()
}


fn ticket_get() -> String {
    // format!("Hello, sessions")
    json!({
        "timestamp": "ok",
        "content": "session list goes here."
    }).to_string()
}
/*
 * data
 */




#[post("/", format = "json", data = "<data>")]
async fn new(data: String, list: Messages<'_>) -> Value {
    // let mut list = list.lock().await;
    // let id = list.len();
    // list.push(message.message.to_string());
    // json!({ "status": "ok", "id": id })

    json!({ "status": "ok", "data": data })
}

#[get("/<class>/<version>/<instance>")]
async fn request_session_ticket(class: String, version: String, instance: Id) -> Value {
    let requestor = class.to_string() + ":" + &*version.to_string() + ":" + &*instance.to_string();
    let requestorKey = requestor.to_string();

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs_f32()
        .to_string();

    json!({ "requestor": requestorKey, "timestamp": timestamp })
}

#[post("/<ticket>", format = "json", data = "<message>")]
async fn post_telemetry(ticket: Id, message: String, list: Messages<'_>) -> Option<Value> {

    // Parse the string of data into serde_json::Value.
    let mut record: Value = serde_json::from_str(&*message).ok()?;
    record["ticket"] = Value::from(ticket.to_string());
    println!("Ticket {} temp {}", record["ticket"], record["temp"]);

    Some(json!({ "status": "ok", "ticket":ticket, "data": record.to_string() }))
}

#[put("/<ticket>", format = "json", data = "<message>")]
async fn update(ticket: Id, message: Json<Message<'_>>, list: Messages<'_>) -> Option<Value> {
    match list.lock().await.get_mut(ticket) {
        Some(existing) => {
            *existing = message.message.to_string();
            Some(json!({ "status": "ok",
                "data": message.message.to_string() }))
        }
        None => None
    }
}

#[get("/<id>", format = "json")]
async fn get(id: Id, list: Messages<'_>) -> Option<Json<Message<'_>>> {
    let list = list.lock().await;

    Some(Json(Message {
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
    rocket::fairing::AdHoc::on_ignite("JSON", |rocket| async {
        rocket.mount("/sessions", routes![update, get, request_session_ticket, session_index])
            .mount("/", routes![index])
            .mount("/telemetry", routes![post_telemetry])
            .register("/sessions", catchers![not_found])
            .manage(MessageList::new(vec![]))
    })
}
