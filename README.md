# NES Emulator in Rust with Python Bindings

## Overview

This project contains a NES (Nintendo Entertainment System) emulator written in Rust with Python bindings. It emulates
the 6502 CPU, PPU (Picture Processing Unit), and other hardware components of the NES, allowing you to play classic NES
games.

## Features

- Full 6502 CPU emulation with all documented and many undocumented opcodes
- PPU emulation with basic rendering capabilities
- Cartridge loading support for NES 1.0 ROM format
- Save state functionality
- Keyboard input handling

## Usage

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

## Limitations

- Audio is not yet implemented
- Memory mappers are not supported
- No support for NES 2.0 ROM format
- Some undocumented CPU opcodes are not implemented
- Render order may not be correct