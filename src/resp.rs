use std::i64;
use std::io::{Read, Write};
use std::net::TcpStream;

/// RESPParser is responsible for parsing Redis Serialization protocol (RESP2)
/// https://redis.io/topics/protocol
///
/// RESP2 is a binary-safe protocol, meaning you can use it to transmit any kind of data, not only strings.
/// This is a huge advantage compared to protocols such as HTTP for instance, where the request or response body can only be a string.
///
/// Architecture
/// Parser holds temporary buffer in which it reads max X bytes from the stream (x is configurable)
/// Reading stops after \r\n is found
///
/// Author: marc7806
///
const TEMP_BUFFER_SIZE: usize = 1024;

pub struct RESPParser {
    // Temporary buffer used for holding a sequence of bytes until the next CRLF( \r\n )
    line_buffer: Vec<u8>,
}

#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Eq)]
#[derive(Hash)]
#[derive(Clone)]
pub enum DataType {
    SimpleString(String),
    Integer(i64),
    // i64 because int can be negative
    // Defines fixed length. It is binary safe. Good to send any kind of data, also \r\n
    BulkString(String),
    Array(Vec<DataType>),
    Error(String),
}

impl RESPParser {
    pub fn new() -> RESPParser {
        return RESPParser {
            line_buffer: Vec::with_capacity(TEMP_BUFFER_SIZE),
        };
    }

    fn read_line(&mut self, stream: &mut TcpStream) -> Option<&Vec<u8>> {
        if self.line_buffer.len() > 0 {
            // clear buffer if it is not empty
            self.line_buffer.clear();
        }

        // check whether there are bytes available to read
        let mut buffer = [0; 1];
        match stream.peek(&mut buffer) {
            Ok(_) => {
                // continue
            }
            Err(_) => {
                // no bytes available to read
                return None;
            }
        }

        // Parse sequence of bytes until next CRLF
        for byte in stream.bytes() {
            let byte = byte.expect("Can not read byte");

            if byte == b'\r' {
                // stop parsing on carriage return
                // read \n byte followed by \r
                stream.read_exact(&mut [0; 1]).expect("Can not read \n byte");
                break;
            }

            self.line_buffer.push(byte);
        }

        // if buffer is empty then return none
        if self.line_buffer.len() == 0 {
            return None;
        }

        return Some(&self.line_buffer);
    }

    pub fn encode(&mut self, data_type: DataType) -> String {
        return match data_type {
            DataType::SimpleString(string) => {
                format!("+{}\r\n", string)
            }
            DataType::Integer(integer) => {
                format!(":{}\r\n", integer)
            }
            DataType::BulkString(string) => {
                format!("${}\r\n{}\r\n", string.len(), string)
            }
            DataType::Array(array) => {
                let mut encoded_array = format!("*{}\r\n", array.len());
                for data_type in array {
                    encoded_array.push_str(&self.encode(data_type));
                }
                encoded_array
            }
            DataType::Error(error) => {
                format!("-{}\r\n", error)
            }
        };
    }

    /// Parse multiple commands from stream until no more bytes are available to read
    /// Method is used for pipelining
    pub fn decode_next_bulk(&mut self, stream: &mut TcpStream) -> Result<Vec<DataType>, String> {
        let mut bulk = Vec::new();

        loop {
            let data_type = self.decode_next(stream);
            match data_type {
                Ok(data_type) => {
                    bulk.push(data_type);
                }
                Err(_) => {
                    // no more bytes to read
                    break;
                }
            }
        }

        return Ok(bulk);
    }

    /// Parses next sequence of bytes from the stream and decodes it to a [`DataType`]
    pub fn decode_next(&mut self, stream: &mut TcpStream) -> Result<DataType, String> {
        let line_option = self.read_line(stream);
        if line_option.is_none() {
            return Err(String::from("No more bytes to read"));
        }

        let line = line_option.unwrap();
        let type_symbol = line[0];

        // parse bytes to data type starting from second byte (first byte is a type symbol)
        let line = &line[1..];

        return match type_symbol {
            // Simple String
            b'+' => {
                Ok(DataType::SimpleString(Self::read_string(line.to_vec())))
            }
            // Integer
            b':' => {
                let integer = Self::read_int(line);
                Ok(DataType::Integer(integer))
            }
            // Bulk String
            b'$' => {
                let length = Self::read_int(line);

                // if length is 0 or negative, then it is empty
                if length <= 0 {
                    return Ok(DataType::BulkString(String::from("")));
                }

                // +2 because of CRLF
                let mut bulk_string_buffer = vec![0; length as usize];
                stream.read_exact(&mut bulk_string_buffer).expect("Can not read bulk string bytes");

                // read ending CRLF
                stream.read_exact(&mut [0; 2]).expect("Can not read \r\n bytes");

                let bulk_string = Self::read_string(bulk_string_buffer);
                Ok(DataType::BulkString(bulk_string))
            }
            // Array
            b'*' => {
                let length = Self::read_int(line);

                // if length is 0 or negative, then it is empty
                if length <= 0 {
                    return Ok(DataType::Array(vec![]));
                }

                let mut array = Vec::with_capacity(length as usize);
                for _ in 0..length {
                    let data_type = self.decode_next(stream).expect("Can not parse next");
                    array.push(data_type);
                }

                Ok(DataType::Array(array))
            }
            // Error
            b'-' => {
                Ok(DataType::Error(Self::read_string(line.to_vec())))
            }
            _ => {
                Err(String::from("Unknown type symbol"))
            }
        };
    }

    fn read_string(buffer: Vec<u8>) -> String {
        return String::from_utf8(buffer).expect("Can not convert bytes to string");
    }

    fn read_int(line: &[u8]) -> i64 {
        return Self::read_string(line.to_vec()).parse::<i64>().expect("Can not parse string to integer");
    }

    pub fn write_to_stream(&mut self, stream: &mut TcpStream, data: Vec<DataType>) {
        let mut encoded_data = String::new();
        for data_type in data {
            encoded_data.push_str(&self.encode(data_type));
        }
        stream.write_all(encoded_data.as_bytes()).expect("Can not write to stream");
    }

    pub fn flush_stream(&mut self, stream: &mut TcpStream) {
        stream.flush().expect("Can not flush stream");
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;
    use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpListener};

    use rand::Rng;

    use super::*;

    #[test]
    fn test_parse_simple_string() {
        // given
        let test_messages = vec![
            "+OK\r\n",
            "+Echo\r\n",
        ];
        let mut stream = get_test_stream(test_messages);
        let mut parser = RESPParser::new();

        // when
        let ok_actual = parser.decode_next(&mut stream).expect("Can not parse next");
        let echo_actual = parser.decode_next(&mut stream).expect("Can not parse next");

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
        let mut stream = get_test_stream(test_messages);
        let mut parser = RESPParser::new();

        // when
        let zero_actual = parser.decode_next(&mut stream).expect("Can not parse next");
        let one_actual = parser.decode_next(&mut stream).expect("Can not parse next");
        let one_hundred_twenty_three_actual = parser.decode_next(&mut stream).expect("Can not parse next");
        let minus_one_actual = parser.decode_next(&mut stream).expect("Can not parse next");
        let minus_one_hundred_twenty_three_actual = parser.decode_next(&mut stream).expect("Can not parse next");

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

    #[test]
    fn test_parse_bulk_string() {
        // given
        let test_messages = vec![
            "$6\r\nfoobar\r\n",
            "$9\r\nabc\r\n2345\r\n",
            "$-1\r\n",
            "$0\r\n\r\n",
        ];
        let mut stream = get_test_stream(test_messages);
        let mut parser = RESPParser::new();

        // when
        let foobar_actual = parser.decode_next(&mut stream).expect("Can not parse next");
        let abcd12345_actual = parser.decode_next(&mut stream).expect("Can not parse next");
        let null_actual = parser.decode_next(&mut stream).expect("Can not parse next");
        let empty_string_actual = parser.decode_next(&mut stream).expect("Can not parse next");

        // then
        let foobar_expected = DataType::BulkString(String::from("foobar"));
        let abcd12345_expected = DataType::BulkString(String::from("abc\r\n2345"));
        let null_expected = DataType::BulkString(String::from(""));
        let empty_string_expected = DataType::BulkString(String::from(""));

        assert_eq!(foobar_expected, foobar_actual);
        assert_eq!(abcd12345_expected, abcd12345_actual);
        assert_eq!(null_expected, null_actual);
        assert_eq!(empty_string_expected, empty_string_actual);
    }

    #[test]
    fn test_parse_array() {
        // given
        let test_messages = vec![
            "*2\r\n$3\r\nfoo\r\n$3\r\nbar\r\n",
            "*4\r\n:1\r\n:2\r\n:3\r\n+echo\r\n",
            "*5\r\n:1\r\n:2\r\n:3\r\n:4\r\n$6\r\nfoobar\r\n",
            "*-1\r\n",
            "*0\r\n",
            "*2\r\n*3\r\n:1\r\n:2\r\n:3\r\n*2\r\n+Foo\r\n-Bar\r\n",
        ];
        let mut stream = get_test_stream(test_messages);
        let mut parser = RESPParser::new();

        // when
        let foo_bar_actual = parser.decode_next(&mut stream).expect("Can not parse next");
        let one_two_three_echo_actual = parser.decode_next(&mut stream).expect("Can not parse next");
        let one_two_three_four_foobar_actual = parser.decode_next(&mut stream).expect("Can not parse next");
        let null_actual = parser.decode_next(&mut stream).expect("Can not parse next");
        let empty_array_actual = parser.decode_next(&mut stream).expect("Can not parse next");
        let nested_array_actual = parser.decode_next(&mut stream).expect("Can not parse next");

        // then
        let foo_bar_expected = DataType::Array(vec![
            DataType::BulkString(String::from("foo")),
            DataType::BulkString(String::from("bar")),
        ]);
        let one_two_three_echo_expected = DataType::Array(vec![
            DataType::Integer(1),
            DataType::Integer(2),
            DataType::Integer(3),
            DataType::SimpleString(String::from("echo")),
        ]);
        let one_two_three_four_foobar_expected = DataType::Array(vec![
            DataType::Integer(1),
            DataType::Integer(2),
            DataType::Integer(3),
            DataType::Integer(4),
            DataType::BulkString(String::from("foobar")),
        ]);
        let null_expected = DataType::Array(vec![]);
        let empty_array_expected = DataType::Array(vec![]);
        let nested_array_expected = DataType::Array(vec![
            DataType::Array(vec![
                DataType::Integer(1),
                DataType::Integer(2),
                DataType::Integer(3),
            ]),
            DataType::Array(vec![
                DataType::SimpleString(String::from("Foo")),
                DataType::Error(String::from("Bar")),
            ]),
        ]);

        assert_eq!(foo_bar_expected, foo_bar_actual);
        assert_eq!(one_two_three_echo_expected, one_two_three_echo_actual);
        assert_eq!(one_two_three_four_foobar_expected, one_two_three_four_foobar_actual);
        assert_eq!(null_expected, null_actual);
        assert_eq!(empty_array_expected, empty_array_actual);
        assert_eq!(nested_array_expected, nested_array_actual);
    }

    #[test]
    fn test_parse_error() {
        // given
        let test_messages = vec![
            "-WRONGTYPE Operation against a key holding the wrong kind of value\r\n",
            "-ERR unknown command 'foobar'\r\n",
        ];
        let mut stream = get_test_stream(test_messages);
        let mut parser = RESPParser::new();

        // when
        let wrong_type_actual = parser.decode_next(&mut stream).expect("Can not parse next");
        let unknown_command_actual = parser.decode_next(&mut stream).expect("Can not parse next");

        // then
        let wrong_type_expected = DataType::Error(String::from("WRONGTYPE Operation against a key holding the wrong kind of value"));
        let unknown_command_expected = DataType::Error(String::from("ERR unknown command 'foobar'"));

        assert_eq!(wrong_type_expected, wrong_type_actual);
        assert_eq!(unknown_command_expected, unknown_command_actual);
    }

    #[test]
    fn test_decode_next_bulk() {
        // given
        let test_messages = vec![
            "*1\r\n$4\r\nPING\r\n*3\r\n$3\r\nSET\r\n$1\r\nk\r\n$1\r\nv\r\n*2\r\n$3\r\nGET\r\n$1\r\nk\r\n",
        ];
        let mut stream = get_test_stream(test_messages);
        let mut parser = RESPParser::new();

        // when
        let bulk = parser.decode_next_bulk(&mut stream).expect("Can not parse next");

        // then
        let expected_bulk = vec![
            DataType::Array(vec![
                DataType::BulkString(String::from("PING")),
            ]),
            DataType::Array(vec![
                DataType::BulkString(String::from("SET")),
                DataType::BulkString(String::from("k")),
                DataType::BulkString(String::from("v")),
            ]),
            DataType::Array(vec![
                DataType::BulkString(String::from("GET")),
                DataType::BulkString(String::from("k")),
            ]),
        ];

        assert_eq!(expected_bulk, bulk);
    }

    fn get_test_stream(messages: Vec<&str>) -> TcpStream {
        let addr = get_test_ipv4();
        let listener = TcpListener::bind(addr).expect("Can not bind test listener for accepting connections");
        let listener_guard = scopeguard::guard(listener, |listener| {
            println!("Closing test listener");
            drop(listener)
        });
        let mut client = TcpStream::connect(addr).expect("Can not create test client to connect to test listener");

        for message in messages {
            client.write(message.as_bytes()).expect("Can not write message to test client");
        }

        return listener_guard.accept().expect("Can not accept client connection on test tcp listener").0;
    }

    fn get_test_ipv4() -> SocketAddr {
        SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), get_random_port()))
    }

    fn get_random_port() -> u16 {
        let mut rng = rand::thread_rng();
        rng.gen_range(1024..65535)
    }
}
