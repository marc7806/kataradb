use std::net::{IpAddr, Ipv4Addr, TcpListener};
use std::os::fd::AsRawFd;

use libc::timespec;

const PORT: i16 = 9977;
const ADDRESS: IpAddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)); // IPv4 addresses are 32bit sized

trait IOMultiplexer {
    fn new() -> Self;
    fn register(&mut self, fd: i32, flags: u16, filter: i16) -> i32;
    fn poll(&mut self, timeout: timespec) -> i32;
    fn close(&mut self);
}

pub struct DarwinIOMultiplexer {
    kq: i32,
}

impl IOMultiplexer for DarwinIOMultiplexer {
    fn new() -> Self {
        let kq = unsafe { libc::kqueue() };
        DarwinIOMultiplexer { kq }
    }

    /// Register a file descriptor with the kernel queue to receive events of a certain type (filter) and with certain flags (flags)
    /// ident: file descriptor
    /// flags: EV_ADD, EV_DELETE, EV_ENABLE, EV_DISABLE, EV_CLEAR, EV_RECEIPT, EV_ONESHOT, EV_DISPATCH, EV_UDATA_SPECIFIC
    /// filter: EVFILT_READ, EVFILT_WRITE, EVFILT_AIO, EVFILT_VNODE, EVFILT_PROC, EVFILT_SIGNAL, EVFILT_TIMER, EVFILT_MACHPORT, EVFILT_FS, EVFILT_USER, EVFILT_VM, EVFILT_SYSCOUNT
    fn register(&mut self, fd: i32, flags: u16, filter: i16) -> i32 {
        let mut event = libc::kevent {
            ident: fd as libc::uintptr_t,
            filter,
            flags,
            fflags: 0,
            data: 0,
            udata: 0 as *mut libc::c_void,
        };

        unsafe { libc::kevent(self.kq, &mut event, 1, std::ptr::null_mut(), 0, std::ptr::null()) }
    }

    /// Poll for events on the kernel queue
    fn poll(&mut self, timeout: timespec) -> i32 {
        let mut event = libc::kevent {
            ident: 0,
            filter: 0,
            flags: 0,
            fflags: 0,
            data: 0,
            udata: 0 as *mut libc::c_void,
        };

        unsafe { libc::kevent(self.kq, std::ptr::null_mut(), 0, &mut event, 1, &timeout) }
    }

    fn close(&mut self) {
        unsafe { libc::close(self.kq) };
    }
}

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

    // listen to incoming connections with max 20k connections

}