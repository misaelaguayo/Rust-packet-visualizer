extern crate pnet;
extern crate pnet_datalink;

use pnet::packet::ethernet::EthernetPacket;

use rand::Rng;
use sdl2::event::Event;
use sdl2::image::{self, InitFlag, LoadTexture};
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::{Texture, WindowCanvas};
use std::env;
use std::io::{self, Write};
use std::process;
use std::thread;
use std::time::Duration;

struct State {
    packets: Vec<Packet>,
}

impl State {
    fn new(width: i32, height: i32) -> State {
        let mut rng = rand::thread_rng();
        let mut rand_pos = || Point::new(rng.gen_range(0..width), rng.gen_range(0..height));
        let packet1_source = rand_pos();
        let packet2_source = rand_pos();
        let packet3_source = rand_pos();
        let packet1 = Packet {
            source: packet1_source,
            destination: rand_pos(),
            position: packet1_source,
            sprite: Rect::new(-300, -300, 300, 300),
            current_frame: 0,
        };
        let packet2 = Packet {
            source: packet2_source,
            destination: rand_pos(),
            position: packet2_source,
            sprite: Rect::new(-300, -300, 300, 300),
            current_frame: 0,
        };
        let packet3 = Packet {
            source: packet3_source,
            destination: rand_pos(),
            position: packet3_source,
            sprite: Rect::new(-300, -300, 300, 300),
            current_frame: 0,
        };
        State {
            packets: vec![packet1, packet2, packet3],
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

fn handle_ethernet_frame(ethernet: &EthernetPacket) {
    println!("{} {}", ethernet.get_source(), ethernet.get_destination());
}

fn packet_handler() {
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
    canvas.set_scale(0.1, 0.1)?;
    let (width, height) = canvas.output_size()?;
    let mut state = State::new(width as i32, height as i32);

    let texture_creator = canvas.texture_creator();
    let texture = texture_creator.load_texture("assets/components/Target2_spritesheet.png")?;

    let mut event_pump = sdl_context.event_pump()?;
    let mut i = 0;
    let _packet_thread = thread::spawn(|| packet_handler());
    'running: loop {
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

        // render packets in transit from source to destination
        for packet in &mut state.packets {
            let delta_x = (packet.destination.x - packet.position.x) / 60;
            let delta_y = (packet.destination.y - packet.position.y) / 60;
            if delta_x != 0 && delta_y != 0 {
                packet.position.x += (packet.destination.x - packet.position.x) / 60;
                packet.position.y += (packet.destination.y - packet.position.y) / 60;
            } else {
                packet.position = packet.source;
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
