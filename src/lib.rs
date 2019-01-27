//! Some utility methods for wrapping `std::io::Read` and `std::io::Write`.

mod eof;
mod ignore;
mod many;
mod pos;
mod short;
mod var_buf;

pub use crate::eof::Eof;
pub use crate::ignore::Ignore;
pub use crate::many::ReadMany;
pub use crate::pos::Pos;
pub use crate::short::ShortRead;
pub use crate::var_buf::VarBufRead;
pub use crate::var_buf::VarBufReader;
