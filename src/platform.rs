use crate::table::Table;

pub trait Download {}

/// A platform.
pub trait Platform {
    /// Returns the descriptive name of the platform. Used for user display purposes.
    /// # Example
    /// ```
    /// use overboost::platform::{Platform, Mazdaspeed6};
    /// let name = <Mazdaspeed6 as Platform>::name();
    /// assert_eq!(name, "Mazdaspeed6 / Mazda 6 MPS / Mazdaspeed Atenza");
    /// ```
    fn name() -> &'static str;

    /// Returns the unique id of the platform. Used for identification.
    /// # Example
    /// ```
    /// use overboost::platform::{Platform, Mazdaspeed6};
    /// let id = <Mazdaspeed6 as Platform>::id();
    /// assert_eq!(id, "mazdaspeed6");
    /// ```
    fn id() -> &'static str;

    /// Searches for table with `id`.
    fn table(&self, id: &str) -> Option<Table>;

    /// Returns byte length of ROM.
    fn rom_length(&self) -> usize;
}

pub struct Mazdaspeed6;

impl Platform for Mazdaspeed6 {
    fn name() -> &'static str {
        "Mazdaspeed6 / Mazda 6 MPS / Mazdaspeed Atenza"
    }

    fn id() -> &'static str {
        "mazdaspeed6"
    }

    fn table(&self, id: &str) -> Option<Table> {
        None
    }

    fn rom_length(&self) -> usize {
        1024 * 1024 * 1024
    }
}
