use std::{io::{BufRead, BufReader, Write}, net::{TcpListener, TcpStream}, thread, time::Duration};

const PORT: &str = "6969";
const LOCALHOST: &str = "127.0.0.1";

struct ThreadWorker {
    id: usize,
    thread: thread::JoinHandle<()>
}

impl ThreadWorker {
    pub fn new(id: usize) -> Self {
        let thread = thread::spawn(|| {});

        ThreadWorker { id, thread }
    }
}

struct ThreadPool {
    workers: Vec<ThreadWorker>
}

impl ThreadPool {
    pub fn new(worker_count: usize) -> Self {
            let mut workers = Vec::with_capacity(worker_count);

            for i in 0..worker_count {
                workers.push(ThreadWorker::new(i));
            }

        ThreadPool {
            workers
        }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static {
         
    }
}

fn main() {
   println!("Starting Multi-threaded TCP Server...");

   let addr = LOCALHOST.to_string() + ":" + PORT;
   let listener = TcpListener::bind(&addr);
   let pool = ThreadPool::new(3);

   match listener {
    Ok(listener) => {
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {

                    pool.execute(|| {
                        handle_connection(stream);
                    });
                }
                Err(e) => { /* connection failed */ }
            } 
        } 
    },
    Err(err) => {
        println!("Failed to bind to {}. See: {}", addr, err);
        return;
    },
   }
}

const OK: &str = "HTTP/1.1 200 OK\r\n\r\n";

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
        "/slow" => {
            thread::sleep(Duration::from_secs(10));
            ("", "200 OK")
        }
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
            return;
        },
    }
}

