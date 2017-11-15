use std::io;
use std::io::Read;

pub struct Pos<R: Read> {
    inner: R,
    position: u64,
}

impl<R: Read> Pos<R> {
    pub fn new(inner: R) -> Self {
        Pos { inner, position: 0 }
    }

    pub fn position(&self) -> u64 {
        self.position
    }

    pub fn into_inner(self) -> R {
        self.inner
    }
}

impl<R: Read> Read for Pos<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self.inner.read(buf) {
            Ok(count) => {
                self.position = self.position.saturating_add(count as u64);
                Ok(count)
            }
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io;
    use std::io::Read;
    use super::Pos;

    #[test]
    fn smoke_cursor() {
        let mut pos = Pos::new(io::Cursor::new(vec![7, 8, 9, 10, 11, 12]));
        assert_eq!(0, pos.position());
        let mut buf = [0u8; 2];
        pos.read_exact(&mut buf).unwrap();
        assert_eq!(&[7, 8], &buf[..]);
        assert_eq!(2, pos.position());

        let mut buf = [0u8; 20];
        assert_eq!(4, pos.read(&mut buf).unwrap());
        assert_eq!(6, pos.position());

        assert_eq!(0, pos.read(&mut buf).unwrap());
        assert_eq!(6, pos.position());
    }
}
