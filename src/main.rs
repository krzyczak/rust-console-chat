use std::{
    io::{prelude::*, BufReader, BufWriter},
    net::{TcpListener, TcpStream}, env,
};

fn main() {
    let mut args = env::args();

    let port = match (&args.next(), &args.next(), args.next()) {
        (Some(_), Some(arg), Some(port)) if arg == &"-p".to_string() => port.to_string(),
        _ => "7878".to_string()
    };

    let host_and_port = format!("127.0.0.1:{}", port);
    println!("Starting server on: {}", host_and_port);

    let bind = TcpListener::bind(format!("127.0.0.1:{}", port));
    match bind {
        Err(err) => println!("{:?}", err),
        Ok(listener) => listen(listener)
    }
}

fn listen(listener: TcpListener) {
    for stream in listener.incoming() {
        let mut stream1 = stream.unwrap(); // TODO: Can use Arc to have internal mutability without
                                           // cloning?
        let mut stream2 = stream1.try_clone().unwrap();

        let mut buf_reader = BufReader::new(&mut stream1);
        let mut buf_writer = BufWriter::new(&mut stream2);

        println!("Before buf writer");
        buf_writer.write_all("Welcome to The Chat! What is your name?\n> ".as_bytes());
        buf_writer.flush();

        let mut user_name = "".to_string();
        buf_reader.read_line(&mut user_name);

        user_name = user_name.trim().to_string();
        if user_name.len() < 1 {
            buf_writer.write_all("Sorry, but you cannot continue without a name...\n".as_bytes());
            buf_writer.flush();
            return ();
        }

        buf_writer.write_all(format!("Hello {}! It's nice to have you here!\n\n({})> ", user_name, user_name).as_bytes());
        buf_writer.flush();

        loop {
            let mut input: String = "".to_string();
            match buf_reader.read_line(&mut input) {
                Ok(_) => {
                    match input.trim() {
                        "exit" => break,
                        other => {
                            println!("LOGGER (INFO): User typed: `{}`.\n", other);
                            buf_writer.write_all(format!("{}: {}\n> ", user_name, other).as_bytes());
                            buf_writer.flush();
                        }
                    }
                },
                Err(err) => println!("Error: {:?}.\nTry again!", err)
            }
        }
    }
}
