use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use url::{Url};
mod utils;
use utils::{HTTPResponse};

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
    let mut buffer: [u8;512] = [0; 512];
    let n = socket.read(&mut buffer[..]).await.unwrap();
    println!("Received {} byte", n);
    let http_request = utils::parse_http_request(&buffer, n).unwrap();
    if http_request.path.starts_with("http://") {
        let path_url = Url::parse(&http_request.path).unwrap();
        println!("{:?}", path_url);
        send_501_error(&mut socket).await;
    } else{
        println!("{:?}", http_request);
        send_501_error(&mut socket).await;
    }
}

async fn send_501_error(socket: &mut TcpStream) {
    let http_response_content = HTTPResponse::create_501_error().build_message();
    if let Err(err) = socket.write(http_response_content.as_bytes()).await{
        panic!(err);
    }
}
