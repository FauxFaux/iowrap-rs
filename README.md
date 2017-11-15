## iowrap

[![](https://img.shields.io/crates/v/iowrap.svg)](https://crates.io/crates/iowrap)

A couple of utilities that I have ended up wanting in various projects,
around `std::io::Read` streams.

 * `Eof` has an `eof()? -> bool` to check if the stream is at the end.
 * `Pos` has an `position() -> u64` to find out where you are in a stream.
 * `Ignore` implements `Read` and `Write` and `Seek` and.. and does nothing.

## License

Don't care. It's listed as `MIT` but do tell me if that's inconvenient.
