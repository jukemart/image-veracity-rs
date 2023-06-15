use std::fmt::{Debug, Display, Formatter};
use std::io::Cursor;

use base64::{engine::general_purpose, Engine};
use blockhash::{blockhash256, Blockhash256};
use image::{
    error::{ImageFormatHint, UnsupportedError},
    io::Reader,
    DynamicImage, ImageError, ImageFormat,
};
use ring::digest::{digest, Context, Digest, SHA256};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::error;

#[derive(Default, Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct VeracityHash {
    pub perceptual_hash: PerceptualHash,
    pub crypto_hash: CryptographicHash,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PerceptualHash(String);

impl Display for PerceptualHash {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Blockhash256> for PerceptualHash {
    fn from(value: Blockhash256) -> Self {
        let p_bytes: [u8; 32] = value.into();
        let perceptual_b64 = general_purpose::URL_SAFE_NO_PAD.encode(p_bytes);
        PerceptualHash(perceptual_b64)
    }
}

impl TryFrom<&str> for PerceptualHash {
    type Error = HashError;

    fn try_from(value: &str) -> Result<PerceptualHash, HashError> {
        if let Ok(decode) = general_purpose::URL_SAFE_NO_PAD.decode(value) {
            let bytes: [u8; 32] = decode
                .try_into()
                .map_err(|decode_err: Vec<u8>| HashError::InvalidLength(decode_err.len()))?;
            Ok(Blockhash256::from(bytes).into())
        } else {
            Err(HashError::InvalidHashString(value.to_string()))
        }
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CryptographicHash(String);

impl Display for CryptographicHash {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Digest> for CryptographicHash {
    fn from(value: Digest) -> Self {
        let crypto_b64 = general_purpose::URL_SAFE_NO_PAD.encode(value);
        CryptographicHash(crypto_b64)
    }
}

impl TryFrom<&str> for CryptographicHash {
    type Error = HashError;

    /// Attempt conversion from &str
    ///
    /// # Arguments
    ///
    /// * `value`: &str
    ///
    /// returns: Result<CryptographicHash, HashError>
    ///
    /// # Examples
    ///
    /// ```
    ///
    /// ```
    fn try_from(value: &str) -> Result<CryptographicHash, HashError> {
        if let Ok(decode) = general_purpose::URL_SAFE_NO_PAD.decode(value) {
            let bytes: [u8; 32] = decode
                .try_into()
                .map_err(|decode_err: Vec<u8>| HashError::InvalidLength(decode_err.len()))?;
            let mut ctx = Context::new(&SHA256);
            ctx.update(&bytes);
            let created_digest = ctx.finish();
            Ok(CryptographicHash::from(created_digest))
        } else {
            Err(HashError::InvalidHashString(value.to_string()))
        }
    }
}

#[inline]
pub fn hash_image(buffer: &[u8]) -> Result<VeracityHash, ImageError> {
    let reader = Reader::new(Cursor::new(buffer)).with_guessed_format()?;
    match reader.format() {
        Some(ImageFormat::Jpeg | ImageFormat::Png) => match reader.decode() {
            Ok(image) => {
                let perceptual_hash = blockhash256(&image);
                let crypto_hash = crypto_image(&image);
                Ok(VeracityHash {
                    perceptual_hash: perceptual_hash.into(),
                    crypto_hash: crypto_hash.into(),
                })
            }
            Err(e) => {
                error!("{}", e.to_string());
                Err(e)
            }
        },
        Some(format) => Err(ImageError::Unsupported(UnsupportedError::from(
            ImageFormatHint::Exact(format),
        ))),
        None => Err(ImageError::Unsupported(UnsupportedError::from(
            ImageFormatHint::Unknown,
        ))),
    }
}

/// Generates a 256-bit cryptographic hash of an image.
///
/// # Examples
///
/// ```
///
/// let img = image::open("resources/test/test_495kb.png").unwrap();
/// let hash = image_veracity::hash::crypto_image(&img);
///
/// assert_eq!(
///     hash,
///     "oY1OmtqoZ32_nUVGgKzmAAdn6Bo0ndvr-YhnDRYju4U",
/// );
///
/// ```
pub fn crypto_image(image: &DynamicImage) -> Digest {
    let pixels = image.as_bytes();
    default_crypto_hash(pixels)
}

fn default_crypto_hash(pixels: &[u8]) -> Digest {
    digest(&SHA256, pixels)
}

#[derive(Error, Debug)]
pub enum HashError {
    #[error("hash string was not valid base64")]
    InvalidHashString(String),
    #[error("hash bytes length not 32")]
    InvalidLength(usize),
}

#[cfg(test)]
mod tests {
    use blockhash::Blockhash256;
    use image::EncodableLayout;
    use ring::test;

    use super::*;

    #[test]
    fn blockhash_persistent_hash() {
        let known_hash = Blockhash256::from([
            0x9c, 0xfd, 0xe0, 0x3d, 0xc4, 0x19, 0x84, 0x67, 0xad, 0x67, 0x1d, 0x17, 0x1c, 0x07,
            0x1c, 0x5b, 0x1f, 0xf8, 0x1b, 0xf9, 0x19, 0xd9, 0x18, 0x18, 0x38, 0xf8, 0xf8, 0x90,
            0xf8, 0x07, 0xff, 0x01,
        ]);
        let known_hex = "9cfde03dc4198467ad671d171c071c5b1ff81bf919d9181838f8f890f807ff01";

        let img = image::open("resources/test/test_495kb.png").unwrap();
        let hash = blockhash256(&img);

        assert_eq!(hash, known_hash);

        assert_eq!(hash.to_string(), known_hex);

        let expected: Vec<u8> = test::from_hex(known_hex).unwrap();
        let hash_bytes: [u8; 32] = hash.into();

        assert_eq!(&hash_bytes, &expected.as_bytes())
    }

    #[test]
    fn blockhash_same_between_formats() {
        // baseline
        let img_png = image::open("resources/test/test_495kb.png").unwrap();
        let hash1 = blockhash256(&img_png);

        let img_jpg = image::open("resources/test/test_from_495kb_png.jpg").unwrap();
        let hash2 = blockhash256(&img_jpg);

        assert_eq!(hash1, hash2);

        // monochrome image
        let img_png = image::open("resources/test/test_1050kb.png").unwrap();
        let hash1 = blockhash256(&img_png);

        let img_jpg = image::open("resources/test/test_from_1050kb_png.jpg").unwrap();
        let hash2 = blockhash256(&img_jpg);

        assert_eq!(hash1, hash2);

        // large jpg -> larger png
        let large_png = image::open("resources/test/test_from_2890kb_jpg.png").unwrap();
        let hash_large_png = blockhash256(&large_png);

        let large_jpg = image::open("resources/test/test_2890kb.jpg").unwrap();
        let hash_large_jpg = blockhash256(&large_jpg);

        assert_eq!(hash_large_png, hash_large_jpg);
    }

    #[test]
    /// Test hashing output does not change across versions
    fn crypto_persistent_hash() {
        let known_hash = "oY1OmtqoZ32_nUVGgKzmAAdn6Bo0ndvr-YhnDRYju4U";

        let img = image::open("resources/test/test_495kb.png").unwrap();
        let crypt_hash: CryptographicHash = crypto_image(&img).into();

        assert_eq!(crypt_hash.0, known_hash)
    }

    #[test]
    /// Test hashing equivalence with known Golang Trillian implementation.
    /// Trillian hasher uses domain prefix of "0" for leaves and "1" for nodes.
    /// This domain separation is for second preimage resistance aka collision avoidance.
    /// Domain separation prefixes:
    /// ```notrust
    /// const (
    ///   RFC6962LeafHashPrefix = 0
    ///   RFC6962NodeHashPrefix = 1
    /// )
    /// ```
    fn crypto_hash_compare_known_golang() {
        // So for a value of [u8; 0, 0, 0, 0, 0, 0, 0, 0] it's actually:
        // let data = [0, 0, 0, 0, 0, 0, 0, 0, 0];
        let data: Vec<u8> = vec![0; 9];
        let expected =
            test::from_hex("3e7077fd2f66d689e0cee6a7cf5b37bf2dca7c979af356d0a31cbc5c85605c7d")
                .unwrap();
        let actual = digest(&SHA256, &data);
        assert_eq!(&expected, &actual.as_ref());

        let image = image::open("resources/test/test_495kb.png").unwrap();

        ////Expected hash created from the test_495kb.png using Golang:
        //package main
        //
        // import (
        // 	"encoding/hex"
        // 	"github.com/transparency-dev/merkle/rfc6962"
        // 	"image"
        // 	"image/png"
        // 	"os"
        // )
        //
        // func createEntry(img image.RGBA) []byte {
        // 	return rfc6962.DefaultHasher.HashLeaf(img.Pix)
        // }
        //
        // func main() {
        // 	file1, _ := os.Open("test_495kb.png")
        // 	defer func(file1 *os.File) {
        // 		_ = file1.Close()
        // 	}(file1)
        //
        // 	img, _ := png.Decode(file1)
        // 	rgba, _ := img.(*image.RGBA)
        // 	entry := createEntry(*rgba)
        // 	println(hex.EncodeToString(entry))
        // }
        let expected =
            test::from_hex("e40e70d70cde3a0edfdc74bd659869b1dccaf05ef69f897ce78b02ccb33fe227")
                .unwrap();
        let mut pixels = image.into_rgba8().as_bytes().to_vec();
        // Add domain prefix
        pixels.insert(0, 0);
        let actual = digest(&SHA256, &pixels);
        assert_eq!(&expected, &actual.as_ref());
    }
}
