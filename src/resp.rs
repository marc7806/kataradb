use std::io::Read;
use std::net::TcpStream;

use crate::TEMP_BUFFER_SIZE;

// RESPParser is responsible for parsing Redis Serialization protocol (RESP)

pub struct RESPParser {
    pub stream: TcpStream,
    // Temporary buffer used for holding a sequence of bytes until the next CRLF( \r\n )
    line_buffer: Vec<u8>,
}

impl RESPParser {
    pub fn new(stream: TcpStream) -> RESPParser {
        return RESPParser {
            stream,
            line_buffer: Vec::with_capacity(TEMP_BUFFER_SIZE),
        };
    }

    fn read_line(&mut self) -> &Vec<u8> {
        if self.line_buffer.len() > 0 {
            // clear buffer if it is not empty
            self.line_buffer.clear();
        }

        // Parse sequence of bytes until next CRLF
        let tcp_stream = self.stream.try_clone().expect("Can not clone stream");
        for byte in tcp_stream.bytes() {
            let byte = byte.expect("Can not read byte");
            println!("{}", byte);

            if byte == b'\r' {
                // stop parsing on carriage return
                break;
            }

            self.line_buffer.push(byte);
        }

        return &self.line_buffer;
    }

    pub fn parse_next(&mut self) -> Result<DataType, String> {
        let line = self.read_line();
        println!("Received: {:?}", line);

        let type_symbol = resolve_type(line);
        println!("Type: {:?}", type_symbol);

        match type_symbol {
            TypeSymbol::SimpleString => {
                let string = String::from_utf8(line.to_vec()).expect("Can not convert to string");
                return Ok(DataType::SimpleString(string));
            }
            TypeSymbol::Error => {
                return Err(String::from("Not implemented"));
            }
            TypeSymbol::Integer => {
                return Err(String::from("Not implemented"));
            }
            TypeSymbol::BulkString => {
                return Err(String::from("Not implemented"));
            }
            TypeSymbol::Array => {
                return Err(String::from("Not implemented"));
            }
        }
    }
}

#[derive(Debug)]
enum TypeSymbol {
    SimpleString,
    Integer,
    BulkString,
    Array,
    Error,
}

#[derive(Debug)]
#[derive(PartialEq)]
pub enum DataType {
    SimpleString(String),
    Integer(i64),
    BulkString(Vec<u8>),
    Array(Vec<DataType>),
    Error(String),
}

fn resolve_type(line: &Vec<u8>) -> TypeSymbol {
    match line[0] {
        b'+' => TypeSymbol::SimpleString,
        b'-' => TypeSymbol::Error,
        b':' => TypeSymbol::Integer,
        b'$' => TypeSymbol::BulkString,
        b'*' => TypeSymbol::Array,
        _ => TypeSymbol::Error
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;
    use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpListener};

    use super::*;

    #[test]
    fn it_works() {
        // given
        let test_messages = vec![
            "+OK\r\n",
            "+Echo\r\n",
        ];
        let stream = get_test_stream(test_messages);
        let mut parser = RESPParser::new(stream);

        // when
        let ok_actual = parser.parse_next().expect("Can not parse next");
        let echo_actual = parser.parse_next().expect("Can not parse next");

        // then
        let ok_expected = DataType::SimpleString(String::from("OK"));
        let echo_expected = DataType::SimpleString(String::from("Echo"));

        assert_eq!(ok_expected, ok_actual);
        assert_eq!(echo_expected, echo_actual);
    }

    fn get_test_stream(messages: Vec<&str>) -> TcpStream {
        let mut listener = TcpListener::bind(get_test_ipv4()).expect("Can not bind test listener for accepting connections");
        let mut client = TcpStream::connect(get_test_ipv4()).expect("Can not create test client to connect to test listener");

        for message in messages {
            client.write(message.as_bytes()).expect("Can not write message to test client");
        }

        return listener.accept().expect("Can not accept client connection on test tcp listener").0;
    }

    fn get_test_ipv4() -> SocketAddr {
        SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 9999))
    }
}