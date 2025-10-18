use std::path::PathBuf;

pub const NOT_FOUND_RESPONSE: &[
    u8;
    90
] = b"HTTP/1.1 404 NOT FOUND\r\nContent-Length: 0\r\nContent-Type: text/plain\r\nConnection: close\r\n\r\n";


pub const BAD_REQUEST_RESPONSE: &[
    u8;
    92
] = b"HTTP/1.1 400 BAD REQUEST\r\nContent-Length: 0\r\nContent-Type: text/plain\r\nConnection: close\r\n\r\n";

pub fn create_response(contents: &Vec<u8>, path: &PathBuf) -> String {
    let content_type = if path.extension().and_then(|s| s.to_str()) == Some("html") {
        "text/html"
    } else if path.extension().and_then(|s| s.to_str()) == Some("css") {
        "text/css"
    } else if path.extension().and_then(|s| s.to_str()) == Some("js") {
        "application/javascript"
    } else {
        "application/octet-stream"
    };

    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: {}\r\nConnection: close\r\n\r\n",
        contents.len(),
        content_type
    );

    return response;
}
