use crate::{BufMut, FallibleBufMut, HttpWriteable, InsufficientSpaceError};

/// HTTP Version.
///
/// This type represents version identifiers of the form
/// `HTTP/<major>.<minor>`. Usually the only parts that most
/// code should be interacting with is the `HTTP_1_0` and `HTTP_1_1`
/// constants.
///
/// Users wanting a custom version identifier should build the
/// first line of the request themselves and then constuct the
/// [`HttpBuilder`](crate::HttpBuilder) themselves using
/// [`HttpBuilder::from_buffer`](crate::HttpBuilder::from_buffer).
#[derive(Copy, Clone, Debug)]
pub struct Version<'data> {
  proto: &'data str,
  major: u8,
  minor: u8,
}

impl<'data> Version<'data> {
  pub(crate) fn write_to<B: BufMut>(
    &self,
    buffer: &mut B,
  ) -> Result<(), InsufficientSpaceError> {
    buffer.try_put_slice(self.proto.as_bytes())?;
    buffer.try_put_u8(b'/')?;
    self.major.write_to(buffer)?;
    buffer.try_put_u8(b'.')?;
    self.minor.write_to(buffer)
  }

  /// Create a HTTP version with the given major and minor version
  /// numbers. When serialized, produces `HTTP/<major>.<minor>`.
  pub const fn http(major: u8, minor: u8) -> Self {
    Self {
      major,
      minor,
      proto: "HTTP",
    }
  }

  /// `HTTP/1.0` version identifier.
  pub const HTTP_1_0: Self = Self::http(1, 0);
  /// `HTTP/1.1` version identifier.
  pub const HTTP_1_1: Self = Self::http(1, 1);

  // No point exposing this for now as it's always "HTTP"
  // pub const fn proto(&self) -> &'data str {
  //   self.proto
  // }

  /// Major version component of this `Version`.
  pub const fn major(&self) -> u8 {
    self.major
  }
  /// Minor version component of this `Version`.
  pub const fn minor(&self) -> u8 {
    self.minor
  }
}
