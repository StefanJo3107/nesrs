use std::collections::HashMap;
use std::fs::OpenOptions;
use std::thread;
use std::time::Duration;
use memmap2::{MmapMut, MmapOptions};
use thiserror::Error;
use crate::api::emulator::Emulator;
use crate::api::ipc::EmulatorServerError::{ParseCommandError, SetKeyEventError};
use crate::hw::joypad::JoypadButton;

pub enum ServerCommands {
    Noop,
    LoadCartridge,
    Reset,
    Step,
    SetKeyEvent,
    GetFrame,
    GetValueAtAddress,
    Stop,
}

impl From<u8> for ServerCommands {
    fn from(value: u8) -> Self {
        match value {
            0 => ServerCommands::LoadCartridge,
            1 => ServerCommands::Reset,
            2 => ServerCommands::Step,
            3 => ServerCommands::SetKeyEvent,
            4 => ServerCommands::GetFrame,
            5 => ServerCommands::GetValueAtAddress,
            6 => ServerCommands::Stop,
            _ => ServerCommands::Noop,
        }
    }
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
    #[error("Memory mapping error: {msg:?}")]
    MemoryMappingError {
        msg: String
    },
}

const COMMAND_FILE_SIZE: usize = 1024;
const STATE_FILE_SIZE: usize = 8192;
const FRAME_FILE_SIZE: usize = 256 * 240 * 3;

pub struct EmulatorServer {
    emulator: Option<Emulator>,

    command_mmap: MmapMut,
    state_mmap: MmapMut,
    frame_mmap: MmapMut,
}
impl EmulatorServer {
    pub fn new(
        command_file: &str,
        state_file: &str,
        frame_file: &str,
    ) -> Result<Self, EmulatorServerError> {

        // Create and setup command file
        let command_mmap = Self::create_memory_map(command_file, COMMAND_FILE_SIZE)?;

        // Create and setup state file
        let state_mmap = Self::create_memory_map(state_file, STATE_FILE_SIZE)?;

        // Create and setup frame file
        let frame_mmap = Self::create_memory_map(frame_file, FRAME_FILE_SIZE)?;

        println!("Shared memory files created:");
        println!("  Commands: {}", command_file);
        println!("  State: {}", state_file);
        println!("  Frame: {}", frame_file);

        Ok(EmulatorServer {
            emulator: None,
            command_mmap,
            state_mmap,
            frame_mmap,
        })
    }

    fn create_memory_map(filename: &str, size: usize) -> Result<MmapMut, EmulatorServerError> {
        let mut file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .truncate(true)
            .open(filename)
            .map_err(|e| EmulatorServerError::MemoryMappingError {
                msg: format!("Failed to create file {}: {}", filename, e)
            })?;

        file.set_len(size as u64)
            .map_err(|e| EmulatorServerError::MemoryMappingError {
                msg: format!("Failed to set file size: {}", e)
            })?;

        let mmap = unsafe {
            MmapOptions::new()
                .map_mut(&file)
                .map_err(|e| EmulatorServerError::MemoryMappingError {
                    msg: format!("Failed to create memory map: {}", e)
                })?
        };

        Ok(mmap)
    }

    fn read_command(&mut self) -> Option<(ServerCommands, Vec<u8>)> {
        let signal_byte = self.command_mmap[COMMAND_FILE_SIZE - 1];
        if signal_byte == 0 {
            return None;
        }

        let command_type = ServerCommands::from(self.command_mmap[0]);
        let payload_len = u32::from_le_bytes([
            self.command_mmap[1],
            self.command_mmap[2],
            self.command_mmap[3],
            self.command_mmap[4],
        ]) as usize;


        let payload = if payload_len > 0 && payload_len < COMMAND_FILE_SIZE - 10 {
            self.command_mmap[5..5 + payload_len].to_vec()
        } else {
            Vec::new()
        };

        // set signal byte to 0 to mark command as read
        self.command_mmap[COMMAND_FILE_SIZE - 1] = 0;

        Some((command_type, payload))
    }

    fn update_frame(&mut self) {
        if let Some(ref emulator) = self.emulator {
            let frame_data = emulator.get_current_frame();
            if frame_data.len() == FRAME_FILE_SIZE {
                self.frame_mmap[..FRAME_FILE_SIZE].copy_from_slice(&frame_data);
            }
        }
    }

    fn write_memory_value_response(&mut self, address: u16, value: u8) {
        self.state_mmap[0..2]
            .copy_from_slice(&address.to_le_bytes());

        self.state_mmap[2] = value;
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

    fn handle_command(&mut self, command: ServerCommands, payload: Vec<u8>) -> Result<(), EmulatorServerError> {
        match command {
            ServerCommands::LoadCartridge => {
                if payload.len() < 4 {
                    return Err(EmulatorServerError::LoadCartridgeError {
                        msg: "Invalid payload length".to_string()
                    });
                }

                let path_len = u32::from_le_bytes([payload[0], payload[1], payload[2], payload[3]]) as usize;
                if payload.len() < 4 + path_len {
                    return Err(EmulatorServerError::LoadCartridgeError {
                        msg: "Invalid payload format".to_string()
                    });
                }

                let path = String::from_utf8(payload[4..4 + path_len].to_vec())
                    .map_err(|_| EmulatorServerError::LoadCartridgeError {
                        msg: "Invalid UTF-8 in ROM path".to_string()
                    })?;

                self.emulator = Some(Emulator::new(&path, false)
                    .map_err(|e| EmulatorServerError::LoadCartridgeError {
                        msg: format!("Failed to load ROM: {}", e)
                    })?);

                println!("Loaded ROM: {}", path);
            }

            ServerCommands::Reset => {
                if let Some(ref mut emulator) = self.emulator {
                    emulator.reset_cpu();
                }
            }

            ServerCommands::Step => {
                if let Some(ref mut emulator) = self.emulator {
                    emulator.step_emulation();
                }
            }

            ServerCommands::SetKeyEvent => {
                if payload.len() >= 5 {
                    let key_code = payload[0];
                    let key_pressed = payload[4] != 0;

                    if let Some(ref mut emulator) = self.emulator {
                        if let Some(joypad_key) = Self::code_to_button(key_code) {
                            emulator.set_key_event(joypad_key, key_pressed);
                        }
                    }
                }
            }

            ServerCommands::GetFrame => {
                self.update_frame();
            }

            ServerCommands::GetValueAtAddress => {
                if let Some(ref emulator) = self.emulator {
                    if payload.len() >= 2 {
                        let address = u16::from_le_bytes([payload[0], payload[1]]);
                        let value = emulator.get_value_at_address(address);
                        self.write_memory_value_response(address, value);
                    }
                }
            }

            ServerCommands::Stop => {
                std::process::exit(0);
            }

            ServerCommands::Noop => {}
        }

        Ok(())
    }
    pub fn run(&mut self) -> Result<(), EmulatorServerError> {
        println!("Shared memory emulator server running...");
        println!("Waiting for commands...");

        loop {
            if let Some((command, payload)) = self.read_command() {
                if let Err(e) = self.handle_command(command, payload) {
                    eprintln!("Error handling command: {}", e);
                }
            }
        }
    }
}

pub fn start_server(command_file: Option<&str>,
                    state_file: Option<&str>,
                    frame_file: Option<&str>,
) -> Result<(), EmulatorServerError> {
    let command_file = command_file.unwrap_or("/tmp/nes_commands");
    let state_file = state_file.unwrap_or("/tmp/nes_state");
    let frame_file = frame_file.unwrap_or("/tmp/nes_frame");

    let mut server = EmulatorServer::new(command_file, state_file, frame_file)?;
    server.run()
}

pub fn start_server_default() -> Result<(), EmulatorServerError> {
    start_server(None, None, None)
}