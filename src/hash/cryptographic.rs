use std::fmt;
use std::fmt::{Display, Formatter};

use base64::prelude::{Engine as _, BASE64_URL_SAFE_NO_PAD};
use hex::{FromHex, FromHexError, ToHex};
use ring::digest::Digest;
use schemars::JsonSchema;
use serde::de::{Error, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use tracing::error;

use crate::hash::HashError;

#[derive(Default, Debug, Clone, PartialEq, Eq, JsonSchema)]
pub struct CryptographicHash([u8; 32]);

impl Serialize for CryptographicHash {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0.encode_hex::<String>())
    }
}

struct CryptographicHashVisitor;

impl<'de> Visitor<'de> for CryptographicHashVisitor {
    type Value = CryptographicHash;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an integer between -2^31 and 2^31")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        match <[u8; 32]>::from_hex(v) {
            Ok(buffer) => Ok(CryptographicHash(buffer)),
            Err(err) => {
                error!(
                    "Could not deserialize CryptographicHash from {}: {}",
                    v, err
                );
                Err(Error::custom(err.to_string()))
            }
        }
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: Error,
    {
        match <[u8; 32]>::from_hex(v) {
            Ok(buffer) => Ok(CryptographicHash(buffer)),
            Err(err) => {
                error!(
                    "Could not deserialize CryptographicHash from String: {}",
                    err
                );
                Err(Error::custom(err.to_string()))
            }
        }
    }
}

impl<'de> Deserialize<'de> for CryptographicHash {
    fn deserialize<D>(deserializer: D) -> Result<CryptographicHash, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(CryptographicHashVisitor)
    }
}

impl AsRef<[u8; 32]> for CryptographicHash {
    fn as_ref(&self) -> &[u8; 32] {
        &self.0
    }
}

impl Display for CryptographicHash {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

impl TryFrom<Digest> for CryptographicHash {
    type Error = HashError;

    fn try_from(value: Digest) -> Result<Self, Self::Error> {
        // convert vec then to definite-sized array
        let buffer: [u8; 32] = Vec::from(value.as_ref())
            .try_into()
            .map_err(|_| HashError::InvalidLength)?;
        Ok(CryptographicHash(buffer))
    }
}

impl TryFrom<Vec<u8>> for CryptographicHash {
    type Error = HashError;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        // convert to definite-sized array
        let buffer: [u8; 32] = value.try_into().map_err(|_| HashError::InvalidLength)?;
        Ok(CryptographicHash(buffer))
    }
}

impl PartialEq<&[u8]> for CryptographicHash {
    fn eq(&self, other: &&[u8]) -> bool {
        self.0 == *other
    }
}

impl FromHex for CryptographicHash {
    type Error = HashError;

    fn from_hex<T: AsRef<[u8]>>(hex: T) -> Result<Self, Self::Error> {
        match <[u8; 32]>::from_hex(hex) {
            Ok(buffer) => Ok(CryptographicHash(buffer)),
            Err(FromHexError::InvalidStringLength | FromHexError::OddLength) => {
                Err(HashError::InvalidLength)
            }
            Err(FromHexError::InvalidHexCharacter { .. }) => Err(HashError::InvalidHexCharacters),
        }
    }
}

impl CryptographicHash {
    pub fn from_b64(value: &str) -> Result<CryptographicHash, HashError> {
        match BASE64_URL_SAFE_NO_PAD.decode(value) {
            Ok(bytes) => {
                let buffer = bytes.try_into().map_err(|_| HashError::InvalidLength)?;
                Ok(CryptographicHash(buffer))
            }
            Err(err) => {
                error!("could not decode from base64 string {}", err);
                Err(HashError::InvalidBase64)
            }
        }
    }

    pub fn to_b64(&self) -> String {
        BASE64_URL_SAFE_NO_PAD.encode(self.0)
    }

    pub fn to_hex(&self) -> String {
        self.0.encode_hex()
    }
}

#[cfg(test)]
mod tests {
    use ring::digest::{digest, SHA256};

    use super::*;

    #[test]
    fn digest_equality() {
        // Using known hashed value of 9 empty bytes
        let crypto = CryptographicHash::from_hex(
            "3e7077fd2f66d689e0cee6a7cf5b37bf2dca7c979af356d0a31cbc5c85605c7d",
        )
        .expect("valid hex");
        let data: Vec<u8> = vec![0; 9];
        let actual_digest = digest(&SHA256, &data);
        assert_eq!(&crypto, &actual_digest.as_ref());
    }
}
