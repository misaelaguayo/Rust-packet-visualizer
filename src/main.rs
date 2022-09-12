extern crate pnet;
extern crate pnet_datalink;

use std::sync::mpsc;

use pnet::packet::arp::ArpPacket;
use pnet::packet::ethernet::{EtherTypes, EthernetPacket};
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::ipv6::Ipv6Packet;
use pnet::packet::Packet;
use rand::Rng;
use sdl2::event::Event;
use sdl2::gfx::primitives::DrawRenderer;
use sdl2::image::{self, InitFlag};
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Point;
use sdl2::render::WindowCanvas;
use std::collections::HashMap;
use std::net::IpAddr;
use std::thread;
use std::time::Duration;

fn handle_arp_packet(tx: &mpsc::Sender<(String, String)>, ethernet: &EthernetPacket) {
    let header = ArpPacket::new(ethernet.payload());
    if let Some(header) = header {
        tx.send((
            header.get_sender_proto_addr().to_string(),
            header.get_target_proto_addr().to_string(),
        ))
        .unwrap();
    }
}

fn handle_ipv6_packet(tx: &mpsc::Sender<(String, String)>, ethernet: &EthernetPacket) {
    let header = Ipv6Packet::new(ethernet.payload());
    if let Some(header) = header {
        tx.send((
            IpAddr::V6(header.get_source()).to_string(),
            IpAddr::V6(header.get_destination()).to_string(),
        ))
        .unwrap();
    }
}

fn handle_ipv4_packet(tx: &mpsc::Sender<(String, String)>, ethernet: &EthernetPacket) {
    let header = Ipv4Packet::new(ethernet.payload());
    if let Some(header) = header {
        tx.send((
            IpAddr::V4(header.get_source()).to_string(),
            IpAddr::V4(header.get_destination()).to_string(),
        ))
        .unwrap();
    }
}

fn handle_ethernet_frame(tx: &mpsc::Sender<(String, String)>, ethernet: &EthernetPacket) {
    match ethernet.get_ethertype() {
        EtherTypes::Ipv4 => handle_ipv4_packet(&tx, ethernet),
        EtherTypes::Ipv6 => handle_ipv6_packet(&tx, ethernet),
        EtherTypes::Arp => handle_arp_packet(&tx, ethernet),
        _ => println!(
            "Unknown packet: {} > {}; ethertype: {:?}",
            ethernet.get_source(),
            ethernet.get_destination(),
            ethernet.get_ethertype(),
        ),
    }
}

fn rand_pos(width: i32, height: i32) -> Point {
    let mut rng = rand::thread_rng();
    Point::new(rng.gen_range(0..width), rng.gen_range(0..height))
}

struct State {
    packets: Vec<PacketStruct>,
    map: HashMap<String, Point>,
}

impl State {
    fn new() -> State {
        State {
            packets: Vec::new(),
            map: HashMap::new(),
        }
    }
}

#[derive(Debug)]
struct PacketStruct {
    source: Point,
    destination: Point,
    position: Point,
}

fn render(canvas: &mut WindowCanvas, color: Color, state: &State) -> Result<(), String> {
    canvas.set_draw_color(color);
    canvas.clear();

    for packet in &state.packets {
        canvas
            .circle(
                packet.source.x as i16,
                packet.source.y as i16,
                16,
                Color::RGB(255, 255, 255),
            )
            .unwrap();
        canvas
            .circle(
                packet.destination.x as i16,
                packet.destination.y as i16,
                16,
                Color::RGB(255, 255, 255),
            )
            .unwrap();
        canvas
            .circle(
                packet.position.x as i16,
                packet.position.y as i16,
                16,
                Color::RGB(255, 255, 255),
            )
            .unwrap();
        canvas
            .line(
                packet.position.x as i16,
                packet.position.y as i16,
                packet.destination.x as i16,
                packet.destination.y as i16,
                Color::RGB(255, 255, 255),
            )
            .unwrap();
    }

    canvas.present();

    Ok(())
}

fn packet_handler(tx: mpsc::Sender<(String, String)>) {
    use pnet_datalink::Channel::Ethernet;
    let interface_names_match = |iface: &pnet_datalink::NetworkInterface| iface.name == "en0";

    // Find the network interface with the provided name
    let interfaces = pnet_datalink::interfaces();
    let interface = interfaces
        .into_iter()
        .filter(interface_names_match)
        .next()
        .unwrap_or_else(|| panic!("No such network interface: en0"));

    // Create a channel to receive on
    let (_, mut rx) = match pnet_datalink::channel(&interface, Default::default()) {
        Ok(Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => panic!("packetdump: unhandled channel type"),
        Err(e) => panic!("packetdump: unable to create channel: {}", e),
    };
    loop {
        match rx.next() {
            Ok(packet) => {
                let ethernet_packet = &EthernetPacket::new(packet).unwrap();
                // tx.send((source, destination)).unwrap();
                handle_ethernet_frame(&tx, &ethernet_packet);
            }
            Err(e) => panic!("packetdump: unable to receive packet: {}", e),
        };
    }
}

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    // prevent rust from dropping this value by creating unused variable
    let _image_context = image::init(InitFlag::PNG | InitFlag::JPG)?;
    let window = video_subsystem
        .window("Rust packet visualizer", 800, 600)
        .position_centered()
        .build()
        .expect("could not initialize video subsystem");

    let mut canvas = window
        .into_canvas()
        .build()
        .expect("could not make a canvas");
    let (width, height) = canvas.output_size()?;
    let mut state = State::new();

    let mut event_pump = sdl_context.event_pump()?;
    let mut i = 0;
    let (tx, rx) = mpsc::channel();
    let _packet_thread = thread::spawn(move || packet_handler(tx));
    'running: loop {
        if let Ok(packet) = rx.try_recv() {
            let (source, destination) = packet;
            println!("Source: {} destination: {}", source, destination);
            if !state.map.contains_key(&source) {
                state
                    .map
                    .insert(source.clone(), rand_pos(width as i32, height as i32));
            }
            if !state.map.contains_key(&destination) {
                state
                    .map
                    .insert(destination.clone(), rand_pos(width as i32, height as i32));
            }
            let packet1 = PacketStruct {
                source: *state.map.get(&source).unwrap(),
                destination: *state.map.get(&destination).unwrap(),
                position: *state.map.get(&source).unwrap(),
            };
            state.packets.push(packet1);
        }
        // Handle events
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }

        // Update
        i = (i + 1) % 255;

        // delete packets which have reached their destination
        state
            .packets
            .retain(|packet| packet.destination != packet.position);

        // render packets in transit from source to destination
        for packet in &mut state.packets {
            let delta_x = (packet.destination.x - packet.position.x) / 10;
            let delta_y = (packet.destination.y - packet.position.y) / 10;
            if delta_x != 0 && delta_y != 0 {
                packet.position.x += (packet.destination.x - packet.position.x) / 10;
                packet.position.y += (packet.destination.y - packet.position.y) / 10;
            }
            else{
                packet.position = packet.destination;
            }
        }

        // Render
        render(&mut canvas, Color::RGB(i, 64, 255 - i), &state)?;

        // Time management!
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
    Ok(())
}
