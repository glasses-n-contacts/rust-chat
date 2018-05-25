extern crate mio;
extern crate chat;

use mio::*;
use mio::tcp::*;
use std::collections::HashMap;

use chat::socket_server::WebSocketServer;
use chat::SERVER_TOKEN;

fn main() {
    let address = "0.0.0.0:10000".parse().unwrap();
    let server_socket = TcpListener::bind(&address).unwrap();
                        
    let mut event_loop = EventLoop::new().unwrap();

    let mut server = WebSocketServer {
        token_counter: 1,        // Starting the token counter from 1
        clients: HashMap::new(), // Creating an empty HashMap
        socket: server_socket    // Handling the ownership of the socket to the struct
    };

    event_loop.register(&server.socket,
                        SERVER_TOKEN,
                        EventSet::readable(),
                        PollOpt::edge()).unwrap();

    event_loop.run(&mut server).unwrap();
}
