use std::net::{IpAddr, Ipv4Addr, TcpListener};
use std::os::fd::AsRawFd;

use libc::timespec;

const PORT: i16 = 9977;
const ADDRESS: IpAddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)); // IPv4 addresses are 32bit sized

trait IOMultiplexer {
    fn new(max_clients: usize) -> Self;
    fn register(&mut self, event: Event) -> i32;
    fn poll(&mut self, timeout: timespec) -> Result<Vec<Event>, String>;
    fn close(&self);
}

pub struct Event {
    pub fd: i32,
    // Describes the type of event
    // EVFILT_READ, EVFILT_WRITE, EVFILT_AIO, EVFILT_VNODE, EVFILT_PROC, EVFILT_SIGNAL, EVFILT_TIMER, EVFILT_MACHPORT, EVFILT_FS, EVFILT_USER, EVFILT_VM, EVFILT_SYSCOUNT
    // EVFILT_READ means that the event is triggered when the file descriptor is ready for reading
    // EV_FILT_READ means that the event is triggered when the file descriptor is ready for writing
    pub filter: i16,
}

impl Event {
    pub fn new(fd: i32, filter: i16) -> Self {
        Event { fd, filter }
    }

    pub fn to_kevent(&self, flags: u16) -> libc::kevent {
        libc::kevent {
            ident: self.fd as libc::uintptr_t,
            filter: self.filter,
            flags,
            fflags: 0,
            data: 0,
            udata: 0 as *mut libc::c_void,
        }
    }
}

pub struct DarwinIOMultiplexer {
    kq: i32,
    // buffer to save the received events from poll
    kq_event_buf: Vec<libc::kevent>,
}

/// kqueue docs see: https://man.freebsd.org/cgi/man.cgi?query=kqueue&sektion=2
impl IOMultiplexer for DarwinIOMultiplexer {
    fn new(max_clients: usize) -> Self {
        // kqueue system call creates a new kernel event queue and returns its file descriptor
        println!("Creating kqueue");
        let kq = unsafe { libc::kqueue() };
        if kq == -1 {
            panic!("Can not create kqueue");
        }
        DarwinIOMultiplexer { kq, kq_event_buf: Vec::with_capacity(max_clients) }
    }

    /// Register a file descriptor with the kernel queue to receive events of a certain type (filter) and with certain flags (flags)
    /// ident: file descriptor
    /// flags: EV_ADD, EV_DELETE, EV_ENABLE, EV_DISABLE, EV_CLEAR, EV_RECEIPT, EV_ONESHOT, EV_DISPATCH, EV_UDATA_SPECIFIC
    /// todo: proper error handling...
    fn register(&mut self, event: Event) -> i32 {
        let add_event_result = unsafe { libc::kevent(self.kq, &mut event.to_kevent(libc::EV_ADD), 1, std::ptr::null_mut(), 0, std::ptr::null()) };
        if add_event_result == -1 {
            panic!("Can not register event for kqueue");
        } else {
            return add_event_result;
        }
    }

    /// Poll for events on the kernel queue
    fn poll(&mut self, timeout: timespec) -> Result<Vec<Event>, String> {
        // kevent writes all changed events into the event buffer array
        let event_count = unsafe { libc::kevent(self.kq, std::ptr::null(), 0, self.kq_event_buf.as_mut_ptr(), self.kq_event_buf.capacity() as i32, &timeout) };
        if event_count == -1 {
            panic!("Can not poll kqueue");
        } else {
            let mut events = Vec::with_capacity(event_count as usize);
            unsafe {
                for i in 0..event_count {
                    events.push(Event::new(self.kq_event_buf.get_unchecked(i as usize).ident as i32, self.kq_event_buf.get_unchecked(i as usize).filter));
                }
            }
            return Ok(events);
        }
    }

    fn close(&self) {
        println!("Closing kqueue");
        let close_result = unsafe { libc::close(self.kq) };
        if close_result == -1 {
            panic!("Can not close kqueue");
        }
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

    // listen to incoming connections
    let io_multiplex = DarwinIOMultiplexer::new(1024);
    let mut io_multiplexer = scopeguard::guard(io_multiplex, |io_multiplexer| io_multiplexer.close());

    let mut event = Event::new(listener_fd, libc::EVFILT_READ);
    io_multiplexer.register(event);


    // loop forever
    loop {
        // poll for events
        let events = io_multiplexer.poll(timespec { tv_sec: 1, tv_nsec: 0 });
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
                    } else {
                        println!("Got new data");
                        let mut buf = [0; 1024];
                        let read_result = unsafe { libc::read(event.fd, buf.as_mut_ptr() as *mut libc::c_void, buf.len()) };
                        if read_result == -1 {
                            panic!("Can not read from file descriptor");
                        } else {
                            println!("Read {read_result} bytes");
                        }
                    }
                }
            }
            Err(e) => {
                panic!("Can not poll for events");
            }
        }
    }

}