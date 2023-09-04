use std::io::{BufRead, Read, Write};

mod resp;
mod io_multiplexer;
mod async_tcp_server;
mod cmd;
mod store;

fn main() {
    println!("Starting kataradb");
    async_tcp_server::setup_server();
}
