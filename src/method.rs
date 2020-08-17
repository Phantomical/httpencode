use crate::{
  is_token, BufMut, FallibleBufMut, InsufficientSpaceError, InvalidMethodError,
};

/// HTTP Method.
///
/// Unless you want to create non-standard methods for a request
/// then most users of this class should just use the method
/// constants provided.
#[derive(Copy, Clone, Debug)]
pub struct Method<'data> {
  method: &'data str,
}

impl<'data> Method<'data> {
  pub(crate) fn write_to<B: BufMut>(
    &self,
    buffer: &mut B,
  ) -> Result<(), InsufficientSpaceError> {
    buffer.try_put_slice(self.method.as_bytes())
  }

  /// Get the string representation of this `Method`.
  #[inline]
  pub const fn as_str(&self) -> &'data str {
    self.method
  }

  /// HTTP GET.
  pub const GET: Self = Self::new("GET");
  /// HTTP HEAD.
  pub const HEAD: Self = Self::new("HEAD");
  /// HTTP POST.
  pub const POST: Self = Self::new("POST");
  /// HTTP PUT.
  pub const PUT: Self = Self::new("PUT");
  /// HTTP DELETE.
  pub const DELETE: Self = Self::new("DELETE");
  /// HTTP CONNECT.
  pub const CONNNECT: Self = Self::new("CONNECT");
  /// HTTP OPTIONS.
  pub const OPTIONS: Self = Self::new("OPTIONS");
  /// HTTP TRACE.
  pub const TRACE: Self = Self::new("TRACE");

  /// Create a custom method from a method string.
  ///
  /// # Errors
  /// Errors if the method is not a sytactically valid
  /// method (Method must be a token as per RFC 7320).
  #[inline]
  pub const fn try_new(method: &'data str) -> Result<Self, InvalidMethodError> {
    if !is_token(method) {
      return Err(InvalidMethodError(()));
    }

    Ok(Self { method })
  }

  /// Create a custom method from a method string.
  ///
  /// # Panics
  /// Panics if the method string is not a syntactically valid method token.
  #[inline]
  pub const fn new(method: &'data str) -> Self {
    match Self::try_new(method) {
      Ok(m) => m,
      Err(_) => const_panic!("Invalid custom method"),
    }
  }

  /// Create a custom method without validating it.
  ///
  /// # Safety
  /// If this function is used to create a syntactically invalid method
  /// then it can be used to create an HTTP request with invalid syntax.
  #[inline]
  pub const unsafe fn new_unchecked(method: &'data str) -> Self {
    Self { method }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn method_roundtrip() {
    let method = Method::new("FOO");

    assert_eq!(method.as_str(), "FOO");
  }

  #[test]
  fn method_new_unchecked_roundtrip() {
    // Ok as long as it doesn't go into a request.
    let method = unsafe { Method::new_unchecked(" ") };
    assert_eq!(method.as_str(), " ");
  }

  macro_rules! invalid_method {
    {
      $( $name:ident => $value:literal; )*
    } => {
      mod invalid_method {
        use super::*;

        $(
          #[test]
          #[should_panic]
          fn $name() {
            let _ = Method::new($value);
          }
        )*
      }
    }
  }

  invalid_method! {
    empty             => "";
    contains_hi_byte  => "\u{0080}";
    contains_space    => " ";
    contains_del      => "\x7F";
    contains_nul      => "\0";
    contains_crlf     => "\r\n";
  }
}
