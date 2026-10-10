//! Canonical map deserialization: duplicate identity keys are
//! rejected instead of collapsing silently.
//!
//! Every authorial map in the Core model (scene nodes, page roots,
//! registries, palette-adjacent lookups) routes through
//! [`deserialize_unique_btree_map`]. The JSON text is the source of
//! truth for identity collisions: two spellings of one key fail the
//! load with a typed error rather than letting the last one win.

use serde::de::{self, Deserialize, Deserializer, MapAccess, Visitor};
use std::collections::BTreeMap;
use std::fmt;
use std::marker::PhantomData;

/// Deserialize a `BTreeMap` refusing duplicate keys with a typed
/// error naming the colliding key.
pub fn deserialize_unique_btree_map<'de, D, K, V>(
    deserializer: D,
) -> Result<BTreeMap<K, V>, D::Error>
where
    D: Deserializer<'de>,
    K: Deserialize<'de> + Ord + fmt::Debug,
    V: Deserialize<'de>,
{
    struct UniqueMap<K, V>(PhantomData<(K, V)>);

    impl<'de, K, V> Visitor<'de> for UniqueMap<K, V>
    where
        K: Deserialize<'de> + Ord + fmt::Debug,
        V: Deserialize<'de>,
    {
        type Value = BTreeMap<K, V>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a JSON object with unique keys")
        }

        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: MapAccess<'de>,
        {
            let mut out = BTreeMap::new();
            while let Some((key, value)) = map.next_entry::<K, V>()? {
                if out.contains_key(&key) {
                    return Err(de::Error::custom(format!(
                        "duplicate identity key rejected: {key:?}"
                    )));
                }
                out.insert(key, value);
            }
            Ok(out)
        }
    }

    deserializer.deserialize_map(UniqueMap(PhantomData))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::id::ObjectId;

    #[test]
    fn unique_maps_round_trip() {
        #[derive(serde::Serialize, serde::Deserialize, PartialEq, Debug)]
        struct Holder {
            #[serde(deserialize_with = "deserialize_unique_btree_map")]
            entries: BTreeMap<ObjectId, String>,
        }
        let mut entries = BTreeMap::new();
        let id = ObjectId::new_v4();
        entries.insert(id, "kept".to_string());
        let holder = Holder { entries };
        let json = serde_json::to_string(&holder).expect("serializes");
        let back: Holder = serde_json::from_str(&json).expect("parses");
        assert_eq!(back, holder);
    }

    #[test]
    fn duplicate_keys_are_rejected_not_collapsed() {
        #[derive(serde::Serialize, serde::Deserialize, Debug)]
        struct Holder {
            #[serde(deserialize_with = "deserialize_unique_btree_map")]
            entries: BTreeMap<ObjectId, String>,
        }
        let id = ObjectId::new_v4();
        let json = format!("{{\"entries\":{{\"{id}\":\"first\",\"{id}\":\"second\"}}}}");
        let error = serde_json::from_str::<Holder>(&json).expect_err("must reject");
        assert!(
            error.to_string().contains("duplicate identity key"),
            "{error}"
        );
    }
}
