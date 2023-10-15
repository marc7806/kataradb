mod resp;
mod io_multiplexer;
mod async_tcp_server;
mod cmd;
mod store;
mod active_expiration;
mod eviction;
mod object_type_encoding;
mod stats;
mod signal;
mod client;

fn main() {
    println!("Starting kataradb");
    async_tcp_server::setup_server();
}
