#![feature(test)]

extern crate test;

use httpencode::*;
use test::Bencher;

#[bench]
fn bench_response(b: &mut Bencher) {
  let body: &[u8] = b"
  <!DOCTYPE html>
  <body>
    Hello World!
  </body>
  ";

  let mut buffer = vec![];

  b.iter(|| -> Result<usize, InsufficientSpaceError> {
    const OK: Status = Status::new(200);

    buffer.clear();

    let mut request = httpencode::response(&mut buffer, Version::HTTP_1_1, OK)?;
    request.header(Header::new("Content-Type", "text/html"))?;
    request.header(Header::new("Content-Length", body.len()))?;
    request.finish()?;

    buffer.extend_from_slice(&body);

    Ok(buffer.len())
  });
}

#[bench]
fn bench_response_fast(b: &mut Bencher) {
  const BODY: &'static [u8] = b"
  <!DOCTYPE html>
  <body>
    Hello World!
  </body>
  ";

  let mut buffer = vec![];

  b.iter(|| -> Result<usize, InsufficientSpaceError> {
    const OK: Status = Status::new(200);
    const CONTENT_TYPE: Header<&str> = Header::new("Content-Type", "text/html");
    const CONTENT_LENGTH: Header<usize> =
      Header::new("Content-Length", BODY.len());

    buffer.clear();

    let mut request = httpencode::response(&mut buffer, Version::HTTP_1_1, OK)?;
    request.header(CONTENT_TYPE)?;
    request.header(CONTENT_LENGTH)?;
    request.finish()?;

    buffer.extend_from_slice(&BODY);

    Ok(buffer.len())
  });
}

#[bench]
fn bench_request(b: &mut Bencher) {
  let mut buffer = vec![];

  b.iter(|| -> Result<usize, InsufficientSpaceError> {
    buffer.clear();

    let mut request = httpencode::request(
      &mut buffer,
      Method::GET,
      Uri::new(b"/wp-content/uploads/2010/03/hello-kitty-darth-vader-pink.jpg"),
      Version::HTTP_1_1,
    )?;

    request.header(Header::new("Host", "www.kittyhell.com"))?;
    request.header(Header::new("User-Agent", "Mozilla/5.0 (Macintosh; U; Intel Mac OS X 10.6; ja-JP-mac; rv:1.9.2.3) Gecko/20100401 Firefox/3.6.3 Pathtraq/0.9"))?;
    request.header(Header::new("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8"))?;
    request.header(Header::new("Accept-Language", "ja,en-us;q=0.7,en;q=0.3"))?;
    request.header(Header::new("Accept-Encoding", "gzip,deflate"))?;
    request.header(Header::new("Accept-Charset", "Shift_JIS,utf-8;q=0.7,*;q=0.7"))?;
    request.header(Header::new("Keep-Alive", 115))?;
    request.header(Header::new("Connection", "keep-alive"))?;
    request.header(Header::new("Cookie", "wp_ozh_wsa_visits=2; wp_ozh_wsa_visit_lasttime=xxxxxxxxxx; __utma=xxxxxxxxx.xxxxxxxxxx.xxxxxxxxxx.xxxxxxxxxx.xxxxxxxxxx.x; __utmz=xxxxxxxxx.xxxxxxxxxx.x.x.utmccn=(referral)|utmcsr=reader.livedoor.com|utmcct=/reader/|utmcmd=referral|padding=under256"))?;

    request.finish()?;

    Ok(buffer.len())
  });
}

#[bench]
fn bench_request_fast(b: &mut Bencher) {
  let mut buffer = vec![];

  b.iter(|| -> Result<usize, InsufficientSpaceError> {
    const URI: Uri = Uri::new_const(b"/wp-content/uploads/2010/03/hello-kitty-darth-vader-pink.jpg");

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

    buffer.clear();

    let mut request = httpencode::request(
      &mut buffer,
      Method::GET,
      URI,
      Version::HTTP_1_1,
    )?;

    for header in HEADERS.iter().copied() {
      request.header(header)?;
    }

    request.finish()?;

    Ok(buffer.len())
  });
}
