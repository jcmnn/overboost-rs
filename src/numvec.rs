use std::io::{Error, Read, Write};

use byteordered::{ByteOrdered, Endianness};
use byteordered::byteorder::WriteBytesExt;

use crate::table::NumVec;

/// DataType for table data
pub enum DataType {
    I8,
    U8,
    I16,
    U16,
    I32,
    U32,
    I64,
    U64,
    F32,
    F64,
}

impl DataType {
    /// # Example
    /// ```
    /// use overboost::numvec::DataType;
    /// let data_type = DataType::I32;
    /// assert_eq!(data_type.byte_size(), 4);
    /// ```
    pub fn byte_size(&self) -> usize {
        match self {
            DataType::I8 | DataType::U8 => 1,
            DataType::I16 | DataType::U16 => 2,
            DataType::I32 | DataType::U32 | DataType::F32 => 4,
            DataType::I64 | DataType::U64 | DataType::F64 => 8,
        }
    }
}

pub trait NumVecWrite {
    /// Writes NumVec to byte stream.
    fn write_num_vec(&mut self, endianness: Endianness, num_vec: &NumVec) -> std::io::Result<()>;
}

impl<T> NumVecWrite for T
    where
        T: Write,
{
    /// Writes NumVec to stream.
    /// This function makes small writes, so a [`BufWriter`] should
    /// be used when not writing to an in-memory buffer.
    /// # Example
    /// ```no_run
    /// use std::fs::File;
    /// use byteordered::Endianness;
    /// use overboost::numvec::{DataType, NumVecRead, NumVecWrite};
    /// use overboost::table::NumVec;
    /// use std::io::BufWriter;
    ///
    /// let mut file = File::create("rom.bin").unwrap();
    /// let mut buff = BufWriter::new(file);
    /// // Write 8 32 bit integers
    /// let nv = NumVec::I32(vec![0, 1, 2, 3, 4, 5, 6, 7, 8]);
    /// let table_data = buff.write_num_vec(Endianness::Big, &nv);
    /// ```
    fn write_num_vec(&mut self, endianness: Endianness, num_vec: &NumVec) -> std::io::Result<()> {
        // Write to stream
        let mut wr = ByteOrdered::runtime(self, endianness);
        match num_vec {
            NumVec::I8(v) => {
                for n in v.iter() {
                    wr.write_i8(*n)?;
                }
            }
            NumVec::U8(v) => {
                for n in v.iter() {
                    wr.write_u8(*n)?;
                }
            }
            NumVec::I16(v) => {
                for n in v.iter() {
                    wr.write_i16(*n)?;
                }
            }
            NumVec::U16(v) => {
                for n in v.iter() {
                    wr.write_u16(*n)?;
                }
            }
            NumVec::I32(v) => {
                for n in v.iter() {
                    wr.write_i32(*n)?;
                }
            }
            NumVec::U32(v) => {
                for n in v.iter() {
                    wr.write_u32(*n)?;
                }
            }
            NumVec::I64(v) => {
                for n in v.iter() {
                    wr.write_i64(*n)?;
                }
            }
            NumVec::U64(v) => {
                for n in v.iter() {
                    wr.write_u64(*n)?;
                }
            }
            NumVec::F32(v) => {
                for n in v.iter() {
                    wr.write_f32(*n)?;
                }
            }
            NumVec::F64(v) => {
                for n in v.iter() {
                    wr.write_f64(*n)?;
                }
            }
        };
        Ok(())
    }
}

pub trait NumVecRead {
    /// Reads NumVec from byte stream.
    fn read_num_vec(
        &mut self,
        data_type: DataType,
        endianness: Endianness,
        length: usize,
    ) -> std::io::Result<NumVec>;
}

impl<T> NumVecRead for T
    where
        T: Read,
{
    /// Reads `NumVec` from stream.
    /// # Example
    /// ```no_run
    /// use std::fs::File;
    /// use byteordered::Endianness;
    /// use overboost::numvec::{DataType, NumVecRead};
    ///
    /// let mut file = File::open("rom.bin").unwrap();
    /// // Read 8 32 bit integers
    /// let table_data = file.read_numvec(DataType::I32, Endianness::Big, 8);
    /// ```
    fn read_num_vec(
        &mut self,
        data_type: DataType,
        endianness: Endianness,
        length: usize,
    ) -> std::io::Result<NumVec> {
        // Seek to offset
        /*if self.seek(SeekFrom::Start(table.offset))? != table.offset {
            return Err(std::io::Error::from(std::io::ErrorKind::UnexpectedEof));
        }*/

        // Read from stream
        let mut rd = ByteOrdered::runtime(self, endianness);
        let data = match data_type {
            DataType::I8 => {
                let mut v = vec![0; length];
                for i in 0..length {
                    v[i] = rd.read_i8()?;
                }
                NumVec::I8(v)
            }
            DataType::U8 => {
                let mut v = vec![0; length];
                for i in 0..length {
                    v[i] = rd.read_u8()?;
                }
                NumVec::U8(v)
            }
            DataType::I16 => {
                let mut v = vec![0; length];
                for i in 0..length {
                    v[i] = rd.read_i16()?;
                }
                NumVec::I16(v)
            }
            DataType::U16 => {
                let mut v = vec![0; length];
                for i in 0..length {
                    v[i] = rd.read_u16()?;
                }
                NumVec::U16(v)
            }
            DataType::I32 => {
                let mut v = vec![0; length];
                for i in 0..length {
                    v[i] = rd.read_i32()?;
                }
                NumVec::I32(v)
            }
            DataType::U32 => {
                let mut v = vec![0; length];
                for i in 0..length {
                    v[i] = rd.read_u32()?;
                }
                NumVec::U32(v)
            }
            DataType::I64 => {
                let mut v = vec![0; length];
                for i in 0..length {
                    v[i] = rd.read_i64()?;
                }
                NumVec::I64(v)
            }
            DataType::U64 => {
                let mut v = vec![0; length];
                for i in 0..length {
                    v[i] = rd.read_u64()?;
                }
                NumVec::U64(v)
            }
            DataType::F32 => {
                let mut v = vec![0_f32; length];
                for i in 0..length {
                    v[i] = rd.read_f32()?;
                }
                NumVec::F32(v)
            }
            DataType::F64 => {
                let mut v = vec![0_f64; length];
                for i in 0..length {
                    v[i] = rd.read_f64()?;
                }
                NumVec::F64(v)
            }
        };

        Ok(data)
    }
}
