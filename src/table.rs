use std::io::{Error, Read, Seek, SeekFrom, Write};

use byteordered::{ByteOrdered, Endianness};
use num::cast::AsPrimitive;

use crate::numvec::DataType;

/// Axis ticks can be stored in memory or evaluated with a function.
pub enum AxisTicks {
    /// Offset to region in memory where axis is stored.
    Memory(u64),

    /// Linear function y = mx + b where `b` is the first argument and `m` is the second.
    Linear(f64, f64),
}

/// Table axis
pub struct Axis {
    /// Unique identifier string
    id: String,

    /// Short axis name
    name: String,

    /// Long axis description
    description: String,

    /// Ticks
    ticks: AxisTicks,
}

/// Interpolation used during table queries
pub enum Interpolation {
    Linear,
}

/// Describes the format of a table.
pub struct Table {
    pub width: usize,
    pub height: usize,

    /// Offset from beginning of ROM
    pub offset: u64,

    /// Printable name of the table
    pub name: String,

    /// Long description of the table
    pub description: String,

    /// Unique identifier
    pub id: String,

    /// X-Axis identifier
    pub x_axis_id: Option<String>,

    /// Y-Axis identifier
    pub y_axis_id: Option<String>,

    pub interpolation: Interpolation,

    pub data_type: DataType,

    pub endianness: Endianness,
}

impl Table {
    /// Returns true if the table contains only one value.
    fn is_scalar(&self) -> bool {
        self.width == 1 && self.height == 1
    }

    /// Returns true if the table height equals 1.
    /// Note: scalar values will return true.
    fn is_one_dimensional(&self) -> bool {
        self.height == 1
    }

    /// Returns the size of the table in bytes.
    fn byte_size(&self) -> usize {
        self.data_type.byte_size() * self.width * self.height
    }

    /// Returns total number of elements in the table (width x height)
    pub fn size(&self) -> usize {
        self.width * self.height
    }

    /// Returns table width
    fn width(&self) -> usize {
        self.width
    }

    /// Returns table height
    fn height(&self) -> usize {
        self.height
    }
}

/// [`NumVec`] contains a vector of any number type. This is useful for table
/// and axis data where we don't know the type at compile time.
#[derive(Debug)]
pub enum NumVec {
    I8(Vec<i8>),
    U8(Vec<u8>),
    I16(Vec<i16>),
    U16(Vec<u16>),
    I32(Vec<i32>),
    U32(Vec<u32>),
    I64(Vec<i64>),
    U64(Vec<u64>),
    F32(Vec<f32>),
    F64(Vec<f64>),
}

/// Run expression over inner vector
macro_rules! expand_numvec {
    ($s:ident, $v:ident, $e:expr) => {
        match $s {
            NumVec::I8($v) => $e,
            NumVec::U8($v) => $e,
            NumVec::I16($v) => $e,
            NumVec::U16($v) => $e,
            NumVec::I32($v) => $e,
            NumVec::U32($v) => $e,
            NumVec::I64($v) => $e,
            NumVec::U64($v) => $e,
            NumVec::F32($v) => $e,
            NumVec::F64($v) => $e,
        }
    };
}

impl NumVec {
    /// Get index from vector. Returns internal type casted to `T`.
    pub fn get<T>(&self, index: usize) -> T
        where
            T: Copy + 'static,
            i8: AsPrimitive<T>,
            u8: AsPrimitive<T>,
            i16: AsPrimitive<T>,
            u16: AsPrimitive<T>,
            i32: AsPrimitive<T>,
            u32: AsPrimitive<T>,
            i64: AsPrimitive<T>,
            u64: AsPrimitive<T>,
            f32: AsPrimitive<T>,
            f64: AsPrimitive<T>,
    {
        expand_numvec!(self, v, v[index].as_())
    }

    /// Casts value to internal type and sets in array.
    pub fn set<T>(&mut self, index: usize, value: T)
        where
            T: AsPrimitive<i8>
            + AsPrimitive<u8>
            + AsPrimitive<i16>
            + AsPrimitive<u16>
            + AsPrimitive<i32>
            + AsPrimitive<u32>
            + AsPrimitive<i64>
            + AsPrimitive<u64>
            + AsPrimitive<f32>
            + AsPrimitive<f64>,
    {
        expand_numvec!(self, v, v[index] = value.as_());
    }

    /// Returns vector length
    pub fn len(&self) -> usize {
        expand_numvec!(self, v, v.len())
    }
}

/// Container for two-dimensional table data
#[derive(Debug)]
pub struct TableData {
    data: NumVec,
    width: usize,
    height: usize,
}

impl TableData {
    /// Returns entry at (col, row) from table casted to type.
    fn get<T>(&self, col: usize, row: usize) -> T
        where
            T: Copy + 'static,
            i8: AsPrimitive<T>,
            u8: AsPrimitive<T>,
            i16: AsPrimitive<T>,
            u16: AsPrimitive<T>,
            i32: AsPrimitive<T>,
            u32: AsPrimitive<T>,
            i64: AsPrimitive<T>,
            u64: AsPrimitive<T>,
            f32: AsPrimitive<T>,
            f64: AsPrimitive<T>,
    {
        self.data.get(row * self.width + col)
    }

    /// Casts value to internal type and sets into table at (col, row).
    pub fn set<T>(&mut self, col: usize, row: usize, value: T)
        where
            T: AsPrimitive<i8>
            + AsPrimitive<u8>
            + AsPrimitive<i16>
            + AsPrimitive<u16>
            + AsPrimitive<i32>
            + AsPrimitive<u32>
            + AsPrimitive<i64>
            + AsPrimitive<u64>
            + AsPrimitive<f32>
            + AsPrimitive<f64>,
    {
        self.data.set(row * self.width + col, value)
    }
}

mod tests {
    use std::io::{Cursor, Seek};

    use super::*;

    #[test]
    fn num_vec() {
        let mut nv = NumVec::U8(vec![0, 1, 2, 3, 4, 5]);
        // Len
        assert_eq!(nv.len(), 6);

        // Get
        assert_eq!(nv.get::<u8>(0), 0_u8);

        // Set
        nv.set(5, 12);
        assert_eq!(nv.get::<u8>(5), 12_u8);
    }

    #[test]
    fn table_read() {
        let table = Table {
            width: 8,
            height: 8,
            offset: 0,
            name: "".to_string(),
            description: "".to_string(),
            id: "".to_string(),
            x_axis_id: None,
            y_axis_id: None,
            interpolation: Interpolation::Linear,
            data_type: DataType::I32,
            endianness: Endianness::Big,
        };

        // Write test data
        let mut buff = Cursor::new(Vec::<u8>::new());

        let mut wr = byteordered::ByteOrdered::be(&mut buff);
        for i in 0..64 {
            wr.write_i32(i);
        }
        // Seek to beginning of buffer
        buff.set_position(0);
        /*
        let table_data = buff.read_table(&table).unwrap();
        for r in 0..8_i32 {
            for c in 0..8_i32 {
                assert_eq!(table_data.get::<i32>(c as usize, r as usize), r * 8 + c);
            }
        }*/
    }

    #[test]
    fn table_write() {}
}
