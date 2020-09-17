use std::env;
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};
mod request;
mod response;
mod utils;
use response::HTTPResponse;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let port = if args.len() == 1 {
        8080
    } else {
        args.get(1).unwrap().parse::<i32>().unwrap()
    };
    let addr = format!("127.0.0.1:{}", port);
    main_loop(&addr).await;
}

async fn main_loop(addr: &str) {
    println!("HTTP Proxy runs at {}", addr);
    let mut listener = TcpListener::bind(addr).await.unwrap();
    loop {
        let (socket, _) = listener.accept().await.unwrap();
        tokio::spawn(async move {
            process(socket).await;
        });
    }
}

async fn process(mut socket: TcpStream) {
    let http_request = utils::read_http_request(&mut socket).await.unwrap();
    if http_request.path.starts_with("http://") {
        let new_request = http_request.build_request_for_proxy();
        if let Some(resp) = utils::do_request(new_request).await {
            socket.write(&resp.build_message()).await.unwrap();
            println!("Forwarded {}", http_request.path);
        }
    } else if http_request.method == "CONNECT" {
        if let Some(_host) = http_request.get_header_value("Host") {
            let ret = utils::do_connect_request(http_request, &mut socket).await;
            if let Some(addr) = ret {
                println!("Forwarded {}", addr);
            } else {
                println!("An unknown HTTPS request");
            }
        }
    } else {
        println!("Unknown request: {:?}", http_request);
        send_501_error(&mut socket).await;
    }
}

async fn send_501_error(socket: &mut TcpStream) {
    let http_response_content = HTTPResponse::create_501_error().build_message();
    if let Err(err) = socket.write(&http_response_content).await {
        panic!(err);
    }
}
