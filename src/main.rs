// Uncomment this block to pass the first stage
use std::net::TcpListener;

use std::io::{Write, Read};
use std::fmt::{self, Display};

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("accepted new connection");
                let mut buffer = Vec::new();
                stream.read_to_end(&mut buffer).unwrap();
                let _ = stream.write_all(response(HttpCode::Ok).as_bytes());
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

enum HttpCode {
  Ok
}

impl Display for HttpCode {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      HttpCode::Ok => write!(f, "200 OK"),
    }
  }
}

fn response(code: HttpCode) -> String {
  format!("HTTP/1.1 {}\r\n\r\n", code)
}

#[cfg(test)]
mod tests {
  use crate::*;

  #[test]
  fn test_ok_response() {
    assert_eq!("HTTP/1.1 200 OK\r\n\r\n", response(HttpCode::Ok));
  }
}
