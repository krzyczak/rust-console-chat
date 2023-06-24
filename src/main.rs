use std::{
    io::{prelude::*, BufReader, BufWriter},
    net::{TcpListener}, env, thread, sync::{mpsc::{self, Receiver, Sender}, Arc, Mutex}, time::Duration
};

fn main() {
    let mut args = env::args();

    let port = match (&args.next(), &args.next(), args.next()) {
        (Some(_), Some(arg), Some(port)) if arg == &"-p".to_string() => port.to_string(),
        _ => "7878".to_string()
    };

    let host_and_port = format!("0.0.0.0:{}", port);
    println!("Starting server on: {}", host_and_port);

    let bind = TcpListener::bind(host_and_port);
    match bind {
        Err(err) => println!("{:?}", err),
        Ok(listener) => listen(listener)
    }
}

fn listen(listener: TcpListener) {
    let broadcasters: Vec<(Receiver<String>, Sender<String>)> = Vec::new();
    let broadcasters = Arc::new(Mutex::new(broadcasters));

    let broadcast_to = Arc::clone(&broadcasters);
    thread::spawn(move || {
        loop {
            let broadcasters = broadcast_to.lock().unwrap();
            let mut messages = vec![];

            for broadcaster in broadcasters.iter() {
                if let Ok(message) = broadcaster.0.try_recv() {
                    messages.push(message);
                }
            }

            for broadcaster in broadcasters.iter() {
                for message in &messages {
                    broadcaster.1.send(message.clone());
                }
            }
            thread::sleep(Duration::from_millis(10));
        }
    });

    for stream in listener.incoming() {
        let (write_sender, write_receiver) = mpsc::channel::<String>();
        let (read_sender, read_receiver) = mpsc::channel::<String>();

        let mut data = broadcasters.lock().unwrap();
        data.push((write_receiver, read_sender));

        thread::spawn(move || {
            let stream = stream.unwrap();
            let stream_reader = stream.try_clone().unwrap(); // TODO: Get rid of cloning.
            let stream_writer = stream.try_clone().unwrap(); // TODO: Get rid of cloning.

            let mut buf_reader = BufReader::new(stream_reader);
            let mut buf_writer = BufWriter::new(stream_writer);

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

            let mut buf_writer_clone = BufWriter::new(stream.try_clone().unwrap());
            let user_name_clone = user_name.clone();
            thread::spawn(move || {
                for message in read_receiver {
                    if message == "exit".to_string() {
                        break;
                    }
                    buf_writer_clone.write_all(format!("\r{}\n({})> ", message, user_name_clone).as_bytes());
                    buf_writer_clone.flush();
                }
            });

            loop {
                let mut input: String = "".to_string();

                match buf_reader.read_line(&mut input) {
                    Ok(_) => {
                        match input.trim() {
                            "exit" => {
                                write_sender.send("exit".to_string());
                                break;
                            },
                            message => {
                                write_sender.send((format!("{}: {}\n", user_name, message)).to_string());
                                thread::sleep(Duration::from_millis(100));
                            }
                        }
                    },
                    Err(err) => println!("Error: {:?}.\nTry again!", err)
                }
            }
        });
    }
}
