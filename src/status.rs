/// HTTP Status Code.
pub struct Status<'msg> {
  code: u16,
  reason: Option<&'msg str>,
}

impl<'msg> Status<'msg> {
  /// Create a new status code and use the default reason phrase.
  ///
  /// Depending on whether the `no-reason-phrase` feature of this
  /// crate is enabled the reason phrase will either be the one
  /// specified in [the IANA status code registry][0] or the empty
  /// string. The only exception is that this crate also supports
  /// HTTP 418 with the `I'm a Teapot` reason phrase.
  ///
  /// # Example
  /// ```
  /// # use httpencode::*;
  /// let status = Status::IM_A_TEAPOT;
  ///
  /// assert_eq!(status.code(), 418);
  /// assert!(status.reason() == "I'm a Teapot" || status.reason() == "");
  /// ```
  ///
  /// [0]: https://www.iana.org/assignments/http-status-codes/http-status-codes.xhtml
  pub const fn new(code: u16) -> Self {
    Self {
      code,
      reason: Self::reason_phrase(code),
    }
  }

  /// Create a status with a custom reason phrase.
  ///
  /// # Example
  /// ```
  /// # use httpencode::*;
  /// let status = Status::with_reason(600, "Not a valid status");
  ///
  /// assert_eq!(status.code(), 600);
  /// assert_eq!(status.reason(), "Not a valid status");
  /// ```
  pub const fn with_reason(code: u16, reason: &'msg str) -> Self {
    Self {
      code,
      reason: Some(reason),
    }
  }

  /// Get the status code for this `Status`.
  pub const fn code(&self) -> u16 {
    self.code
  }

  /// Get the reason phrase for this `Status` or `""` if it doesn't have
  /// one.
  pub const fn reason(&self) -> &str {
    match self.reason {
      Some(reason) => reason,
      None => "",
    }
  }

  #[cfg(not(feature = "no-reason-phrase"))]
  const fn reason_phrase(mut code: u16) -> Option<&'static str> {
    code = match code {
      0..=99 => return None,
      code if code as usize > REASON_PHRASES.len() + 100 => return None,
      code => code,
    };

    REASON_PHRASES[(code - 100) as usize]
  }

  #[cfg(feature = "no-reason-phrase")]
  const fn reason_phrase(_: u16) -> Option<&'static str> {
    None
  }
}

macro_rules! decl_status {
  {
    $(
      $( #[$attr:meta] )*
      $value:literal => $name:ident;
    )*
  } => {
    $(
      $( #[$attr] )*
      pub const $name: Self = Self::new($value);
    )*
  }
}

// It's probably not necessary to document every HTTP status code here.
#[allow(missing_docs)]
impl<'msg> Status<'msg> {
  decl_status! {
    // 1xx codes
    100 => CONTINUE;
    101 => SWITCHING_PROTOCOLS;
    102 => PROCESSING;
    103 => EARLY_HINTS;

    // 2xx codes
    200 => OK;
    201 => CREATED;
    202 => ACCEPTED;
    203 => NON_AUTHORITATIVE_INFORMATION;
    204 => NO_CONTENT;
    205 => RESET_CONTENT;
    206 => PARTIAL_CONTENT;
    207 => MULTI_STATUS;
    208 => ALREADY_REPORTED;
    226 => IM_USED;

    // 3xx codes
    300 => MULTIPLE_CHOICES;
    301 => MOVED_PERMANENTLY;
    302 => FOUND;
    303 => SEE_OTHER;
    304 => NOT_MODIFIED;
    305 => USE_PROXY;
    // 306
    307 => TEMPORARY_REDIRECT;
    308 => PERMANENT_REDIRECT;

    // 4xx codes
    400 => BAD_REQUEST;
    401 => UNAUTHORIZED;
    402 => PAYMENT_REQUIRED;
    403 => FORBIDDEN;
    404 => NOT_FOUND;
    405 => METHOD_NOT_ALLOWED;
    406 => NOT_ACCEPTABLE;
    407 => PROXY_AUTHENTICATION_REQUIRED;
    408 => REQUEST_TIMEOUT;
    409 => CONFLICT;
    410 => GONE;
    411 => LENGTH_REQUIRED;
    412 => PRECONDITION_FAILED;
    413 => PAYLOAD_TOO_LARGE;
    414 => URI_TOO_LONG;
    415 => UNSUPPORTED_MEDIA_TYPE;
    416 => RANGE_NOT_SATISFIABLE;
    417 => EXPECTATION_FAILED;
    418 => IM_A_TEAPOT;
    // 419-420
    421 => MISDIRECTED_REQUEST;
    422 => UNPROCESSEABLE_ENTITY;
    423 => LOCKED;
    424 => FAILED_DEPENDENCY;
    425 => TOO_EARLY;
    426 => UPGRADE_REQUIRED;
    // 427
    428 => PRECONDITION_REQUIRED;
    429 => TOO_MANY_REQUESTS;
    // 430
    431 => REQUEST_HEADER_FIELDS_TOO_LARGE;
    // 432-451
    451 => UNAVAILABLE_FOR_LEGAL_REASONS;

    // 5xx codes
    500 => INTERNAL_SERVER_ERROR;
    501 => NOT_IMPLEMENTED;
    502 => BAD_GATEWAY;
    503 => SERVICE_UNAVAILABLE;
    504 => GATEWAY_TIMEOUT;
    505 => HTTP_VERSION_NOT_SUPPORTED;
    506 => VARIANT_ALSO_NEGOTIATES;
    507 => INSUFFICIENT_STORAGE;
    508 => LOOP_DETECTED;
    // 509
    510 => NOT_EXTENDED;
    511 => NETWORK_AUTHENTICATION_REQUIRED;
  }
}

macro_rules! min {
  () => { 0 };
  ($a:expr) => { $a };
  ($a:expr, $b:expr) => {{
    let a = $a;
    let b = $b;

    if a < b { a } else { b }
  }};
  ($a:expr, $b:expr $(, $rest:expr )+) => {
    min!(min!($a, $b), min!($( $rest ),*))
  }
}

macro_rules! max {
  () => { 0 };
  ($a:expr) => { $a };
  ($a:expr, $b:expr) => {{
    let a = $a;
    let b = $b;

    if a > b { a } else { b }
  }};
  ($a:expr, $b:expr $(, $rest:expr )+) => {
    max!(max!($a, $b), max!($( $rest ),*))
  }
}

macro_rules! arraytable {
  [
    $(
      [$index:literal] = $value:expr
    ),* $(,)?
  ] => {{
    const MAX: usize = max!($( $index ),*);
    const MIN: usize = min!($( $index ),*);

    let mut value = [None; MAX - MIN + 1];

    $( value[$index - MIN] = Some($value); )*

    value
  }}
}

#[cfg_attr(not(features="no-reason-phrase"), allow(dead_code))]
const REASON_PHRASES: &[Option<&str>] = &arraytable![
  // 1xx codes
  [100] = "Continue",
  [101] = "Switching Protocols",
  [102] = "Processing",
  [103] = "Early Hints",
  // 2xx codes
  [200] = "OK",
  [201] = "Created",
  [202] = "Accepted",
  [203] = "Non-Authoritative Information",
  [204] = "No Content",
  [205] = "Reset Content",
  [206] = "Partial Content",
  [207] = "Multi-Status",
  [208] = "Already Reported",
  [226] = "IM Used",
  // 3xx codes
  [300] = "Multiple Choices",
  [301] = "Moved Permanently",
  [302] = "Found",
  [303] = "See Other",
  [304] = "Not Modified",
  [305] = "Use Proxy",
  // 306 is unused but reserved
  [307] = "Temporary Redirect",
  [308] = "Permanent Redirect",
  // 4xx codes
  [400] = "Bad Request",
  [401] = "Unauthorized",
  [402] = "Payment Required",
  [403] = "Forbidden",
  [404] = "Not Found",
  [405] = "Method Not Allowed",
  [406] = "Not Acceptable",
  [407] = "Proxy Authentication Required",
  [408] = "Request Timeout",
  [409] = "Conflict",
  [410] = "Gone",
  [411] = "Length Required",
  [412] = "Precondition Failed",
  [413] = "Payload Too Large",
  [414] = "URI Too Long",
  [415] = "Unsupported Media Type",
  [416] = "Range Not Satisfiable",
  [417] = "Expectation Failed",
  [418] = "I'm a Teapot",
  // 419-420 are unassigned
  [421] = "Misdirected Request",
  [422] = "Unprocessable Entity",
  [423] = "Locked",
  [424] = "Failed Dependency",
  [425] = "Too Early",
  [426] = "Upgrade Required",
  // 427 is unassigned
  [428] = "Precondition Required",
  [429] = "Too Many Requests",
  // 430 is unassigned
  [431] = "Request Header Fields Too Large",
  // 432-451 are unassigned
  [451] = "Unavailable for Legal Reasons",
  // 5xx codes
  [500] = "Internal Server Error",
  [501] = "Not Implemented",
  [502] = "Bad Gateway",
  [503] = "Service Unavailable",
  [504] = "Gateway Timeout",
  [505] = "HTTP Version Not Supported",
  [506] = "Variant Also Negotiates",
  [507] = "Insufficient Storage",
  [508] = "Loop Detected",
  // 509 is unassigned
  [510] = "Not Extended",
  [511] = "Network Authentication Required"
];
