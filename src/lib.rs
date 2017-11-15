//! Some utility methods for wrapping `std::io::Read` and `std::io::Write`.

mod eof;
mod ignore;
mod pos;

pub use eof::Eof;
pub use ignore::Ignore;
pub use pos::Pos;
