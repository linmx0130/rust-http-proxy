use bytes::Bytes;
use indoc::indoc;

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
}

#[derive(Debug)]
pub struct HTTPResponse {
    pub status_code: u32,
    pub body: String
}

impl HTTPResponse{
    fn get_status_description(&self) -> &str{
        match self.status_code {
            501 => "501 Not Implemented",
            _ => panic!("Status code {} not supported", self.status_code)
        }
    }
    fn get_header_text(&self) -> String{
        let mut lines: Vec<String> = Vec::new();
        lines.push(format!("HTTP/1.1 {}", self.get_status_description()));
        lines.push(format!("Status: {}", self.get_status_description()));
        lines.push(String::from("Connection: Close"));
        lines.join("\r\n").to_string()
    }
    pub fn build_message(&self) -> String{
        let mut ret = String::from(self.get_header_text());
        ret.push_str("\r\n\r\n");
        ret.push_str(&self.body);
        ret
    }
    pub fn create_501_error() -> Self {
        HTTPResponse{
            status_code: 501,
            body: String::from(indoc! {"
                <html><body><h1>501 Not Implemented</h1>
                <p>This proxy doesn't support this protocol.</p></body></html>\n"
            })
        }
    }
}

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