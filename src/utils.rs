use bytes::{Bytes, Buf, BytesMut, BufMut};
use tokio::net::{TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use crate::request::{HTTPRequest};
use crate::response::{HTTPResponse};

type ChunkedBuffer = [u8;4096];

pub fn parse_http_request(data: &[u8], n: usize) -> Option<HTTPRequest> { 
    let bufs = String::from_utf8(data[..n].to_vec()).unwrap();
    let mut headers = Vec::new();
    let lines: Vec<&str> = bufs.split("\r\n").collect();
    if let Some((command, extra)) = lines.split_first(){
        // offset: the size of command and headers
        let mut offset = command.len() + 2;
        // first line: method path protocol
        let command_split : Vec<&str> = command.split(' ').collect();
        if let [method, path, protocol] = command_split[..3]{
            // analyze the headers
            for line in extra{
                if let Some(idx) = line.find(": "){
                    let key = &line[0..idx];
                    let value = & line[idx+2..];
                    headers.push((key.to_string(), value.to_string()));
                    offset += line.len() + 2;
                }else {
                    break;
                }
            }
            
            return Some(HTTPRequest{
                method: method.to_string(),
                path: path.to_string(),
                protocol: protocol.to_string(),
                headers: headers, 
                body: Bytes::copy_from_slice(&data[offset+2..n])});
        }
    }
    None
}

pub async fn do_request(request: HTTPRequest) -> Option<HTTPResponse> {
    let host = request.get_header_value("Host").unwrap();
    if let Some(addr) = lookup_an_address(host).await {
        let mut socket = TcpStream::connect(addr).await.unwrap();
        let msg = request.build_message();
        if let Err(err) = socket.write(&msg).await {
            println!("{:?}", err);
            return None;
        };
        read_http_response(&mut socket).await
    } else {
        None
    }
}

async fn lookup_an_address(host: &str) -> Option<std::net::SocketAddr>{
    let full_host = if let Some(_) = host.find(":") {
        String::from(host)
    } else {
        format!("{}:80", host)
    };
    let mut addrs = tokio::net::lookup_host(full_host).await.unwrap();
    if let Some(addr) = addrs.next() {
        Some(addr)
    } else {
        None
    }
}

async fn read_http_response(socket: &mut TcpStream) -> Option<HTTPResponse> {
    let mut body_buffer = BytesMut::new();
    loop {
        let mut buffer: ChunkedBuffer = [0; 4096];
        let c_size_option = socket.read(&mut buffer).await;
        if let Ok(c_size) = c_size_option {
            body_buffer.put(&buffer[..c_size]);
            let resp_parsed = HTTPResponse::parse_message(&body_buffer.clone().to_bytes());
            if let Some(resp) = resp_parsed {
                if let Some(size_str) = &resp.get_header_value("Content-Length") {
                    if size_str.parse::<usize>().unwrap() == resp.body.len() {
                        return Some(resp);
                    }
                }
            }
            if c_size == 0 {
                break;
            }
        } else{
            break;
        }
    };
    HTTPResponse::parse_message(&body_buffer.to_bytes())
}
