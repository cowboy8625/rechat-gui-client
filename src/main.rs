// #![windows_subsystem = "windows"]
mod rechat;
use std::io::{ErrorKind, Read, Write};
use std::net::TcpStream;
use std::sync::mpsc::{self, TryRecvError};
use std::thread;
use std::time::Duration;
use json;

use rechat::ReChat;

const LOCAL           : &str  = "45.79.53.62:6000";
const MSG_SIZE        : usize = 2048;
const USER_NAME_SIZE  : usize = 16;
const JSON_SIZE       : usize   = 28;
const DATA_SIZE       : usize = MSG_SIZE + USER_NAME_SIZE + JSON_SIZE;
const WIDTH           : i32   = 800;
const HEIGHT          : i32   = 800;
const USERS_X         : i32   = 0;
const USERS_Y         : i32   = 0;
const USERS_W         : i32   = 200;
const USERS_H         : i32   = HEIGHT;
const CHAT_BOX_X      : i32   = USERS_W;
const CHAT_BOX_Y      : i32   = HEIGHT - CHAT_BOX_H;
const CHAT_BOX_W      : i32   = WIDTH - USERS_W;
const CHAT_BOX_H      : i32   = 25;
const MESSAGE_X       : i32   = USERS_W;
const MESSAGE_Y       : i32   = 0;
const MESSAGE_W       : i32   = WIDTH - USERS_W;
const MESSAGE_H       : i32   = HEIGHT - CHAT_BOX_H;


fn main() {
    let mut client = TcpStream::connect(LOCAL).expect("Stream failed to connect");
    client.set_nonblocking(true).expect("failed to initiate non-blocking");

    let (chat_box_tx, chat_box_rx) = mpsc::channel::<String>();
    let (message_tx, message_rx)   = mpsc::channel::<(String, String)>();

    thread::spawn(move || loop {
        let mut buff = vec![0; DATA_SIZE];
        match client.read_exact(&mut buff) {
            Ok(_) => {
                let (username, message) = get_data_from_json(&buff);
                if let (Some(user), Some(msg)) = (username, message) {
                    message_tx.send((user, msg)).expect("Unable to send received message over Channel");
                }
            },
            Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
            Err(_) => {
                eprintln!("connection with server was severed");
                break;
            }
        }

        match chat_box_rx.try_recv() {
            Ok(text) => {
                let mut msg = vec![' '; MSG_SIZE];
                // FIXME: change this to utf8
                text.chars().enumerate().for_each(|(i, c)| {
                    if let Some(letter) = msg.get_mut(i) {
                        *letter = c;
                    }
                });
                let msg: String = msg.iter().collect();
                let json_data = json_formater("cowboy          ", msg.as_str());
                let byte_array = json_data.into_bytes();
                client.write_all(&byte_array).expect("writing to socket failed");
            },
            Err(TryRecvError::Empty) => (),
            Err(TryRecvError::Disconnected) => break
        }

        thread::sleep(Duration::from_millis(100));
    });

    let mut rechat = ReChat::new(chat_box_tx, message_rx);
    rechat.mainloop();
}

/* Data Transaction Functions */

fn json_formater(username: &str, message: &str) -> String {
    let data = json::object!{
        username: username,
        message: message,
    };
    data.dump()
}

fn get_data_from_json(byte_array: &[u8]) -> (Option<String>, Option<String>) {
    let data_string = String::from_utf8(byte_array.to_vec()).expect("Invalid utf8 message");

    match json::parse(&format!(r#"{}"#, data_string)) {
        Ok(json_value) => {
            match json_value {
                json::JsonValue::Object(data) => {
                    // let username = data.get_mut("username").and_then(json::JsonValue::take_string);
                    // let message  = data.get_mut("message").and_then(json::JsonValue::take_string);
                    let username = data.get("username").map(ToString::to_string);
                    let message  = data.get("message").map(ToString::to_string);
                    (username, message)
                },
                _ => (None, None),
            }
        },
        Err(_) => (None, None),
    }
}
