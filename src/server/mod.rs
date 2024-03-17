use std::net::{TcpListener, TcpStream};
use std::io::{BufRead, Read, Write};
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use lazy_static::lazy_static;

mod user;

lazy_static! {
    static ref CONNS: Arc<Mutex<Vec<TcpStream>>> = Arc::new(Mutex::new(Vec::new()));
}
// a simple server, allowing concurrent connections. each time one client sends a message, it is broadcasted to all other clients.
pub struct Server {
    address: String,
}

impl Server {
    pub fn new(address: &str) -> Server {
        Server {
            address: address.to_string(),
        }
    }

    pub fn run(&mut self) {
        let listener = TcpListener::bind(&self.address).unwrap();
        let (tx, rx) = mpsc::channel::<String>();

        // start a new thread to handle incoming connections
        thread::spawn(move || {
            for stream in listener.incoming() {
                let tx = tx.clone();
                match stream {
                    Ok(stream) => {
                        thread::spawn(move || {
                            CONNS.lock().unwrap().push(stream.try_clone().unwrap());
                            handle_client(stream, &tx);
                        });
                    }
                    Err(e) => {
                        println!("Error: {}", e);
                    }
                }
            }
        });

        // start a new thread to handle messages from clients
        loop {
            match rx.recv() {
                Ok(msg) => {
                    // all messages are in the form "username/password/message"
                    let parts: Vec<&str> = msg.split('/').collect();

                    if parts.len() != 3 {
                        continue;
                    }

                    let uname = parts[0];
                    let pass = parts[1];
                    let msg = parts[2];

                    println!("parts: {:?}", parts);

                    if !user::check_pw(uname, pass).unwrap() {
                        println!("Invalid username/password");
                        continue;
                    }

                    for conn in CONNS.lock().unwrap().iter_mut() {
                        let _ = conn.write_all(format!("{}: {}\n", uname, msg).as_bytes());
                    }
                }
                Err(e) => {
                    println!("Error: {}", e);
                    break;
                }
            }
        }
    }
}

fn handle_client(mut stream: TcpStream, tx: &Sender<String>) {
    // first message sent from client is in the format "username,password"
    // check if the username and password are valid
    // if valid, continue, else close the connection
    let mut reader = std::io::BufReader::new(&stream);

    let mut line = String::new();

    reader.read_line(&mut line).unwrap();

    let parts: Vec<&str> = line.trim().split('/').collect();

    if parts.len() != 2 {
        stream.shutdown(std::net::Shutdown::Both).unwrap();
    }

    // if the message is in the form "create/<username>/<password>", create a new user
    if parts[0] == "create" {
        user::create_user(parts[1], parts[2]).unwrap();

        stream.write(b"User created").unwrap();

        stream.shutdown(std::net::Shutdown::Both).unwrap();
        return;
    } else {
        if !user::check_pw(parts[0], parts[1]).unwrap() {
            stream.shutdown(std::net::Shutdown::Both).unwrap();
            return;
        }
    }


    let mut buffer = [0; 1024];
    loop {
        match stream.read(&mut buffer) {
            Ok(n) => {
                if n == 0 {
                    break;
                }
                let msg = String::from_utf8_lossy(&buffer[..n]).to_string();
                tx.send(msg).unwrap();
            }
            Err(e) => {
                println!("Error: {}", e);
                break;
            }
        }
    }
}