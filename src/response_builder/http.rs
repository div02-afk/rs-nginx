use std::path::Path;

pub const NOT_FOUND_RESPONSE: &[
    u8;
    90
] = b"HTTP/1.1 404 NOT FOUND\r\nContent-Length: 0\r\nContent-Type: text/plain\r\nConnection: close\r\n\r\n";

pub const BAD_REQUEST_RESPONSE: &[
    u8;
    92
] = b"HTTP/1.1 400 BAD REQUEST\r\nContent-Length: 0\r\nContent-Type: text/plain\r\nConnection: close\r\n\r\n";

pub fn create_response(contents: &[u8], path: &Path) -> String {
    let content_type = get_file_type(path);

    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: {}\r\nConnection: close\r\n\r\n",
        contents.len(),
        content_type
    );

    response
}

pub fn get_file_type(path: &Path) -> String {
    let content_type = if path.extension().and_then(|s| s.to_str()) == Some("html") {
        "text/html"
    } else if path.extension().and_then(|s| s.to_str()) == Some("css") {
        "text/css"
    } else if path.extension().and_then(|s| s.to_str()) == Some("js") {
        "application/javascript"
    } else {
        "application/octet-stream"
    };

    content_type.to_string()
}
