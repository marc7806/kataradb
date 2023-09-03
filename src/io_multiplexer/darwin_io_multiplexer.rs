use libc::timespec;

use crate::io_multiplexer::io_multiplexer::{Event, IOMultiplexer};

/// Implementation of the IOMultiplexer trait for the Darwin (macOS) operating system
/// uses the kqueue system call
/// see docs: https://man.freebsd.org/cgi/man.cgi?query=kqueue&sektion=2
///
/// We are using libc (C standard library) to make system calls as this library is officially supported by the rust team and has no significant performance downsides

pub struct DarwinIOMultiplexer {
    kq: i32,
    // buffer to save the received events from poll
    kq_event_buf: Vec<libc::kevent>,
    // buffer to save the converted events from kq_event_buf
    kdb_events: Vec<Event>,
}

impl IOMultiplexer for DarwinIOMultiplexer {
    fn new(max_clients: usize) -> Self {
        println!("Creating kqueue");

        // kqueue system call creates a new kernel event queue and returns its file descriptor
        let kq = unsafe { libc::kqueue() };
        if kq == -1 {
            panic!("Can not create kqueue");
        }

        DarwinIOMultiplexer {
            kq,
            kq_event_buf: Vec::with_capacity(max_clients),
            kdb_events: Vec::with_capacity(max_clients),
        }
    }

    /// Register a file descriptor with the kernel queue to receive events of a certain type (filter) and with certain flags (flags)
    /// ident: file descriptor
    /// flags: EV_ADD, EV_DELETE, EV_ENABLE, EV_DISABLE, EV_CLEAR, EV_RECEIPT, EV_ONESHOT, EV_DISPATCH, EV_UDATA_SPECIFIC
    fn register(&mut self, event: Event) -> i32 {
        let add_event_result = unsafe { libc::kevent(self.kq, &mut event.to_kevent(libc::EV_ADD), 1, std::ptr::null_mut(), 0, std::ptr::null()) };

        if add_event_result == -1 {
            panic!("Can not register event for kqueue");
        }

        return add_event_result;
    }

    /// Poll for events on the kernel queue
    fn poll(&mut self, timeout: timespec) -> Result<Vec<Event>, String> {
        // kevent writes all changed events into the event buffer array
        let event_count = unsafe { libc::kevent(self.kq, std::ptr::null_mut(), 0, self.kq_event_buf.as_mut_ptr(), self.kq_event_buf.capacity() as i32, &timeout) };

        if event_count == -1 {
            panic!("Can not poll kqueue");
        }

        unsafe {
            for i in 0..event_count {
                let event = self.kq_event_buf.get_unchecked(i as usize);
                let converted_event = Event::from_kevent(event);

                if self.kdb_events.len() <= i as usize {
                    self.kdb_events.push(converted_event);
                } else {
                    self.kdb_events[i as usize] = converted_event;
                }
            }
        }

        return Ok(self.kdb_events[0..event_count as usize].to_vec());
    }

    fn close(&self) {
        println!("Closing kqueue");

        let close_result = unsafe { libc::close(self.kq) };

        if close_result == -1 {
            panic!("Can not close kqueue");
        }
    }
}