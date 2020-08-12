use crate::{util::ilog10, BufMut, FallibleBufMut, InsufficientSpaceError};

/// Types that can safely be written out as an http value.
///
/// Implementors of this trait should ensure that the data
/// they write to the buffer cannot be used to inject unexpected
/// headers into the buffer. That is, any instances of `\r\n`
/// that occur outside of quotes should be followed by at least
/// one linear whitespace (`' '` or `'\n'`).
///
/// As an example, encoding of `&[u8]` will insert a `\t` after any
/// `\r\n` instance that doesn't occur within a quoted region.
pub trait HttpWriteable {
  /// Write out the representation of this type to the buffer.
  ///
  /// If there is not enough room within the buffer this method
  /// should return an error instead of panicking.
  ///
  /// # Note for Implementors
  /// The methods on the extension trait
  /// [`FallibleBufMut`](crate::util::FallibleBufMut) should be
  /// helpful when implementing this method.
  fn write_to<B: BufMut>(
    &self,
    buffer: &mut B,
  ) -> Result<(), InsufficientSpaceError>;
}

fn reverse<T>(range: &mut [T]) {
  if range.len() < 2 {
    return;
  }

  let (head, tail) = range.split_at_mut(range.len() / 2);
  let iter = head.iter_mut().zip(tail.iter_mut().rev());

  for (a, b) in iter {
    core::mem::swap(a, b)
  }
}

fn find_unquoted_crlf(bytes: &[u8]) -> UnquotedCRLFIterator {
  UnquotedCRLFIterator {
    bytes,
    inquotes: false,
    offset: 0,
  }
}

struct UnquotedCRLFIterator<'a> {
  bytes: &'a [u8],
  inquotes: bool,
  offset: usize,
}

impl UnquotedCRLFIterator<'_> {
  fn find_next(&mut self) -> Option<usize> {
    memchr::memchr3(b'\\', b'"', b'\r', &self.bytes[self.offset..])
      .map(|pos| pos + self.offset)
  }

  fn advance_to(&mut self, amount: usize) {
    self.offset = amount.min(self.bytes.len());
  }
}

impl<'a> Iterator for UnquotedCRLFIterator<'a> {
  type Item = usize;

  fn next(&mut self) -> Option<Self::Item> {
    while let Some(mut pos) = self.find_next() {
      match &self.bytes[pos..] {
        [b'\\', ..] => pos += 1,
        [b'"', ..] => self.inquotes = !self.inquotes,
        [b'\r', b'\n', ..] if !self.inquotes => {
          self.advance_to(pos + 2);
          return Some(pos);
        }
        [b'\r', ..] => (),
        _ => unreachable!(),
      }

      self.advance_to(pos + 1);
    }

    None
  }
}

macro_rules! writable_unsigned {
  ($ty:ident) => {
    impl HttpWriteable for $ty {
      fn write_to<B: BufMut>(
        &self,
        buffer: &mut B,
      ) -> Result<(), InsufficientSpaceError> {
        let mut bytes = [0u8; ilog10(Self::MAX as u128)];
        let mut value = *self;

        if value == 0 {
          return buffer.try_put_u8(b'0');
        }

        let mut i = 0;
        while value != 0 {
          let rem = value % 10;
          value /= 10;
          bytes[i] = b'0' + (rem as u8);
          i += 1;
        }

        reverse(&mut bytes[..i]);

        buffer.try_put_slice(&bytes[..i])
      }
    }
  };
}

writable_unsigned!(u8);
writable_unsigned!(u16);
writable_unsigned!(u32);
writable_unsigned!(u64);
writable_unsigned!(u128);
writable_unsigned!(usize);

macro_rules! writable_signed {
  ($sty:ident, $uty:ident) => {
    impl HttpWriteable for $sty {
      fn write_to<B: BufMut>(
        &self,
        buffer: &mut B,
      ) -> Result<(), InsufficientSpaceError> {
        let mut value = *self as $uty;

        if *self < 0 {
          buffer.try_put_u8(b'-')?;
          value = !value + 1;
        }

        value.write_to(buffer)
      }
    }
  };
}

writable_signed!(i8, u8);
writable_signed!(i16, u16);
writable_signed!(i32, u32);
writable_signed!(i64, u64);
writable_signed!(i128, u128);
writable_signed!(isize, usize);

impl HttpWriteable for &'_ [u8] {
  fn write_to<B: BufMut>(
    &self,
    buffer: &mut B,
  ) -> Result<(), InsufficientSpaceError> {
    let data = *self;
    let mut prev = 0;

    for idx in find_unquoted_crlf(data) {
      let temp = &data[idx..];
      match temp {
        [b'\r', b'\n', b' ', ..] | [b'\r', b'\n', b'\t', ..] => (),
        [b'\r', b'\n', ..] => {
          buffer.try_put_slice(&data[prev..idx + 2])?;
          buffer.try_put_u8(b'\t')?;
          prev = idx + 2;
        }
        _ => unreachable!("Unquoted CRLF instance did not start with CRLF"),
      }
    }

    buffer.try_put_slice(&data[prev..data.len()])
  }
}

impl HttpWriteable for &'_ str {
  #[inline]
  fn write_to<B: BufMut>(
    &self,
    buffer: &mut B,
  ) -> Result<(), InsufficientSpaceError> {
    self.as_bytes().write_to(buffer)
  }
}

impl<W> HttpWriteable for &'_ W
where
  W: HttpWriteable,
{
  #[inline]
  fn write_to<B: BufMut>(
    &self,
    buffer: &mut B,
  ) -> Result<(), InsufficientSpaceError> {
    <W as HttpWriteable>::write_to(*self, buffer)
  }
}

#[cfg(feature = "std")]
mod with_std {
  use super::*;
  use std::borrow::Cow;

  impl HttpWriteable for Vec<u8> {
    #[inline]
    fn write_to<B: BufMut>(
      &self,
      buffer: &mut B,
    ) -> Result<(), InsufficientSpaceError> {
      self.as_slice().write_to(buffer)
    }
  }

  impl HttpWriteable for String {
    #[inline]
    fn write_to<B: BufMut>(
      &self,
      buffer: &mut B,
    ) -> Result<(), InsufficientSpaceError> {
      self.as_str().write_to(buffer)
    }
  }

  impl<W> HttpWriteable for Cow<'_, W>
  where
    W: HttpWriteable + Clone,
  {
    #[inline]
    fn write_to<B: BufMut>(
      &self,
      buffer: &mut B,
    ) -> Result<(), InsufficientSpaceError> {
      <W as HttpWriteable>::write_to(&*self, buffer)
    }
  }
}
