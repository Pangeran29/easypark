use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(sqlx::Type, Debug)]
pub struct UTC(pub DateTime<Utc>);

const FORMAT: &'static str = "%Y-%m-%d %H:%M:%S";

impl Serialize for UTC {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let date = format!("{}", self.0.format(FORMAT));

        serializer.serialize_str(&date)
    }
}

impl<'de> Deserialize<'de> for UTC {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let dt = NaiveDateTime::parse_from_str(&s, FORMAT).map_err(serde::de::Error::custom)?;
        let format = DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc);
        Ok(UTC(format))
    }
}
