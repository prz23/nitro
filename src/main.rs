#![feature(proc_macro_hygiene, decl_macro)]

use rocket::{get, routes};
use vsock_proxy::starter::Proxy;
use vsock_proxy::starter2::Proxy2;
use std::net::{IpAddr, Ipv4Addr};
use std::io::{Read, Write};
use tempfile::NamedTempFile;
use clap::{App, AppSettings, Arg};

#[get("/")]
fn hello() -> &'static str {
    "Hello, Rust 2018!"
}

fn main() {
    let matches = App::new("Vsock-TCP proxy")
        .about("Vsock-TCP proxy")
        .setting(AppSettings::DisableVersion)
        .arg(
            Arg::with_name("proxy_type")
                .help("1 enclave or 2 instance")
                .required(true),
        ).get_matches();

    let proxy_type = matches
        .value_of("proxy_type")
        // This argument is required, so clap ensures it's available
        .unwrap();
    let proxy_type = proxy_type
        .parse::<u16>()
        .map_err(|_| "Remote port is not valid").unwrap();

    if proxy_type == 1u16{
        forward_server(9000,"http://127.0.0.1:9000".to_string());
        std::thread::sleep(std::time::Duration::from_secs(1));
        rocket::ignite().mount("/", routes![hello]).launch();
    }else if proxy_type == 2u16 {
        instance_server(9000,"http://127.0.0.1:9000".to_string());
    }else {
        // Create a listening TCP server on port 9000
        std::thread::spawn(move || {
            use std::net::{ SocketAddr, TcpStream, TcpListener};
            let server = TcpListener::bind("127.0.0.1:9000").expect("server bind");
            let (mut stream, _) = server.accept().expect("server accept");

            // Read request
            let mut buf = [0; 19];
            //let mut buf = Vec::new();

            stream.read(&mut buf).expect("server read");
            // let msg = str::from_utf8(&buf).expect("from_utf8");
            //assert_eq!(msg, "client2server");
            println!("server_recv {:?}",buf);

            // Write response
            stream.write_all(b"server2client").expect("server write");
        });
    }
}

fn forward_server(vsock_port:u32,enclave_url:String){
    let addr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)).to_string();
    let mut file = NamedTempFile::new().unwrap();
    file.write_all(
        b"allowlist:\n\
            - {address: 127.0.0.1, port: 9000}",
    )
        .unwrap();
    let proxy = Proxy::new(
        vsock_proxy::starter::VSOCK_PROXY_PORT,
        &addr,
        9000,
        2,
        file.path().to_str(),
        false,
        false,
    ).unwrap();

    // Start proxy in a different thread
    let ret = proxy.sock_listen();
    println!("server sock_listen");
    let listener = ret.expect("proxy listen");
    std::thread::spawn(move || {
        let _ret = proxy.sock_accept(&listener).expect("proxy accept");
    });
}

fn instance_server(vsock_port:u32,enclave_url:String){
    let addr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)).to_string();
    let mut file = NamedTempFile::new().unwrap();
    file.write_all(
        b"allowlist:\n\
            - {address: 127.0.0.1, port: 9000}",
    )
        .unwrap();
    let proxy = Proxy2::new(
        vsock_proxy::starter::VSOCK_PROXY_PORT,
        &addr,
        9000,
        2,
        file.path().to_str(),
        false,
        false,
    ).unwrap();

    // Start proxy in a different thread
    let ret = proxy.sock_listen();
    println!("server sock_listen {:?}",ret);
    let listener = ret.expect("proxy listen");
    let _ret = proxy.sock_accept(&listener).expect("proxy accept");
}