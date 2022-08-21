extern crate pnet;
extern crate pnet_datalink;

use pnet::packet::ethernet::EthernetPacket;

use std::env;
use std::io::{self, Write};
use std::process;

fn handle_ethernet_frame(ethernet: &EthernetPacket) {
    println!("{} {}", ethernet.get_source(), ethernet.get_destination());
}

fn main() {
    use pnet_datalink::Channel::Ethernet;

    let iface_name = match env::args().nth(1) {
        Some(n) => n,
        None => {
            writeln!(io::stderr(), "USAGE: packetdump <NETWORK INTERFACE>").unwrap();
            process::exit(1);
        }
    };
    let interface_names_match = |iface: &pnet_datalink::NetworkInterface| iface.name == iface_name;

    // Find the network interface with the provided name
    let interfaces = pnet_datalink::interfaces();
    let interface = interfaces
        .into_iter()
        .filter(interface_names_match)
        .next()
        .unwrap_or_else(|| panic!("No such network interface: {}", iface_name));

    // Create a channel to receive on
    let (_, mut rx) = match pnet_datalink::channel(&interface, Default::default()) {
        Ok(Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => panic!("packetdump: unhandled channel type"),
        Err(e) => panic!("packetdump: unable to create channel: {}", e),
    };

    loop {
        match rx.next() {
            Ok(packet) => {
                handle_ethernet_frame(&EthernetPacket::new(packet).unwrap());
            }
            Err(e) => panic!("packetdump: unable to receive packet: {}", e),
        }
    }
}
