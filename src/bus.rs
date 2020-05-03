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
        self.memory[(address) as usize]
    }

    pub fn store(&mut self, value: u8, address: u16) {
        self.memory[(address) as usize] = value;
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

    // TODO: Needs refactor for CI
    #[test]
    #[ignore]
    fn test_read_file() {
        let mut tmp_dir = home_dir().unwrap();
        tmp_dir.push(".bash_history");
        assert_ne!(read_file(tmp_dir.as_path()).len(), 0)
    }
}