# NES emulator in rust

## Description
![Donkey kong screenshot](https://i.imgur.com/Bn5FSSp.png)

[NES](https://en.wikipedia.org/wiki/Nintendo_Entertainment_System) emulator written in rust. Unofficial opcodes are not supported. Only mapper 0 is implemented. 


## Usage

Tested only on Windows.

```rust
cargo run --package r_nes --bin r_nes
```


## Features:

- [x] CPU
  - [x] Official opcodes
  - [ ] Unofficial opcodes 
- [x] PPU
- [x] PAD
- [ ] APU
- [x] Mappers
  - [x] Mapper 0



[![Build Status](https://travis-ci.com/kregoslup/r_nes.svg?branch=master)](https://travis-ci.com/kregoslup/r_nes)