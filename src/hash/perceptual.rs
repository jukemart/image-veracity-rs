use std::fmt;
use std::fmt::{Display, Formatter};

use base64::prelude::{Engine as _, BASE64_URL_SAFE_NO_PAD};
use blockhash::Blockhash256;
use hex::{FromHex, FromHexError, ToHex};
use schemars::JsonSchema;
use serde::de::{Error, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use tracing::error;

use crate::hash::HashError;

#[derive(Default, Debug, Clone, PartialEq, Eq, JsonSchema)]
pub struct PerceptualHash([u8; 32]);

impl Serialize for PerceptualHash {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0.encode_hex::<String>())
    }
}

struct PerceptualHashVisitor;

impl<'de> Visitor<'de> for PerceptualHashVisitor {
    type Value = PerceptualHash;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an integer between -2^31 and 2^31")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        match <[u8; 32]>::from_hex(v) {
            Ok(buffer) => Ok(PerceptualHash(buffer)),
            Err(err) => {
                error!("Could not deserialize PerceptualHash from {}: {}", v, err);
                Err(Error::custom(err.to_string()))
            }
        }
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: Error,
    {
        match <[u8; 32]>::from_hex(v) {
            Ok(buffer) => Ok(PerceptualHash(buffer)),
            Err(err) => {
                error!("Could not deserialize PerceptualHash from String: {}", err);
                Err(Error::custom(err.to_string()))
            }
        }
    }
}

impl<'de> Deserialize<'de> for PerceptualHash {
    fn deserialize<D>(deserializer: D) -> Result<PerceptualHash, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(PerceptualHashVisitor)
    }
}

impl AsRef<[u8; 32]> for PerceptualHash {
    fn as_ref(&self) -> &[u8; 32] {
        &self.0
    }
}

impl Display for PerceptualHash {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.0.encode_hex::<String>())
    }
}

impl From<Blockhash256> for PerceptualHash {
    fn from(value: Blockhash256) -> Self {
        PerceptualHash(value.into())
    }
}

impl PartialEq<Blockhash256> for PerceptualHash {
    fn eq(&self, other: &Blockhash256) -> bool {
        let block_buffer: [u8; 32] = <Blockhash256>::into(*other);
        self.0 == block_buffer
    }
}

impl FromHex for PerceptualHash {
    type Error = HashError;

    fn from_hex<T: AsRef<[u8]>>(hex: T) -> Result<Self, Self::Error> {
        match <[u8; 32]>::from_hex(hex) {
            Ok(buffer) => Ok(PerceptualHash(buffer)),
            Err(FromHexError::InvalidStringLength | FromHexError::OddLength) => {
                Err(HashError::InvalidLength)
            }
            Err(FromHexError::InvalidHexCharacter { .. }) => Err(HashError::InvalidHexCharacters),
        }
    }
}

impl PerceptualHash {
    pub fn from_b64(value: &str) -> Result<PerceptualHash, HashError> {
        match BASE64_URL_SAFE_NO_PAD.decode(value) {
            Ok(bytes) => {
                let buffer = bytes.try_into().map_err(|_| HashError::InvalidLength)?;
                Ok(PerceptualHash(buffer))
            }
            Err(err) => {
                error!("could not decode from base64 string {}", err);
                Err(HashError::InvalidBase64)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blockhash_equality() {
        let crypto = PerceptualHash::from_hex(
            "9cfde03dc4198467ad671d171c071c5b1ff81bf919d9181838f8f890f807ff01",
        )
        .expect("valid hex");
        let blockhash = Blockhash256::from([
            0x9c, 0xfd, 0xe0, 0x3d, 0xc4, 0x19, 0x84, 0x67, 0xad, 0x67, 0x1d, 0x17, 0x1c, 0x07,
            0x1c, 0x5b, 0x1f, 0xf8, 0x1b, 0xf9, 0x19, 0xd9, 0x18, 0x18, 0x38, 0xf8, 0xf8, 0x90,
            0xf8, 0x07, 0xff, 0x01,
        ]);
        assert_eq!(&crypto, &blockhash);
    }
}
