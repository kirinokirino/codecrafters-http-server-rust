use tokio::fs::{read_to_string, try_exists, File};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};

use std::env::args;

#[tokio::main]
async fn main() {
    let args: Vec<String> = args().skip(1).collect();
    let directory = if args.len() == 2 {
        assert!(args[0] == "--directory");
        Some(args[1].clone())
    } else {
        None
    };

    let address = "127.0.0.1:4221";
    let listener = TcpListener::bind(address).await.unwrap();

    loop {
        let (mut stream, _) = listener.accept().await.unwrap();
        let directory = directory.clone();
        println!("Accepted request");
        tokio::spawn(async move {
            handle_connection(stream, directory).await;
        });
    }
}

async fn handle_connection(mut stream: TcpStream, directory: Option<String>) {
    println!("Handling request");
    let mut buffer = [0u8; 1024 * 8];
    let read = stream.read(&mut buffer).await.unwrap();
    assert!(read > 0);
    let request_full = String::from_utf8(buffer[0..read].to_vec()).unwrap();
    let request: Vec<&str> = request_full.lines().collect();

    let (method, path) = parse_header(&request[0]);
    let response = if method == HttpMethod::Post {
        let body: String = request
            .iter()
            .skip_while(|line| !line.is_empty())
            .skip(1)
            .copied()
            .collect();
        let mut file = File::create(format!("{}{path}", directory.unwrap()))
            .await
            .unwrap();
        file.write_all(body.as_bytes()).await;
        "HTTP/1.1 201 CREATED".to_owned()
    } else if path == "/" {
        "HTTP/1.1 200 OK\r\n\r\n".to_owned()
    } else if path.starts_with("/echo/") {
        let echo = path.split("/echo/").skip(1).next().unwrap();
        let ok = "HTTP/1.1 200 OK\r\n";
        let content_type = "Content-Type: text/plain\r\n";
        let content_length = format!("Content-Length: {}\r\n", echo.len());
        format!("{ok}{content_type}{content_length}\r\n{echo}")
    } else if path.starts_with("/user-agent") {
        let user_agent = request
            .iter()
            .find(|line| line.starts_with("User-Agent"))
            .unwrap();
        let user_agent = user_agent.split(' ').skip(1).take(1).next().unwrap();

        let ok = "HTTP/1.1 200 OK\r\n";
        let content_type = "Content-Type: text/plain\r\n";
        let content_length = format!("Content-Length: {}\r\n", user_agent.len());
        format!("{ok}{content_type}{content_length}\r\n{user_agent}")
    } else if path.starts_with("/files/") {
        if let Some(dir) = directory {
            let file_path = path.split("/files/").skip(1).next().unwrap();
            let file_path = format!("{dir}{file_path}");
            println!("file: {file_path}");
            if try_exists(&file_path).await.unwrap() {
                let contents = read_to_string(&file_path).await.unwrap();
                let ok = "HTTP/1.1 200 OK\r\n";
                let content_type = "Content-Type: application/octet-stream\r\n";
                let content_length = format!("Content-Length: {}\r\n", contents.len());
                format!("{ok}{content_type}{content_length}\r\n{contents}")
            } else {
                "HTTP/1.1 404 NOT FOUND\r\n\r\n".to_owned()
            }
        } else {
            "HTTP/1.1 404 NOT FOUND\r\n\r\n".to_owned()
        }
    } else {
        "HTTP/1.1 404 NOT FOUND\r\n\r\n".to_owned()
    };

    stream.write_all(response.as_bytes()).await;
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

#[derive(Debug, Eq, PartialEq)]
enum HttpMethod {
    Get,
    Post,
}

enum HttpStatus {
    Ok,
    NotFound,
}
