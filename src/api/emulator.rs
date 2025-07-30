use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use crate::hw::bus::Bus;
use crate::hw::cartridge::Cartridge;
use crate::hw::cpu::CPU;
use crate::hw::joypad;
use crate::hw::joypad::{Joypad, JoypadButton};
use crate::hw::ppu::PPU;
use crate::rendering::frame::Frame;
use crate::rendering::renderer;

pub struct Emulator {
    cpu: Arc<RefCell<CPU<'static>>>,
}

impl Emulator {
    pub fn new(cartridge_path: &str, keyboard_input: bool) -> anyhow::Result<Self> {
        // init sdl2
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();
        let window = video_subsystem
            .window("NESRS", (256.0 * 3.0) as u32, (240.0 * 3.0) as u32)
            .position_centered()
            .build()
            .unwrap();

        let canvas = Rc::new(RefCell::new(window.into_canvas().present_vsync().build().unwrap()));
        let event_pump = Rc::new(RefCell::new(sdl_context.event_pump().unwrap()));
        let canvas_clone = canvas.clone();
        canvas_clone.borrow_mut().set_scale(3.0, 3.0).unwrap();


        let bytes: Vec<u8> = std::fs::read(cartridge_path)?;
        let crt = Cartridge::new(bytes)?;

        let frame = Rc::new(RefCell::new(Frame::new()));

        // init joypad
        let mut key_map = HashMap::new();
        key_map.insert(Keycode::Down, joypad::JoypadButton::DOWN);
        key_map.insert(Keycode::UP, joypad::JoypadButton::UP);
        key_map.insert(Keycode::Right, joypad::JoypadButton::RIGHT);
        key_map.insert(Keycode::Left, joypad::JoypadButton::LEFT);
        key_map.insert(Keycode::Space, joypad::JoypadButton::SELECT);
        key_map.insert(Keycode::Return, joypad::JoypadButton::START);
        key_map.insert(Keycode::A, joypad::JoypadButton::BUTTON_A);
        key_map.insert(Keycode::S, joypad::JoypadButton::BUTTON_B);

        // the game cycle
        let bus = Bus::new(Some(crt), move |ppu: &PPU, joypad: &mut Joypad| {
            let frame_clone = frame.clone();
            let mut frame_mut = frame_clone.borrow_mut();
            let canvas_clone = canvas.clone();
            let mut canvas_mut = canvas_clone.borrow_mut();
            let event_pump_clone = event_pump.clone();
            let mut event_pump_mut = event_pump_clone.borrow_mut();
            let creator = canvas_mut.texture_creator();
            let mut texture = creator
                .create_texture_target(PixelFormatEnum::RGB24, 256, 240)
                .unwrap();

            renderer::render(ppu, &mut frame_mut);
            texture.update(None, &frame_mut.data, 256 * 3).unwrap();

            canvas_mut.copy(&texture, None, None).unwrap();

            canvas_mut.present();

            if keyboard_input {
                for event in event_pump_mut.poll_iter() {
                    match event {
                        Event::Quit { .. }
                        | Event::KeyDown {
                            keycode: Some(Keycode::Escape),
                            ..
                        } => std::process::exit(0),
                        Event::KeyDown { keycode, .. } => {
                            if let Some(key) = key_map.get(&keycode.unwrap_or(Keycode::Ampersand)) {
                                joypad.set_button_pressed_status(*key, true);
                            }
                        }
                        Event::KeyUp { keycode, .. } => {
                            if let Some(key) = key_map.get(&keycode.unwrap_or(Keycode::Ampersand)) {
                                joypad.set_button_pressed_status(*key, false);
                            }
                        }

                        _ => { /* do nothing */ }
                    }
                }
            }
        });

        let cpu = Arc::new(RefCell::new(CPU::new(bus)));
        Ok(Self {
            cpu,
        })
    }

    pub fn set_key_event(&mut self, key: JoypadButton, pressed: bool) {
        let cpu_clone = Arc::clone(&self.cpu);
        let mut cpu_borrow = cpu_clone.borrow_mut();
        if pressed {
            cpu_borrow.bus.set_key_to_press(key);
        } else {
            cpu_borrow.bus.set_key_to_release(key);
        }
    }

    pub fn reset_cpu(&mut self) {
        let cpu_clone = Arc::clone(&self.cpu);
        let mut cpu_borrow = cpu_clone.borrow_mut();
        cpu_borrow.reset();
    }

    pub fn step_emulation(&mut self) {
        let cpu_clone = Arc::clone(&self.cpu);
        let mut cpu_borrow = cpu_clone.borrow_mut();
        cpu_borrow.step(|_| {});
    }

    pub fn get_current_frame(&self) -> Vec<u8> {
        let cpu_clone = Arc::clone(&self.cpu);
        let cpu_borrow = cpu_clone.borrow_mut();
        let mut frame = Frame::new();
        renderer::render(&cpu_borrow.bus.ppu, &mut frame);
        frame.data
    }
}