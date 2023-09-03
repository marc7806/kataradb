use libc::{timespec, uintptr_t};

pub trait IOMultiplexer {
    fn new(max_clients: usize) -> Self;
    fn register(&mut self, event: Event) -> i32;
    fn poll(&mut self, timeout: timespec) -> Result<Vec<Event>, String>;
    fn close(&self);
}

#[derive(Clone)]
pub struct Event {
    pub fd: i32,
    // filter describes the type of event to monitor
    // * EVFILT_READ means that the event is triggered when the file descriptor is ready for reading
    // * EV_FILT_READ means that the event is triggered when the file descriptor is ready for writing
    pub filter: i16,
    // flag that indicates whether a connection got closed by the client
    pub connection_closed: bool,
}

impl Event {
    pub fn new(fd: i32, filter: i16) -> Self {
        Event { fd, filter, connection_closed: false }
    }

    pub fn to_kevent(&self, flags: u16) -> libc::kevent {
        libc::kevent {
            ident: self.fd as uintptr_t,
            filter: self.filter,
            flags,
            fflags: 0,
            data: 0,
            udata: std::ptr::null_mut(),
        }
    }

    pub fn from_kevent(event: &libc::kevent) -> Self {
        // use bitwise-and to check if the connection got closed
        let connection_closed = event.flags & libc::EV_EOF != 0;

        Event {
            fd: event.ident as i32,
            filter: event.filter,
            connection_closed
        }
    }
}