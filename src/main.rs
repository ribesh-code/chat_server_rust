use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};

struct ChatRoom {
    name: String,
    users: Vec<TcpStream>,
    history: Vec<String>,
}

impl ChatRoom {
    fn new(name: String) -> ChatRoom {
        ChatRoom {
            name,
            users: vec![],
            history: vec![],
        }
    }

    fn broadcast(&mut self, message: &str) {
        for user in &mut self.users {
            let _ = user.write_fmt(format_args!("{}\n", message));
        }
    }

    fn add_user(&mut self, user: TcpStream) {
        self.users.push(user);
    }

    fn add_message(&mut self, message: String) {
        self.history.push(message);
    }
}

fn handle_client(
    stream: TcpStream,
    rooms: Arc<Mutex<HashMap<String, ChatRoom>>>,
) -> std::io::Result<()> {
    let mut reader = BufReader::new(&stream);
    let mut username = String::new();
    let mut room_name = String::new();

    reader.read_line(&mut username)?;
    reader.read_line(&mut room_name)?;

    let username = username.trim().to_string();
    let room_name = room_name.trim().to_string();

    let message = format!("{} has joined the room", username);

    let mut rooms_gaurd = rooms.lock().unwrap();
    let room = rooms_gaurd
        .entry(room_name.clone())
        .or_insert_with(|| ChatRoom::new(room_name.clone()));

    room.add_user(stream.try_clone()?);
    room.add_message(message.clone());
    room.broadcast(&message);

    let mut message_buf = String::new();

    loop {
        message_buf.clear();
        reader.read_line(&mut message_buf)?;
        let message = message_buf.trim().to_string();
        if message.is_empty() {
            continue;
        }

        if message.to_lowercase() == "exit" {
            let message = format!("{} has left the room.", username);

            room.broadcast(&message);
            break;
        }

        let full_message = format!("{}: {}", username, message);
        room.add_message(full_message.clone());
        room.broadcast(&full_message);
    }

    Ok(())
}

fn main() -> std::io::Result<()> {
    let rooms = Arc::new(Mutex::new(HashMap::new()));

    let listener = TcpListener::bind("127.0.0.1:8080")?;

    for stream in listener.incoming() {
        let rooms = rooms.clone();
        std::thread::spawn(move || {
            if let Err(err) = handle_client(stream.unwrap(), rooms) {
                println!("Error {}", err)
            }
        });
    }
    Ok(())
}
