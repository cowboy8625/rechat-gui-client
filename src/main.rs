use std::io::{ErrorKind, Read, Write};
use std::net::TcpStream;
use std::sync::mpsc::{self, TryRecvError};
use std::thread;
use std::time::Duration;
use std::sync::{Arc, Mutex};

use fltk::{enums::*, text::*, app::*, frame::*, window::*};

const LOCAL     : &str  = "45.79.53.62:6000";
const MSG_SIZE  : usize = 100;
const WIDTH     : i32   = 800;
const HEIGHT    : i32   = 800;
// const PAD_Y     : i32   = 5;
// const PAD_X     : i32   = 5;
const USERS_X   : i32   = 0;
const USERS_Y   : i32   = 0;
const USERS_W   : i32   = 200;
const USERS_H   : i32   = HEIGHT;
const CHAT_BOX_X: i32   = USERS_W;
const CHAT_BOX_Y: i32   = HEIGHT - CHAT_BOX_H;
const CHAT_BOX_W: i32   = WIDTH - USERS_W;
const CHAT_BOX_H: i32   = 25;
const MESSAGE_X : i32   = USERS_W;
const MESSAGE_Y : i32   = 0;
const MESSAGE_W : i32   = WIDTH - USERS_W;
const MESSAGE_H : i32   = HEIGHT - CHAT_BOX_H;


fn main() {
    let mut client = TcpStream::connect(LOCAL).expect("Stream failed to connect");
    client.set_nonblocking(true).expect("failed to initiate non-blocking");

    let (chat_box_tx, chat_box_rx) = mpsc::channel::<String>();
    let (message_tx, message_rx) = mpsc::channel::<String>();

    thread::spawn(move || loop {
        let mut buff = vec![0; MSG_SIZE];
        match client.read_exact(&mut buff) {
            Ok(_) => {
                let msg = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                let msg = String::from_utf8(msg).expect("Invalid utf8 message");
                message_tx.send(msg).expect("Unable to send received message over Channel");
            },
            Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
            Err(_) => {
                println!("connection with server was severed");
                break;
            }
        }

        match chat_box_rx.try_recv() {
            Ok(msg) => {
                let mut buff = msg.clone().into_bytes();
                buff.resize(MSG_SIZE, 0);
                client.write_all(&buff).expect("writing to socket failed");
            }, 
            Err(TryRecvError::Empty) => (),
            Err(TryRecvError::Disconnected) => break
        }

        thread::sleep(Duration::from_millis(100));
    });

    let app = App::default().with_scheme(AppScheme::Plastic);
    let mut wind = Window::new(100, 100, WIDTH, HEIGHT, "ReChat");
    // let mut frame = Frame::new(0, 0, 400, 200, "");
    // let mut but = Button::new(160, 210, 80, 40, "Click me!");

    // Display all users
    let mut _u_mes = create_user_display();

    // Display Messages
    let mut r_mes = create_text_display();
    r_mes.handle2(move |f, _ev| {
        match message_rx.try_recv() {
            Ok(msg) => {
                let mut buffer = TextBuffer::default();
                buffer.append(&msg);
                f.set_buffer(Some(buffer));
                true
            }
            Err(_) => {
                false
            }
        }
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
                    // Key::Enter, Key::ShiftL) => {
                    //     println!("SHIFT + ENTER");
                    // },
                    _ => false,
                }
            },
            _ => false,
        }
    });


    wind.end();
    wind.show();
    // but.set_callback(move || frame.set_label("Hello World!"));
    app.run().unwrap();
}

fn create_user_display() -> TextDisplay {
    let mut user_display = TextDisplay::default();
    user_display.set_size(USERS_W, USERS_H);
    user_display.set_pos(USERS_X, USERS_Y);
    user_display.set_buffer(Some(TextBuffer::default()));
    user_display.set_color(Color::Black);
    user_display.set_text_color(Color::White);
    //user_display.set_color(Color::Cyan);
    user_display
}

fn create_text_display() -> TextDisplay {
    let mut text_display = TextDisplay::default();
    text_display.set_size(MESSAGE_W, MESSAGE_H);
    text_display.set_pos(MESSAGE_X, MESSAGE_Y);
    text_display.set_buffer(Some(TextBuffer::default()));
    text_display.set_color(Color::Black);
    text_display.set_text_color(Color::White);
    text_display
}

fn create_message_box() -> TextEditor {
    let mut message_box = TextEditor::default();
    message_box.set_size(CHAT_BOX_W, CHAT_BOX_H);
    message_box.set_pos(CHAT_BOX_X, CHAT_BOX_Y);
    message_box.set_buffer(Some(TextBuffer::default()));
    message_box.set_scrollbar_width(0);
    message_box.set_cursor_style(TextCursor::Simple);
    message_box.set_cursor_color(Color::White);
    message_box.set_text_color(Color::White);
    message_box.set_insert_mode(true);
    message_box.set_tooltip("Input Message Here");
    message_box.set_color(Color::Black);
    message_box
}

