use std::net::{TcpStream};
use crate::resp::DataType;

pub struct ClientConnection {
    pub stream: TcpStream,
    pub is_transaction_active: bool,
    pub cmd_queue: Vec<DataType>,
}

impl ClientConnection {
    pub fn new(stream: TcpStream) -> Self {
        ClientConnection {
            stream,
            is_transaction_active: false,
            cmd_queue: Vec::new(),
        }
    }
}
