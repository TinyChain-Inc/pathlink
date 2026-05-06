//! A URI which supports IPv4, IPv6, domain names, and segmented [`Path`]s.

use std::cmp::Ordering;
use std::fmt;
use std::iter;
use std::str::FromStr;

use derive_more::Display;
use get_size::GetSize;
use idna::domain_to_ascii;
use smallvec::SmallVec;

pub use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

pub use hr_id::{label, Id, Label, ParseError};

mod path;
#[cfg(feature = "serialize")]
mod serial;
#[cfg(feature = "stream")]
mod stream;

pub use path::*;

/// A port number
pub type Port = u16;

type Segments<T> = SmallVec<[T; 8]>;

/// An owned or borrowed [`Link`] or [`Path`] which can be parsed as a URL.
pub enum ToUrl<'a> {
    Link(Link),
    LinkRef(&'a Link),
    Path(PathBuf),
    PathRef(&'a [PathSegment]),
}

impl<'a> ToUrl<'a> {
    /// Construct a new [`Link`] from this URL.
    pub fn to_link(&self) -> Link {
        match self {
            Self::Link(link) => (*link).clone(),
            Self::LinkRef(link) => (**link).clone(),
            Self::Path(path) => path.clone().into(),
            Self::PathRef(path) => PathBuf::from_slice(path).into(),
        }
    }

    /// Borrow the [`Host`] component of this link, if any.
    pub fn host(&self) -> Option<&Host> {
        match self {
            Self::Link(link) => link.host(),
            Self::LinkRef(link) => link.host(),
            _ => None,
        }
    }

    /// Borrow the [`Path`] component of this link.
    pub fn path(&self) -> &[PathSegment] {
        match self {
            Self::Link(link) => link.path(),
            Self::LinkRef(link) => link.path(),
            Self::Path(path) => path,
            Self::PathRef(path) => path,
        }
    }

    /// Construct a new URL of the given type from this link.
    pub fn parse<Url>(&self) -> Result<Url, <Url as FromStr>::Err>
    where
        Url: FromStr,
    {
        self.to_string().parse()
    }
}

impl<'a> fmt::Display for ToUrl<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Link(link) => fmt::Display::fmt(link, f),
            Self::LinkRef(link) => fmt::Display::fmt(link, f),
            Self::Path(path) => fmt::Display::fmt(path, f),
            Self::PathRef(path) => {
                if path.is_empty() {
                    f.write_str("/")?;
                }

                for segment in path.iter() {
                    write!(f, "/{segment}")?;
                }

                Ok(())
            }
        }
    }
}

impl<'a> From<Link> for ToUrl<'a> {
    fn from(link: Link) -> Self {
        Self::Link(link)
    }
}

impl<'a> From<&'a Link> for ToUrl<'a> {
    fn from(link: &'a Link) -> Self {
        Self::LinkRef(link)
    }
}

impl<'a> From<PathBuf> for ToUrl<'a> {
    fn from(path: PathBuf) -> Self {
        Self::Path(path)
    }
}

impl<'a> From<&'a [PathSegment]> for ToUrl<'a> {
    fn from(path: &'a [PathSegment]) -> Self {
        Self::PathRef(path)
    }
}

/// The protocol portion of a [`Link`] (e.g. "http")
#[derive(Copy, Clone, Debug, Default, Hash, Eq, PartialEq, get_size_derive::GetSize)]
pub enum Protocol {
    #[default]
    HTTP,
    HTTPS,
}

impl PartialOrd for Protocol {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Protocol {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Self::HTTP, Self::HTTP) => Ordering::Equal,
            (Self::HTTP, Self::HTTPS) => Ordering::Less,
            (Self::HTTPS, Self::HTTP) => Ordering::Greater,
            (Self::HTTPS, Self::HTTPS) => Ordering::Equal,
        }
    }
}

impl fmt::Display for Protocol {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match self {
            Self::HTTP => "http",
            Self::HTTPS => "https",
        })
    }
}

/// A network address
#[derive(Clone, Debug, Display, Eq, PartialEq, Hash)]
pub enum Address {
    IPv4(Ipv4Addr),
    IPv6(Ipv6Addr),
    Domain(String),
}

impl Default for Address {
    fn default() -> Self {
        Self::LOCALHOST
    }
}

impl Address {
    pub const LOCALHOST: Self = Self::IPv4(Ipv4Addr::LOCALHOST);

    /// Return this [`Address`] as an [`IpAddr`].
    pub fn as_ip(&self) -> Option<IpAddr> {
        match self {
            Self::IPv4(addr) => Some((*addr).into()),
            Self::IPv6(addr) => Some((*addr).into()),
            Self::Domain(_) => None,
        }
    }

    /// Return `true` if this is the [`Address`] of the local host.
    pub fn is_localhost(&self) -> bool {
        match self {
            Self::IPv4(addr) => addr.is_loopback(),
            Self::IPv6(addr) => addr.is_loopback(),
            Self::Domain(domain) => domain.eq_ignore_ascii_case("localhost"),
        }
    }
}

impl PartialOrd for Address {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Address {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Self::IPv4(this), Self::IPv4(that)) => this.cmp(that),
            (Self::IPv6(this), Self::IPv6(that)) => this.cmp(that),
            (Self::Domain(this), Self::Domain(that)) => this.cmp(that),
            (Self::IPv4(_), _) => Ordering::Less,
            (Self::IPv6(_), Self::IPv4(_)) => Ordering::Greater,
            (Self::IPv6(_), Self::Domain(_)) => Ordering::Less,
            (Self::Domain(_), _) => Ordering::Greater,
        }
    }
}

impl GetSize for Address {
    fn get_size(&self) -> usize {
        match self {
            Self::IPv4(_) => 4,
            Self::IPv6(_) => 16,
            Self::Domain(domain) => domain.len(),
        }
    }
}

impl From<Ipv4Addr> for Address {
    fn from(addr: Ipv4Addr) -> Address {
        Self::IPv4(addr)
    }
}

impl From<Ipv6Addr> for Address {
    fn from(addr: Ipv6Addr) -> Address {
        Self::IPv6(addr)
    }
}

impl From<IpAddr> for Address {
    fn from(addr: IpAddr) -> Address {
        match addr {
            IpAddr::V4(addr) => Self::IPv4(addr),
            IpAddr::V6(addr) => Self::IPv6(addr),
        }
    }
}

impl PartialEq<Ipv4Addr> for Address {
    fn eq(&self, other: &Ipv4Addr) -> bool {
        match self {
            Self::IPv4(addr) => addr == other,
            _ => false,
        }
    }
}

impl PartialEq<Ipv6Addr> for Address {
    fn eq(&self, other: &Ipv6Addr) -> bool {
        match self {
            Self::IPv6(addr) => addr == other,
            _ => false,
        }
    }
}

impl PartialEq<IpAddr> for Address {
    fn eq(&self, other: &IpAddr) -> bool {
        use IpAddr::*;

        match other {
            V4(addr) => self == addr,
            V6(addr) => self == addr,
        }
    }
}

/// The host component of a [`Link`] (e.g. "http://127.0.0.1:8702")
#[derive(Clone, Debug, Hash, Eq, PartialEq, get_size_derive::GetSize)]
pub struct Host {
    protocol: Protocol,
    address: Address,
    port: Option<Port>,
}

impl Host {
    /// Check if the address of this [`Host`] is the local host.
    pub fn is_localhost(&self) -> bool {
        self.address.is_localhost()
    }

    /// Check if this [`Host`] matches the host and port of the given `public_addr` or localhost.
    pub fn is_loopback(&self, public_addr: Option<&Host>) -> bool {
        if let Some(addr) = public_addr {
            self == addr || (self.is_localhost() && self.port == addr.port)
        } else {
            self.is_localhost()
        }
    }

    /// Return the [`Protocol`] to use with this [`Host`].
    pub fn protocol(&self) -> Protocol {
        self.protocol
    }

    /// Return the [`Address`] of this [`Host`].
    pub fn address(&self) -> &Address {
        &self.address
    }

    /// Return the [`Port`] which this [`Host`] is listening on.
    pub fn port(&self) -> Option<Port> {
        self.port
    }
}

impl FromStr for Host {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Host, ParseError> {
        if !s.starts_with("http://") {
            if !s.starts_with("https://") {
                return Err(format!("invalid protocol: {}", s).into());
            }
        }

        let (protocol, s) = if let Some(s) = s.strip_prefix("http://") {
            (Protocol::HTTP, s)
        } else {
            (Protocol::HTTPS, s.strip_prefix("https://").unwrap())
        };

        let (address, port): (Address, Option<u16>) = if s.contains("::") {
            let mut segments: Segments<&str> = s.split("::").collect();
            let port: Option<u16> = if segments.last().unwrap().contains(':') {
                let last_segment: Segments<&str> = segments.pop().unwrap().split(':').collect();
                if last_segment.len() == 2 {
                    segments.push(last_segment[0]);

                    let port = last_segment[1].parse().map_err(|cause| {
                        format!("{} is not a valid port number: {}", last_segment[1], cause)
                    })?;

                    Some(port)
                } else {
                    return Err(format!("invalid IPv6 address: {}", s).into());
                }
            } else {
                None
            };

            let address = segments.join("::");
            let address: Ipv6Addr = address.parse().map_err(|cause| {
                ParseError::from(format!(
                    "{} is not a valid IPv6 address: {}",
                    address, cause
                ))
            })?;

            (address.into(), port)
        } else {
            let (address, port) = if s.contains(':') {
                let segments: Segments<&str> = s.split(':').collect();
                if segments.len() == 2 {
                    let port: u16 = segments[1].parse().map_err(|cause| {
                        ParseError::from(format!(
                            "{} is not a valid port number: {}",
                            segments[1], cause
                        ))
                    })?;

                    (segments[0], Some(port))
                } else {
                    return Err(format!("invalid network address: {}", s).into());
                }
            } else {
                (s, None)
            };

            let address = if let Ok(address) = address.parse::<Ipv4Addr>() {
                Address::from(address)
            } else {
                let domain = domain_to_ascii(address).map_err(|cause| {
                    ParseError::from(format!(
                        "invalid domain name {address}: {cause:?}"
                    ))
                })?;

                Address::Domain(domain)
            };

            (address, port)
        };

        Ok(Host {
            protocol,
            address,
            port,
        })
    }
}

impl PartialOrd for Host {
    fn partial_cmp(&self, other: &Host) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Host {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.protocol.cmp(&other.protocol) {
            Ordering::Equal => match self.address.cmp(&other.address) {
                Ordering::Equal => self.port.cmp(&other.port),
                ordering => ordering,
            },
            ordering => ordering,
        }
    }
}

impl From<(Protocol, Address)> for Host {
    fn from(components: (Protocol, Address)) -> Self {
        let (protocol, address) = components;
        Self {
            protocol,
            address,
            port: None,
        }
    }
}

impl From<(Address, Port)> for Host {
    fn from(components: (Address, Port)) -> Self {
        let (address, port) = components;
        Self {
            protocol: Protocol::default(),
            address,
            port: Some(port),
        }
    }
}

impl From<(Protocol, Address, Port)> for Host {
    fn from(components: (Protocol, Address, Port)) -> Self {
        let (protocol, address, port) = components;
        Self {
            protocol,
            address,
            port: Some(port),
        }
    }
}

impl From<(Protocol, Address, Option<Port>)> for Host {
    fn from(components: (Protocol, Address, Option<Port>)) -> Self {
        let (protocol, address, port) = components;
        Self {
            protocol,
            address,
            port,
        }
    }
}

impl fmt::Display for Host {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(port) = self.port {
            write!(f, "{}://{}:{}", self.protocol, self.address, port)
        } else {
            write!(f, "{}://{}", self.protocol, self.address)
        }
    }
}

/// An HTTP Link with an optional [`Address`] and [`PathBuf`]
#[derive(Clone, Default, Eq, Hash, PartialEq, get_size_derive::GetSize)]
pub struct Link {
    host: Option<Host>,
    path: PathBuf,
}

impl Link {
    /// Create a new [`Link`] with the given [`Host`] and [`PathBuf`].
    pub fn new(host: Host, path: PathBuf) -> Self {
        Self {
            host: Some(host),
            path,
        }
    }

    /// Consume this [`Link`] and return its [`Host`] and [`PathBuf`].
    pub fn into_inner(self) -> (Option<Host>, PathBuf) {
        (self.host, self.path)
    }

    /// Consume this [`Link`] and return its [`Host`].
    pub fn into_host(self) -> Option<Host> {
        self.host
    }

    /// Consume this [`Link`] and return its [`PathBuf`].
    pub fn into_path(self) -> PathBuf {
        self.path
    }

    /// Borrow this [`Link`]'s [`Host`], if it has one.
    pub fn host(&self) -> Option<&Host> {
        self.host.as_ref()
    }

    /// Borrow this [`Link`]'s path.
    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    /// Borrow this [`Link`]'s path mutably.
    pub fn path_mut(&mut self) -> &mut PathBuf {
        &mut self.path
    }

    /// Append the given [`PathSegment`] to this [`Link`] and return it.
    pub fn append<S: Into<PathSegment>>(mut self, segment: S) -> Self {
        self.path = self.path.append(segment);
        self
    }
}

impl Extend<PathSegment> for Link {
    fn extend<I: IntoIterator<Item = PathSegment>>(&mut self, iter: I) {
        self.path.extend(iter)
    }
}

impl PartialEq<[PathSegment]> for Link {
    fn eq(&self, other: &[PathSegment]) -> bool {
        if self.host.is_some() {
            return false;
        }

        &self.path == other
    }
}

impl PartialEq<String> for Link {
    fn eq(&self, other: &String) -> bool {
        self == other.as_str()
    }
}

impl PartialEq<str> for Link {
    fn eq(&self, other: &str) -> bool {
        let other = other.strip_suffix('/').unwrap_or(other);

        if other.is_empty() {
            return false;
        }

        other.parse::<Link>().is_ok_and(|other| self == &other)
    }
}

impl From<Host> for Link {
    fn from(host: Host) -> Link {
        Link {
            host: Some(host),
            path: PathBuf::default(),
        }
    }
}

impl From<PathLabel> for Link {
    fn from(path: PathLabel) -> Self {
        PathBuf::from(path).into()
    }
}

impl From<PathBuf> for Link {
    fn from(path: PathBuf) -> Link {
        Link { host: None, path }
    }
}

impl From<(Host, PathBuf)> for Link {
    fn from(tuple: (Host, PathBuf)) -> Link {
        let (host, path) = tuple;
        Link {
            host: Some(host),
            path,
        }
    }
}

impl From<(Option<Host>, PathBuf)> for Link {
    fn from(tuple: (Option<Host>, PathBuf)) -> Link {
        let (host, path) = tuple;
        Link { host, path }
    }
}

impl FromStr for Link {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Link, ParseError> {
        if s.starts_with('/') {
            return Ok(Link {
                host: None,
                path: s.parse()?,
            });
        } else if !s.starts_with("http://") && !s.starts_with("https://") {
            return Err(format!("cannot parse {} as a Link: invalid protocol", s).into());
        }

        let s = s.strip_suffix('/').unwrap_or(s);

        let segments: Segments<&str> = s.split('/').collect();

        if segments.is_empty() {
            return Err(format!("invalid Link: {}", s).into());
        }

        let host: Host = segments[..3].join("/").parse()?;

        let segments = &segments[3..];

        let segments = segments
            .iter()
            .map(|s| s.parse())
            .collect::<Result<Segments<PathSegment>, ParseError>>()?;

        Ok(Link {
            host: Some(host),
            path: iter::FromIterator::from_iter(segments),
        })
    }
}

impl PartialOrd for Link {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Link {
    fn cmp(&self, other: &Self) -> Ordering {
        match (&self.host, &other.host) {
            (None, None) => self.path.cmp(&other.path),
            (Some(this), Some(that)) => match this.cmp(that) {
                Ordering::Equal => self.path.cmp(&other.path),
                ordering => ordering,
            },
            (Some(_), _) => Ordering::Less,
            (_, Some(_)) => Ordering::Greater,
        }
    }
}

impl TryFrom<Link> for PathBuf {
    type Error = ParseError;

    fn try_from(link: Link) -> Result<Self, Self::Error> {
        if link.host.is_none() {
            Ok(link.path)
        } else {
            Err(ParseError::from(format!(
                "expected a path but found {}",
                link
            )))
        }
    }
}

impl fmt::Debug for Link {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(host) = &self.host {
            write!(f, "{:?}{:?}", host, self.path)
        } else {
            fmt::Debug::fmt(&self.path, f)
        }
    }
}

impl fmt::Display for Link {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(host) = &self.host {
            write!(f, "{}{}", host, self.path)
        } else {
            fmt::Display::fmt(&self.path, f)
        }
    }
}

#[cfg(feature = "hash")]
impl<D: async_hash::Digest> async_hash::Hash<D> for Link {
    fn hash(self) -> async_hash::Output<D> {
        async_hash::Hash::<D>::hash(&self)
    }
}

#[cfg(feature = "hash")]
impl<D: async_hash::Digest> async_hash::Hash<D> for &Link {
    fn hash(self) -> async_hash::Output<D> {
        if self == &Link::default() {
            async_hash::default_hash::<D>()
        } else {
            async_hash::Hash::<D>::hash(self.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Address, Host, Link, Protocol};

    #[test]
    fn parse_http_host_ipv4() {
        let host: Host = "http://127.0.0.1:80".parse().expect("host");
        assert_eq!(host.protocol(), Protocol::HTTP);
        assert_eq!(host.port(), Some(80));
        assert_eq!(host.address(), &Address::from(std::net::Ipv4Addr::new(127, 0, 0, 1)));
    }

    #[test]
    fn parse_https_host_domain() {
        let host: Host = "https://example.com:443".parse().expect("host");
        assert_eq!(host.protocol(), Protocol::HTTPS);
        assert_eq!(host.port(), Some(443));
        assert_eq!(host.address(), &Address::Domain("example.com".to_string()));
    }

    #[test]
    fn parse_https_idn_host_normalizes_to_ascii() {
        let host: Host = "https://bücher.example".parse().expect("host");
        assert_eq!(
            host.address(),
            &Address::Domain("xn--bcher-kva.example".to_string())
        );
    }

    #[test]
    fn parse_invalid_domain_name_fails() {
        let err = "https://-bad-.com"
            .parse::<Host>()
            .expect_err("invalid domain should fail");
        assert!(err.to_string().contains("invalid domain name"));
    }

    #[test]
    fn parse_link_accepts_https() {
        let link: Link = "https://example.com/a/b".parse().expect("link");
        assert_eq!(link.to_string(), "https://example.com/a/b");
    }
}
