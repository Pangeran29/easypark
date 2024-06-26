pub mod config {
    use once_cell::sync::Lazy;
    use jsonwebtoken::{DecodingKey, EncodingKey};

    pub struct Keys {
        pub encoding: EncodingKey,
        pub decoding: DecodingKey,
    }

    impl Keys {
        pub fn new(secret: &[u8]) -> Self {
            Self {
                encoding: EncodingKey::from_secret(secret),
                decoding: DecodingKey::from_secret(secret),
            }
        }
    }

    pub static KEYS: Lazy<Keys> = Lazy::new(|| {
        let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
        Keys::new(secret.as_bytes())
    });
}