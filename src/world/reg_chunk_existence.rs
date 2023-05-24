use std::{fs::File, io::Read, path::Path};

#[derive(Clone, Copy)]
pub struct ExistenceBitset(pub u64);

impl ExistenceBitset {
    pub const EMPTY: Self = Self(0);

    pub fn read_from_file(f: &mut File) -> ExistenceBitset {
        let mut buf = [0; 8];
        f.read_exact(&mut buf).unwrap();
        ExistenceBitset(u64::from_le_bytes(buf))
    }

    pub fn read_from_fs(path: &Path) -> ExistenceBitset {
        let mut f = File::open(path).unwrap();
        Self::read_from_file(&mut f)
    }
}

impl std::fmt::Debug for ExistenceBitset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f)?;
        for i in 0..64 {
            let chr = if crate::bitmanip::nth_bit_set(self.0, i) {
                'X'
            } else {
                '_'
            };
            write!(f, "{chr}")?;
            if (i + 1) % 8 == 0 {
                writeln!(f)?;
            }
        }
        Ok(())
    }
}
