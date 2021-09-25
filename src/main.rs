#![allow(dead_code, unused_variables)]
use std::collections::HashMap;
use std::io;
use std::net::Ipv4Addr;
use std::process::Command;
use tun_tap::{Iface, Mode};
mod tcp;

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
struct Quad {
    src: (Ipv4Addr, u16),
    dst: (Ipv4Addr, u16),
}

fn main() -> io::Result<()> {
    // Create the tun interface.
    let &mut nic = Iface::new("tun0", Mode::Tun).unwrap();

    // Create a location for the bytes to be written to and then continually read and print the
    // bytes that are read
    let mut buff = vec![0; 1504];
    let mut connections: HashMap<Quad, tcp::State> = Default::default();
    loop {
        // Every read is one packet. If the buffer is too small, bad luck, it gets truncated.
        let nbytes = nic.recv(&mut buff).unwrap();
        let _eth_flags = u16::from_be_bytes([buff[0], buff[1]]);
        let eth_proto = u16::from_be_bytes([buff[2], buff[3]]);
        if eth_proto != 0x0800 {
            // if the ehertype is not IPv4 go to next loop iteration
            continue;
        }
        let ip_packet = &buff[4..];
        match etherparse::Ipv4HeaderSlice::from_slice(ip_packet) {
            Ok(ip_header) => {
                let src = ip_header.source_addr();
                let dst = ip_header.destination_addr();
                if ip_header.protocol() != 0x06 {
                    // if the packet is not tcp go to next iteration
                    continue;
                }
                let tcp_packet = &ip_packet[ip_header.slice().len()..];
                match etherparse::TcpHeaderSlice::from_slice(tcp_packet) {
                    Ok(tcp_header) => {
                        // Payload is from the end of the tcp_header until the total number of bytes
                        // recieved but not necessarily the full size of the buffer in memory
                        let payload = &tcp_packet[tcp_header.slice().len()..nbytes];
                        connections
                            .entry(Quad {
                                src: (src, tcp_header.source_port()),
                                dst: (dst, tcp_header.destination_port()),
                            })
                            .or_default()
                            .on_packet(ip_header, tcp_header, payload, &mut nic)?;
                    }
                    Err(e) => {
                        eprintln!("ignoring weird tcp packet {:?}", e);
                    }
                }
            }
            Err(e) => {
                eprintln!("igrnoring weird packet {:?}", e);
            }
        }
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
