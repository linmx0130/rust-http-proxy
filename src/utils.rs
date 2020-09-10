use bytes::Bytes;

#[derive(Debug)]
pub struct HTTPRequest {
    method: String,
    path: String,
    protocol: String,
    headers: Vec<(String, String)>,
    body: Bytes
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