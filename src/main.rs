use std::io::{self, Write};
use std::thread;
use std::time::{Duration, Instant};

fn main() {
    countdown(10);
}

fn countdown(t: u64) {
    let now = Instant::now();
    while now.elapsed().as_secs() <= t {
        let remaining = t - now.elapsed().as_secs();
        let hours = remaining / 3600;
        let minutes = (remaining % 3600) / 60;
        let seconds = remaining % 60;
        print!("\r{:02}:{:02}:{:02}", hours, minutes, seconds);
        io::stdout().flush().unwrap();
        thread::sleep(Duration::from_millis(1000));
    }
    println!();
}
