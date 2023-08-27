use std::i64;
use std::io::Read;
use std::net::TcpStream;

use crate::TEMP_BUFFER_SIZE;

/// RESPParser is responsible for parsing Redis Serialization protocol (RESP)

pub struct RESPParser {
    pub stream: TcpStream,
    // Temporary buffer used for holding a sequence of bytes until the next CRLF( \r\n )
    line_buffer: Vec<u8>,
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
    Integer(i64), // i64 because int can be negative
    BulkString(String),
    Array(Vec<DataType>),
    Error(String),
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
        let mut tcp_stream = &self.stream.try_clone().expect("Can not clone stream");
        for byte in tcp_stream.bytes() {
            let byte = byte.expect("Can not read byte");
            println!("{}", byte);

            if byte == b'\r' {
                // stop parsing on carriage return
                // read \n byte followed by \r
                &tcp_stream.read_exact(&mut [0; 1]).expect("Can not read \n byte");
                break;
            }

            self.line_buffer.push(byte);
        }

        return &self.line_buffer;
    }

    /// Parses next sequence of bytes from the stream and decodes it to a [`DataType`]
    pub fn parse_next(&mut self) -> Result<DataType, String> {
        let line = self.read_line();
        println!("Received: {:?}", line);

        let type_symbol = line[0];

        // parse bytes to data type starting from second byte (first byte is a type symbol)
        let line = &line[1..];

        match type_symbol {
            // Simple String
            b'+' => {
                let string = String::from_utf8(line.to_vec()).expect("Can not convert bytes to string");
                return Ok(DataType::SimpleString(string));
            }
            // Integer
            b':' => {
                let integer = String::from_utf8(line.to_vec()).expect("Can not convert bytes to string").parse::<i64>().expect("Can not parse string to integer");
                return Ok(DataType::Integer(integer));
            }
            // Bulk String
            b'$' => {
                let string = String::from_utf8(line.to_vec()).expect("Can not convert bytes to string");
                return Ok(DataType::BulkString(string));
            }
            // Array
            b'*' => {
                let string = String::from_utf8(line.to_vec()).expect("Can not convert bytes to string");
                let integer = string.parse::<i64>().expect("Can not parse string to integer");
                return Ok(DataType::Integer(integer));
            }
            // Error
            b'-' => {
                let string = String::from_utf8(line.to_vec()).expect("Can not convert bytes to string");
                return Ok(DataType::Error(string));
            }
            _ => {
                return Err(String::from("Unknown type symbol"));
            }
        }


    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;
    use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpListener};

    use super::*;

    #[test]
    fn test_parse_simple_string() {
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

    #[test]
    fn test_parse_integer() {
        // given
        let test_messages = vec![
            ":0\r\n",
            ":1\r\n",
            ":123\r\n",
            ":-1\r\n",
            ":-123\r\n",
        ];
        let stream = get_test_stream(test_messages);
        let mut parser = RESPParser::new(stream);

        // when
        let zero_actual = parser.parse_next().expect("Can not parse next");
        let one_actual = parser.parse_next().expect("Can not parse next");
        let one_hundred_twenty_three_actual = parser.parse_next().expect("Can not parse next");
        let minus_one_actual = parser.parse_next().expect("Can not parse next");
        let minus_one_hundred_twenty_three_actual = parser.parse_next().expect("Can not parse next");

        // then
        let zero_expected = DataType::Integer(0);
        let one_expected = DataType::Integer(1);
        let one_hundred_twenty_three_expected = DataType::Integer(123);
        let minus_one_expected = DataType::Integer(-1);
        let minus_one_hundred_twenty_three_expected = DataType::Integer(-123);

        assert_eq!(zero_expected, zero_actual);
        assert_eq!(one_expected, one_actual);
        assert_eq!(one_hundred_twenty_three_expected, one_hundred_twenty_three_actual);
        assert_eq!(minus_one_expected, minus_one_actual);
        assert_eq!(minus_one_hundred_twenty_three_expected, minus_one_hundred_twenty_three_actual);
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