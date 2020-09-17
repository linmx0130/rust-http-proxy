use crate::request::HTTPRequest;
use crate::response::HTTPResponse;
use bytes::{Buf, BufMut, BytesMut};
use native_tls;
use native_tls::Identity;
use native_tls::TlsConnector;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio_native_tls::TlsAcceptor;

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
async fn read_http_response<Stream>(socket: &mut Stream) -> Option<HTTPResponse>
where
    Stream: tokio::io::AsyncRead + std::marker::Unpin,
{
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
pub async fn read_http_request<Stream>(socket: &mut Stream) -> Option<HTTPRequest>
where
    Stream: tokio::io::AsyncRead + std::marker::Unpin,
{
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

/// connect request: HTTPS requests
pub async fn do_connect_request(req: HTTPRequest, client_socket: &mut TcpStream) -> Option<String> {
    // return 200 OK to get the real request
    client_socket
        .write(b"HTTP/1.1 200 OK\r\n\r\n")
        .await
        .unwrap();
    // TLS acceptor initialization
    let der = include_bytes!("keyStore.p12");
    let cert = Identity::from_pkcs12(der, "foobar").unwrap();
    let tls_acceptor = TlsAcceptor::from(native_tls::TlsAcceptor::builder(cert).build().unwrap());
    let mut tls_stream = tls_acceptor
        .accept(client_socket)
        .await
        .expect("TLS accept error");

    // the real request for sending to the target server
    let real_req_option = read_http_request(&mut tls_stream).await;
    if let Some(real_req) = real_req_option {
        let addr_str = req.get_header_value("Host").unwrap();
        let addr = lookup_an_address(addr_str).await.unwrap();
        let socket = TcpStream::connect(&addr).await.unwrap();
        let cx = tokio_native_tls::TlsConnector::from(TlsConnector::builder().build().unwrap());
        let target_domain = real_req.get_header_value("Host").unwrap();

        let mut socket = cx.connect(target_domain, socket).await.unwrap();
        socket.write(&real_req.build_message()).await.unwrap();
        let resp = read_http_response(&mut socket).await.unwrap();
        let mut msg = resp.build_message();
        tls_stream.write(&msg.to_bytes()).await.unwrap();
        Some(format!("https://{}{}", target_domain, real_req.path))
    } else {
        None
    }
}
