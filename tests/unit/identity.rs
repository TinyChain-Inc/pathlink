use crate::{Id, Link, PathBuf};

#[test]
fn escaped_segments_round_trip_without_normalizing_identity() {
    assert!(Link::from(PathBuf::new()).eq("/"));

    #[cfg(feature = "hash")]
    {
        use async_hash::{Hash, Sha256};

        assert_eq!(
            Hash::<Sha256>::hash(PathBuf::new()),
            async_hash::default_hash::<Sha256>()
        );
    }

    for name in ["λ", "名字", "percent%name", "%2f", "release+build"] {
        let path = PathBuf::from(name.parse::<Id>().unwrap());
        let wire = path.to_string();
        assert_eq!(wire.parse::<PathBuf>().unwrap(), path);
        assert!(path.eq(wire.as_str()));
        assert_eq!(wire, format!("/{name}"));

        assert_eq!(
            crate::ToUrl::from(path.clone()).to_string(),
            crate::ToUrl::from(&path[..]).to_string()
        );
        let uri = crate::ToUrl::from(&path[..]).to_string();
        assert_eq!(uri.parse::<Link>().unwrap().path(), &path);
        if name == "%2f" {
            assert_eq!(uri, "/%252f");
        }

        #[cfg(feature = "hash")]
        {
            use async_hash::{Hash, Sha256};

            assert_eq!(
                Hash::<Sha256>::hash(&path),
                Hash::<Sha256>::hash(wire.as_str())
            );
        }

        let link = Link::from(path.clone());
        assert_eq!(link.to_string().parse::<Link>().unwrap(), link);

        #[cfg(feature = "hash")]
        {
            use async_hash::{Hash, Sha256};

            assert_eq!(
                Hash::<Sha256>::hash(&link),
                Hash::<Sha256>::hash(link.to_string())
            );
        }

        #[cfg(feature = "serialize")]
        {
            let native = serde_json::to_string(&path).unwrap();
            assert_eq!(serde_json::from_str::<String>(&native).unwrap(), wire);
            assert_eq!(serde_json::from_str::<PathBuf>(&native).unwrap(), path);

            let encoded = serde_json::to_string(&link).unwrap();
            assert_eq!(serde_json::from_str::<Link>(&encoded).unwrap(), link);
            let decoded: Link =
                serde_json::from_value(serde_json::Value::String(link.to_string())).unwrap();
            assert_eq!(decoded, link);
        }
    }

    for malformed in [
        "http:", "http://", "https://", "/%", "/%zz", "/%2e", "/%2e%2e", "/%2f", "/%5c",
    ] {
        assert!(malformed.parse::<Link>().is_err());
    }
}
