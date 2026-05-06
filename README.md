# pathlink

A URL type whose path can also be used as a filesystem path, for Rust.

`pathlink` supports absolute HTTP and HTTPS links with IPv4, bracketed IPv6,
and domain-name hosts, plus absolute path-only links:

```rust
use pathlink::{Domain, Link};

let http: Link = "http://127.0.0.1:8702/api".parse().expect("HTTP link");
let https: Link = "https://example.com/api".parse().expect("HTTPS link");
let idn: Link = "https://bücher.example/api".parse().expect("IDN link");
let local: Link = "/api".parse().expect("path-only link");

assert_eq!(idn.to_string(), "https://xn--bcher-kva.example/api");

let domain = Domain::new("EXAMPLE.COM").expect("domain");
assert_eq!(domain.as_str(), "example.com");
```

Host parsing intentionally accepts only the URL components represented by
`Host`: a scheme, host address, and optional port. Userinfo, paths, queries,
and fragments are rejected when parsing a `Host`.

Domain names are normalized to ASCII/Punycode through the `url` crate's IDNA
handling and then validated as DNS-style labels. Display, equality, ordering,
hashing, and serialization use that normalized form. `Address::Domain` stores a
validated `Domain` value backed by immutable shared string storage, so domain
addresses cannot be constructed from an arbitrary `String` and cloning them does
not clone the underlying domain string.
