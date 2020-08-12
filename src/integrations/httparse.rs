use crate::{CheckedField, CheckedValue, Header};

impl<'a> From<httparse::Header<'a>> for Header<'a, CheckedValue<'a>> {
  fn from(header: httparse::Header<'a>) -> Self {
    // The only case where these can fire is if someone is manually creating
    // httparse Header instances so in this case the conversions are acceptable.
    let name = CheckedField::try_new(header.name)
      .expect("Unable to parse HTTP header name");
    let value = CheckedValue::try_new(header.value)
      .expect("Unable to parse HTTP header value");

    Header::checked_new(name, value)
  }
}
