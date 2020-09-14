use crate::request::HTTPRequest;
use crate::response::HTTPResponse;
use bytes::{Buf, BufMut, BytesMut};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

type ChunkedBuffer = [u8; 4096];

/// Send the request to its destination and return the response it received.
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

/// Find the address of the host.
async fn lookup_an_address(host: &str) -> Option<std::net::SocketAddr> {
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

/// Read HTTP response from the socket.
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
        } else {
            break;
        }
    }
    HTTPResponse::parse_message(&body_buffer.to_bytes())
}

/// Read HTTP request from the socket.
pub async fn read_http_request(socket: &mut TcpStream) -> Option<HTTPRequest> {
    let mut buffer = BytesMut::new();
    loop {
        let mut chuncked_buffer: ChunkedBuffer = [0; 4096];
        if let Ok(c_size) = socket.read(&mut chuncked_buffer).await {
            buffer.put(&chuncked_buffer[..c_size]);
            let req_parsed = HTTPRequest::parse_message(&buffer.clone().to_bytes());
            if let Some(req) = req_parsed {
                if let Some(size_str) = &req.get_header_value("Content-Length") {
                    if size_str.parse::<usize>().unwrap() == req.body.len() {
                        return Some(req);
                    }
                } else {
                    // no content length found, directly return the valid request
                    break;
                }
            }
            if c_size == 0 {
                break;
            }
        } else {
            break;
        }
    }
    HTTPRequest::parse_message(&buffer.to_bytes())
}
