use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
mod utils;
mod request;
mod response;
use response::{HTTPResponse};

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

async fn process(mut socket: TcpStream){
    let mut buffer: [u8;4096] = [0; 4096];
    let n = socket.read(&mut buffer[..]).await.unwrap();
    let http_request = utils::parse_http_request(&buffer, n).unwrap();
    if http_request.path.starts_with("http://") {
        let new_request = http_request.build_request_for_proxy();
        if let Some(resp) = utils::do_request(new_request).await {
            socket.write(&resp.build_message()).await.unwrap();
            println!("Forwarded {}", http_request.path);
        }
        
    } else{
        println!("Unknown request: {:?}", http_request);
        send_501_error(&mut socket).await;
    }
}

async fn send_501_error(socket: &mut TcpStream) {
    let http_response_content = HTTPResponse::create_501_error().build_message();
    if let Err(err) = socket.write(&http_response_content).await{
        panic!(err);
    }
}
