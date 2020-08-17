use crate::{
  validate_uri, BufMut, FallibleBufMut, InsufficientSpaceError, InvalidUriError,
};

/// The URI component of an HTTP request.
///
/// This type does just enough validation to ensure that it will produce
/// a parseable HTTP header. In effect, the only validation is that this
/// URI does not contain any of `' '`, `'\r'`, or `'\n'`
///
/// For general use `new` and `try_new` should be preferred as they are
/// faster at performing validation but `new_const` and `try_new_const`
/// are provided for use in const contexts.
///
/// # Example
/// ```
/// # use httpencode::*;
/// let _ = Uri::try_new(b"https://example.com").unwrap(); // Works
/// let _ = Uri::try_new(b"/test#anchor").unwrap();        // Works
/// let _ = Uri::try_new(b"/\xFF").unwrap();               // Works
///
/// let _ = Uri::try_new(b"/uri with spaces").unwrap_err(); // Not allowed
/// let _ = Uri::try_new(b"/uri\nnewline").unwrap_err();    // Not allowed
/// let _ = Uri::try_new(b"/uri\rlinefeed").unwrap_err();   // Not allowed
/// ```
#[derive(Copy, Clone, Debug)]
pub struct Uri<'data> {
  uri: &'data [u8],
}

impl<'data> Uri<'data> {
  pub(crate) fn write_to<B: BufMut>(
    &self,
    buffer: &mut B,
  ) -> Result<(), InsufficientSpaceError> {
    buffer.try_put_slice(self.uri)
  }

  /// Create a `Uri` instance with the provided byte string.
  ///
  /// # Panics
  /// Panics if `uri` contains any invalid characters.
  pub fn new(uri: &'data [u8]) -> Self {
    match Self::try_new(uri) {
      Ok(uri) => uri,
      Err(_) => panic!("URI contained invalid character"),
    }
  }

  /// Create a `Uri` instance with the provided byte string.
  ///
  /// # Errors
  /// Returns an error if `uri` contains any invalid characters.
  pub fn try_new(uri: &'data [u8]) -> Result<Self, InvalidUriError> {
    let is_valid =
      !uri.is_empty() && memchr::memchr3(b' ', b'\r', b'\n', uri).is_none();

    if !is_valid {
      return Err(InvalidUriError(()));
    }

    Ok(Self { uri })
  }

  /// Create a `Uri` instance with the provided byte string without
  /// checking for validity.
  ///
  /// # Safety
  /// If `uri` contains any invalid characters then any HTTP request
  /// or response constructed using that URI will have invalid syntax.
  pub const unsafe fn new_unchecked(uri: &'data [u8]) -> Self {
    Self { uri }
  }

  /// Create a `Uri` instance with the provided byte string.
  ///
  /// If this method is not being used in a const context then `try_new`
  /// should be preferred as it will likely be faster.
  ///
  /// # Errors
  /// Returns an error if `uri` contains any invalid characters.
  pub const fn try_new_const(
    uri: &'data [u8],
  ) -> Result<Self, InvalidUriError> {
    if !validate_uri(uri) {
      return Err(InvalidUriError(()));
    }

    Ok(Self { uri })
  }

  /// Create a `Uri` instance with the provided byte string.
  ///
  /// If this method is not being used in a const context then `new`
  /// should be preferred as it will likely be faster.
  ///
  /// # Panics
  /// Panics if `uri` contains any invalid characters.
  pub const fn new_const(uri: &'data [u8]) -> Self {
    match Self::try_new_const(uri) {
      Ok(uri) => uri,
      Err(_) => const_panic!("URI contained invalid character"),
    }
  }

  /// Get the contents of this URI as a byte slice.
  pub const fn as_bytes(&self) -> &'data [u8] {
    self.uri
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn uri_round_trip() {
    let uri = Uri::new(b"/test.html");
    assert_eq!(uri.as_bytes(), b"/test.html");

    let bytes: &[u8] = b"/const?ababababba#const.fn";
    let const_uri = Uri::new_const(bytes);
    assert_eq!(const_uri.as_bytes(), bytes);
  }

  #[test]
  fn uri_new_unchecked_round_trip() {
    // Ok as long as it never goes into a request.
    let uri = unsafe { Uri::new_unchecked(b" ") };
    assert_eq!(uri.as_bytes(), b" ");
  }

  macro_rules! uri_invalid {
    {
      $( $name:ident => $value:literal; )*
    } => {
      mod invalid_uri {
        use super::*;

        $(
          #[test]
          #[should_panic]
          fn $name() {
            let _ = Uri::new($value);
          }
        )*

        mod const_ {
          use super::*;

          $(
            #[test]
            #[should_panic]
            fn $name() {
              let _ = Uri::new_const($value);
            }
          )*
        }
      }
    }
  }

  uri_invalid! {
    empty       => b"";
    only_space  => b" ";
    only_crlf   => b"\r\n";

    contains_space => b"https://contains space.com/";
    contains_crlf  => b"contains\r\newline.com/example";
    contains_cr    => b"has\rCR";
    contains_lf    => b"has\nLF";
  }
}
