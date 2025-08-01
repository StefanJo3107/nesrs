use std::collections::HashMap;
use thiserror::Error;
use crate::api::emulator::Emulator;
use crate::api::ipc::EmulatorServerError::{ParseCommandError, SetKeyEventError};
use crate::hw::joypad::JoypadButton;

pub enum ServerCommands {
    LoadCartridge,
    Reset,
    Step,
    SetKeyEvent,
    GetFrame,
    GetValueAtAddress,
    Stop,
}

lazy_static::lazy_static! {
    pub static ref COMMANDS: HashMap<u8, ServerCommands> = {
        let mut map = HashMap::new();
        map.insert(0, ServerCommands::LoadCartridge);
        map.insert(1, ServerCommands::Reset);
        map.insert(2, ServerCommands::Step);
        map.insert(3, ServerCommands::SetKeyEvent);
        map.insert(4, ServerCommands::GetFrame);
        map.insert(5, ServerCommands::GetValueAtAddress);
        map.insert(6, ServerCommands::Stop);
        map
    };
}

#[derive(Error, Debug)]
pub enum EmulatorServerError {
    #[error("Parse command error: {msg:?}")]
    ParseCommandError {
        msg: String,
    },
    #[error("Handle command error: {msg:?}")]
    HandleCommandError {
        msg: String
    },
    #[error("Load cartridge error: {msg:?}")]
    LoadCartridgeError {
        msg: String
    },
    #[error("Set key event error: {msg:?}")]
    SetKeyEventError {
        msg: String
    },
}

pub struct EmulatorServer {
    emulator: Option<Emulator>,
    socket: zmq::Socket,
}

impl EmulatorServer {
    pub fn new(port: &str) -> anyhow::Result<Self> {
        let context = zmq::Context::new();
        let socket = context.socket(zmq::REP)?;
        let addr = format!("tcp://127.0.0.1:{}", port);
        socket.bind(&addr)?;

        println!("NES Emulator server listening on {}", addr);

        Ok(EmulatorServer {
            emulator: None,
            socket,
        })
    }

    fn code_to_button(key: u8) -> Option<JoypadButton> {
        match key {
            0b00010000 => Some(JoypadButton::UP),
            0b00100000 => Some(JoypadButton::DOWN),
            0b01000000 => Some(JoypadButton::LEFT),
            0b10000000 => Some(JoypadButton::RIGHT),
            0b00000001 => Some(JoypadButton::BUTTON_A),
            0b00000010 => Some(JoypadButton::BUTTON_B),
            0b00000100 => Some(JoypadButton::SELECT),
            0b00001000 => Some(JoypadButton::START),
            _ => None,
        }
    }

    fn parse_command(&self, data: &[u8]) -> anyhow::Result<(u8, Vec<u8>)> {
        if data.is_empty() {
            return Err(ParseCommandError { msg: String::from("Empty command") }.into());
        }

        let command_type = data[0];
        let payload = data[1..].to_vec();
        Ok((command_type, payload))
    }

    fn send_success(&self) -> zmq::Result<()> {
        let response = vec![0u8];
        self.socket.send(&response, 0)
    }

    fn send_error(&self, message: &str) -> zmq::Result<()> {
        let mut response = vec![1u8];
        let msg_bytes = message.as_bytes();
        response.extend_from_slice(&(msg_bytes.len() as u32).to_le_bytes());
        response.extend_from_slice(msg_bytes);
        self.socket.send(&response, 0)
    }

    fn send_frame(&self, frame_data: Vec<u8>) -> Result<(), zmq::Error> {
        let mut response = vec![2u8];
        response.extend_from_slice(&(frame_data.len() as u32).to_le_bytes());
        response.extend_from_slice(&frame_data);
        self.socket.send(&response, 0)
    }

    fn send_value_at_address(&self, value: u8) -> Result<(), zmq::Error> {
        let mut response = vec![3u8];
        response.push(value);
        self.socket.send(&response, 0)
    }

    fn handle_command(&mut self, command_type: u8, payload: Vec<u8>) -> anyhow::Result<()> {
        if let Some(command) = COMMANDS.get(&command_type) {
            match command {
                ServerCommands::LoadCartridge => {
                    if payload.len() < 4 {
                        return Err(EmulatorServerError::LoadCartridgeError { msg: String::from("Invalid payload, payload length is smaller than 4") }.into());
                    }
                    let path_len = u32::from_le_bytes([payload[0], payload[1], payload[2], payload[3]]) as usize;
                    if payload.len() < 4 + path_len {
                        return Err(EmulatorServerError::LoadCartridgeError { msg: String::from("Invalid payload, payload length is incorrect") }.into());
                    }
                    let path = String::from_utf8(payload[4..4 + path_len].to_vec())
                        .map_err(|_| EmulatorServerError::LoadCartridgeError { msg: String::from("Invalid UTF-8 in ROM path") })?;

                    self.emulator = Some(Emulator::new(&path, false)?);
                    self.send_success().map_err(|e| e)?;
                }
                ServerCommands::Reset => {
                    if let Some(ref mut emulator) = self.emulator {
                        emulator.reset_cpu();
                        self.send_success().map_err(|e| e)?;
                    } else {
                        self.send_error("No ROM loaded").map_err(|e| e)?;
                    }
                }
                ServerCommands::Step => {
                    if let Some(ref mut emulator) = self.emulator {
                        emulator.step_emulation();
                        self.send_success().map_err(|e| e)?;
                    } else {
                        self.send_error("No ROM loaded").map_err(|e| e)?;
                    }
                }
                ServerCommands::SetKeyEvent => {
                    if payload.len() < 2 {
                        return Err(SetKeyEventError { msg: String::from("Invalid payload, payload length is smaller than 2") }.into());
                    }
                    let key_code = payload[0];
                    let key_pressed = payload[1];

                    if let Some(ref mut emulator) = self.emulator {
                        if let Some(joypad_key) = EmulatorServer::code_to_button(key_code) {
                            emulator.set_key_event(joypad_key, key_pressed != 0);
                            self.send_success().map_err(|e| e)?;
                        } else {
                            self.send_error(&format!("Invalid key: {}", key_code)).map_err(|e| e)?;
                        }
                    } else {
                        self.send_error("No ROM loaded").map_err(|e| e)?;
                    }
                }
                ServerCommands::GetFrame => {
                    if let Some(ref emulator) = self.emulator {
                        let frame_data = emulator.get_current_frame();
                        self.send_frame(frame_data).map_err(|e| e)?;
                    } else {
                        self.send_error("No ROM loaded").map_err(|e| e)?;
                    }
                }
                ServerCommands::GetValueAtAddress => {
                    if payload.len() < 2 {
                        return Err(SetKeyEventError { msg: String::from("Invalid payload, payload length is smaller than 1") }.into());
                    }
                    let address = u16::from_le_bytes([payload[0], payload[1]]);
                    if let Some(ref mut emulator) = self.emulator {
                        let value = emulator.get_value_at_address(address);
                        self.send_value_at_address(value).map_err(|e| e)?;
                    } else {
                        self.send_error("No ROM loaded").map_err(|e| e)?;
                    }
                }
                ServerCommands::Stop => {
                    println!("Shutting down emulator server...");
                    std::process::exit(0);
                }
            }
            return Ok(());
        }
        Err(EmulatorServerError::HandleCommandError { msg: String::from("Unknown command") }.into())
    }

    pub fn run(&mut self) -> anyhow::Result<()> {
        loop {
            let msg = self.socket.recv_bytes(0)?;

            let (command_type, payload) = match self.parse_command(&msg) {
                Ok((cmd_type, payload)) => (cmd_type, payload),
                Err(e) => {
                    self.send_error(&format!("Failed to parse command: {}", e))?;
                    continue;
                }
            };

            if let Err(e) = self.handle_command(command_type, payload) {
                println!("Error handling command: {}", e);
            }
        }
    }
}

pub fn start_server(port: &str) -> anyhow::Result<()> {
    let mut server = EmulatorServer::new(port)?;
    server.run()
}