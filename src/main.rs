#[macro_use] extern crate rocket;

#[cfg(test)] mod tests;

mod json;
mod msgpack;
mod uuid;
mod observatory;

#[launch]
fn rocket() -> _ {
    rocket::build()
        // .attach(json::stage())
        .attach(msgpack::stage())
        .attach(uuid::stage())
        .attach(observatory::stage())
}
