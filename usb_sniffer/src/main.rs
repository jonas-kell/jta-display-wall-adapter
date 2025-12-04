use std::io::{self, Read};
use std::time::Duration;

fn main() -> io::Result<()> {
    // let port_path = "/dev/serial/by-id/usb-Alex_Taradov_USB_Sniffer_Lite__RP2040__7A6B5C58-if00";
    let port_path = "COM4";

    println!("Try open port...");
    let mut port = serialport::new(port_path, 3_000_000)
        .timeout(Duration::from_millis(500))
        .open()?;
    println!("Port opened!");

    let mut buf = [0u8; 4096];

    std::thread::sleep(std::time::Duration::from_millis(50)); // probably not needed if we correctly set the things below

    println!("Writing terminal ready...");
    port.write_data_terminal_ready(true)?; // only needed on windows
    port.write_request_to_send(true)?; // not yet seen what difference this makes, but supposedly a good idea
    println!("Terminal ready set!");

    println!("Starting first capture...");
    port.write_all(b"s")?;
    println!("First capture started...");

    loop {
        match port.read(&mut buf) {
            Ok(n) => {
                // Print raw text from sniffer firmware
                let decoded = String::from_utf8_lossy(&buf[..n]);

                for decoded_line in decoded.lines() {
                    if decoded_line.contains("DATA") {
                        println!("{}", decoded_line);
                    }

                    if decoded_line.starts_with("Total:") {
                        // we got all the data, restart scanning
                        port.write_all(b"s")?;
                        println!("Restarted capture");
                    }
                }
            }
            Err(ref e) if e.kind() == io::ErrorKind::TimedOut => {
                port.write_all(b"p")?; // stop capture to let us read (we want data at least every timeout if possible)
            }
            Err(e) => {
                print!("Unknown read error: {}", e.to_string());
            }
        }
    }
}
