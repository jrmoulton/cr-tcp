use std::io;
use std::process::Command;

use tun_tap::{Iface, Mode};

fn main() -> io::Result<()> {
    // Create the tun interface.
    let nic = Iface::new("tun0", Mode::Tun).unwrap();

    // Configure the „local“ (kernel) endpoint.
    cmd("ip", &["addr", "add", "dev", nic.name(), "192.126.0.2/24"]);
    cmd("ip", &["link", "set", "up", "dev", nic.name()]);
    println!(
        "Created interface {}. Send some packets into it and see they're printed here",
        nic.name()
    );
    println!("You can for example ping 10.107.1.3 (it won't answer) \n");

    // Create a location for the bytes to be written to and then continually read and print the
    // bytes that are read
    let mut buff = vec![0; 1504];
    loop {
        // Every read is one packet. If the buffer is too small, bad luck, it gets truncated.
        let nbytes = nic.recv(&mut buff).unwrap();
        assert!(nbytes >= 4);
        eprintln!("Read {}: {:x?}", nbytes, &buff[..nbytes]);
    }
}

/// Run a shell command. Panic if it fails in any way.
fn cmd(cmd: &str, args: &[&str]) {
    let ecode = Command::new(cmd)
        .args(args)
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
    assert!(ecode.success(), "Failed to execte {}", cmd);
}
