//! HTTP 1.0/1.1 header encoding library focused on providing fast
//! encoding of HTTP headers without requiring any allocations
//! except those required by the output buffer type. This library
//! is meant to only allow the creation of syntactically valid HTTP
//! requests/responses but it does not perform any semantic checking
//! of the resulting data (e.g. checking that Content-Length is valid.)
//!
//! # Examples
//! Write out a request header:
//! ```
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use httpencode::{Method, Version, Uri, Header};
//!
//! // First we need a buffer to output to.
//! let mut buffer = vec![];
//!
//! let mut request = httpencode::request(
//!     &mut buffer,
//!     Method::GET,
//!     Uri::new(b"/example.html"),
//!     Version::HTTP_1_1
//! )?;
//!
//! // Set our custom user agent
//! request.header(Header::new("User-Agent", "Crustacean/0.1"))?;
//! request.finish()?;
//!
//! // Send request to server here...
//! # Ok(())
//! # }
//! ```
//!
//! Write out a response:
//! ```
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use httpencode::{Method, Version, Status, Header};
//!
//! let body: &[u8] = b"
//! <!DOCTYPE html>
//! <body>
//!   Hello World!
//! </body>
//! ";
//!
//! // First we need a buffer to write to
//! let mut buffer = vec![];
//!
//! let mut request = httpencode::response(
//!     &mut buffer,
//!     Version::HTTP_1_1,
//!     Status::OK
//! )?;
//!
//! request.header(Header::new("Content-Type", "text/html"))?;
//! request.header(Header::new("Content-Length", body.len()))?;
//! request.finish()?;
//!
//! buffer.extend_from_slice(&body);
//!
//! // Send to client here...
//! # Ok(())
//! # }
//! ```

#![cfg_attr(not(feature = "std"), no_std)]
#![warn(missing_docs)]

// There are rust versions where invalid array indexing allows for
// panicking but actual panic! calls in constants are not. For these
// versions of rust we'll default to a bad panic message.
#[cfg(not(has_const_panic))]
macro_rules! const_panic {
  ($message:literal) => {{
    let x: [crate::util::Never; 0] = [];

    #[allow(unconditional_panic)]
    x[$message.len()]
  }};
}

#[cfg(has_const_panic)]
macro_rules! const_panic {
  ($message:literal) => {
    panic!($message)
  };
}

pub use bytes::BufMut;

mod errors;
mod header;
mod integrations;
mod method;
mod status;
mod uri;
mod util;
mod version;
mod writable;

pub use crate::header::{CheckedField, CheckedValue, Header};
pub use crate::method::Method;
pub use crate::status::Status;
pub use crate::uri::Uri;
pub use crate::util::FallibleBufMut;
pub use crate::version::Version;
pub use crate::writable::HttpWriteable;

const CRLF: [u8; 2] = *b"\r\n";

/// A custom HTTP method contained invalid characters.
///
/// Invalid characters are defined according to the token spec in
/// RFC 7230:
/// ```text
/// token = 1*tchar
/// tchar = "!" / "#" / "$" / "%" / "&" / "'" / "*"
///       / "+" / "-" / "." / "^" / "_" / "`" / "|" / "~"
///       / DIGIT / ALPHA
///       ; any VCHAR, except delimiters
/// ```
#[derive(Debug)]
pub struct InvalidMethodError(());

/// A URI contained an invalid character (either ' ', '\r', or '\n').
#[derive(Debug)]
pub struct InvalidUriError(());

/// A header field name contained an invalid character.
///
/// Invalid characters are defined according to the token spec in
/// RFC 7230:
/// ```text
/// token = 1*tchar
/// tchar = "!" / "#" / "$" / "%" / "&" / "'" / "*"
///       / "+" / "-" / "." / "^" / "_" / "`" / "|" / "~"
///       / DIGIT / ALPHA
///       ; any VCHAR, except delimiters
/// ```
#[derive(Debug)]
pub struct InvalidHeaderError(());

/// The target buffer doesn't have enough space to write out the desired data.
#[derive(Default, Debug)]
pub struct InsufficientSpaceError(());

/// Start an HTTP-style request with the given method, uri, and protocol
/// version.
///
/// This method is exactly the same as [`HttpBuilder::request`][0].
///
/// # Example
/// ```
/// # use httpencode::*;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let mut builder = request(
///     vec![],
///     Method::GET,
///     Uri::try_new(b"/")?,
///     Version::HTTP_1_1
/// )?;
/// let output = builder.finish()?;
///
/// assert_eq!(
///   std::str::from_utf8(&output)?,
///   "GET / HTTP/1.1\r\n\r\n"
/// );
/// # Ok(())
/// # }
/// ```
///
/// [0]: crate::HttpBuilder::request
pub fn request<B: BufMut>(
  buffer: B,
  method: Method,
  request_target: Uri,
  version: Version,
) -> Result<HttpBuilder<B>, InsufficientSpaceError> {
  HttpBuilder::request(buffer, method, request_target, version)
}

/// Start an HTTP-style response with the given version and status.
///
/// By default this includes a reason phrase with the status. If the
/// `no-reason-phrase` feature is specified then the reason phrase will
/// be kept blank.
///
/// # Example
/// ```
/// # use httpencode::*;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let mut builder = response(
///     vec![],
///     Version::HTTP_1_0,
///     Status::with_reason(418, "I'm a Teapot")
/// )?;
/// let output = builder.finish()?;
///
/// assert_eq!(
///   output,
///   b"HTTP/1.0 418 I'm a Teapot\r\n\r\n"
/// );
/// # Ok(())
/// # }
/// ```
pub fn response<B: BufMut>(
  buffer: B,
  version: Version,
  status: Status,
) -> Result<HttpBuilder<B>, InsufficientSpaceError> {
  HttpBuilder::response(buffer, version, status)
}

/// Build an HTTP 1.1/1.0-style request or response and write it out to
/// the provided buffer.
pub struct HttpBuilder<B: BufMut> {
  buffer: B,
}

impl<B: BufMut> HttpBuilder<B> {
  /// Start an HTTP-style request with the given method, uri, and protocol
  /// version.
  ///
  /// # Example
  /// ```
  /// # use httpencode::*;
  /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
  /// let mut builder = HttpBuilder::request(
  ///     vec![],
  ///     Method::GET,
  ///     Uri::try_new(b"/")?,
  ///     Version::HTTP_1_1
  /// )?;
  /// let output = builder.finish()?;
  ///
  /// assert_eq!(
  ///   std::str::from_utf8(&output)?,
  ///   "GET / HTTP/1.1\r\n\r\n"
  /// );
  /// # Ok(())
  /// # }
  /// ```
  pub fn request(
    mut buffer: B,
    method: Method,
    request_target: Uri,
    version: Version,
  ) -> Result<Self, InsufficientSpaceError> {
    method.write_to(&mut buffer)?;
    buffer.try_put_u8(b' ')?;
    request_target.write_to(&mut buffer)?;
    buffer.try_put_u8(b' ')?;
    version.write_to(&mut buffer)?;
    buffer.try_put_slice(&CRLF)?;

    Ok(Self { buffer })
  }

  /// Start an HTTP-style response with the given version and status.
  ///
  /// By default this includes a reason phrase with the status. If the
  /// `no-reason-phrase` feature is specified then the reason phrase will
  /// be kept blank.
  ///
  /// # Example
  /// ```
  /// # use httpencode::*;
  /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
  /// let mut builder = HttpBuilder::response(
  ///     vec![],
  ///     Version::HTTP_1_0,
  ///     Status::with_reason(418, "I'm a Teapot")
  /// )?;
  /// let output = builder.finish()?;
  ///
  /// assert_eq!(
  ///   output,
  ///   b"HTTP/1.0 418 I'm a Teapot\r\n\r\n"
  /// );
  /// # Ok(())
  /// # }
  /// ```
  pub fn response(
    mut buffer: B,
    version: Version,
    status: Status,
  ) -> Result<Self, InsufficientSpaceError> {
    version.write_to(&mut buffer)?;
    buffer.try_put_u8(b' ')?;
    status.code().write_to(&mut buffer)?;
    buffer.try_put_u8(b' ')?;
    buffer.try_put_slice(status.reason().as_bytes())?;
    buffer.try_put_slice(&CRLF)?;

    Ok(Self { buffer })
  }

  /// Write out a HTTP header field.
  ///
  /// # Example
  /// ```
  /// # use httpencode::*;
  /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
  /// let mut builder = HttpBuilder::response(
  ///     vec![],
  ///     Version::HTTP_1_1,
  ///     Status::with_reason(418, "I'm a Teapot")
  /// )?;
  /// builder.header(Header::new("Foo", "Bar"))?;
  /// builder.header(Header::new("Content-Type", "text/json"))?;
  /// builder.header(Header::new("Content-Length", 0))?;
  /// let output = builder.finish()?;
  ///
  /// assert_eq!(
  ///   std::str::from_utf8(&output)?,
  ///   "HTTP/1.1 418 I'm a Teapot\r\n\
  ///   Foo: Bar\r\n\
  ///   Content-Type: text/json\r\n\
  ///   Content-Length: 0\r\n\
  ///   \r\n"
  /// );
  /// # Ok(())
  /// # }
  /// ```
  pub fn header<'data, V: HttpWriteable, H: Into<Header<'data, V>>>(
    &mut self,
    header: H,
  ) -> Result<&mut Self, InsufficientSpaceError> {
    header.into().write_to(&mut self.buffer)?;
    Ok(self)
  }

  /// Finish off the HTTP header and return the `BufMut` instance that
  /// was being written to.
  ///
  /// The client can then write the HTTP body directly into the buffer,
  /// if desired.
  pub fn finish(mut self) -> Result<B, InsufficientSpaceError> {
    self.buffer.try_put_slice(&CRLF)?;
    Ok(self.buffer)
  }

  /// Construct an HttpBuilder from an existing stream without writing
  /// a request line or a status line.
  ///
  /// # Example
  /// ```
  /// # use httpencode::*;
  /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
  /// // Say we need a custom non-standard request line for some reason.
  /// let buffer = (&b"GET /example MY_OWN_PROTOCOL\r\n"[..]).to_owned();
  /// let mut builder = HttpBuilder::from_buffer(buffer);
  /// builder.header(Header::new("Foo", "Bar"))?;
  /// let output = builder.finish()?;
  ///
  /// assert_eq!(
  ///     output,
  ///     b"GET /example MY_OWN_PROTOCOL\r\n\
  ///     Foo: Bar\r\n\
  ///     \r\n"
  /// );
  /// # Ok(())
  /// # }
  /// ```
  pub fn from_buffer(buffer: B) -> Self {
    Self { buffer }
  }

  /// Return the existing buffer without adding the extra blank line
  /// required to terminate the HTTP header section.
  ///
  /// This can be used in combination with `from_buffer` to inject
  /// custom data into the middle of an HTTP request/response.
  pub fn into_inner(self) -> B {
    self.buffer
  }
}

const fn is_token(token: &str) -> bool {
  // According to RFC 7230 this is the valid set of chars in a token.
  //
  // token = 1*tchar
  // tchar = "!" / "#" / "$" / "%" / "&" / "'" / "*"
  //       / "+" / "-" / "." / "^" / "_" / "`" / "|" / "~"
  //       / DIGIT / ALPHA
  //       ; any VCHAR, except delimiters
  const fn is_allowed(byte: u8) -> bool {
    const MASK: u128 = 0x57FFFFFFC7FFFFFE03FF2CFA00000000u128;
    const MASKLO: u64 = MASK as u64;
    const MASKHI: u64 = (MASK >> 64) as u64;

    match byte {
      0..=63 => (MASKLO >> byte) & 1 == 1,
      64..=127 => (MASKHI >> (byte & 63)) & 1 == 1,
      _ => false,
    }
  }

  let mut i = 0;
  let bytes = token.as_bytes();
  while i < bytes.len() {
    if !is_allowed(bytes[i]) {
      return false;
    }
    i += 1;
  }

  !bytes.is_empty()
}

/// Validates that the uri doesn't contain space, CR, or LF
const fn validate_uri(uri: &[u8]) -> bool {
  let mut i = 0;
  while i < uri.len() {
    match uri[i] {
      b' ' | b'\r' | b'\n' => return false,
      _ => i += 1,
    }
  }

  !uri.is_empty()
}
