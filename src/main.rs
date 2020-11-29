// #![windows_subsystem = "windows"]

use std::io::{ErrorKind, Read, Write};
use std::net::TcpStream;
use std::sync::mpsc::{self, TryRecvError};
use std::thread;
use std::time::Duration;
use std::sync::{Arc, Mutex};

use fltk::{enums::*, text::*, app::*, window::*};
use json;

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
    // let (message_tx, message_rx) = mpsc::channel::<String>();
    let (message_tx, message_rx) = fltk::app::channel::<([char; USER_NAME_SIZE], [char; MSG_SIZE])>();
    let mut last_user_message: Option<String> = None;

    thread::spawn(move || loop {
        let mut buff = vec![0; DATA_SIZE];
        match client.read_exact(&mut buff) {
            Ok(_) => {
                let (username, message) = get_data_from_json(&buff);
                if let (Some(user), Some(msg)) = (username, message) {
                    let msg = parse_message_to_array(msg.as_str());
                    let username = parse_username_to_array(user.as_str());
                    message_tx.send((username, msg));// .expect("Unable to send received message over Channel");
                }
            },
            Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
            Err(_) => {
                println!("connection with server was severed");
                break;
            }
        }

        match chat_box_rx.try_recv() {
            Ok(text) => {
                let mut msg = vec![' '; MSG_SIZE];
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

    let message_list: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

    let app = App::default().with_scheme(AppScheme::Base);
    let mut wind = Window::new(100, 100, WIDTH, HEIGHT, "ReChat");

    // Display all users
    let mut _u_mes = create_user_display();

    // Display Messages
    let mut r_mes = create_text_display();
    let ml = Arc::clone(&message_list);
    let mut rm = r_mes.clone();
    r_mes.set_callback(move || {
    });

    // User typed message
    let mut mes_box = create_message_box();
    mes_box.handle2(move |f, ev| {
        use fltk::enums::*;
        match ev {
            Event::KeyUp => {
                match event_key() {
                    Key::Enter => {
                        if let Some(buffer) = f.buffer() {
                            let text = buffer.text().trim().to_string();
                            if !text.is_empty() {
                                chat_box_tx.send(text).expect("Transmite to Thread Failed");
                            }
                            f.set_buffer(Some(TextBuffer::default()));
                        }
                        true
                    },
                    _ => false,
                }
            },
            _ => false,
        }
    });

    fltk::app::set_focus(&mes_box);
    wind.end();
    wind.show();
    while app.wait() {
        match message_rx.recv() {
            Some((user, buff)) => {
                // Turn user/msg Byte Array into String.
                let user: String = user.iter().collect();
                let msg: String = buff.iter().collect();
                let message: String;

                let mut with_user_name = false;
                if let Some(last) = &last_user_message {
                    if !(*last == user.trim().to_string()) {
                        with_user_name = true;
                    }
                } else {
                    last_user_message = Some(user.trim().to_string());
                    with_user_name = true;
                }
                if with_user_name {
                    message = format!("<{}>\n{}", user.trim(), msg.trim());
                } else {
                    message = format!("{}", msg.trim());
                }
                let mut data = ml.lock().expect("Failed to get message list.");
                data.push(message);

                let text = data.join("\n");
                let mut buffer = TextBuffer::default();
                buffer.set_text(text.as_str());
                rm.set_buffer(Some(buffer));
            }
            None => {}
        }
    }
}

/* GUI Wiget Creation Functions */

fn create_user_display() -> TextDisplay {
    let mut user_display = TextDisplay::default()
        .with_size(USERS_W, USERS_H)
        .with_pos(USERS_X, USERS_Y);
    user_display.set_buffer(Some(TextBuffer::default()));
    user_display.set_color(Color::Black);
    user_display.set_text_color(Color::White);
    user_display
}

fn create_text_display() -> TextDisplay {
    let mut text_display = TextDisplay::default()
        .with_size(MESSAGE_W, MESSAGE_H)
        .with_pos(MESSAGE_X, MESSAGE_Y);
    text_display.set_buffer(Some(TextBuffer::default()));
    text_display.set_color(Color::Black);
    text_display.set_text_color(Color::White);
    text_display.wrap_mode(WrapMode::AtPixel, MESSAGE_W - 10);
    text_display
}

fn create_message_box() -> TextEditor {
    let mut message_box = TextEditor::default()
        .with_size(CHAT_BOX_W, CHAT_BOX_H)
        .with_pos(CHAT_BOX_X, CHAT_BOX_Y);
    message_box.set_buffer(Some(TextBuffer::default()));
    message_box.set_scrollbar_width(0);
    message_box.set_cursor_style(TextCursor::Simple);
    message_box.set_cursor_color(Color::White);
    message_box.set_text_color(Color::White);
    message_box.set_insert_mode(true);
    message_box.set_tooltip("Input Message Here");
    message_box.set_color(Color::Black);
    message_box.wrap_mode(WrapMode::AtPixel, CHAT_BOX_W - 10);
    message_box
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
                    let username = data.get("username").map(|i| i.to_string());
                    let message  = data.get("message").map(|i| i.to_string());
                    (username, message)
                },
                _ => (None, None),
            }
        },
        Err(_) => (None, None),
    }
}

fn parse_message_to_array(message: &str) -> [char; MSG_SIZE] {
    let mut your_array = [' '; MSG_SIZE];
    message.chars()
        .zip(your_array.iter_mut())
        .for_each(|(b, ptr)| *ptr = b);
    your_array
}

fn parse_username_to_array(message: &str) -> [char; USER_NAME_SIZE] {
    let mut your_array = [' '; USER_NAME_SIZE];
    message.chars()
        .zip(your_array.iter_mut())
        .for_each(|(b, ptr)| *ptr = b);
    your_array
}
