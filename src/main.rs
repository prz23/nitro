#![feature(proc_macro_hygiene, decl_macro)]

use rocket::{get, routes};
use vsock_sample::{vsock_forward_server};

#[get("/")]
fn hello() -> &'static str {
    "Hello, Rust 2018!"
}

fn main() {
    forward_server(5533,"http://0.0.0.0:8000".to_string());
    rocket::ignite().mount("/", routes![hello]).launch();
}

fn forward_server(vsock_port:u32,enclave_url:String){
    std::thread::spawn(move || {
        vsock_forward_server(vsock_port,&enclave_url);
    });
}