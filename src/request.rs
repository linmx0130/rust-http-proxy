use bytes::{Bytes};
use url::{Url};

#[derive(Debug)]
pub struct HTTPRequest {
    pub method: String,
    pub path: String,
    pub protocol: String,
    pub headers: Vec<(String, String)>,
    pub body: Bytes
}

impl HTTPRequest {
    pub fn get_header_value<'a> (&'a self, key: &str) -> Option<&'a str>{
        for (k, v) in &self.headers {
            if k == key{
                return Some(v.as_ref())
            }
        }
        None
    }

    pub fn build_message(&self) -> Bytes {
        let mut ret = String::new();
        let command_line = format!("{} {} {}\r\n", self.method, self.path, self.protocol);
        ret.push_str(&command_line);
        for (key, value) in &self.headers {
            ret.push_str(&format!("{}: {}\r\n", key, value))
        };
        ret.push_str("\r\n");
        Bytes::from(ret)
    }

    pub fn build_request_for_proxy(&self) -> Self{
        if !self.path.starts_with("http://") {
            panic!("Not a http request for proxy!");
        }
        let path_url = Url::parse(&self.path).unwrap();
        {
            let path = match path_url.query() {
                Some(query) => format!("{}?{}", path_url.path(),query),
                None => String::from(path_url.path())
            };
            let mut headers = Vec::new();
            headers.push((String::from("Host"), String::from(path_url.host_str().unwrap())));
            for (key, value) in &self.headers {
                match key.as_str(){
                    "Host" => {},
                    "Proxy-Connection" => {},
                    _ =>{
                        headers.push((key.clone(), value.clone()));
                    }
                }
            }
            HTTPRequest{
                method: String::from(&self.method),
                protocol: String::from("HTTP/1.1"),
                path,
                headers,
                body: self.body.clone()
            }
        }
    }
}