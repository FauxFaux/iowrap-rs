//! Some utility methods for wrapping `std::io::Read` and `std::io::Write`.

mod eof;
mod ignore;
mod many;
mod pos;
mod short;

pub use eof::Eof;
pub use ignore::Ignore;
pub use many::ReadMany;
pub use pos::Pos;
pub use short::ShortRead;
