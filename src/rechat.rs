use crate::*;
use std::sync::mpsc;
use fltk::{enums::*, frame::*, text::*, app::*, window::*};

pub struct ReChat {
    message_rx: mpsc::Receiver<(String, String)>,
    _app: App,
    window: Window,
    chat_box: TextEditor,
    received_box: TextDisplay,
    users_box: TextDisplay,
    message_list: Vec<String>,
    last_user_message: Option<String>,
}

impl ReChat {
    pub fn new(
        chat_box_tx: mpsc::Sender<String>,
        message_rx: mpsc::Receiver<(String, String)>
        ) -> Self {
        let _app = App::default().with_scheme(AppScheme::Base);
        let mut window = Window::new(100, 100, WIDTH, HEIGHT, "ReChat");
        let mut chat_box = create_chat_box();
        let received_box = create_received_box();
        let users_box = create_users_box();
        window.set_callback(|| {
            if event_key() == fltk::enums::Key::Escape {
                quit();
                std::process::exit(0);
            }
        });
        chat_box.handle2(move |f, ev| {
            use fltk::enums::*;
            if ev == Event::KeyUp && event_key() == Key::Enter {
                if let Some(buffer) = f.buffer() {
                    let text = buffer.text().trim().to_string();
                    if !text.is_empty() {
                        chat_box_tx.send(text).expect("Transmite to Thread Failed");
                    }
                    f.set_buffer(Some(TextBuffer::default()));
                }
                true
            } else {
                false
            }
        });

        Self {
            message_rx,
            _app,
            window,
            chat_box,
            received_box,
            users_box,
            message_list:  Vec::new(),
            last_user_message: None,
        }
    }

    pub fn mainloop(&mut self) {
        fltk::app::set_focus(&self.chat_box);
        self.window.end();
        self.window.show();
        loop {
            match self.message_rx.try_recv() {
                Ok((user, msg)) => {
                    // Turn user/msg Byte Array into String.
                    let message: String;

                    let mut with_user_name = false;
                    if let Some(last) = &self.last_user_message {
                        if !(*last == user.trim().to_string()) {
                            with_user_name = true;
                        } else {
                            self.last_user_message = Some(user.trim().to_string());
                        }
                    } else {
                        self.last_user_message = Some(user.trim().to_string());
                        with_user_name = true;
                    }
                    if with_user_name {
                        message = format!("<{}>\n{}", user.trim(), msg.trim());
                    } else {
                        message = format!("{}", msg.trim());
                    }
                    self.message_list.push(message);

                    let text = self.message_list.join("\n");
                    let mut buffer = TextBuffer::default();
                    buffer.set_text(text.as_str());
                    self.received_box.set_buffer(Some(buffer));
                },
                Err(_) => { },
            }
            wait_for(0.001).unwrap()
        }
    }
}

fn create_users_box() -> TextDisplay {
    let mut users_box = TextDisplay::default()
        .with_size(USERS_W, USERS_H)
        .with_pos(USERS_X, USERS_Y);
    users_box.set_buffer(Some(TextBuffer::default()));
    users_box.set_color(Color::Black);
    users_box.set_text_color(Color::White);
    users_box
}

fn create_received_box() -> TextDisplay {
    let mut image = fltk::image::SharedImage::load("D:/Minecraft stuff/Photos/Cowboy-image-final.jpg").expect("lksadjfkaj");
    image.scale(MESSAGE_W,MESSAGE_H,true, true);
    let mut received_box = TextDisplay::default()
        .with_size(MESSAGE_W, MESSAGE_H)
        .with_pos(MESSAGE_X, MESSAGE_Y);
    received_box.set_buffer(Some(TextBuffer::default()));
    received_box.set_color(Color::Black);
    received_box.set_text_color(Color::White);
    received_box.wrap_mode(WrapMode::AtPixel, MESSAGE_W - 10);
    received_box.set_image(Some(image));
    received_box
}

fn create_chat_box() -> TextEditor {
    let mut chat_box = TextEditor::default()
        .with_size(CHAT_BOX_W, CHAT_BOX_H)
        .with_pos(CHAT_BOX_X, CHAT_BOX_Y);
    chat_box.set_buffer(Some(TextBuffer::default()));
    chat_box.set_scrollbar_width(0);
    chat_box.set_cursor_style(TextCursor::Simple);
    chat_box.set_cursor_color(Color::White);
    chat_box.set_text_color(Color::White);
    chat_box.set_insert_mode(true);
    chat_box.set_tooltip("Input Message Here");
    chat_box.set_color(Color::Black);
    chat_box.wrap_mode(WrapMode::AtPixel, CHAT_BOX_W - 10);
    chat_box
}
