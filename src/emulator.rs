use std::{
    thread::sleep,
    time::{Duration, Instant},
};

use pixels::PixelsBuilder;
use winit::{
    event::{ElementState, Event, VirtualKeyCode, WindowEvent},
    event_loop::EventLoop,
    window::WindowBuilder,
};

use crate::{
    interpreter::Chip8Interpreter,
    memory::CosmacRAM,
    peripherals::{Beeper, Tone},
    Result,
};

type Chip8 = Chip8Interpreter<fastrand::Rng>;

const INSTRUCTIONS_FREQ_HZ: u64 = 700; // number of CHIP-8 instructions performed per second
const INSTRUCTION_DURATION: Duration = Duration::from_micros(1_000_000 / INSTRUCTIONS_FREQ_HZ);
const DISPLAY_SCALE_FACTOR: u32 = 16;
const TONE_FREQ_HZ: u32 = 440;

pub fn run(chip8_program: &[u8]) -> Result<()> {
    // Initialise CHIP-8 RAM/"CPU"
    let mut ram = CosmacRAM::new();
    ram.load_chip8_program(chip8_program)?;
    let mut chip8 = Chip8::new(fastrand::Rng::new());
    chip8.reset(&mut ram);

    // Set up devices (screen, keyboard and audio)
    env_logger::init();
    let event_loop = EventLoop::new();

    let window = {
        let size = winit::dpi::LogicalSize::new(64, 32);
        let scaled_size = winit::dpi::LogicalSize::new(
            size.width * DISPLAY_SCALE_FACTOR,
            size.height * DISPLAY_SCALE_FACTOR,
        );
        WindowBuilder::new()
            .with_title("CHIP-8 Emulator")
            .with_inner_size(scaled_size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture =
            pixels::SurfaceTexture::new(window_size.width, window_size.height, &window);
        let mut pixels = PixelsBuilder::new(64, 32, surface_texture)
            .enable_vsync(true)
            .build()
            .unwrap();

        // initialise frame buffer
        pixels
            .frame_mut()
            .copy_from_slice(&rgba_pixels_from_cosmac_display_buffer(&ram));

        pixels
    };

    let beeper = Beeper::new(TONE_FREQ_HZ);

    // run the main event loop
    event_loop.run(move |event, _, control_flow| {
        control_flow.set_poll();

        match event {
            Event::MainEventsCleared => {
                let is_draw_instruction = Chip8::is_on_draw_instruction(&ram);

                let start = Instant::now();
                chip8.step(&mut ram);
                sleep(start + INSTRUCTION_DURATION - Instant::now());

                // update tone
                let tone_should_be_sounding = Chip8::is_tone_sounding(&ram);
                if tone_should_be_sounding && !beeper.is_tone_on() {
                    beeper.start_tone();
                } else if !tone_should_be_sounding && beeper.is_tone_on() {
                    beeper.stop_tone();
                }

                // update display (waits for VBLANK)
                if is_draw_instruction {
                    window.request_redraw();
                }
            }
            Event::RedrawRequested(_) => {
                pixels
                    .frame_mut()
                    .copy_from_slice(&rgba_pixels_from_cosmac_display_buffer(&ram));

                // vsync is enabled in render call, but need to simulate it for case
                // when window is minimised, as graphics library doesn't wait for VBLANKs
                // when the window is not on the screen.
                let target_render_time = Instant::now() + Duration::from_micros(16_667);
                pixels.render().unwrap();
                let now = Instant::now();
                if now < target_render_time {
                    sleep(target_render_time - now);
                }
            }
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    control_flow.set_exit();
                }
                WindowEvent::KeyboardInput { input, .. } => {
                    if input.state == ElementState::Released {
                        Chip8::set_current_key_press(&mut ram, None);
                    } else if let Some(key_code) = input.virtual_keycode {
                        Chip8::set_current_key_press(
                            &mut ram,
                            match key_code {
                                VirtualKeyCode::Key1 => Some(0x1),
                                VirtualKeyCode::Key2 => Some(0x2),
                                VirtualKeyCode::Key3 => Some(0x3),
                                VirtualKeyCode::Q => Some(0x4),
                                VirtualKeyCode::W => Some(0x5),
                                VirtualKeyCode::E => Some(0x6),
                                VirtualKeyCode::A => Some(0x7),
                                VirtualKeyCode::S => Some(0x8),
                                VirtualKeyCode::D => Some(0x9),
                                VirtualKeyCode::X => Some(0x0),
                                VirtualKeyCode::Z => Some(0xA),
                                VirtualKeyCode::C => Some(0xB),
                                VirtualKeyCode::Key4 => Some(0xC),
                                VirtualKeyCode::R => Some(0xD),
                                VirtualKeyCode::F => Some(0xE),
                                VirtualKeyCode::V => Some(0xF),
                                _ => None,
                            },
                        );
                    }
                }
                _ => (),
            },
            _ => (),
        }
    });
}

fn rgba_pixels_from_cosmac_display_buffer(ram: &CosmacRAM) -> Vec<u8> {
    ram.display_buffer()
        .iter()
        .flat_map(|pixel_byte| {
            let mut color_pixels = [[0xFFu8, 0xFF, 0xFF, 0xFF]; 8]; // default to 8 white pixels
            for (i, rgb_pixel) in color_pixels.iter_mut().enumerate() {
                if pixel_byte & (1 << (7 - i)) != 0 {
                    // make pixel black
                    rgb_pixel[0] = 0x00; // R
                    rgb_pixel[1] = 0x00; // G
                    rgb_pixel[2] = 0x00; // B
                }
            }
            color_pixels
        })
        .flatten()
        .collect()
}
