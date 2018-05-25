extern crate http_muncher;

use super::http_parser::HttpParser;
use self::http_muncher::Parser;
use std::cell::RefCell;

pub enum ClientState {
    AwaitingHandshake(RefCell<Parser<HttpParser>>),
    HandshakeResponse,
    Connected
}
