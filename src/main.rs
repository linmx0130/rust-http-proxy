use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt};
mod utils;

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
    let host = http_request.get_header_value("Host").unwrap();
    println!("{}", host);
}
