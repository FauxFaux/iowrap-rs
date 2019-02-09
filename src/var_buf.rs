use std::io;
use std::io::BufRead;
use std::io::Read;

/// An "extension" of `std::io::BufRead`, for which `fill_*` can be forced to read.
///
/// `BufRead`'s `fill_buf` may return only 1 byte, and will not re-fill until you consume.
///
/// `fill_many` will try to fill the internal buffer to the `target` requested,
/// giving up only if the underlying reader hits end-of-file, or returns an error
///
/// `fill_at_least` will only return success if the internal buffer contains at least
/// `target` bytes.
///
/// `consume` advances the internal pointer, same as with `BufRead`.
pub trait VarBufRead {
    /// Advance the internal pointer, so that `fill_*` and `read*` will no longer return
    /// the consumed bytes.
    fn consume(&mut self, amt: usize);

    /// Try hard to return a buffer of at least `target` bytes. If the end of file is
    /// reached, then the buffer will be shorter. If we already have sufficient bytes in
    /// memory, then no reads will be performed, and the larger buffer will be returned.
    ///
    /// Other errors (except interruption) are returned as-is.
    fn fill_many(&mut self, target: usize) -> io::Result<&[u8]>;

    /// Return a buffer of at least `target` bytes, by repeatedly reading from the
    /// underlying reader. If the underlying reader reaches end-of-file, an error will
    /// be returned.
    ///
    /// Other errors (except interruption) are returned as-is.
    fn fill_at_least(&mut self, target: usize) -> io::Result<&[u8]> {
        let buf = self.fill_many(target)?;
        if buf.len() < target {
            return Err(io::ErrorKind::UnexpectedEof.into());
        }
        Ok(buf)
    }

    /// Read
    fn read_until_limit(&mut self, delim: u8, limit: usize) -> Result<Vec<u8>, io::Error> {
        let buf = self.fill_many(limit)?;
        if let Some(end) = memchr::memchr(delim, buf) {
            let ret = buf[..end].to_vec();
            self.consume(end + 1);
            return Ok(ret);
        }

        Err(io::ErrorKind::NotFound.into())
    }
}

pub struct VarBufReader<R> {
    inner: R,
    data: Vec<u8>,
}

impl<R: Read> VarBufReader<R> {
    pub fn new(inner: R) -> VarBufReader<R> {
        VarBufReader {
            inner,
            data: Vec::new(),
        }
    }
}

impl<R: Read> VarBufRead for VarBufReader<R> {
    fn consume(&mut self, amt: usize) {
        assert!(amt <= self.data.len());
        self.data.drain(..amt);
    }

    fn fill_many(&mut self, target: usize) -> Result<&[u8], io::Error> {
        while self.data.len() < target {
            let mut buf = [0u8; 8 * 1024];
            let read = self.inner.read(&mut buf)?;
            if 0 == read {
                break;
            }
            self.data.extend(&buf[..read]);
        }

        Ok(&self.data)
    }
}

impl<R: Read> BufRead for VarBufReader<R> {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        self.fill_many(1)
    }

    fn consume(&mut self, amt: usize) {
        VarBufRead::consume(self, amt)
    }
}

impl<R: Read> Read for VarBufReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, io::Error> {
        let found = self.fill_many(buf.len())?;
        let valid = buf.len().min(found.len());
        buf[..valid].copy_from_slice(&found[..valid]);
        VarBufRead::consume(self, valid);
        Ok(valid)
    }
}

#[cfg(test)]
mod tests {
    use std::io;
    use std::io::Cursor;
    use std::io::Read;

    use byteorder::ReadBytesExt;

    use crate::ShortRead;

    use super::VarBufRead;
    use super::VarBufReader;

    #[test]
    fn fill_then_read() {
        let mut vb = VarBufReader::new(ShortRead::new(
            Cursor::new(b"hello"),
            vec![1, 1, 2, 1, 99].into_iter(),
        ));
        assert_eq!(b"hell", &vb.fill_many(4).unwrap()[..4]);
        assert_eq!(b'h', vb.read_u8().unwrap());
        let mut buf = [0u8; 4];
        assert_eq!(4, vb.read(&mut buf).unwrap());
        assert_eq!(b"ello", &buf);
    }

    #[test]
    fn read_then_fill() {
        let mut vb = VarBufReader::new(ShortRead::new(
            Cursor::new(b"hello world"),
            vec![1, 1, 2, 1, 99].into_iter(),
        ));
        assert_eq!(b'h', vb.read_u8().unwrap());
        assert_eq!(b"ell", &vb.fill_many(5).unwrap()[..3]);
        assert_eq!(b'e', vb.read_u8().unwrap());
        vb.consume("llo ".len());
        assert_eq!(b"world", &vb.fill_many(7).unwrap());
    }

    #[test]
    fn double_fill() {
        let mut vb = VarBufReader::new(ShortRead::new(
            Cursor::new(b"hello world"),
            vec![1, 1, 2, 1, 99].into_iter(),
        ));
        assert_eq!(b"he", &vb.fill_many(2).unwrap()[..2]);
        assert_eq!(b"hell", &vb.fill_many(4).unwrap()[..4]);
        vb.consume(3);
        assert_eq!(b"lo", &vb.fill_many(2).unwrap()[..2]);
        assert_eq!(b'l', vb.read_u8().unwrap());
        assert_eq!(b'o', vb.read_u8().unwrap());
    }

    #[test]
    fn eof() {
        let mut vb = VarBufReader::new(ShortRead::new(
            Cursor::new(b"hello world"),
            vec![1, 1, 2, 1, 99].into_iter(),
        ));
        assert_eq!(b"hello world", &vb.fill_many(100).unwrap());

        vb.consume("hello wor".len());
        assert_eq!(b"ld", &vb.fill_many(100).unwrap());
        assert_eq!(b"ld", &vb.fill_at_least(2).unwrap());
        assert_eq!(
            io::ErrorKind::UnexpectedEof,
            vb.fill_at_least(3).unwrap_err().kind()
        );

        vb.consume(1);
        assert_eq!(b"d", &vb.fill_at_least(1).unwrap());
        assert_eq!(
            io::ErrorKind::UnexpectedEof,
            vb.fill_at_least(2).unwrap_err().kind()
        );

        vb.consume(1);
        assert_eq!(b"", &vb.fill_many(1).unwrap());
        assert_eq!(
            io::ErrorKind::UnexpectedEof,
            vb.fill_at_least(1).unwrap_err().kind()
        );
    }

    #[test]
    fn read_short() {
        let mut vb = VarBufReader::new(ShortRead::new(
            Cursor::new(b"hello there world"),
            vec![1, 1, 2, 1, 99].into_iter(),
        ));
        assert_eq!(
            io::ErrorKind::NotFound,
            vb.read_until_limit(b' ', 3).unwrap_err().kind()
        );
        assert_eq!(
            io::ErrorKind::NotFound,
            vb.read_until_limit(b' ', 4).unwrap_err().kind()
        );
        assert_eq!(
            io::ErrorKind::NotFound,
            vb.read_until_limit(b' ', 5).unwrap_err().kind()
        );
        assert_eq!(b"hello", vb.read_until_limit(b' ', 6).unwrap().as_slice());
        assert_eq!(b"there", vb.read_until_limit(b' ', 6).unwrap().as_slice());
        assert_eq!(
            io::ErrorKind::NotFound,
            vb.read_until_limit(b' ', 200).unwrap_err().kind()
        );
        assert_eq!(b"world", &vb.fill_many(5).unwrap()[..5]);
    }
}
