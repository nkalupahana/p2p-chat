const SERVER_IP: &str = "34.67.17.122:8888";

use ctrlc;
use eframe::{egui, epi};
use std::env;
use std::io;
use std::io::*;
use std::net::UdpSocket;
use std::process;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::{panic, thread, time::Duration};

const CONN_FAIL: &str = "Couldn't connect to address";
const SEND_FAIL: &str = "Couldn't send message";
const RECV_FAIL: &str = "Couldn't receive message";

struct App {
    socket: Arc<UdpSocket>,
    messages: Arc<Mutex<Vec<(String, String)>>>,
    cur_message: String,
}

impl App {
    pub fn new(socket: Arc<UdpSocket>) -> (App, Arc<Mutex<Vec<(String, String)>>>) {
        let messages = Arc::new(Mutex::new(Vec::new()));
        let returned_messages = messages.clone();

        (
            App {
                socket: socket,
                messages,
                cur_message: String::new(),
            },
            returned_messages,
        )
    }
}

impl epi::App for App {
    fn name(&self) -> &str {
        "P2P Messaging Client"
    }

    fn update(&mut self, ctx: &egui::CtxRef, _frame: &mut epi::Frame<'_>) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let mut frame = egui::Frame::default();
            frame.stroke = egui::Stroke::none();
            egui::TopBottomPanel::bottom("sendMessage")
                .frame(frame)
                .show_inside(ui, |ui| {
                    ui.with_layout(egui::Layout::left_to_right(), |ui| {
                        let widget = egui::TextEdit::singleline(&mut self.cur_message)
                            .hint_text("Enter a message to send");
                        let text_edit = ui.add(widget);
                        if ui.button("Send").clicked() || ui.input().key_released(egui::Key::Enter)
                        {
                            let trimmed = self.cur_message.trim();
                            if !trimmed.is_empty() {
                                self.messages
                                    .lock()
                                    .unwrap()
                                    .push(("You".to_string(), trimmed.to_string()));
                                self.socket
                                    .send(trimmed.as_bytes())
                                    .expect("error: unable to send message");
                                self.cur_message.clear();
                            }
                            text_edit.request_focus();
                        };
                    });
                });

            let spacing = 10.0;
            egui::TopBottomPanel::top("conversation")
                .min_height(ui.available_height() - spacing)
                .show_inside(ui, |ui| {
                    egui::ScrollArea::vertical()
                        .stick_to_bottom()
                        .show(ui, |ui| {
                            ui.set_min_width(ui.available_width());
                            for (sender, message) in self.messages.lock().unwrap().iter() {
                                ui.label(format!("{}: {}", sender, message));
                            }
                        });
                });
            ui.add_space(spacing);
        });
    }

    fn on_exit(&mut self) {
        self.socket.send(b"FIN").expect("couldn't send message");
    }
}

fn main() -> std::io::Result<()> {
    {
        // Get keyword for connection
        let mut keyword = String::new();
        loop {
            print!("Enter a keyword to identify your connection: ");
            io::stdout().flush().unwrap();
            io::stdin().read_line(&mut keyword).unwrap();
            if keyword.trim().len() != 0 {
                break;
            };
        }

        // Use Arc for reference counting, so socket can be shared between threads later
        let socket = Arc::new(UdpSocket::bind("0.0.0.0:0")?);

        // Connect and send message
        socket.connect(SERVER_IP).expect(CONN_FAIL);
        println!("Sending keyword and waiting for peer... (expires after 1 minute)");
        socket.send(keyword.as_bytes()).expect(SEND_FAIL);

        // Wait for peer to connect (waiting for "peer IP:port" message)
        let mut buf = [0; 100];
        let (len, _) = socket.recv_from(&mut buf).expect(RECV_FAIL);
        let peer = String::from_utf8(buf[..len].to_vec())
            .unwrap()
            .trim()
            .to_string();
        if peer == "Keyword invalid" {
            panic!("Keyword rejected by server.");
        }

        // Connect to peer (hole punch!)
        println!(
            "Connected to {}. Use Ctrl-C to terminate the connection.",
            peer,
        );
        socket.connect(peer).expect(CONN_FAIL);
        socket.send(b"SYN").expect(SEND_FAIL);

        // Register Ctrl-C handler to notify peer when connection is closed
        {
            let socket = socket.clone();
            ctrlc::set_handler(move || {
                socket.send(b"FIN").expect("couldn't send message");
                process::exit(0);
            })
            .unwrap();
        }

        let is_console_mode = Arc::new(AtomicBool::new(false));
        let args: Vec<String> = env::args().collect();
        if args.len() > 1 && args.last().unwrap() == "--console" {
            is_console_mode.store(true, Ordering::SeqCst);
        }

        let (app, messages) = App::new(socket.clone());
        // Receive thread: Display messages as they're received
        {
            let is_console_mode = is_console_mode.clone();
            let socket = socket.clone();
            let messages = messages.clone();
            thread::spawn(move || {
                loop {
                    // Wait for message
                    let mut buf = [0; 100];
                    let (len, _) = socket.recv_from(&mut buf).expect(RECV_FAIL);
                    let message = String::from_utf8(buf[..len].to_vec()).unwrap();

                    // If the message isn't keep-alive:
                    if message != "SYN" {
                        // FIN is for peer close, so terminate, else display message
                        if message == "FIN" {
                            println!("Connection closed by peer.");
                            process::exit(0);
                        } else {
                            if is_console_mode.load(Ordering::SeqCst) {
                                print!("\r< {}\n> ", message);
                                io::stdout().flush().unwrap();
                            } else {
                                messages.lock().unwrap().push(("Peer".to_string(), message));
                            }
                        }
                    }
                }
            });
        }

        // keep-alive thread: even if messages aren't being sent, this
        // keeps messages going so the NAT keeps the ports open
        {
            let socket = socket.clone();
            thread::spawn(move || loop {
                socket.send(b"SYN").expect(SEND_FAIL);
                thread::sleep(Duration::from_secs(1));
            });
        }

        // Prevent any output on panic
        panic::set_hook(Box::new(|_info| {
            // do nothing
        }));

        if !is_console_mode.load(Ordering::SeqCst) {
            // Create a window in order to send and receive messages simultaneously without collisions
            let native_options = eframe::NativeOptions::default();
            // Try to see if a window can be launched
            let _ = panic::catch_unwind(|| {
                eframe::run_native(Box::new(app), native_options);
            });
        }

        // If we get here, either the user doesn't have a display or they ran in console mode --
        // so let's get that all set up
        is_console_mode.store(true, Ordering::SeqCst);
        loop {
            let mut message = String::new();
            print!("> ");
            io::stdout().flush().unwrap();
            io::stdin()
                .read_line(&mut message)
                .expect("Couldn't read from stdin");
            socket.send(message.trim().as_bytes()).expect(SEND_FAIL);
        }
    }
}
