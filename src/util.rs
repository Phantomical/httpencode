use crate::{BufMut, InsufficientSpaceError};
use bytes::Buf;
use core::mem::size_of_val;

// Hack used to refer to the never type in stable rust.
//
// Needed so that the return type of the const_panic macro
// deduces to !.
pub(crate) trait RetTypeHack {
  type Return;
}

impl<T> RetTypeHack for fn() -> T {
  type Return = T;
}

pub(crate) type Never = <fn() -> ! as RetTypeHack>::Return;

type Result<T = (), E = InsufficientSpaceError> = core::result::Result<T, E>;

macro_rules! declare_ext {
  {
    $(
      $( #[$attr:meta] )*
      fn $try_name:ident ($src:ident : $ty:ty) => $name:ident ( $size:expr );
    )*
  } => {
    $(
      $( #[$attr] )*
      #[inline]
      fn $try_name (&mut self, $src : $ty) -> Result {
        if self.remaining_mut() < $size {
          return Err(InsufficientSpaceError::default());
        }

        self.$name($src);
        Ok(())
      }
    )*
  }
}

#[allow(missing_docs)]
pub trait FallibleBufMut: BufMut {
  #[inline]
  fn try_put<T: Buf>(&mut self, src: T) -> Result
  where
    Self: Sized,
  {
    if self.remaining_mut() < src.remaining() {
      return Err(InsufficientSpaceError::default());
    }

    self.put(src);
    Ok(())
  }

  #[inline]
  fn try_put_uint(&mut self, n: u64, nbytes: usize) -> Result {
    if self.remaining_mut() < nbytes {
      return Err(InsufficientSpaceError::default());
    }

    self.put_uint(n, nbytes);
    Ok(())
  }
  #[inline]
  fn try_put_uint_le(&mut self, n: u64, nbytes: usize) -> Result {
    if self.remaining_mut() < nbytes {
      return Err(InsufficientSpaceError::default());
    }

    self.put_uint_le(n, nbytes);
    Ok(())
  }

  #[inline]
  fn try_put_int(&mut self, n: i64, nbytes: usize) -> Result {
    if self.remaining_mut() < nbytes {
      return Err(InsufficientSpaceError::default());
    }

    self.put_int(n, nbytes);
    Ok(())
  }
  #[inline]
  fn try_put_int_le(&mut self, n: i64, nbytes: usize) -> Result {
    if self.remaining_mut() < nbytes {
      return Err(InsufficientSpaceError::default());
    }

    self.put_int_le(n, nbytes);
    Ok(())
  }

  declare_ext! {
    fn try_put_slice(src: &[u8]) => put_slice(src.len());

    fn try_put_u8(n: u8)        => put_u8(size_of_val(&n));
    fn try_put_i8(n: i8)        => put_i8(size_of_val(&n));
    fn try_put_u16(n: u16)      => put_u16(size_of_val(&n));
    fn try_put_u16_le(n: u16)   => put_u16_le(size_of_val(&n));
    fn try_put_i16(n: i16)      => put_i16(size_of_val(&n));
    fn try_put_i16_le(n: i16)   => put_i16_le(size_of_val(&n));
    fn try_put_u32(n: u32)      => put_u32(size_of_val(&n));
    fn try_put_u32_le(n: u32)   => put_u32_le(size_of_val(&n));
    fn try_put_i32(n: i32)      => put_i32(size_of_val(&n));
    fn try_put_i32_le(n: i32)   => put_i32_le(size_of_val(&n));
    fn try_put_u64(n: u64)      => put_u64(size_of_val(&n));
    fn try_put_u64_le(n: u64)   => put_u64_le(size_of_val(&n));
    fn try_put_i64(n: i64)      => put_i64(size_of_val(&n));
    fn try_put_i64_le(n: i64)   => put_i64_le(size_of_val(&n));
    fn try_put_u128(n: u128)    => put_u128(size_of_val(&n));
    fn try_put_u128_le(n: u128) => put_u128_le(size_of_val(&n));
    fn try_put_i128(n: i128)    => put_i128(size_of_val(&n));
    fn try_put_i128_le(n: i128) => put_i128_le(size_of_val(&n));

    fn try_put_f32(n: f32)      => put_f32(size_of_val(&n));
    fn try_put_f32_le(n: f32)   => put_f32_le(size_of_val(&n));
    fn try_put_f64(n: f64)      => put_f64(size_of_val(&n));
    fn try_put_f64_le(n: f64)   => put_f64_le(size_of_val(&n));
  }
}

impl<B: BufMut> FallibleBufMut for B {}

pub(crate) const fn ilog10(mut x: u128) -> usize {
  let mut result = 0;

  if x == 0 {
    return 1;
  }

  while x != 0 {
    result += 1;
    x /= 10;
  }

  result
}

#[cfg(test)]
mod tests {
  use super::*;

  macro_rules! ilog10_tests {
    {
      $(
        $expected:expr => [ $( $test:expr ),* ]
      ),* $(,)?
    } => {
      $(
        $(
          assert_eq!(ilog10($test), $expected);
        )*
      )*
    }
  }

  #[test]
  fn ilog10_success() {
    ilog10_tests! {
      1 => [0, 1, 9],
      2 => [10, 99],
      3 => [100, 999],
      4 => [1000, 9999],
      5 => [10000, 99999],
      6 => [100000, 999999],
      7 => [1000000, 9999999],
      8 => [10000000, 99999999],
      9 => [100000000, 999999999],
      10=> [1000000000, 9999999999],
      11=> [10000000000, 99999999999],
      12=> [100000000000, 999999999999],
      13=> [1000000000000, 9999999999999],
      14=> [10000000000000, 99999999999999],
      15=> [100000000000000, 999999999999999],
      16=> [1000000000000000, 9999999999999999],
      17=> [10000000000000000, 99999999999999999],
      18=> [100000000000000000, 999999999999999999],
      19=> [1000000000000000000, 9999999999999999999],
    }
  }

  #[test]
  fn u8_buffer_too_short() {
    use crate::HttpWriteable;

    let val = b' ';
    let mut buffer = vec![];

    assert!(val.write_to(&mut &mut buffer[..]).is_err());
  }
}
