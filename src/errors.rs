use crate::{
  InsufficientSpaceError, InvalidHeaderError, InvalidMethodError,
  InvalidUriError,
};

use core::fmt::{Display, Formatter, Result};

impl Display for InvalidHeaderError {
  fn fmt(&self, f: &mut Formatter<'_>) -> Result {
    f.write_str("Header contained invalid character")
  }
}

impl Display for InvalidMethodError {
  fn fmt(&self, f: &mut Formatter<'_>) -> Result {
    f.write_str("Custom HTTP method contained invalid character")
  }
}

impl Display for InvalidUriError {
  fn fmt(&self, f: &mut Formatter<'_>) -> Result {
    f.write_str("URI contained invalid character")
  }
}

impl Display for InsufficientSpaceError {
  fn fmt(&self, f: &mut Formatter<'_>) -> Result {
    f.write_str("Buffer had insufficient space")
  }
}

#[cfg(feature = "std")]
mod with_std {
  use super::*;
  use std::error::Error;

  impl Error for InvalidHeaderError {}
  impl Error for InvalidMethodError {}
  impl Error for InvalidUriError {}
  impl Error for InsufficientSpaceError {}
}
