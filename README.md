# NES Emulator in Rust with Python Bindings

## Overview

This project contains a NES (Nintendo Entertainment System) emulator written in Rust with Python bindings. It emulates the 6502 CPU, PPU (Picture Processing Unit), and other hardware components of the NES, allowing you to play classic NES games.

## Features

- Full 6502 CPU emulation with all documented and many undocumented opcodes
- PPU emulation with basic rendering capabilities
- Cartridge loading support for NES 1.0 ROM format
- Save state functionality
- Python bindings
- Keyboard input handling
- Breakpoint support via memory triggers

## Requirements
- Rust
- SDL2 development libraries (for the graphical interface)

## Installation

### Standalone binary
1. Clone the repository:
```bash
git clone https://github.com/StefanJo3107/nesrs
cd nesrs
```
2. Run the project:
```bash
cargo run
```

### Python bindings
1. Clone the repository from `develop-py` branch:
```bash
git clone https://github.com/StefanJo3107/nesrs --branch develop-py
cd nesrs
```
2. Install maturin:
```bash
pip install maturin
```
3. Activate venv environment:
```bash
source path_to_your_venv
```
4. Build and install wheel package using maturin:
```bash
maturin develop -r
```

## Usage

### Running from Rust
```rust
use nesrs::api::emulator::{Emulator, EmulatorTrigger};

fn main() {
    let mut emu = Emulator::new(
        "/path/to/game.nes",
        true,  // enable keyboard input
        vec![EmulatorTrigger::MemEquals { addr: 0x67, value: 0 }]
    ).unwrap();
    
    emu.reset_cpu();
    
    loop {
        let trigger = emu.step_emulation();
        if trigger {
            emu.reset_cpu();
        }
    }
}
```

### Python bindings
```python
import nesrs

emu = nesrs.Emulator("/path/to/game.nes", True)

while True:
    emu.step_emulation()
    
    frame = emu.get_current_frame()
    
    value = emu.get_value_at_address(0x1234)
    
    emu.set_key_event(KEY_UP, True)
```

## Key bindings
When keyboard input is enabled:
- Arrow keys: Directional pad
- Space: Select button
- Enter: Start button
- A: A button
- S: B button
- Escape: Quit emulator

## File Formats
- .nes files - NES 1.0 ROMs
- .cpu files - Serialized CPU states (save states)

## Debugging Features
You can set memory triggers for certain memory conditions (currently only equality):
```rust
EmulatorTrigger::MemEquals { addr: 0x67, value: 0 }
```

## Limitations
- Audio is not yet implemented
- Memory mappers are not supported
- No support for NES 2.0 ROM format
- Some undocumented CPU opcodes are not implemented
- Render order may not be correct
