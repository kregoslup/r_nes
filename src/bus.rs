use std::path::Path;
use std::fs::File;
use std::io::Read;
use dirs::home_dir;
use std::fmt::Debug;

#[derive(Debug)]
pub struct Bus {
    memory: Vec<u8>
}

impl Bus {
    pub fn load_rom(path: &Path) -> Bus {
        let loaded_rom: Vec<u8> = read_file(path);
        return Bus::new(loaded_rom)
    }

    pub fn new(memory: Vec<u8>) -> Bus {
        Bus {
            memory,
        }
    }

    pub fn fetch(&self, address: u16) -> u8 {
        self.memory[address as usize]
    }
}

fn read_file(path: &Path) -> Vec<u8> {
    let mut file = File::open(path).unwrap();
    let mut data = Vec::new();
    file.read_to_end(&mut data);
    return data;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_file() {
        let mut tmp_dir = home_dir().unwrap();
        tmp_dir.push(".bash_history");
        assert_ne!(read_file(tmp_dir.as_path()).len(), 0)
    }

    #[test]
    fn test_load_rom_not_empty() {
        let mut tmp_dir = home_dir().unwrap();
        tmp_dir.push(".bash_history");
        let bus = Bus::load_rom(tmp_dir.as_path());
        assert_ne!(memory.fetch_byte_at_offset(0), 0)
    }
}