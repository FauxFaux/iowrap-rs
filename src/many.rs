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

        loop {
            let read = self.read(&mut buf[pos..])?;

            if 0 == read {
                return Ok(pos);
            }

            pos += read;

            if buf.len() == pos {
                return Ok(pos);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use many::ReadMany;
    use short::ShortRead;
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
}
