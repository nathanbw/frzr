use signal_hook::{consts::SIGINT, consts::SIGPIPE, iterator::Signals};
use std::{error::Error, thread, time::Duration};

fn main() -> Result<(), Box<dyn Error>> {
    let mut signals = Signals::new(&[SIGINT, SIGPIPE])?;

    thread::spawn(move || {
        for sig in signals.forever() {
            println!("Received signal {:?}", sig);
        }
    });

    let mut i = 0;
    while i < 5 {
        i = i + 1;
        thread::sleep(Duration::from_secs(1));
        eprintln!("stderr is here");
        println!("stdout stuff is here");
    }
    Ok(())
}

