use crate::error::Error;
use std::io::{stdin, BufRead};
use std::sync::mpsc::{channel, Receiver};
use std::thread::{self, JoinHandle};
use tracing::{trace, warn};

pub fn stream_input(num_lines: usize) -> (Receiver<Result<Vec<String>, Error>>, JoinHandle<()>) {
    trace!("Opening channel for input reader");
    let (tx, rx) = channel::<Result<Vec<String>, Error>>();
    let thread_handle = thread::spawn(move || {
        trace!("Reading input");
        let input = stdin().lock();
        trace!("Splitting input");
        let mut input_lines = input.split(b'\n');

        loop {
            trace!("Reading lines");
            let mut maybe_err = None;
            let mut lines = Vec::with_capacity(num_lines);
            for _ in 0..num_lines {
                match input_lines.next() {
                    Some(Ok(buf)) => {
                        trace!("Got lines");
                        let line = String::from_utf8_lossy(&buf).to_string();
                        lines.push(line);
                    }
                    Some(Err(err)) => {
                        warn!("Error reading input lines: {err}");
                        maybe_err = Some(err);
                        break;
                    }
                    None => {
                        trace!("No new lines");
                        return;
                    }
                }
            }
            if let Err(err) = tx.send(Ok(lines)) {
                warn!("Error sending input streaming result: {err}");
                return;
            }
            if let Some(read_err) = maybe_err {
                warn!("Got read error streaming input: {read_err}");
                if let Err(_send_err) = tx.send(Err(Error::StreamingSend)) {
                    return;
                }
            };
        }
    });
    (rx, thread_handle)
}
