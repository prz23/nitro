#![feature(proc_macro_hygiene, decl_macro)]

use rocket::{get, routes};
use vsock_proxy::starter::Proxy;
use vsock_proxy::starter2::Proxy2;
use std::net::{IpAddr, Ipv4Addr};
use std::io::{Read, Write};
use tempfile::NamedTempFile;
use clap::{App, AppSettings, Arg};
use vsock_sample::server_port;
use std::thread::JoinHandle;

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
        ).arg(
        Arg::with_name("cid")
            .help("cid for client")
            .required(true),
        ).get_matches();

    let proxy_type = matches
        .value_of("proxy_type")
        // This argument is required, so clap ensures it's available
        .unwrap();
    let proxy_type = proxy_type
        .parse::<u16>()
        .map_err(|_| "Remote port is not valid").unwrap();

    let cid = matches
        .value_of("cid")
        // This argument is required, so clap ensures it's available
        .unwrap();
    let cid = cid
        .parse::<u16>()
        .map_err(|_| "Remote port is not valid").unwrap();

    if proxy_type == 1u16{
        forward_server(9000,"http://127.0.0.1:9000".to_string());
        std::thread::sleep(std::time::Duration::from_secs(1));
        rocket::ignite().mount("/", routes![hello]).launch();
    }else if proxy_type == 2u16 {
        instance_server(cid as u32);
    }else if proxy_type == 3u16 {
        rocket::ignite().mount("/", routes![hello]).launch();
    }else if proxy_type == 4u16 {
        forward_server(9000,"http://127.0.0.1:9000".to_string());
    }else if proxy_type == 5u16 {
        forward_server_nix(9000,"http://127.0.0.1:9000".to_string());
    }else if proxy_type == 6u16 {
        start_normal_server();
    }else if proxy_type == 7u16 {
        server_port(9000);
    }else if proxy_type == 8u16 {
        forward_server_nix(9000,"http://127.0.0.1:9000".to_string());
        start_normal_server();
    }else if proxy_type == 9u16 {
        std::thread::spawn(move ||{
            instance_server(cid as u32);
        });
        std::thread::sleep(std::time::Duration::from_secs(1));
        http_client();
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
    let handle = std::thread::spawn(move || {
        loop {
            let _ret = proxy.sock_accept(&listener).expect("proxy accept");
        }
    });
    handle.join().unwrap();
}

fn forward_server_nix(vsock_port:u32,enclave_url:String){
    let addr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)).to_string();
    // let mut file = NamedTempFile::new().unwrap();
    // file.write_all(
    //     b"allowlist:\n\
    //         - {address: 127.0.0.1, port: 9000}",
    // )
    //     .unwrap();
    let proxy = Proxy::new(
        vsock_proxy::starter::VSOCK_PROXY_PORT,
        &addr,
        9000,
        2,
        None, //file.path().to_str(),
        false,
        false,
    ).unwrap();

    // Start proxy in a different thread
    let ret = proxy.sock_listen_nix();
    println!("server sock_listen");
    let listener = ret.expect("proxy listen");
    let handle = std::thread::spawn(move || {
        loop {
            let _ret = proxy.sock_accept_nix(&listener).expect("proxy accept");
        }
    });
}

fn start_normal_server(){
    // Create a listening TCP server on port 9000
    let handle = std::thread::spawn(move || {
        use std::net::{ SocketAddr, TcpStream, TcpListener};
        println!("start_normal_server ");
        let server = TcpListener::bind("127.0.0.1:9000").expect("server bind");
        println!("start_normal_server listen ");

        loop {
            let (mut stream, _) = server.accept().expect("server accept");

            // Read request
            let mut buf = [0; 19];
            //let mut buf = Vec::new();

            stream.read(&mut buf).expect("server read");
            // let msg = str::from_utf8(&buf).expect("from_utf8");
            //assert_eq!(msg, "client2server");
            println!("server_recv {:?}", buf);

            // Write response
            stream.write_all(b"server2client").expect("server write");
        }
    });
    handle.join().unwrap();
}
fn instance_server(cid:u32){
    let addr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)).to_string();
    let mut file = NamedTempFile::new().unwrap();
    file.write_all(
        b"allowlist:\n\
            - {address: 127.0.0.1, port: 9000}",
    )
        .unwrap();
    let proxy = Proxy2::new(
        cid,
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
    loop {
        proxy.sock_accept(&listener).expect("proxy accept");
    }
}

pub fn http_client(){
    let handleb = std::thread::spawn(||{
        let mut stream = std::net::TcpStream::connect("127.0.0.1:9000").unwrap();
        // Write request
        stream.write_all(b"client2server").expect("client write");
        println!("client_send");

        // Read response
        let mut buf = [0; 13];

        stream.read_exact(&mut buf).expect("client read");
        println!("client_recv {:?}",buf);
    });
    handleb.join().unwrap();
}