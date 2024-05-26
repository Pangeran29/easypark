use std::fmt::Formatter;

use serde::{de::Visitor, Deserialize, Deserializer, Serialize, Serializer};
use time::{Format, OffsetDateTime};

#[derive(sqlx::Type)]
pub struct Timestamptz(pub OffsetDateTime);

impl Serialize for Timestamptz {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(&self.0.lazy_format(Format::Rfc3339))
    }
}

impl<'de> Deserialize<'de> for Timestamptz {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct StrVisitor;

        // By providing our own `Visitor` impl, we can access the string data without copying.
        //
        // We could deserialize a borrowed `&str` directly but certain deserialization modes
        // of `serde_json` don't support that, so we'd be forced to always deserialize `String`.
        //
        // `serde_with` has a helper for this but it can be a bit overkill to bring in
        // just for one type: https://docs.rs/serde_with/latest/serde_with/#displayfromstr
        //
        // We'd still need to implement `Display` and `FromStr`, but those are much simpler
        // to work with.
        //
        // However, I also wanted to demonstrate that it was possible to do this with Serde alone.
        impl Visitor<'_> for StrVisitor {
            type Value = Timestamptz;

            fn expecting(&self, f: &mut Formatter) -> std::fmt::Result {
                f.pad("expected string")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                OffsetDateTime::parse(v, Format::Rfc3339)
                    .map(Timestamptz)
                    .map_err(E::custom)
            }
        }

        deserializer.deserialize_str(StrVisitor)
    }
}
