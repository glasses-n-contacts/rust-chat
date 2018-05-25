extern crate mio;
extern crate sha1;

use self::mio::*;
use self::mio::tcp::*;
use std::collections::HashMap;
use socket_client::WebSocketClient;

pub struct WebSocketServer {
    pub socket: TcpListener,
    pub clients: HashMap<Token, WebSocketClient>,
    pub token_counter: usize
}

impl Handler for WebSocketServer {
    type Timeout = usize;
    type Message = ();

    fn ready(&mut self, event_loop: &mut EventLoop<WebSocketServer>,
            token: Token, events: EventSet)
    {
        // Are we dealing with the read event?
        if events.is_readable() {
            match token {
                ::SERVER_TOKEN => {
                    let client_socket = match self.socket.accept() {
                        Err(e) => {
                            println!("Accept error: {}", e);
                            return;
                        },
                        Ok(None) => unreachable!("Accept has returned 'None'"),
                        Ok(Some((sock, _addr))) => sock
                    };

                    self.token_counter += 1;
                    let new_token = Token(self.token_counter);

                    self.clients.insert(new_token, WebSocketClient::new(client_socket));
                    event_loop.register(&self.clients[&new_token].socket, new_token, EventSet::readable(),
                                PollOpt::edge() | PollOpt::oneshot()).unwrap();
                },
                token => {
                    let mut client = self.clients.get_mut(&token).unwrap();
                    client.read();
                    event_loop.reregister(&client.socket, token,
                        client.interest, // Providing `interest` from the client's struct
                        PollOpt::edge() | PollOpt::oneshot()).unwrap();
                }
            }
        }

        // Handle write events that are generated whenever
        // the socket becomes available for a write operation:
        if events.is_writable() {
            let client = self.clients.get_mut(&token).unwrap();
            client.write();
            event_loop.reregister(&client.socket, token, client.interest,
                                PollOpt::edge() | PollOpt::oneshot()).unwrap();
        }
    }
}
