use std::thread;
use crossbeam_channel::{bounded, Receiver};
use libc::{c_int, SIGINT, SIGTERM};
use signal_hook::iterator::Signals;

pub fn listen_for_shutdown_signals() -> Result<Receiver<c_int>, String> {
    let (sender, receiver) = bounded(100);

    let signals = Signals::new(&[SIGINT, SIGTERM]);
    thread::spawn(move || {
        for sig in signals.unwrap().forever() {
            println!("Received shutdown signal {:?}", sig);
            println!("Shutting down...");
            let _ = sender.send(sig);
        }
    });

    Ok(receiver)
}
