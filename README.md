# HTTPEncode - no_std HTTP 1.0/1.1 encoding

httpencode is a low-level library for encoding HTTP 1.0/1.1 request
and response headers. It is designed to allow the maximum amount of
validation to be done at compile time and to be otherwise performant.

## Compiling without the standard library

httpencode links to the standard library by default, but you can disable
the `std` feature if you want to use it in a `#![no_std]` crate:

```toml
[dependencies]
httpencode { git="...", default-features = false }
```

## License
This library is dual-licensed under both Apache-2.0 or MIT. You can
choose between either one of them if you use this work.

```
SPDX-License-Identifier: Apache-2.0 OR MIT
```