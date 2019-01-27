use std::io;
use std::io::Read;

/// Intentionally return short reads, to test `Read` code.
///
/// The `decider` iterator gets to decide how short a read should be.
/// A read length of 0 generates an `ErrorKind::Interrupted` error.
/// When the iterator runs out before the reader, `read` will always
/// return zero-length reads (EOF).
///
/// Currently, no effort is made to make reads longer, if the underlying
/// reader naturally returns short reads.
///
/// # Examples
///
/// Short read:
///
/// ```rust
/// # use std::io;
/// # use std::io::Read;
/// let mut naughty = iowrap::ShortRead::new(
///         io::Cursor::new(b"1234567890"),
///         vec![2, 3, 4, 5, 6].into_iter()
/// );
/// let mut buf = [0u8; 10];
/// // A `Cursor` would normally return the whole ten bytes here,
/// // but we've limited it to two bytes.
/// assert_eq!(2, naughty.read(&mut buf).unwrap());
/// ```
///
/// Interrupted read:
///
/// ```rust
/// # use std::io;
/// # use std::io::Read;
/// let mut interrupting = iowrap::ShortRead::new(
///         io::Cursor::new(b"123"),
///         vec![0, 1, 0].into_iter()
/// );
/// let mut buf = [0u8; 10];
/// assert_eq!(io::ErrorKind::Interrupted,
///         interrupting.read(&mut buf).unwrap_err().kind());
/// ```
pub struct ShortRead<R: Read, I: Iterator<Item = usize>> {
    inner: R,
    decider: I,
}

impl<R: Read, I: Iterator<Item = usize>> Read for ShortRead<R, I> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let wanted = match self.decider.next() {
            Some(0) => return Err(io::Error::from(io::ErrorKind::Interrupted)),
            Some(wanted) => wanted,
            None => return Ok(0),
        };
        let wanted = wanted.min(buf.len());

        let buf = &mut buf[..wanted];
        self.inner.read(buf)
    }
}

impl<R: Read, I: Iterator<Item = usize>> ShortRead<R, I> {
    pub fn new(inner: R, decider: I) -> Self {
        ShortRead { inner, decider }
    }

    pub fn into_inner(self) -> R {
        self.inner
    }
}

#[cfg(test)]
mod tests {
    use crate::short::ShortRead;
    use std::io;
    use std::io::Read;

    #[test]
    fn shorten() {
        let mut naughty = ShortRead::new(
            io::Cursor::new(b"1234567890"),
            vec![2, 3, 4, 5, 6].into_iter(),
        );
        let mut buf = [0u8; 10];
        assert_eq!(2, naughty.read(&mut buf).unwrap());
        assert_eq!(b"12", &buf[..2]);
        assert_eq!(3, naughty.read(&mut buf).unwrap());
        assert_eq!(b"345", &buf[..3]);
        assert_eq!(4, naughty.read(&mut buf).unwrap());
        assert_eq!(b"6789", &buf[..4]);
        assert_eq!(1, naughty.read(&mut buf).unwrap());
        assert_eq!(b"0", &buf[..1]);

        assert_eq!(0, naughty.read(&mut buf).unwrap());
        assert_eq!(0, naughty.read(&mut buf).unwrap());
    }

    #[test]
    fn interrupt() {
        let mut interrupting = ShortRead::new(io::Cursor::new(b"12"), vec![0, 1, 0, 1].into_iter());
        let mut buf = [0; 1];

        assert_eq!(
            io::ErrorKind::Interrupted,
            interrupting.read(&mut buf).unwrap_err().kind()
        );
        assert_eq!(1, interrupting.read(&mut buf).unwrap());
        assert_eq!(
            io::ErrorKind::Interrupted,
            interrupting.read(&mut buf).unwrap_err().kind()
        );
        assert_eq!(1, interrupting.read(&mut buf).unwrap());
    }
}
