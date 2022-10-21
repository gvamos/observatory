#[macro_use] extern crate rocket;

#[cfg(test)] mod tests;

mod session;
mod msgpack;
mod uuid;

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(session::stage())
        .attach(msgpack::stage())
        .attach(uuid::stage())
}
