use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::{Host, Link, PathBuf};

impl<'de> Deserialize<'de> for Host {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let inner: &str = Deserialize::deserialize(deserializer)?;
        inner.parse().map_err(serde::de::Error::custom)
    }
}

impl Serialize for Host {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.to_string().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Link {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let inner: &str = Deserialize::deserialize(deserializer)?;
        inner.parse().map_err(serde::de::Error::custom)
    }
}

impl Serialize for Link {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.to_string().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for PathBuf {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let inner: &str = Deserialize::deserialize(deserializer)?;
        inner.parse().map_err(serde::de::Error::custom)
    }
}

impl Serialize for PathBuf {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.to_string().serialize(serializer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{from_str, to_string};

    fn assert_serde<T: Serialize + for<'de> Deserialize<'de>>() {}

    #[test]
    fn host_link_pathbuf_have_serde_impls() {
        assert_serde::<Host>();
        assert_serde::<Link>();
        assert_serde::<PathBuf>();
    }

    #[test]
    fn host_serde_roundtrip() {
        let host: Host = "https://example.com:443".parse().expect("host");
        let encoded = to_string(&host).expect("serialize host");
        let decoded: Host = from_str(&encoded).expect("deserialize host");
        assert_eq!(decoded, host);
    }

    #[test]
    fn link_serde_roundtrip() {
        let link: Link = "https://bücher.example/a/b".parse().expect("link");
        let encoded = to_string(&link).expect("serialize link");
        let decoded: Link = from_str(&encoded).expect("deserialize link");
        assert_eq!(decoded, link);
    }
}
