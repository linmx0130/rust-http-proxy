use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};
mod request;
mod response;
mod utils;
use response::HTTPResponse;

#[tokio::main]
async fn main() {
    main_loop().await;
}

async fn main_loop() {
    let mut listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
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
