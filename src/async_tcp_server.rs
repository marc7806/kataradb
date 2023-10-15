use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, TcpListener};
use std::os::fd::{AsRawFd, RawFd};

use libc::{timespec};

use crate::active_expiration::ActiveExpirationManager;
use crate::client::ClientConnection;
use crate::cmd::command::SimpleCommand::BGREWRITEAOF;
use crate::cmd::handler::CommandHandler;
use crate::io_multiplexer::darwin_io_multiplexer::DarwinIOMultiplexer;
use crate::io_multiplexer::io_multiplexer::{Event, IOMultiplexer};
use crate::signal::listen_for_shutdown_signals;
use crate::store::Store;

const PORT: i16 = 9977;
const ADDRESS: IpAddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
const MAX_CLIENT_CONNECTIONS: usize = 1024;

pub fn setup_server() {
    let (listener, listener_fd) = setup_tcp_listener();
    let mut store = Store::new();
    start_event_loop(listener, listener_fd, &mut store);
}

fn start_event_loop(listener: TcpListener, listener_fd: RawFd, store: &mut Store) {
    // listen to process signals
    let signal_receiver = listen_for_shutdown_signals().expect("Can not listen for process signals");

    // listen to incoming connections
    let mut io_multiplexer = DarwinIOMultiplexer::new(MAX_CLIENT_CONNECTIONS);

    // register tcp server socket - needed in order to listen for new client connections
    let event = Event::new(listener_fd, libc::EVFILT_READ);
    io_multiplexer.register(event).expect("Can not register TCP server socket");

    // if the client connection goes out of scope, the connection will be closed. Because of this we need to store the connections
    let mut client_connections = HashMap::new();

    let mut command_handler = CommandHandler::new();

    let mut active_expiration_manager = ActiveExpirationManager::new(3000);

    // event loop
    loop {
        // check for shutdown signals
        match signal_receiver.try_recv() {
            Ok(_) => {
                cleanup(&mut io_multiplexer, store, &mut command_handler);
                std::process::exit(0);
            }
            Err(_) => {}
        }

        active_expiration_manager.run_loop(store);

        let events = io_multiplexer.poll(timespec { tv_sec: 0, tv_nsec: 0 });

        match events {
            Ok(events) => {
                for event in events {

                    if event.fd == listener_fd {
                        println!("New client connection");
                        let (stream, _) = listener.accept().expect("Can not accept connection");
                        stream.set_nonblocking(true).expect("Cannot set non-blocking");

                        let stream_fd = stream.as_raw_fd();
                        let event = Event::new(stream_fd, libc::EVFILT_READ);
                        match io_multiplexer.register(event) {
                            Ok(_) => {
                                client_connections.insert(stream_fd, ClientConnection::new(stream));
                            }
                            Err(e) => {
                                println!("{}", e);
                                continue;
                            }
                        }
                    } else {
                        let mut client_connection = client_connections.get_mut(&event.fd).expect("Can not get stream");

                        if event.has_data {
                            command_handler.handle_bulk(&mut client_connection, store);
                        }

                        if event.connection_closed {
                            println!("Connection got closed by client");
                            client_connections.remove(&event.fd);
                            continue;
                        }
                    }

                }
            }
            Err(e) => {
                println!("{}", e);
            }
        }
    }
}

fn cleanup(io_multiplexer: &mut DarwinIOMultiplexer, store: &mut Store, cmd_handler: &mut CommandHandler) {
    io_multiplexer.close();
    cmd_handler.execute_simple_command(&BGREWRITEAOF, &mut Vec::new(), store);
}

fn setup_tcp_listener() -> (TcpListener, RawFd) {
    println!("Setting up tcp listener...");
    let listener = TcpListener::bind(format!("{ADDRESS}:{PORT}")).expect("Can not create TCP listener");
    listener.set_nonblocking(true).expect("Cannot set non-blocking");
    // a file descriptor is a number that uniquely identifies an open file in a computer's operating system
    let listener_fd = listener.as_raw_fd();
    (listener, listener_fd)
}
