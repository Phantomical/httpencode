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

  const fn make_unchecked(method: &'data str) -> Self {
    Self { method }
  }

  /// Get the string representation of this `Method`.
  pub const fn as_str(&self) -> &'data str {
    self.method
  }

  /// HTTP GET.
  pub const GET: Self = Self::make_unchecked("GET");
  /// HTTP HEAD.
  pub const HEAD: Self = Self::make_unchecked("HEAD");
  /// HTTP POST.
  pub const POST: Self = Self::make_unchecked("POST");
  /// HTTP PUT.
  pub const PUT: Self = Self::make_unchecked("PUT");
  /// HTTP DELETE.
  pub const DELETE: Self = Self::make_unchecked("DELETE");
  /// HTTP CONNECT.
  pub const CONNNECT: Self = Self::make_unchecked("CONNECT");
  /// HTTP OPTIONS.
  pub const OPTIONS: Self = Self::make_unchecked("OPTIONS");
  /// HTTP TRACE.
  pub const TRACE: Self = Self::make_unchecked("TRACE");

  /// Create a custom method from a method string.
  ///
  /// # Errors
  /// Errors if the method is not a sytactically valid
  /// method (Method must be a token as per RFC 7320).
  pub const fn try_new(method: &'data str) -> Result<Self, InvalidMethodError> {
    if !is_token(method) {
      return Err(InvalidMethodError(()));
    }

    Ok(Self::make_unchecked(method))
  }

  /// Create a custom method from a method string.
  ///
  /// # Panics
  /// Panics if the method string is not a syntactically valid method token.
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
  pub const unsafe fn new_unchecked(method: &'data str) -> Self {
    Self::make_unchecked(method)
  }
}
