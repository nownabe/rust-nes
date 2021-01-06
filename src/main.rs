use std::env;
use std::fs::File;
use std::io::prelude::*;

#[macro_use]
extern crate log;

// for rendering window
extern crate image;

// https://docs.rs/piston_window/0.116.0/piston_window/index.html
extern crate piston_window;
use piston_window::{PistonWindow, WindowSettings, Texture, TextureContext, TextureSettings};
use piston_window::OpenGL;
use piston_window::G2dTexture;
use piston_window::{RenderEvent, Transformed}; // render_args(), scale()
use piston_window::{clear, image as piston_image};

mod cassette;
mod cpu;
mod instruction;
mod memory;
mod ppu;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env::set_var("RUST_LOG", "debug");
    env_logger::init();

    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        error!("ROM filename is required");
        return Err("ROM filename is required".into());
    }
    let rom_filename = &args[1];

    debug!("ROM file = {}", rom_filename);

    let mut f = File::open(rom_filename)?;
    let mut buf = Vec::new();
    let _ = f.read_to_end(&mut buf)?;

    let cassette = cassette::Cassette::new(buf);

    if !cassette.is_ines() {
        error!("ROM must be iNES format");
        return Err("ROM must be iNES format".into());
    }

    let program_data = rom.program_data();
    debug!("program data length = {} bytes", program_data.len());

    let mut mem = memory::Memory::new();

    let mut cpu = cpu::Cpu::new();
    cpu.load_program_from_cassette(&mut mem, &cassette);

    let mut ppu = ppu::Ppu::new();
    ppu.init(&mut mem, &cassette);

    display_sprites(&ppu);

    let scale = 4;
    let width = ppu::VISIBLE_SCREEN_WIDTH as u32 * scale;
    let height = ppu::VISIBLE_SCREEN_HEIGHT as u32 * scale;

    let opengl = OpenGL::V3_2;
    let mut window: PistonWindow = WindowSettings::new("Rust NES", (width, height))
        .exit_on_esc(true)
        .graphics_api(opengl)
        .build()
        .unwrap();

    let mut canvas = image::ImageBuffer::new(width, height);
    let mut texture_context = TextureContext {
        factory: window.factory.clone(),
        encoder: window.factory.create_command_buffer().into(),
    };

    let mut texture: G2dTexture = Texture::from_image(
            &mut texture_context,
            &canvas,
            &TextureSettings::new()
        ).unwrap();

    while let Some(e) = window.next() {
        if let Some(_) = e.render_args() {
            let cycle = cpu.tick(&mut mem);
            ppu.step(&mut mem, cycle);

            let screen = ppu.screen;

            for x in 0..ppu::VISIBLE_SCREEN_WIDTH {
                for y in 0..ppu::VISIBLE_SCREEN_HEIGHT {
                    let color = ppu.screen[y][x];
                    canvas.put_pixel(
                        x as u32,
                        y as u32,
                        image::Rgba([color[0], color[1], color[2], 255])
                    );
                }
            }

            texture.update(&mut texture_context, &canvas).unwrap();
            window.draw_2d(&e, |c, g, device| {
                texture_context.encoder.flush(device);
                clear([1.0; 4], g);
                piston_image(&texture, c.transform.scale(scale as f64, scale as f64), g);
            });
        }
    }

    loop {
        let cycle = cpu.tick(&mut mem);
        ppu.step(&mut mem, cycle);
    }

    Ok(())
}

// https://github.com/PistonDevelopers/piston-examples/blob/master/src/paint.rs
fn display_sprites(ppu: &ppu::Ppu) {
    const COLORS: [image::Rgba::<u8>; 4] = [
        image::Rgba([0, 0, 0, 255]),
        image::Rgba([63, 63, 63, 255]),
        image::Rgba([127, 127, 127, 255]),
        image::Rgba([255, 255, 255, 255]),
    ];

    let scale = 4;
    let width = 256 * scale;
    let height = 64 * scale;

    let opengl = OpenGL::V3_2;
    let mut window: PistonWindow = WindowSettings::new("Rust NES", (width, height))
        .exit_on_esc(true)
        .graphics_api(opengl)
        .build()
        .unwrap();

    let mut canvas = image::ImageBuffer::new(width, height);
    let mut texture_context = TextureContext {
        factory: window.factory.clone(),
        encoder: window.factory.create_command_buffer().into(),
    };

    let mut texture: G2dTexture = Texture::from_image(
            &mut texture_context,
            &canvas,
            &TextureSettings::new()
        ).unwrap();

    while let Some(e) = window.next() {
        if let Some(_) = e.render_args() {
            for i in 0..=0xFF {
                let sprite = ppu.get_sprite(i);

                let offset_x = (i as u32) % 32 * 8;
                let offset_y = (i as u32) / 32 * 8;

                for x in 0..8 {
                    for y in 0..8 {

                        let color = COLORS[sprite.get(x, y) as usize];
                        canvas.put_pixel(
                            offset_x+x as u32,
                            offset_y+y as u32,
                            color);
                    }
                }
            }

            texture.update(&mut texture_context, &canvas).unwrap();
            window.draw_2d(&e, |c, g, device| {
                texture_context.encoder.flush(device);
                clear([1.0; 4], g);
                piston_image(&texture, c.transform.scale(scale as f64, scale as f64), g);
            });
        }
    }
}
