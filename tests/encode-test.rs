use httpencode::*;
use std::error::Error;

fn escape(bytes: &[u8]) -> String {
  bytes
    .iter()
    .copied()
    .flat_map(std::ascii::escape_default)
    .map(|x| x as char)
    .collect()
}

fn write<W: HttpWriteable>(val: W) -> String {
  let mut buf = vec![];
  val.write_to(&mut buf).unwrap();
  escape(&buf)
}

#[test]
fn test_writable_integer() {
  assert_eq!(write(-10i8), "-10");
  assert_eq!(write(-100000000000000000000i128), "-100000000000000000000");

  assert_eq!(write(-0i8), "0");
  assert_eq!(write(0u8), "0");
}

#[test]
fn quoted_crlf_field() -> Result<(), Box<dyn Error>> {
  let mut req = HttpBuilder::request(
    vec![],
    Method::GET,
    Uri::new(b"/"),
    Version::HTTP_1_1,
  )?;

  req.header(Header::new(
    "Quoted",
    "\"Quoted with newline\r\nand another line\" but also \r\none outside",
  ))?;

  let output = req.finish()?;

  assert_eq!(
    std::str::from_utf8(&output)?,
    "GET / HTTP/1.1\r\n\
    Quoted: \"Quoted with newline\r\n\
    and another line\" but also \r\n\
    \tone outside\r\n\
    \r\n"
  );

  Ok(())
}

#[test]
fn crlf_followed_by_space() -> Result<(), Box<dyn Error>> {
  let mut builder =
    HttpBuilder::response(vec![], Version::HTTP_1_0, Status::new(200))?;

  builder.header(Header::new(
    "Includes-Space",
    "Multiline string\r\n with space after CRLF",
  ))?;

  let output = builder.finish()?;

  assert_eq!(
    std::str::from_utf8(&output)?,
    "HTTP/1.0 200 OK\r\n\
    Includes-Space: Multiline string\r\n with space after CRLF\r\n\
    \r\n"
  );

  Ok(())
}

#[test]
fn empty_uri_not_ok() {
  assert!(Uri::try_new(b"").is_err());
  assert!(Uri::try_new_const(b"").is_err());
}

#[test]
fn empty_header_field_not_ok() {
  assert!(Header::try_new("", "Blah").is_err());
}

#[test]
fn empty_header_value_ok() {
  assert!(Header::try_new("Foo", "").is_ok());
}

// This test is lifted from the inverse test within httparse
#[test]
fn large_request() -> Result<(), Box<dyn Error>> {
  const URI: Uri = Uri::new_const(
    b"/wp-content/uploads/2010/03/hello-kitty-darth-vader-pink.jpg",
  );

  const HEADERS: &[Header<&str>] = &[
      Header::new("Host",             "www.kittyhell.com"),
      Header::new("User-Agent",       "Mozilla/5.0 (Macintosh; U; Intel Mac OS X 10.6; ja-JP-mac; rv:1.9.2.3) Gecko/20100401 Firefox/3.6.3 Pathtraq/0.9"),
      Header::new("Accept",           "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8"),
      Header::new("Accept-Language",  "ja,en-us;q=0.7,en;q=0.3"),
      Header::new("Accept-Encoding",  "gzip,deflate"),
      Header::new("Accept-Charset",   "Shift_JIS,utf-8;q=0.7,*;q=0.7"),
      Header::new("Keep-Alive",       "115"),
      Header::new("Connection",       "keep-alive"),
      Header::new("Cookie",           "wp_ozh_wsa_visits=2; wp_ozh_wsa_visit_lasttime=xxxxxxxxxx; __utma=xxxxxxxxx.xxxxxxxxxx.xxxxxxxxxx.xxxxxxxxxx.xxxxxxxxxx.x; __utmz=xxxxxxxxx.xxxxxxxxxx.x.x.utmccn=(referral)|utmcsr=reader.livedoor.com|utmcct=/reader/|utmcmd=referral|padding=under256")
    ];

  let mut request =
    httpencode::request(vec![], Method::GET, URI, Version::HTTP_1_1)?;

  for header in HEADERS.iter().copied() {
    request.header(header)?;
  }

  let output = request.finish()?;

  assert_eq!(
    output,
    b"\
    GET /wp-content/uploads/2010/03/hello-kitty-darth-vader-pink.jpg HTTP/1.1\r\n\
    Host: www.kittyhell.com\r\n\
    User-Agent: Mozilla/5.0 (Macintosh; U; Intel Mac OS X 10.6; ja-JP-mac; rv:1.9.2.3) Gecko/20100401 Firefox/3.6.3 Pathtraq/0.9\r\n\
    Accept: text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8\r\n\
    Accept-Language: ja,en-us;q=0.7,en;q=0.3\r\n\
    Accept-Encoding: gzip,deflate\r\n\
    Accept-Charset: Shift_JIS,utf-8;q=0.7,*;q=0.7\r\n\
    Keep-Alive: 115\r\n\
    Connection: keep-alive\r\n\
    Cookie: wp_ozh_wsa_visits=2; wp_ozh_wsa_visit_lasttime=xxxxxxxxxx; __utma=xxxxxxxxx.xxxxxxxxxx.xxxxxxxxxx.xxxxxxxxxx.xxxxxxxxxx.x; __utmz=xxxxxxxxx.xxxxxxxxxx.x.x.utmccn=(referral)|utmcsr=reader.livedoor.com|utmcct=/reader/|utmcmd=referral|padding=under256\r\n\
    \r\n"
  );

  Ok(())
}
