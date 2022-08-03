use sha2::{Sha256, Digest};
use std::fs;
use std::io::Read;

fn main() {
    // random.dat created with: dd if=/dev/random of=random.dat bs=1M count=4
    // This program was tested during development with: cargo run && sha256sum random.dat
    // and visually comparing the results
    let mut the_file = match fs::File::open("./random.dat") {
        Ok(file) => file,
        Err(e) => {
            println!("An error occurred: {}", e);
            return;
        }
    };

    let mut hasher = Sha256::new();

    let mut buf: [u8; 4096] = [0; 4096]; // Read 4k at a time
    loop {
        let num_bytes_read = match the_file.read(&mut buf) {
            Ok(thing) => thing,
            Err(e) => {
                println!("An error occurred: {}", e);
                return;
            }
        };
        //println!("num_bytes_read: {}", num_bytes_read);
        if num_bytes_read == 0 {
            break;
        }
        // read bytes from the file, pass them to the hasher:
        hasher.update(&buf[..num_bytes_read]);
    }

    let result = hasher.finalize();
    println!("{:x}", result);
}