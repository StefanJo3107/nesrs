use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::rc::Rc;
use std::sync::Arc;
use postcard::to_stdvec;
use pyo3::{pyclass, pymethods};
use pyo3::types::PyAnyMethods;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use crate::hw::bus::Bus;
use crate::hw::cartridge::Cartridge;
use crate::hw::cpu::CPU;
use crate::hw::joypad::{Joypad, JoypadButton};
use crate::hw::memory::Memory;
use crate::hw::ppu::PPU;
use crate::rendering::frame::Frame;
use crate::rendering::renderer;

#[derive(PartialEq)]
pub enum LoadFormat {
    NES,
    CPU,
    Unknown,
}

#[pyclass(unsendable)]
pub enum EmulatorTrigger {
    MemEquals { addr: u16, value: u8 },
}

#[pyclass(unsendable)]
pub struct Emulator {
    cpu: Arc<RefCell<CPU<'static>>>,
    triggers: Vec<EmulatorTrigger>,
    load_format: LoadFormat,
    cartridge_path: String,
}

#[pymethods]
impl Emulator {
    #[new]
    pub fn new_trigerless(cartridge_path: &str, keyboard_input: bool) -> Self {
        Emulator::new(cartridge_path, keyboard_input, vec![]).unwrap()
    }

    pub fn set_key_event(&mut self, key: u8, pressed: bool) {
        let cpu_clone = Arc::clone(&self.cpu);
        let mut cpu_borrow = cpu_clone.borrow_mut();
        if pressed {
            let button = JoypadButton::from_bits(key).unwrap_or(JoypadButton::UP);
            cpu_borrow.bus.set_key_to_press(button);
        } else {
            let button = JoypadButton::from_bits(key).unwrap_or(JoypadButton::UP);
            cpu_borrow.bus.set_key_to_release(button);
        }
    }

    pub fn reset_cpu(&mut self) {
        let cpu_clone = Arc::clone(&self.cpu);
        let mut cpu_borrow = cpu_clone.borrow_mut();
        if self.load_format == LoadFormat::NES {
            cpu_borrow.reset();
        } else {
            let callback = cpu_borrow.bus.gameloop_callback.take();
            let bytes: Vec<u8> = std::fs::read(self.cartridge_path.as_str()).unwrap();
            let mut cpu = Emulator::deserialize_cpu(bytes);
            cpu.bus.gameloop_callback = callback;
            self.cpu = Arc::new(RefCell::new(cpu));
            self.set_key_event(JoypadButton::START.bits(), false);
        }
    }

    // returns true if breakpoint is hit
    pub fn step_emulation(&mut self) -> bool {
        let cpu_clone = Arc::clone(&self.cpu);
        let mut cpu_borrow = cpu_clone.borrow_mut();
        cpu_borrow.step(|_| {});
        self.check_triggers(&mut *cpu_borrow)
    }

    pub fn get_current_frame(&self) -> Vec<u8> {
        let cpu_clone = Arc::clone(&self.cpu);
        let cpu_borrow = cpu_clone.borrow_mut();
        let data = &cpu_borrow.bus.ppu.current_frame.data;
        data.clone()
    }

    pub fn get_value_at_address(&self, address: u16) -> u8 {
        let cpu_clone = Arc::clone(&self.cpu);
        let mut cpu_borrow = cpu_clone.borrow_mut();
        let value = cpu_borrow.mem_read(address);
        value
    }
}

impl Emulator {
    pub fn new(cartridge_path: &str, keyboard_input: bool, triggers: Vec<EmulatorTrigger>) -> anyhow::Result<Self> {
        // init sdl2
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();
        let window = video_subsystem
            .window("NESRS", (256.0 * 3.0) as u32, (240.0 * 3.0) as u32)
            .position_centered()
            .build()?;

        let canvas = Rc::new(RefCell::new(window.into_canvas().build()?));
        let event_pump = Rc::new(RefCell::new(sdl_context.event_pump().unwrap()));
        let canvas_clone = canvas.clone();
        canvas_clone.borrow_mut().set_scale(3.0, 3.0).unwrap();

        // init joypad
        let mut key_map = HashMap::new();
        key_map.insert(Keycode::Down, JoypadButton::DOWN);
        key_map.insert(Keycode::Up, JoypadButton::UP);
        key_map.insert(Keycode::Right, JoypadButton::RIGHT);
        key_map.insert(Keycode::Left, JoypadButton::LEFT);
        key_map.insert(Keycode::Space, JoypadButton::SELECT);
        key_map.insert(Keycode::Return, JoypadButton::START);
        key_map.insert(Keycode::A, JoypadButton::BUTTON_A);
        key_map.insert(Keycode::S, JoypadButton::BUTTON_B);


        let path = Path::new(cartridge_path);
        let mut load_format = LoadFormat::Unknown;

        if let Some(extension) = path.extension() {
            if extension == "cpu" {
                load_format = LoadFormat::CPU;
            } else if extension == "nes" {
                load_format = LoadFormat::NES;
            } else {
                load_format = LoadFormat::Unknown;
            }
        }

        if load_format == LoadFormat::Unknown {
            panic!("Unsupported cartridge format!");
        }


        let bytes: Vec<u8> = std::fs::read(cartridge_path)?;

        if load_format == LoadFormat::NES {
            let crt = Cartridge::new(bytes)?;

            // the game cycle
            let bus = Bus::new(Some(crt), move |ppu: &mut PPU, joypad: &mut Joypad| {
                let mut frame = Frame::new();
                let canvas_clone = canvas.clone();
                let mut canvas_mut = canvas_clone.borrow_mut();
                let event_pump_clone = event_pump.clone();
                let mut event_pump_mut = event_pump_clone.borrow_mut();
                let creator = canvas_mut.texture_creator();
                let mut texture = creator
                    .create_texture_target(PixelFormatEnum::RGB24, 256, 240)
                    .unwrap();

                renderer::render(ppu, &mut frame);
                texture.update(None, &frame.data, 256 * 3).unwrap();
                ppu.current_frame = frame;
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
                                    joypad.set_button_pressed_status(key, true);
                                }
                            }
                            Event::KeyUp { keycode, .. } => {
                                if let Some(key) = key_map.get(&keycode.unwrap_or(Keycode::Ampersand)) {
                                    joypad.set_button_pressed_status(key, false);
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
                triggers,
                load_format: LoadFormat::NES,
                cartridge_path: String::from(cartridge_path),
            })
        } else {
            let mut cpu = Emulator::deserialize_cpu(bytes);
            cpu.bus.gameloop_callback = Some(Box::new(move |ppu: &mut PPU, joypad: &mut Joypad| {
                let mut frame = Frame::new();
                let canvas_clone = canvas.clone();
                let mut canvas_mut = canvas_clone.borrow_mut();
                let event_pump_clone = event_pump.clone();
                let mut event_pump_mut = event_pump_clone.borrow_mut();
                let creator = canvas_mut.texture_creator();
                let mut texture = creator
                    .create_texture_target(PixelFormatEnum::RGB24, 256, 240)
                    .unwrap();

                renderer::render(ppu, &mut frame);
                texture.update(None, &frame.data, 256 * 3).unwrap();
                ppu.current_frame = frame;
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
                                    joypad.set_button_pressed_status(key, true);
                                }
                            }
                            Event::KeyUp { keycode, .. } => {
                                if let Some(key) = key_map.get(&keycode.unwrap_or(Keycode::Ampersand)) {
                                    joypad.set_button_pressed_status(key, false);
                                }
                            }

                            _ => { /* do nothing */ }
                        }
                    }
                }
            }));
            Ok(Self { cpu: Arc::new(RefCell::new(cpu)), triggers, load_format: LoadFormat::CPU, cartridge_path: String::from(cartridge_path) })
        }
    }

    pub fn take_cpu_snapshot(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let cpu_clone = Arc::clone(&self.cpu);
        let cpu_borrow = cpu_clone.borrow_mut();
        println!("Cpu state:: PC: {}, A: {}, X: {}, Y: {}", cpu_borrow.program_counter, cpu_borrow.register_a, cpu_borrow.register_x, cpu_borrow.register_y);
        let bytes = to_stdvec(&*cpu_borrow)?;
        let mut file = File::create(path)?;
        file.write_all(&bytes)?;
        Ok(())
    }

    fn deserialize_cpu(data: Vec<u8>) -> CPU<'static> {
        let mut new_cpu: CPU = postcard::from_bytes(data.as_slice())
            .expect("Failed to deserialize cpu");
        new_cpu
    }

    fn check_triggers(&self, cpu: &mut CPU) -> bool {
        self.triggers.iter().any(|trigger| match trigger {
            EmulatorTrigger::MemEquals { addr, value } => cpu.mem_read(*addr) == *value
        })
    }
}