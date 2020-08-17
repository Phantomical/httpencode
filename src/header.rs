use crate::{
  is_token, BufMut, FallibleBufMut, HttpWriteable, InsufficientSpaceError,
  InvalidHeaderError, CRLF,
};

/// Field name wrapper allowing a field to be checked for validity at
/// compile time.
///
/// # Example
/// ```
/// # use httpencode::*;
///
/// const CONTENT_TYPE: CheckedField = CheckedField::new("Content-Type");
///
/// let header = Header::checked_new(CONTENT_TYPE, "text/plain");
/// ```
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct CheckedField<'data>(&'data str);

impl<'data> CheckedField<'data> {
  /// Create a `CheckedField` from the provided field name.
  ///
  /// # Errors
  /// Returns an error if `name` is not a valid HTTP header field name.
  /// See the docs for
  /// [`InvalidHeaderError`](crate::InvalidHeaderError)
  /// for details.
  pub const fn try_new(name: &'data str) -> Result<Self, InvalidHeaderError> {
    if !is_token(name) {
      return Err(InvalidHeaderError(()));
    }

    Ok(Self(name))
  }

  /// Create a `CheckedField` from the given field name.
  ///
  /// # Panics
  /// Panics if `name` is not a valid field name for an HTTP header.
  /// See the docs for
  /// [`InvalidHeaderError`](crate::InvalidHeaderError)
  /// for details.
  pub const fn new(name: &'data str) -> Self {
    match Self::try_new(name) {
      Ok(field) => field,
      Err(_) => const_panic!("Invalid HTTP header field name"),
    }
  }

  /// Get this `CheckedField` instance as a string.
  pub const fn as_str(&self) -> &'data str {
    self.0
  }
}

/// Pre-checked HTTP field value.
///
/// This is useful for cases where you want to avoid the overhead of
/// inserting spaces after invalid CRLF sequences that is done when using a
/// string or byte-slice as a header value.
///
/// If you know that your header value is valid or want to run the checks
/// in advance (or at compile time) then you'll want to use this type.
///
/// # Example
/// ```
/// # use httpencode::*;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // We can create the header as a constant
/// const ACCEPT_HEADER: Header<CheckedValue> = Header::new(
///   "Accept",
///   CheckedValue::new(b"text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8")
/// );
///
/// let mut req = response(vec![], Version::HTTP_1_1, Status::OK)?;
///
/// // So now this part is just a byte-level copy with no validation required
/// // (except checking that it fits in the destination buffer).
/// req.header(ACCEPT_HEADER)?;
///
/// # Ok(())
/// # }
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct CheckedValue<'data>(&'data [u8]);

impl<'data> CheckedValue<'data> {
  /// Create a `CheckedValue` from the provided byte slice.
  ///
  /// # Errors
  /// Returns an error if `value` contains a CRLF not immediately
  /// followed by linear whitespace (`' '` or `'\t'`).
  pub const fn try_new(value: &'data [u8]) -> Result<Self, InvalidHeaderError> {
    if !Self::check_valid_const(value) {
      return Err(InvalidHeaderError(()));
    }

    Ok(Self(value))
  }

  /// Create a `CheckedValue` from the provided byte slice.
  ///
  /// # Panics
  /// Panics if `value` contains a CRLF not immediately followed by
  /// linear whitespace (`' '` or `'\t'`).
  pub const fn new(value: &'data [u8]) -> Self {
    match Self::try_new(value) {
      Ok(value) => value,
      Err(_) => const_panic!("Header contained invalid character"),
    }
  }

  /// Create a `CheckedValue` without checking to see that `value` meets
  /// the requirements for a valid HTTP header value.
  ///
  /// # Safety
  /// Breaking the requirements of this function won't cause memory
  /// unsafety. However, if `value` contains a CRLF not immediately
  /// followed by linear whitespace (`' '` or `'\t'`) then any HTTP
  /// headers emitted using this value will not be syntactically valid.
  pub const unsafe fn new_unchecked(value: &'data [u8]) -> Self {
    Self(value)
  }

  /// Access the underlying byte slice of of this value.
  pub const fn as_bytes(&self) -> &'data [u8] {
    self.0
  }

  const fn check_valid_const(value: &[u8]) -> bool {
    let mut prev = 0;
    while let Some(idx) = Self::memchr_const(b'\r', value, prev) {
      prev = match value.len() - idx {
        0 | 1 => break,
        2 => match value[1] {
          b'\n' => return false,
          _ => break,
        },
        _ => match (value[1], value[2]) {
          (b'\n', b' ') | (b'\n', b'\t') => 3,
          (b'\n', _) => return false,
          _ => 1,
        },
      } + idx;
    }

    true
  }

  const fn memchr_const(
    needle: u8,
    haystack: &[u8],
    start: usize,
  ) -> Option<usize> {
    let mut idx = start;

    while idx < haystack.len() {
      if haystack[idx] == needle {
        return Some(idx);
      }

      idx += 1;
    }

    None
  }
}

impl HttpWriteable for CheckedValue<'_> {
  fn write_to<B: BufMut>(
    &self,
    buffer: &mut B,
  ) -> Result<(), InsufficientSpaceError> {
    buffer.try_put_slice(self.0)
  }
}

/// A key-value pair representing an HTTP header.
///
/// # Example
/// ```
/// # use httpencode::*;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let mut req = response(vec![], Version::HTTP_1_1, Status::new(200))?;
/// req.header(Header::new("Content-Type", "text/plain"))?;
/// # Ok(())
/// # }
/// ```
#[derive(Copy, Clone, Debug)]
pub struct Header<'data, V> {
  pub(crate) field: CheckedField<'data>,
  pub(crate) value: V,
}

impl<'data, V> Header<'data, V> {
  /// Create a new header with the provided field name and value.
  ///
  /// # Panics
  /// Panics if `field` is not a valid HTTP header field name.
  /// See the docs for
  /// [`InvalidHeaderError`](crate::InvalidHeaderError)
  /// for details.
  pub const fn new(field: &'data str, value: V) -> Self {
    let field = match CheckedField::try_new(field) {
      Ok(field) => field,
      Err(_) => const_panic!("Header field contained invalid character"),
    };

    Self { field, value }
  }

  /// Create a new header using the provided field name and value.
  pub const fn checked_new(field: CheckedField<'data>, value: V) -> Self {
    Self { field, value }
  }
}

impl<'data, V: HttpWriteable> Header<'data, V> {
  /// Create a new header using the provided field name and value.
  ///
  /// # Errors
  /// Returns an error if `field` is not a valid HTTP header field.
  /// See the docs for
  /// [`InvalidHeaderError`](crate::InvalidHeaderError)
  /// for details.
  pub fn try_new(
    field: &'data str,
    value: V,
  ) -> Result<Self, InvalidHeaderError> {
    let field = match CheckedField::try_new(field) {
      Ok(field) => field,
      Err(e) => return Err(e),
    };

    Ok(Self { field, value })
  }

  pub(crate) fn write_to<B: BufMut>(
    &self,
    buf: &mut B,
  ) -> Result<(), InsufficientSpaceError> {
    buf.try_put_slice(self.field.as_str().as_bytes())?;
    buf.try_put_slice(b": ")?;
    self.value.write_to(buf)?;
    buf.try_put_slice(&CRLF)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn checked_field_new_valid() {
    let _ = CheckedField::new("Content-Type");
  }

  #[test]
  #[should_panic]
  fn checked_field_new_invalid() {
    let _ = CheckedField::new("Spaced Header");
  }

  macro_rules! checked_field_invalid {
    {
      $( $name:ident => $value:literal; )*
    } => {
      mod invalid_checked_field {
        use super::*;

        $(
          #[test]
          #[should_panic]
          fn $name() {
            let _ = CheckedField::new($value);
          }
        )*
      }
    }
  }

  checked_field_invalid! {
    contains_crlf       => "Has\r\nNewline";
    contains_del        => "Has\x7FDEL";
    contains_spaces     => "Inner    Spaces";
    contains_high_byte  => "HI\u{0080}BYTE";

    empty => "";
  }

  macro_rules! checked_value_valid {
    {
      $( $name:ident => $value:literal; )*
    } => {
      mod invalid_checked_value {
        use super::*;

        $(
          #[test]
          fn $name() {
            let v1 = CheckedValue::new($value);
            let v2 = unsafe { CheckedValue::new_unchecked($value) };

            assert!(v1 == v2);
            assert!(v1.as_bytes() == $value);
          }
        )*
      }
    }
  }

  #[test]
  fn checked_value_invalid() {
    assert!(CheckedValue::try_new(b"\r\n").is_err());
    assert!(CheckedValue::try_new(b"\r\na").is_err());
    assert!(CheckedValue::try_new(b"\r\n\r\n").is_err());
  }

  #[test]
  #[should_panic]
  fn checked_value_invalid_crlf() {
    let _ = CheckedValue::new(b"\r\n");
  }

  checked_value_valid! {
    contains_nul  => b"\0";
    empty         => b"";
    invalid_utf8  => b"\xff\x00";
    crlf_w_space  => b"\r\n ";
    crlf_w_tab    => b"\r\n\t";
    multispace    => b"spaced     out";
    cr            => b"\r";
    cr_space      => b"\r ";
    cr_space_a    => b"\r a";
  }
}
