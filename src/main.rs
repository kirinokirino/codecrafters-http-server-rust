use std::io::{prelude::*, BufReader};
use std::net::{TcpListener, TcpStream};

fn main() {
    let address = "127.0.0.1:4221";
    let listener = TcpListener::bind(address).unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        handle_connection(stream);
    }
}

fn handle_connection(mut stream: TcpStream) {
    let reader = BufReader::new(&mut stream);
    let request: Vec<String> = reader
        .lines()
        .map(|line| line.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    let (method, path) = parse_header(&request[0]);
    let response = if path == "/" {
        "HTTP/1.1 200 OK\r\n\r\n".to_owned()
    } else if path.starts_with("/echo/") {
        let echo = path.split("/echo").skip(1).next().unwrap();
        let ok = "HTTP/1.1 200 OK\r\n";
        let content_type = "Content-Type: text/plain\r\n";
        let content_length = format!("Content-Length: {}\r\n", echo.len());
        format!("{ok}{content_type}{content_length}\r\n{echo}")
    } else {
        "HTTP/1.1 404 NOT FOUND\r\n\r\n".to_owned()
    };

    stream.write_all(response.as_bytes());
}

fn parse_header(line: &str) -> (HttpMethod, String) {
    let parts: Vec<&str> = line.split_whitespace().take(3).collect();
    assert!(parts[2].starts_with("HTTP"));
    let method = match parts[0] {
        "GET" => HttpMethod::Get,
        "POST" => HttpMethod::Post,
        _ => panic!("Unknown http method"),
    };
    (method, parts[1].to_owned())
}

#[derive(Debug)]
enum HttpMethod {
    Get,
    Post,
}

enum HttpStatus {
    Ok,
    NotFound,
}
