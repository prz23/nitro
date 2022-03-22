#![feature(proc_macro_hygiene, decl_macro)]

use rocket::{get, routes};

#[get("/")]
fn hello() -> &'static str {
    "Hello, Rust 2018!"
}

fn main() {
    forward_server();
    rocket::ignite().mount("/", routes![hello]).launch();
}

fn forward_server(){
    std::thread::spawn(move || { });
}