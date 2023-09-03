use std::collections::HashMap;
use std::io::Read;
use std::net::{IpAddr, Ipv4Addr, TcpListener};
use std::os::fd::AsRawFd;

use libc::timespec;

use crate::io_multiplexer::darwin_io_multiplexer::DarwinIOMultiplexer;
use crate::io_multiplexer::io_multiplexer::{Event, IOMultiplexer};

const PORT: i16 = 9977;
const ADDRESS: IpAddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)); // IPv4 addresses are 32bit sized

/// We are using libc (C standard library) to make system calls as this library is officially supported by the rust team and has no significant performance downsides
pub fn setup_server() {
    let listener = TcpListener::bind(format!("{ADDRESS}:{PORT}")).expect("Can not create TCP listener");
    listener.set_nonblocking(true).expect("Cannot set non-blocking");

    println!("Setting up listener");

    // a file descriptor is a number that uniquely identifies an open file in a computer's operating system
    let listener_fd = listener.as_raw_fd();

    // get process id using libc crate
    let pid = unsafe { libc::getpid() };
    println!("PID: {pid}");

    // listen to incoming connections
    let io_multiplex = DarwinIOMultiplexer::new(1024);
    let mut io_multiplexer = scopeguard::guard(io_multiplex, |io_multiplexer| io_multiplexer.close());

    let mut event = Event::new(listener_fd, libc::EVFILT_READ);
    io_multiplexer.register(event);

    let mut client_connections = HashMap::new();

    // loop forever
    loop {
        // poll for events infinitely
        let events = io_multiplexer.poll(timespec { tv_sec: 0, tv_nsec: 0 });
        match events {
            Ok(events) => {
                for event in events {
                    if event.fd == listener_fd {
                        println!("Got new connection");
                        let (stream, _) = listener.accept().expect("Can not accept connection");
                        stream.set_nonblocking(true).expect("Cannot set non-blocking");
                        let stream_fd = stream.as_raw_fd();
                        let mut event = Event::new(stream_fd, libc::EVFILT_READ);
                        io_multiplexer.register(event);

                        client_connections.insert(stream_fd, stream);
                    } else {
                        println!("Got new data");
                        let stream = client_connections.get_mut(&event.fd).expect("Can not get stream");

                        if event.connection_closed {
                            println!("Connection got closed by client");
                            client_connections.remove(&event.fd);
                            continue;
                        }

                        let mut buf = [0; 1024];
                        stream.read(&mut buf).expect("Can not read from stream");
                        println!("Got data: {:?}", buf);
                    }
                }
            }
            Err(e) => {
                panic!("Can not poll for events");
            }
        }
    }
}