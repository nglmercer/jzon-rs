use jzon_rs_compat as facade;
use serde::{Deserialize, Serialize};
use serde_json as reference_json;
use std::cell::Cell;

struct FailsOnce(Cell<usize>);
impl Serialize for FailsOnce {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let count = self.0.get();
        self.0.set(count + 1);
        if count == 0 {
            return Err(serde::ser::Error::custom("first invocation"));
        }
        serializer.serialize_u8(1)
    }
}
#[test]
fn serializer_callbacks_run_once() {
    for mode in 0..6 {
        let value = FailsOnce(Cell::new(0));
        let error = match mode {
            0 => facade::to_string(&value).map(|_| ()),
            1 => facade::to_string_pretty(&value).map(|_| ()),
            2 => facade::to_vec(&value).map(|_| ()),
            3 => facade::to_vec_pretty(&value).map(|_| ()),
            4 => facade::to_writer(Vec::new(), &value),
            _ => facade::to_writer_pretty(Vec::new(), &value),
        }
        .unwrap_err();
        assert_eq!(error.to_string(), "first invocation");
        assert_eq!(value.0.get(), 1);
    }
}
thread_local! { static CALLS: Cell<usize> = const { Cell::new(0) }; }
struct Failure;
impl<'de> Deserialize<'de> for Failure {
    fn deserialize<D: serde::Deserializer<'de>>(_: D) -> Result<Self, D::Error> {
        CALLS.with(|count| count.set(count.get() + 1));
        Err(serde::de::Error::custom("user error"))
    }
}
#[test]
fn deserializer_callbacks_run_once() {
    for mode in 0..3 {
        CALLS.with(|c| c.set(0));
        let error = match mode {
            0 => facade::from_str::<Failure>("null"),
            1 => facade::from_slice::<Failure>(b"null"),
            _ => facade::from_reader::<_, Failure>(&b"null"[..]),
        }
        .err()
        .unwrap();
        assert!(error.is_data());
        assert_eq!(error.to_string(), "user error");
        CALLS.with(|c| assert_eq!(c.get(), 1));
    }
}

struct Reader {
    bytes: &'static [u8],
    position: usize,
    fail_at: usize,
    interrupt: bool,
}
impl std::io::Read for Reader {
    fn read(&mut self, output: &mut [u8]) -> std::io::Result<usize> {
        if self.interrupt {
            self.interrupt = false;
            return Err(std::io::ErrorKind::Interrupted.into());
        }
        if self.position >= self.fail_at {
            return Err(std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                "reader failed",
            ));
        }
        if self.position == self.bytes.len() || output.is_empty() {
            return Ok(0);
        }
        output[0] = self.bytes[self.position];
        self.position += 1;
        Ok(1)
    }
}
fn error_signature<T>(
    result: Result<T, facade::Error>,
) -> Result<T, (String, usize, usize, facade::error::Category)> {
    result.map_err(|e| (e.to_string(), e.line(), e.column(), e.classify()))
}
#[test]
fn streaming_reader_error_precedence() {
    for (bytes, fail_at) in [
        (b"!null".as_slice(), 2),
        (b"[1,2]", 99),
        (b"[1,2]", 3),
        (b"[1,", 99),
        (b"null", 4),
    ] {
        let reader = || Reader {
            bytes,
            position: 0,
            fail_at,
            interrupt: true,
        };
        assert_eq!(
            error_signature(facade::from_reader::<_, facade::Value>(reader())),
            error_signature(reference_json::from_reader::<_, reference_json::Value>(
                reader()
            ))
        );
    }
}

struct Writer {
    output: Vec<u8>,
    budget: usize,
    interrupt: bool,
}
impl std::io::Write for Writer {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if self.interrupt {
            self.interrupt = false;
            return Err(std::io::ErrorKind::Interrupted.into());
        }
        if self.output.len() >= self.budget {
            return Err(std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                "writer failed",
            ));
        }
        let count = bytes.len().min(1).min(self.budget - self.output.len());
        self.output.extend_from_slice(&bytes[..count]);
        Ok(count)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
struct Composite<'a>(&'a Cell<usize>);
impl Serialize for Composite<'_> {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeSeq;
        self.0.set(self.0.get() + 1);
        let mut seq = s.serialize_seq(Some(2))?;
        seq.serialize_element("first")?;
        self.0.set(self.0.get() + 1);
        seq.serialize_element(&FailsOnce(Cell::new(0)))?;
        seq.end()
    }
}
#[test]
fn streaming_writer_partial_output_and_callbacks() {
    for pretty in [false, true] {
        for budget in [0, 1, 4, 10, 100] {
            let mut ours = Writer {
                output: Vec::new(),
                budget,
                interrupt: true,
            };
            let mut reference = Writer {
                output: Vec::new(),
                budget,
                interrupt: true,
            };
            let a = Cell::new(0);
            let b = Cell::new(0);
            let candidate = if pretty {
                facade::to_writer_pretty(&mut ours, &Composite(&a))
            } else {
                facade::to_writer(&mut ours, &Composite(&a))
            };
            let oracle = if pretty {
                reference_json::to_writer_pretty(&mut reference, &Composite(&b))
            } else {
                reference_json::to_writer(&mut reference, &Composite(&b))
            };
            assert_eq!(error_signature(candidate), error_signature(oracle));
            assert_eq!(ours.output, reference.output);
            assert_eq!(a.get(), b.get());
        }
    }
}
