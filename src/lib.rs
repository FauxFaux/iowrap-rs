//! Some utility methods for wrapping `std::io::Read` and `std::io::Write`.

mod eof;
mod pos;

pub use eof::Eof;
pub use pos::Pos;
