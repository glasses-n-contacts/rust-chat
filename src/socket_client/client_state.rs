#[derive(PartialEq)]
pub enum ClientState {
    AwaitingHandshake,
    HandshakeResponse,
    Connected
}
