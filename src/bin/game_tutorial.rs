extern crate pnet;
extern crate pnet_datalink;

use pnet::packet::ethernet::EthernetPacket;
use std::sync::mpsc;

use rand::Rng;
use sdl2::event::Event;
use sdl2::image::{self, InitFlag, LoadTexture};
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::{Texture, WindowCanvas};
use std::collections::HashMap;
use std::thread;
use std::time::Duration;

fn rand_pos(width: i32, height: i32) -> Point {
    let mut rng = rand::thread_rng();
    Point::new(rng.gen_range(0..width), rng.gen_range(0..height))
}

struct State {
    packets: Vec<Packet>,
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
struct Packet {
    source: Point,
    destination: Point,
    position: Point,
    sprite: Rect,
    current_frame: i32,
}

fn render(
    canvas: &mut WindowCanvas,
    color: Color,
    texture: &Texture,
    state: &State,
) -> Result<(), String> {
    canvas.set_draw_color(color);
    canvas.clear();

    let (width, height) = canvas.output_size()?;
    for packet in &state.packets {
        let (frame_width, frame_height) = packet.sprite.size();
        let current_frame = Rect::new(
            packet.sprite.x() + frame_width as i32 * packet.current_frame,
            packet.sprite.y() + frame_height as i32 * 1,
            frame_width,
            frame_height,
        );
        let screen_position = packet.position + Point::new(width as i32 / 2, height as i32 / 2);
        // Treat the center of the screen as the (0,0) coordinate
        let screen_rect = Rect::from_center(screen_position, frame_width, frame_height);
        canvas.copy(texture, current_frame, screen_rect)?;
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
                let source = ethernet_packet.get_source().to_string();
                let destination = ethernet_packet.get_destination().to_string();
                println!("source: {} destination {}", source, destination);
                tx.send((source, destination)).unwrap();
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
        .window("game tutorial", 800, 600)
        .position_centered()
        .build()
        .expect("could not initialize video subsystem");

    let mut canvas = window
        .into_canvas()
        .build()
        .expect("could not make a canvas");
    canvas.set_scale(0.6, 0.6)?;
    let (width, height) = canvas.output_size()?;
    let mut state = State::new();

    let texture_creator = canvas.texture_creator();
    let texture = texture_creator.load_texture("assets/components/Target2_spritesheet.png")?;

    let mut event_pump = sdl_context.event_pump()?;
    let mut i = 0;
    let (tx, rx) = mpsc::channel();
    let _packet_thread = thread::spawn(move || packet_handler(tx));
    'running: loop {
        if let Ok(packet) = rx.try_recv() {
            let (source, destination) = packet;
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
            let packet1 = Packet {
                source: *state.map.get(&source).unwrap(),
                destination: *state.map.get(&destination).unwrap(),
                position: *state.map.get(&source).unwrap(),
                sprite: Rect::new(-300, -300, 300, 300),
                current_frame: 0,
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
            let delta_x = (packet.destination.x - packet.position.x) / 60;
            let delta_y = (packet.destination.y - packet.position.y) / 60;
            if delta_x != 0 && delta_y != 0 {
                packet.position.x += (packet.destination.x - packet.position.x) / 60;
                packet.position.y += (packet.destination.y - packet.position.y) / 60;
            }
            if i % 10 == 0 {
                packet.current_frame = (packet.current_frame + 1) % 4;
            }
        }

        // Render
        render(&mut canvas, Color::RGB(i, 64, 255 - i), &texture, &state)?;

        // Time management!
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
    Ok(())
}
