use bytes::Bytes;
use url::Url;
/// The struct for HTTP Request
#[derive(Debug)]
pub struct HTTPRequest {
    /// HTTP method of this request. Generally, it can be GET/POST/PUT...
    ///
    /// See https://developer.mozilla.org/en-US/docs/Web/HTTP/Methods for details.
    pub method: String,
    /// The path of content that is requested.
    pub path: String,
    /// The protocol for this request. For now, only HTTP/1.1 is supported.
    pub protocol: String,
    /// HTTP Headers, represented by key-value pairs
    pub headers: Vec<(String, String)>,
    /// Body of the request
    pub body: Bytes,
}

impl HTTPRequest {
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

    /// Build a message for this HTTPRequest.
    /// The return data can be sent to the server.
    pub fn build_message(&self) -> Bytes {
        let mut ret = String::new();
        let command_line = format!("{} {} {}\r\n", self.method, self.path, self.protocol);
        ret.push_str(&command_line);
        for (key, value) in &self.headers {
            match key.as_str() {
                "Connection" => ret.push_str("Connection: close\r\n"),
                "Proxy-Connection" => {}
                _ => {
                    ret.push_str(&format!("{}: {}\r\n", key, value));
                }
            }
        }
        ret.push_str("\r\n");
        Bytes::from(ret)
    }

    /// Build a new HTTPRequest based on this request as a proxy.
    ///
    /// The new request should be sent to real destination of this request
    /// and indicated the client is a proxy.
    pub fn build_request_for_proxy(&self) -> Self {
        if !self.path.starts_with("http://") {
            panic!("Not a http request for proxy!");
        }
        let path_url = Url::parse(&self.path).unwrap();
        {
            let path = match path_url.query() {
                Some(query) => format!("{}?{}", path_url.path(), query),
                None => String::from(path_url.path()),
            };
            let mut headers = Vec::new();
            let host = self
                .get_header_value("Host")
                .unwrap_or(path_url.host_str().unwrap());
            headers.push((String::from("Host"), String::from(host)));
            for (key, value) in &self.headers {
                match key.as_str() {
                    "Host" => {}
                    "Connection" => {}
                    "Proxy-Connection" => {}
                    _ => {
                        headers.push((key.clone(), value.clone()));
                    }
                }
            }
            headers.push((String::from("Connection"), String::from("close")));
            headers.push((String::from("Proxy-Connection"), String::from("close")));
            HTTPRequest {
                method: String::from(&self.method),
                protocol: String::from("HTTP/1.1"),
                path,
                headers,
                body: self.body.clone(),
            }
        }
    }

    /// Parsing the message of a request to create an HTTPRequest
    pub fn parse_message(buf: &Bytes) -> Option<Self> {
        let mut pointer = 0;
        let mut last_end = 0;
        let mut method = String::new();
        let mut path = String::new();
        let mut protocol = String::new();
        let mut headers = Vec::new();

        while pointer + 1 < buf.len() {
            if buf.get(pointer).unwrap() == &('\r' as u8)
                && buf.get(pointer + 1).unwrap() == &('\n' as u8)
            {
                let new_line = String::from_utf8(buf[last_end..pointer].to_vec()).unwrap();
                last_end = pointer + 2;
                pointer = pointer + 2;
                if new_line.ends_with("HTTP/1.1") {
                    let items: Vec<&str> = new_line.split(' ').collect();
                    method = String::from(*items.get(0).unwrap());
                    path = String::from(*items.get(1).unwrap());
                    protocol = String::from(*items.get(2).unwrap());
                } else if new_line.len() == 0 {
                    break;
                } else {
                    if let Some(spliter) = new_line.find(": ") {
                        let key = String::from(&new_line[..spliter]);
                        let value = String::from(&new_line[spliter + 2..]);
                        headers.push((key, value));
                    } else {
                        return None;
                    }
                }
            } else {
                pointer = pointer + 1;
            }
        }
        if protocol.len() == 0 {
            return None;
        }
        Some(HTTPRequest {
            method: method,
            path: path,
            protocol: protocol,
            headers: headers,
            body: Bytes::copy_from_slice(&buf[last_end..]),
        })
    }
}
