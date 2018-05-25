extern crate mio;

use self::mio::*;

pub const SERVER_TOKEN: Token = Token(0);

pub mod socket_client;
pub mod socket_server;