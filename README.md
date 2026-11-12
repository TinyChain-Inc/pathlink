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

For HTTP and HTTPS links, an empty path is equivalent to the root path according
to IETF RFC 9110 Section 4.2.3, so parsing `https://example.com` canonicalizes and
displays it as `https://example.com/`.

Domain names are normalized to ASCII/Punycode through the `url` crate's IDNA
handling and then validated as DNS-style labels. Display, equality, ordering,
hashing, and serialization use that normalized form. `Address::Domain` stores a
validated `Domain` value backed by immutable shared string storage, so domain
addresses cannot be constructed from an arbitrary `String` and cloning them does
not clone the underlying domain string.

## Literal paths and URI text

`Id`, `Path`, `PathBuf`, and `PathLabel` contain and display literal identifier
text. Native path parsing splits segments without percent-decoding. Their string
serialization is literal in JSON and TBON; the codec supplies its own escaping.

`Link` is a URI: it parses escapes once and displays canonical escaped segments.
Use `ToUrl::from(&path[..])` when formatting a native path as a request target,
or construct a Link directly from the path. A literal native path `/a%2fb`
becomes URI `/a%252fb`; URI `/a%2fb` instead attempts to include a slash in one
identifier and is rejected. Unicode follows the same explicit conversion boundary.
Symbolic references and ordinary map keys are not URI text.

Native path strings and cryptographic path digests for escaped names intentionally
change to literal text. Link serialization/hashes, host rules, and the empty-path
hash sentinel are unchanged. There is no legacy escape interpretation or migration.
