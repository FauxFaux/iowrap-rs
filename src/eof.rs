use std::io;
use std::io::Read;

/// Track whether a stream has hit the end of file.
///
/// This allows slightly nicer code when reading lots of items using an uncooperative api,
/// or when you want an `UnexpectedEof` only after the first byte.
/// It also simplifies determining the last chunk when processing
/// input in fixed-size chunks.
///
/// `Eof::eof()` and – for consistency – the `Read` implementation of `Eof`
/// ignore `ErrorKind::Interrupted` errors and retry the operation.
///
/// # Example
///
/// ```rust,no_run
/// use std::io;
/// use std::io::Read;
/// use std::fs::File;
/// use iowrap::Eof;
///
/// fn load() -> io::Result<Vec<u64>> {
///   let mut ret = Vec::new();
///   let mut file = Eof::new(File::open("foo.bin")?);
///   while !file.eof()? {
///     ret.push(third_party::parse_thing(&mut file));
///   }
///   Ok(ret)
/// }
///
/// mod third_party {
///   use std::io::Read;
///   pub fn parse_thing<R: Read>(mut from: R) -> u64 {
///     let mut buf = [0u8; 8];
///     from.read_exact(&mut buf).unwrap();
///     u64::from(buf[0]) // oops!
///   }
/// }
/// ```
pub struct Eof<R: Read> {
    inner: R,
    next: Option<u8>,
}

impl<R: Read> Eof<R> {
    pub fn new(inner: R) -> Self {
        Eof { inner, next: None }
    }

    /// Test if we are at the end of the stream.
    /// If false, then a proceeding `read()` will always succeed.
    pub fn eof(&mut self) -> io::Result<bool> {
        if self.next.is_some() {
            return Ok(false);
        }

        let mut buf = [0u8; 1];
        Ok(loop {
            break match self.inner.read(&mut buf) {
                Ok(0) => true,
                Ok(1) => {
                    self.next = Some(buf[0]);
                    false
                }
                Ok(_) => unreachable!(),
                Err(ref e) if e.kind() == io::ErrorKind::Interrupted => continue,
                Err(e) => return Err(e),
            };
        })
    }

    /// The buffered value, which we read while checking for EOF.
    pub fn held_state(&self) -> Option<u8> {
        self.next
    }

    pub fn get_ref(&self) -> &R {
        &self.inner
    }

    pub fn get_mut(&mut self) -> &mut R {
        &mut self.inner
    }

    pub fn into_inner(self) -> R {
        self.inner
    }
}

impl<R: Read> Read for Eof<R> {
    /// For consistency with `eof()`, this implementation retries the
    /// operation on `ErrorKind::Interrupted` errors.
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }

        if let Some(val) = self.next {
            buf[0] = val;
            self.next = None;
            return Ok(1);
        }

        loop {
            match self.inner.read(buf) {
                Err(ref e) if e.kind() == io::ErrorKind::Interrupted => continue,
                e => return e,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Eof;
    use crate::short::ShortRead;
    use std::io;
    use std::io::Read;

    #[test]
    fn smoke_cursor() {
        let mut eof = Eof::new(io::Cursor::new(vec![7, 8, 9, 10, 11, 12]));
        assert_eq!(None, eof.held_state(), "nothing held initially");
        assert_eq!(
            false,
            eof.eof().unwrap(),
            "there's bytes to read, we're not at the end"
        );
        assert_eq!(
            false,
            eof.eof().unwrap(),
            "we weren't at the end before, so we're not now"
        );
        assert_eq!(Some(7), eof.held_state(), "the state is the first byte");
        assert_eq!(
            false,
            eof.eof().unwrap(),
            "viewing the state doesn't move us to the end"
        );

        let mut buf = [0u8; 2];
        eof.read_exact(&mut buf).unwrap();

        assert_eq!(None, eof.held_state(), "reading consumed the state");
        assert_eq!(
            false,
            eof.eof().unwrap(),
            "reading two bytes didn't push us past the end"
        );
        assert_eq!(
            Some(9),
            eof.held_state(),
            "checking the eof read the third byte into state"
        );

        let mut buf = [0u8; 20];
        assert_eq!(
            1,
            eof.read(&mut buf).unwrap(),
            "[implementation detail] read will only return one byte"
        );
        assert_eq!(9, buf[0], "it was the right byte");
        assert_eq!(3, eof.read(&mut buf).unwrap(), "there's three more bytes");
        assert_eq!(
            None,
            eof.held_state(),
            "there's no state after some reading"
        );
        assert_eq!(true, eof.eof().unwrap(), "we're at the end");
        assert_eq!(None, eof.held_state(), "there's still no state");

        eof.get_mut().get_mut().push(100);
        assert_eq!(
            false,
            eof.eof().unwrap(),
            "if the underlying reader starts returning data again, so do we"
        );
    }

    #[test]
    fn interrupted_read() {
        let take_a_break = ShortRead::new(
            io::Cursor::new(b"12345"),
            vec![0, 1, 0, 1, 0, 0, 0, 9, 9, 0, 1].into_iter(),
        );
        let mut eof = Eof::new(take_a_break);

        let mut buf = [0u8; 2];

        assert_eq!(
            false,
            eof.eof().unwrap(),
            "skip interruption at the beginning"
        );
        assert_eq!(1, eof.read(&mut buf).unwrap());
        assert_eq!(b'1', buf[0]);

        assert_eq!(
            1,
            eof.read(&mut buf).unwrap(),
            "skip interruption while reading"
        );
        assert_eq!(b'2', buf[0]);

        assert_eq!(false, eof.eof().unwrap(), "skip multiple interruptions");
        assert_eq!(1, eof.read(&mut buf).unwrap());
        assert_eq!(b'3', buf[0]);
        assert_eq!(2, eof.read(&mut buf).unwrap());
        assert_eq!(b"45", &buf);

        assert_eq!(true, eof.eof().unwrap(), "skip interruption before eof");
    }
}
