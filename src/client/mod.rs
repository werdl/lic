
use std::io::{Read, Write};
use std::net::TcpStream;

pub struct ClientOpts {
    pub port: u16,

    pub host: String,
}

pub struct Client {
    opts: ClientOpts,
    connection: TcpStream,
}

impl Client {
    pub fn new(opts: ClientOpts) -> std::io::Result<Self> {
        let connection = TcpStream::connect(format!("{}:{}", opts.host, opts.port))?;
        Ok(Self {
            opts,
            connection,
        })
    }

    pub fn send(&mut self, data: String) -> std::io::Result<()> {
        self.connection.write_all(data.as_bytes())?;
        Ok(())
    }

    pub fn run(&mut self) -> std::io::Result<()> {
        eprint!("Details (<username>/<password>): ");

        // first, spawn a thread to read from the server, and print to stdout
        let mut connection = self.connection.try_clone()?;

        // first, take uname and password from the user
        let mut buffer = String::new();


        std::io::stdin().read_line(&mut buffer)?;
        self.send(buffer.clone())?;

        // if we are here, the username and password are valid
        let uname = buffer.split('/').next().unwrap().to_string();
        let pass = buffer.split('/').last().unwrap().strip_suffix("\n").unwrap();

        let uname_for_later = uname.clone();

        std::thread::spawn(move || {
            let mut newbuffer = [0; 1024];
            loop {
                match connection.read(&mut newbuffer) {
                    Ok(0) => {
                        eprintln!("Server closed connection");
                        break;
                    }
                    Ok(n) => {
                        let data = std::str::from_utf8(&newbuffer[..n]).unwrap();
                        if data.split(":").next().unwrap() != uname.clone() {
                            print!("{}", data);
                        }
                    }
                    Err(err) => {
                        eprintln!("Error reading from server: {}", err);
                        break;
                    }
                }
            }
        });

        // then, read from stdin and send to the server
        let mut buffer = String::new();
        loop {
            std::io::stdin().read_line(&mut buffer)?;
            // form: username/password/message
            self.send(format!("{}/{}/{}", uname_for_later.clone(), pass, buffer.strip_suffix("\n").unwrap_or(&buffer)))?;
            buffer.clear();
        }
    }
}