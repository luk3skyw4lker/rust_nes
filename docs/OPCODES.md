# 6502 Opcode Map

Opcodes from `src/CPU/instructions.rs` (`INSTRUCTIONS` array, index = opcode).

- **Total opcodes:** 256 (`0x00`–`0xFF`)
- **Listed below:** 150 (NOP excluded)
- **NOP slots accounted for but omitted:** 106

## Oxyron audit (Step 1.16)

Compared against [Oxyron 6502 opcode matrix](https://www.oxyron.de/html/opcodes02.html).

| Check                                              | Result        |
| -------------------------------------------------- | ------------- |
| Official opcodes matching name/mode/base cycles    | **151 / 151** |
| Undocumented opcodes stubbed as `NOP` (Phase 1 OK) | **105**       |
| Remaining mismatches                               | **0**         |

All **official** opcodes match Oxyron for mnemonic, addressing mode, and base cycle count.

Undocumented/illegal opcodes (including extra `NOP`s, `KIL`, `SLO`, … and `$EB` SBC) remain `NOP` stubs — fine for Phase 1 / nestest’s official path.

### Plan spot-checks

| Item                                  | Status                |
| ------------------------------------- | --------------------- |
| All branches → `REL`, base cycles 2   | PASS                  |
| `STA` abs,X (`0x9D`) / abs,Y (`0x99`) | PASS (`ABX`/`ABY`, 5) |
| `LDX`/`LDY` immediate (`0xA2`/`0xA0`) | PASS                  |
| `STX` zp,Y (`0x96`)                   | PASS                  |

Opcodes marked `*` on Oxyron may take +1 cycle on page cross (and branches +1 if taken). Your CPU adds those at runtime via addressing modes / `branch_if`; the table stores the **base** count only.

## Opcode table (NOP omitted)

| Opcode | Instruction | Mode                 | Cycles |
| ------ | ----------- | -------------------- | ------ |
| `0x00` | BRK         | Implied (`IMP`)      | 7      |
| `0x01` | ORA         | (Indirect,X) (`IZX`) | 6      |
| `0x05` | ORA         | Zero Page (`ZP0`)    | 3      |
| `0x06` | ASL         | Zero Page (`ZP0`)    | 5      |
| `0x08` | PHP         | Implied (`IMP`)      | 3      |
| `0x09` | ORA         | Immediate (`IMM`)    | 2      |
| `0x0A` | ASL         | Implied (`IMP`)      | 2      |
| `0x0D` | ORA         | Absolute (`ABS`)     | 4      |
| `0x0E` | ASL         | Absolute (`ABS`)     | 6      |
| `0x10` | BPL         | Relative (`REL`)     | 2      |
| `0x11` | ORA         | (Indirect),Y (`IZY`) | 5      |
| `0x15` | ORA         | Zero Page,X (`ZPX`)  | 4      |
| `0x16` | ASL         | Zero Page,X (`ZPX`)  | 6      |
| `0x18` | CLC         | Implied (`IMP`)      | 2      |
| `0x19` | ORA         | Absolute,Y (`ABY`)   | 4      |
| `0x1D` | ORA         | Absolute,X (`ABX`)   | 4      |
| `0x1E` | ASL         | Absolute,X (`ABX`)   | 7      |
| `0x20` | JSR         | Absolute (`ABS`)     | 6      |
| `0x21` | AND         | (Indirect,X) (`IZX`) | 6      |
| `0x24` | BIT         | Zero Page (`ZP0`)    | 3      |
| `0x25` | AND         | Zero Page (`ZP0`)    | 3      |
| `0x26` | ROL         | Zero Page (`ZP0`)    | 5      |
| `0x28` | PLP         | Implied (`IMP`)      | 4      |
| `0x29` | AND         | Immediate (`IMM`)    | 2      |
| `0x2A` | ROL         | Implied (`IMP`)      | 2      |
| `0x2C` | BIT         | Absolute (`ABS`)     | 4      |
| `0x2D` | AND         | Absolute (`ABS`)     | 4      |
| `0x2E` | ROL         | Absolute (`ABS`)     | 6      |
| `0x30` | BMI         | Relative (`REL`)     | 2      |
| `0x31` | AND         | (Indirect),Y (`IZY`) | 5      |
| `0x35` | AND         | Zero Page,X (`ZPX`)  | 4      |
| `0x36` | ROL         | Zero Page,X (`ZPX`)  | 6      |
| `0x38` | SEC         | Implied (`IMP`)      | 2      |
| `0x39` | AND         | Absolute,Y (`ABY`)   | 4      |
| `0x3D` | AND         | Absolute,X (`ABX`)   | 4      |
| `0x3E` | ROL         | Absolute,X (`ABX`)   | 7      |
| `0x40` | RTI         | Implied (`IMP`)      | 6      |
| `0x41` | EOR         | (Indirect,X) (`IZX`) | 6      |
| `0x45` | EOR         | Zero Page (`ZP0`)    | 3      |
| `0x46` | LSR         | Zero Page (`ZP0`)    | 5      |
| `0x48` | PHA         | Implied (`IMP`)      | 3      |
| `0x49` | EOR         | Immediate (`IMM`)    | 2      |
| `0x4A` | LSR         | Implied (`IMP`)      | 2      |
| `0x4C` | JMP         | Absolute (`ABS`)     | 3      |
| `0x4D` | EOR         | Absolute (`ABS`)     | 4      |
| `0x4E` | LSR         | Absolute (`ABS`)     | 6      |
| `0x50` | BVC         | Relative (`REL`)     | 2      |
| `0x51` | EOR         | (Indirect),Y (`IZY`) | 5      |
| `0x55` | EOR         | Zero Page,X (`ZPX`)  | 4      |
| `0x56` | LSR         | Zero Page,X (`ZPX`)  | 6      |
| `0x58` | CLI         | Implied (`IMP`)      | 2      |
| `0x59` | EOR         | Absolute,Y (`ABY`)   | 4      |
| `0x5D` | EOR         | Absolute,X (`ABX`)   | 4      |
| `0x5E` | LSR         | Absolute,X (`ABX`)   | 7      |
| `0x60` | RTS         | Implied (`IMP`)      | 6      |
| `0x61` | ADC         | (Indirect,X) (`IZX`) | 6      |
| `0x65` | ADC         | Zero Page (`ZP0`)    | 3      |
| `0x66` | ROR         | Zero Page (`ZP0`)    | 5      |
| `0x68` | PLA         | Implied (`IMP`)      | 4      |
| `0x69` | ADC         | Immediate (`IMM`)    | 2      |
| `0x6A` | ROR         | Implied (`IMP`)      | 2      |
| `0x6C` | JMP         | Indirect (`IND`)     | 5      |
| `0x6D` | ADC         | Absolute (`ABS`)     | 4      |
| `0x6E` | ROR         | Absolute (`ABS`)     | 6      |
| `0x70` | BVS         | Relative (`REL`)     | 2      |
| `0x71` | ADC         | (Indirect),Y (`IZY`) | 5      |
| `0x75` | ADC         | Zero Page,X (`ZPX`)  | 4      |
| `0x76` | ROR         | Zero Page,X (`ZPX`)  | 6      |
| `0x78` | SEI         | Implied (`IMP`)      | 2      |
| `0x79` | ADC         | Absolute,Y (`ABY`)   | 4      |
| `0x7D` | ADC         | Absolute,X (`ABX`)   | 4      |
| `0x7E` | ROR         | Absolute,X (`ABX`)   | 7      |
| `0x81` | STA         | (Indirect,X) (`IZX`) | 6      |
| `0x84` | STY         | Zero Page (`ZP0`)    | 3      |
| `0x85` | STA         | Zero Page (`ZP0`)    | 3      |
| `0x86` | STX         | Zero Page (`ZP0`)    | 3      |
| `0x88` | DEY         | Implied (`IMP`)      | 2      |
| `0x8A` | TXA         | Implied (`IMP`)      | 2      |
| `0x8C` | STY         | Absolute (`ABS`)     | 4      |
| `0x8D` | STA         | Absolute (`ABS`)     | 4      |
| `0x8E` | STX         | Absolute (`ABS`)     | 4      |
| `0x90` | BCC         | Relative (`REL`)     | 2      |
| `0x91` | STA         | (Indirect),Y (`IZY`) | 6      |
| `0x94` | STY         | Zero Page,X (`ZPX`)  | 4      |
| `0x95` | STA         | Zero Page,X (`ZPX`)  | 4      |
| `0x96` | STX         | Zero Page,Y (`ZPY`)  | 4      |
| `0x98` | TYA         | Implied (`IMP`)      | 2      |
| `0x99` | STA         | Absolute,Y (`ABY`)   | 5      |
| `0x9A` | TXS         | Implied (`IMP`)      | 2      |
| `0x9D` | STA         | Absolute,X (`ABX`)   | 5      |
| `0xA0` | LDY         | Immediate (`IMM`)    | 2      |
| `0xA1` | LDA         | (Indirect,X) (`IZX`) | 6      |
| `0xA2` | LDX         | Immediate (`IMM`)    | 2      |
| `0xA4` | LDY         | Zero Page (`ZP0`)    | 3      |
| `0xA5` | LDA         | Zero Page (`ZP0`)    | 3      |
| `0xA6` | LDX         | Zero Page (`ZP0`)    | 3      |
| `0xA8` | TAY         | Implied (`IMP`)      | 2      |
| `0xA9` | LDA         | Immediate (`IMM`)    | 2      |
| `0xAA` | TAX         | Implied (`IMP`)      | 2      |
| `0xAC` | LDY         | Absolute (`ABS`)     | 4      |
| `0xAD` | LDA         | Absolute (`ABS`)     | 4      |
| `0xAE` | LDX         | Absolute (`ABS`)     | 4      |
| `0xB0` | BCS         | Relative (`REL`)     | 2      |
| `0xB1` | LDA         | (Indirect),Y (`IZY`) | 5      |
| `0xB4` | LDY         | Zero Page,X (`ZPX`)  | 4      |
| `0xB5` | LDA         | Zero Page,X (`ZPX`)  | 4      |
| `0xB6` | LDX         | Zero Page,Y (`ZPY`)  | 4      |
| `0xB8` | CLV         | Implied (`IMP`)      | 2      |
| `0xB9` | LDA         | Absolute,Y (`ABY`)   | 4      |
| `0xBA` | TSX         | Implied (`IMP`)      | 2      |
| `0xBC` | LDY         | Absolute,X (`ABX`)   | 4      |
| `0xBD` | LDA         | Absolute,X (`ABX`)   | 4      |
| `0xBE` | LDX         | Absolute,Y (`ABY`)   | 4      |
| `0xC0` | CPY         | Immediate (`IMM`)    | 2      |
| `0xC1` | CMP         | (Indirect,X) (`IZX`) | 6      |
| `0xC4` | CPY         | Zero Page (`ZP0`)    | 3      |
| `0xC5` | CMP         | Zero Page (`ZP0`)    | 3      |
| `0xC6` | DEC         | Zero Page (`ZP0`)    | 5      |
| `0xC8` | INY         | Implied (`IMP`)      | 2      |
| `0xC9` | CMP         | Immediate (`IMM`)    | 2      |
| `0xCA` | DEX         | Implied (`IMP`)      | 2      |
| `0xCC` | CPY         | Absolute (`ABS`)     | 4      |
| `0xCD` | CMP         | Absolute (`ABS`)     | 4      |
| `0xCE` | DEC         | Absolute (`ABS`)     | 6      |
| `0xD0` | BNE         | Relative (`REL`)     | 2      |
| `0xD1` | CMP         | (Indirect),Y (`IZY`) | 5      |
| `0xD5` | CMP         | Zero Page,X (`ZPX`)  | 4      |
| `0xD6` | DEC         | Zero Page,X (`ZPX`)  | 6      |
| `0xD8` | CLD         | Implied (`IMP`)      | 2      |
| `0xD9` | CMP         | Absolute,Y (`ABY`)   | 4      |
| `0xDD` | CMP         | Absolute,X (`ABX`)   | 4      |
| `0xDE` | DEC         | Absolute,X (`ABX`)   | 7      |
| `0xE0` | CPX         | Immediate (`IMM`)    | 2      |
| `0xE1` | SBC         | (Indirect,X) (`IZX`) | 6      |
| `0xE4` | CPX         | Zero Page (`ZP0`)    | 3      |
| `0xE5` | SBC         | Zero Page (`ZP0`)    | 3      |
| `0xE6` | INC         | Zero Page (`ZP0`)    | 5      |
| `0xE8` | INX         | Implied (`IMP`)      | 2      |
| `0xE9` | SBC         | Immediate (`IMM`)    | 2      |
| `0xEC` | CPX         | Absolute (`ABS`)     | 4      |
| `0xED` | SBC         | Absolute (`ABS`)     | 4      |
| `0xEE` | INC         | Absolute (`ABS`)     | 6      |
| `0xF0` | BEQ         | Relative (`REL`)     | 2      |
| `0xF1` | SBC         | (Indirect),Y (`IZY`) | 5      |
| `0xF5` | SBC         | Zero Page,X (`ZPX`)  | 4      |
| `0xF6` | INC         | Zero Page,X (`ZPX`)  | 6      |
| `0xF8` | SED         | Implied (`IMP`)      | 2      |
| `0xF9` | SBC         | Absolute,Y (`ABY`)   | 4      |
| `0xFD` | SBC         | Absolute,X (`ABX`)   | 4      |
| `0xFE` | INC         | Absolute,X (`ABX`)   | 7      |

## Counts by instruction

| Instruction | Opcode count                     |
| ----------- | -------------------------------- |
| ADC         | 8                                |
| AND         | 8                                |
| ASL         | 5                                |
| BCC         | 1                                |
| BCS         | 1                                |
| BEQ         | 1                                |
| BIT         | 2                                |
| BMI         | 1                                |
| BNE         | 1                                |
| BPL         | 1                                |
| BRK         | 1                                |
| BVC         | 1                                |
| BVS         | 1                                |
| CLC         | 1                                |
| CLD         | 1                                |
| CLI         | 1                                |
| CLV         | 1                                |
| CMP         | 8                                |
| CPX         | 3                                |
| CPY         | 3                                |
| DEC         | 4                                |
| DEX         | 1                                |
| DEY         | 1                                |
| EOR         | 8                                |
| INC         | 4                                |
| INX         | 1                                |
| INY         | 1                                |
| JMP         | 2                                |
| JSR         | 1                                |
| LDA         | 8                                |
| LDX         | 5                                |
| LDY         | 5                                |
| LSR         | 5                                |
| NOP         | 106 _(omitted from table above)_ |
| ORA         | 8                                |
| PHA         | 1                                |
| PHP         | 1                                |
| PLA         | 1                                |
| PLP         | 1                                |
| ROL         | 5                                |
| ROR         | 5                                |
| RTI         | 1                                |
| RTS         | 1                                |
| SBC         | 8                                |
| SEC         | 1                                |
| SED         | 1                                |
| SEI         | 1                                |
| STA         | 7                                |
| STX         | 3                                |
| STY         | 3                                |
| TAX         | 1                                |
| TAY         | 1                                |
| TSX         | 1                                |
| TXA         | 1                                |
| TXS         | 1                                |
| TYA         | 1                                |

**Sum of opcode counts:** 256 / 256
