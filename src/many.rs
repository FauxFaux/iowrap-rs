use std::io;
use std::io::Read;

/// Retry `read` if it read short, to check we're at the end of the file.
///
/// `read` is allowed to return fewer bytes than requested, even if
/// we're not at the end of the file.
///
/// `read_exact` has undefined behaviour if we hit the end of the file
/// while reading. It will return an error, but won't necessarily have
/// put the lost bytes into the buffer.
///
/// `read_many` will only return a short read if the underlying reader
/// returns `0`, indicating an end of file condition.
pub trait ReadMany {
    /// Try quite hard to fill `buf` with bytes.
    ///
    /// Give up if the underlying  reader returns an end-of-file
    /// condition or error only, not if it's just a bit lazy.
    ///
    /// Errors from the underlying reader will be returned as-is.
    fn read_many(&mut self, buf: &mut [u8]) -> io::Result<usize>;
}

impl<T: Read> ReadMany for T {
    fn read_many(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut pos = 0;

        while pos < buf.len() {
            match self.read(&mut buf[pos..]) {
                Ok(0) => break,
                Ok(read) => pos += read,
                Err(ref e) if e.kind() == io::ErrorKind::Interrupted => {}
                Err(e) => return Err(e),
            }
        }

        Ok(pos)
    }
}

#[cfg(test)]
mod tests {
    use crate::many::ReadMany;
    use crate::short::ShortRead;
    use std::io;

    #[test]
    fn short_read() {
        let mut naughty =
            ShortRead::new(io::Cursor::new(b"1234567890"), vec![2, 1, 4, 5].into_iter());
        let mut buf = [0u8; 3];
        assert_eq!(3, naughty.read_many(&mut buf).unwrap());
        assert_eq!(b"123", &buf);

        let mut buf = [0u8; 12];
        assert_eq!(7, naughty.read_many(&mut buf).unwrap());
        assert_eq!(b"4567890", &buf[..7]);
    }

    #[test]
    fn interrupted_read() {
        let mut take_a_break = ShortRead::new(io::Cursor::new(b"12345"), vec![2, 0, 3].into_iter());
        let mut buf = [0u8; 5];
        assert_eq!(5, take_a_break.read_many(&mut buf).unwrap());
        assert_eq!(b"12345", &buf);
    }
}
