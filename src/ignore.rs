use std::fmt;

use std::io;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;

/// Ignore all IO requests made on this object.
///
///  * For `Write`, that means reporting success always, but not doing anything.
///  * For `Read` and `Seek`, that means acting like an immutable empty file.
///
/// This may confuse things which use `Write` and `Seek` together, as writing to an
/// `Ignore` does not advance its seek position, which is not what would happen with a `File` or
/// `Cursor` or similar. The `Seek` implementation ignores out of bound seeks, negative seeks,
/// etc., in the hope of maybe mitigating this a bit.
///
/// # Example
/// ```rust
/// use std::io;
/// use iowrap::Ignore;
///
/// fn main() {
///     assert_eq!(0, io::copy(&mut Ignore::new(), &mut Ignore::new()).unwrap());
/// }
///
/// ```
pub struct Ignore {}

impl Ignore {
    pub fn new() -> Self {
        Ignore {}
    }
}

impl Write for Ignore {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        Ok(buf.len())
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }

    #[inline]
    fn write_all(&mut self, mut _buf: &[u8]) -> io::Result<()> {
        Ok(())
    }

    #[inline]
    fn write_fmt(&mut self, _fmt: fmt::Arguments) -> io::Result<()> {
        Ok(())
    }
}

impl Read for Ignore {
    #[inline]
    fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
        Ok(0)
    }

    #[inline]
    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        if buf.is_empty() {
            Ok(())
        } else {
            Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "there is nothing in this Ignore",
            ))
        }
    }
}

impl Seek for Ignore {
    #[inline]
    fn seek(&mut self, _pos: SeekFrom) -> io::Result<u64> {
        Ok(0)
    }
}
