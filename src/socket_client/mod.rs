extern crate mio;
extern crate sha1;
extern crate rustc_serialize;
extern crate http_muncher;

mod client_state;
mod http_parser;
mod frame;

use self::mio::*;
use self::mio::tcp::*;
use std::collections::HashMap;
use self::rustc_serialize::base64::{ToBase64, STANDARD};
use std::cell::RefCell;
use std::rc::Rc;
use std::fmt;
use self::client_state::ClientState;
use self::http_parser::HttpParser;
use self::http_muncher::Parser;
use self::frame::{OpCode, WebSocketFrame};

fn gen_key(key: &String) -> String {
    let mut m = sha1::Sha1::new();
    let mut buf = [0u8; 20];

    m.update(key.as_bytes());
    m.update("258EAFA5-E914-47DA-95CA-C5AB0DC85B11".as_bytes());

    m.output(&mut buf);

    return buf.to_base64(STANDARD);
}

pub struct WebSocketClient {
    pub socket: TcpStream,
    pub headers: Rc<RefCell<HashMap<String, String>>>,
    // Adding a new `interest` property:
    pub interest: EventSet,

    // Add a client state:
    pub state: ClientState,

    outgoing: Vec<WebSocketFrame>
}

impl WebSocketClient {
    fn read_handshake(&mut self) {
        loop {
            let mut buf = [0; 2048];
            match self.socket.try_read(&mut buf) {
                Err(e) => {
                    println!("Error while reading socket: {:?}", e);
                    return
                },
                Ok(None) =>
                    // Socket buffer has got no more bytes.
                    break,
                Ok(Some(_len)) => {
                    let is_upgrade = if let ClientState::AwaitingHandshake(ref parser_state) = self.state {
                        let mut parser = parser_state.borrow_mut();
                        parser.parse(&buf);
                        parser.is_upgrade()
                    } else { false };

                    if is_upgrade {
                        // Change the current state
                        self.state = ClientState::HandshakeResponse;

                        // Change current interest to `Writable`
                        self.interest.remove(EventSet::readable());
                        self.interest.insert(EventSet::writable());
                        break;
                    }
                }
            }
        }
    }

    fn read_frame(&mut self) {
        let frame = WebSocketFrame::read(&mut self.socket);
        match frame {
            Ok(frame) => {
                match frame.get_opcode() {
                    OpCode::TextFrame => {
                        println!("{:?}", frame);
                        let reply_frame = WebSocketFrame::from("Hi there!");
                        self.outgoing.push(reply_frame);
                    },
                    OpCode::Ping => {
                        println!("ping/pong");
                        self.outgoing.push(WebSocketFrame::pong(&frame));
                    },
                    OpCode::ConnectionClose => {
                        self.outgoing.push(WebSocketFrame::close_from(&frame));
                    },
                    _ => {}
                }
                self.interest.remove(EventSet::readable());
                self.interest.insert(EventSet::writable());
            }
            Err(e) => println!("error while reading frame: {}", e)
        }
    }

    pub fn read(&mut self) {
        match self.state {
            ClientState::AwaitingHandshake(_) => self.read_handshake(),
            // Add a new state handler:
            ClientState::Connected => self.read_frame(),
            _ => {}
        }
    }

    pub fn write(&mut self) {
        match self.state {
            ClientState::HandshakeResponse => {
                self.write_handshake();
            },
            ClientState::Connected => {
                // Add a boolean flag:
                let mut close_connection = false;

                for frame in self.outgoing.iter() {
                    if let Err(e) = frame.write(&mut self.socket) {
                        println!("error on write: {}", e);
                    }

                    // Check if there's a frame that wants to close the connection:
                    if frame.is_close() {
                        close_connection = true;
                    }
                }

                self.outgoing.clear();

                self.interest.remove(EventSet::writable());
                // Add a `hup` event if we want to close the connection:
                if close_connection {
                    self.interest.insert(EventSet::hup());
                } else {
                    self.interest.insert(EventSet::readable());
                }
            },
            _ => {}
        }
    }

    fn write_handshake(&mut self) {
        // Get the headers HashMap from the Rc<RefCell<...>> wrapper:
        let headers = self.headers.borrow();

        // Find the header that interests us, and generate the key from its value:
        let response_key = gen_key(&headers.get("Sec-WebSocket-Key").unwrap());

        // We're using special function to format the string.
        // You can find analogies in many other languages, but in Rust it's performed
        // at the compile time with the power of macros. We'll discuss it in the next
        // part sometime.
        let response = fmt::format(format_args!("HTTP/1.1 101 Switching Protocols\r\n\
                                                Connection: Upgrade\r\n\
                                                Sec-WebSocket-Accept: {}\r\n\
                                                Upgrade: websocket\r\n\r\n", response_key));

        // Write the response to the socket:
        self.socket.try_write(response.as_bytes()).unwrap();

        // Change the state:
        self.state = ClientState::Connected;

        // And change the interest back to `readable()`:
        self.interest.remove(EventSet::writable());
        self.interest.insert(EventSet::readable());
    }

    pub fn new(socket: TcpStream) -> WebSocketClient {
        let headers = Rc::new(RefCell::new(HashMap::new()));

        WebSocketClient {
            socket: socket,

            // We're making a first clone of the `headers` variable
            // to read its contents:
            headers: headers.clone(),

            // Initial events that interest us
            interest: EventSet::readable(),

            // Initial state
            state: ClientState::AwaitingHandshake(RefCell::new(Parser::request(HttpParser {
                current_key: None,
                headers: headers.clone()
            }))),

            outgoing: Vec::new()
        }
    }
}
