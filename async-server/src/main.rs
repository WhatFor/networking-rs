
use std::{io::{BufRead, BufReader, Write}, net::{TcpListener, TcpStream}, sync::{mpsc::{self, SendError}, Arc, Mutex}, thread, time::Duration};

const PORT: &str = "6969";
const LOCALHOST: &str = "127.0.0.1";

struct ThreadWorker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>
}

impl ThreadWorker {
    pub fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Self {
        let thread = Some(thread::spawn(move || loop {
            let job = receiver.lock().unwrap().recv();

            match job {
                Ok(job) => {
                    println!("Got job on thread {id}");
                    job();
                },
                Err(_) => {
                    println!("Thread {id} closing...");
                    break;
                },
            }
        }));

        ThreadWorker { id, thread }
    }
}

type Job = Box<dyn FnOnce() + Send + 'static>;

struct ThreadPool {
    workers: Vec<ThreadWorker>,
    sender: Option<mpsc::Sender<Job>>
}

impl ThreadPool {
    pub fn new(worker_count: usize) -> Self {
            let mut workers = Vec::with_capacity(worker_count);
            let (sender, receiver) = mpsc::channel();
            let receiver = Arc::new(Mutex::new(receiver));

            for i in 0..worker_count {
                workers.push(ThreadWorker::new(i, Arc::clone(&receiver)));
            }

        ThreadPool {
            workers,
            sender: Some(sender),
        }
    }

    pub fn execute<F>(&self, f: F) -> Result<(), SendError<Job>>
    where
        F: FnOnce() + Send + 'static {
         let job = Box::new(f);

         self.sender.as_ref().unwrap().send(job)
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {

        drop(self.sender.take());

        for worker in &mut self.workers {
            println!("Shutting down worker {}...", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
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

                    let execute =  pool.execute(|| {
                        handle_connection(stream);
                    });

                    if let Err(e) = execute {
                        println!("Failed to handle connection. See: {}", e);
                    }
                }
                Err(e) => { 
                    println!("Connection failed: {e}");
                 }
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
        },
    }
}


