use std::{io::{BufRead, BufReader, Write}, net::{TcpListener, TcpStream}};

const PORT: &str = "6969";
const LOCALHOST: &str = "127.0.0.1";

fn main() {
   println!("Starting Single-threaded TCP Server...");

   let addr = LOCALHOST.to_string() + ":" + PORT;
   let listener = TcpListener::bind(&addr);

   match listener {
    Ok(listener) => {
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    handle_connection(stream);
                }
                Err(_) => { /* connection failed */ }
            } 
        } 
    },
    Err(err) => {
        println!("Failed to bind to {}. See: {}", addr, err);
    },
   }
}

fn handle_connection(mut stream: TcpStream) {
    println!("Connection established with {}", stream.peer_addr().unwrap());

    let reader = BufReader::new(&stream);

    let mut lines_itr = reader.lines();
    let request_info = match lines_itr.next() {
        Some(info) => {
            match info {
                Ok(ok) => ok,
                Err(_) => {
                    println!("Failed to read request info");
                    return;
                },
            }
        }
        None => {
            println!("Failed to read request info");
            return;
        },
    };

    let mut info_parts = request_info.split_whitespace();
    let method = info_parts.next().unwrap();
    let path = info_parts.next().unwrap();
    let proto = info_parts.next().unwrap();

    println!("[INFO] {method} {path} ({proto})");

    if !method.eq_ignore_ascii_case("GET") {
        stream.write_all("HTTP/1.1 405 Method Not Allowed\r\n\r\n".as_bytes()).unwrap();
        return;
    }

    let (response, code) = match path {
        "/" => {
            ("{\"status\": \"ok\"}", "200 OK")
        },
        "/index.html" => {
            ("<html><p>hi</p></html>", "200 OK")
        },
        _ => {
            ("", "404 NOT FOUND")
        },
    };

    let content_length = response.len().to_string();
    let resp = format!("HTTP/1.1 {code}\r\nContent-Length: {content_length}\r\n\r\n{response}");

    match stream.write_all(resp.as_bytes()) {
        Ok(response) => {
            println!("Response: {:#?}", response);
        },
        Err(err) => {
            println!("Failed to write to stream. See: {}", err);
        },
    }
}
