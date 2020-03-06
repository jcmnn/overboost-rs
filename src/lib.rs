use std::fs::File;
use std::io::{Error, Read, SeekFrom};
use std::ops::{Index, Mul};
use std::path::Path;
use std::slice::SliceIndex;

pub use byteordered::Endianness;

use crate::platform::Platform;
use crate::table::{Table, TableData};

pub mod datalink;
pub mod numvec;
pub mod platform;
pub mod table;

pub struct Rom {
    data: Vec<u8>,
}

impl Rom {
    /// Returns table.
    pub fn read_table(&self, table: &Table) -> TableData {
        unimplemented!()
    }
}

pub trait RomRead {
    fn read_rom(&mut self, size: usize) -> std::io::Result<Rom>;
}

impl<T> RomRead for T
    where
        T: Read,
{
    /// Reads ROM data from stream.
    fn read_rom(&mut self, size: usize) -> std::io::Result<Rom> {
        let mut data = vec![0; size];
        self.read_exact(&mut data)?;
        Ok(Rom { data })
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;

    use crate::numvec::NumVecRead;
    use crate::platform::{Mazdaspeed6, Platform};

    use super::*;
    use super::table::NumVec;

    #[cfg(unix)]
    #[test]
    fn socketcan() {
        use socketcan::CANSocket;
        let socket = CANSocket::open("vcan0").unwrap();
    }
}
