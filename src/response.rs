use bytes::{BufMut, Bytes, BytesMut};
use indoc::indoc;

/// The struct for HTTP Response
#[derive(Debug)]
pub struct HTTPResponse {
    /// Status code of the response
    pub status_code: u32,
    /// HTTP Headers, represented by key-value pairs
    pub headers: Vec<(String, String)>,
    /// Body of the response
    pub body: Bytes,
}

impl HTTPResponse {
    /// Get the value of a specific key in the headers.
    ///
    /// Return `None` if the key isn't found.
    pub fn get_header_value<'a>(&'a self, key: &str) -> Option<&'a str> {
        for (k, v) in &self.headers {
            if k == key {
                return Some(v.as_ref());
            }
        }
        None
    }

    /// Map the status code to its description
    fn get_status_description(&self) -> &str {
        match self.status_code {
            100 => "100 Continue",
            200 => "200 OK",
            301 => "301 Moved Permanently",
            304 => "304 Not Modified",
            400 => "400 Bad Request",
            401 => "401 Unauthorized",
            403 => "403 Forbidden",
            404 => "404 Not Found",
            405 => "405 Method Not Allowed",
            418 => "418 I'm a teapot",
            500 => "500 Internal Server Error",
            501 => "501 Not Implemented",
            502 => "502 Bad Gateway",
            _ => panic!("Status code {} not supported", self.status_code),
        }
    }

    /// Build HTTPResponse header part message
    fn get_header_text(&self) -> String {
        let mut lines: Vec<String> = Vec::new();
        lines.push(format!("HTTP/1.1 {}", self.get_status_description()));
        for (key, value) in &self.headers {
            if key == "Connection" {
                lines.push(String::from("Connection: close"));
            } else {
                lines.push(format!("{}: {}", key, value));
            }
        }
        lines.join("\r\n").to_string()
    }

    /// Build a message for this HTTPResponse.
    /// The return data can be sent to the client.
    pub fn build_message(&self) -> Bytes {
        let mut ret = String::from(self.get_header_text());
        ret.push_str("\r\n\r\n");
        let mut buf = BytesMut::new();
        buf.put(ret.as_bytes());
        buf.put(self.body.as_ref());
        Bytes::from(buf)
    }

    /// Create a 501 Not Implemented response.
    pub fn create_501_error() -> Self {
        let headers = vec![(String::from("Connection"), String::from("close"))];
        HTTPResponse {
            status_code: 501,
            body: Bytes::from(indoc! {"
                <html><body><h1>501 Not Implemented</h1>
                <p>This proxy doesn't support this protocol.</p></body></html>\n"
            }),
            headers,
        }
    }

    /// Parse HTTP response message to this struct
    pub fn parse_message(buf: &Bytes) -> Option<Self> {
        let mut pointer = 0;
        let mut last_end = 0;
        let mut status_code = 0;
        let mut headers = Vec::new();
        while pointer + 1 < buf.len() {
            if buf.get(pointer) == Some(&('\r' as u8))
                && buf.get(pointer + 1) == Some(&('\n' as u8))
            {
                let new_line = String::from_utf8(buf[last_end..pointer].to_vec()).unwrap();
                last_end = pointer + 2;
                pointer = pointer + 2;
                if new_line.starts_with("HTTP/1.1") {
                    let items: Vec<&str> = new_line.split(' ').collect();
                    status_code = items.get(1).unwrap().to_string().parse::<u32>().unwrap();
                } else if new_line.len() == 0 {
                    break;
                } else {
                    let spliter = new_line.find(": ").unwrap();
                    let key = String::from(&new_line[..spliter]);
                    let value = String::from(&new_line[spliter + 2..]);
                    headers.push((key, value));
                }
            } else {
                pointer = pointer + 1;
            }
        }
        if status_code != 0 {
            Some(HTTPResponse {
                status_code,
                headers,
                body: Bytes::copy_from_slice(&buf[last_end..]),
            })
        } else {
            None
        }
    }
}
