# rust_nes

A Nintendo Entertainment System (NES) emulator written from scratch in Rust.

This is a hands-on learning project: every piece of emulation logic — CPU, bus, cartridge, PPU, APU, mappers — is meant to be hand-written, with the Rust standard library preferred over third-party crates. The goal is to understand how the console works by building it, not to ship the most accurate emulator overnight.

Inspired by the architecture and teaching style of [javidx9 / One Lone Coder](https://www.youtube.com/@javidx9) and [Akitando](https://www.youtube.com/@Akitando).

## Status

Work in progress. The focus right now is a correct **MOS 6502** CPU core:

| Area | Status |
| ---- | ------ |
| Opcode decode table (256 entries) | Scaffolded — official opcodes mapped |
| Addressing modes | Implemented |
| Opcode execution | In progress (toward nestest) |
| Interrupts (`reset` / `irq` / `nmi`) | Skeleton |
| Bus | Flat 64KB RAM (NES memory map still ahead) |
| Cartridge / mappers | Not started |
| PPU / APU / controllers / UI | Not started |

The long-term path is: make the CPU trustworthy → real NES bus + cartridge → PPU → display → APU → more mappers. See [`docs/plans/EMULATOR_PLAN.md`](docs/plans/EMULATOR_PLAN.md) for the full roadmap.

## Why Rust

Rust is a good fit for an emulator: explicit ownership of shared hardware (CPU, PPU, and cart all touch the bus), no accidental aliasing bugs, and a strong testing story for validating against known-good ROMs like [nestest](https://www.nesdev.org/wiki/Emulator_tests).

## Project layout

```text
rust_nes/
├── src/
│   ├── main.rs          # Root binary (placeholder for now)
│   └── CPU/             # 6502 CPU library
│       ├── main.rs      # Registers, flags, step/reset/interrupts
│       ├── bus.rs       # Memory bus
│       └── instructions.rs
├── docs/
│   ├── OPCODES.md       # Opcode map audit
│   └── plans/           # Phase plans (0, 1, end-to-end)
└── Cargo.toml
```

The CPU crate currently has **zero dependencies**. Emulation core stays that way for as long as possible; a thin window/audio crate may appear later only at the presentation edge.

## Build

Requires a recent [Rust toolchain](https://rustup.rs/).

```bash
# Build the workspace / root crate
cargo build

# Build and test the CPU crate
cargo build -p CPU
cargo test -p CPU
```

There is no playable frontend yet — validation is via unit tests and (soon) nestest-style ROM harnesses.

## Design notes

- **Instruction table + function pointers** — OLC-style decode: fetch opcode → resolve addressing mode → execute → count cycles.
- **Correctness before features** — a broken CPU makes every later subsystem lie to you.
- **Test ROMs as oracles** — prefer nestest / blargg over eyeballing games early.
- **Bus-centric machine** — CPU/PPU/APU talk through memory-mapped I/O; a thin `Nes` shell will own the whole system later.

Opcode reference for this codebase: [`docs/OPCODES.md`](docs/OPCODES.md).

## References

- [NESdev Wiki](https://www.nesdev.org/wiki/Nesdev_Wiki)
- [Obelisk 6502 reference](https://www.nesdev.org/obelisk-6502-guide/reference.html)
- [Emulator tests (nestest, blargg, …)](https://www.nesdev.org/wiki/Emulator_tests)
- [Oxyron 6502 opcode matrix](https://www.oxyron.de/html/opcodes02.html)

## License

[MIT](LICENSE)
