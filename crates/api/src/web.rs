use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::fs;

// Starts web server and returns first request.
pub fn get_request() -> String {
    let listener = TcpListener::bind("127.0.0.1:6767").unwrap();
    for stream in listener.incoming() {
        let mut stream = stream.unwrap();
        let mut buffer = [0; 1024];
        stream.read(&mut buffer).unwrap();
        let contents = get_html_response();
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}",
            contents.len(),
            contents
        );
        stream.write(response.as_bytes()).unwrap();
        stream.flush().unwrap();
        let x = String::from_utf8_lossy(&buffer[..]);
        let string = x.to_string();
        return string;
    }
    panic!("Failed to return request");
}

fn get_html_response() -> String {
    "\
    <html>\
    <head>\
        <title>dashboard</title>\
    </head>\
    <body>\
        <h1>You may now close this window</h1>\
    </body>\
    </html>\
    ".to_string()
}
