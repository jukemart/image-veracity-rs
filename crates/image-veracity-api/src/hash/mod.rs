use std::fmt::Debug;
use std::io::Cursor;

use blockhash::blockhash256;
use image::{io::Reader, DynamicImage, ImageFormat};
use ring::digest::{digest, Digest, SHA256};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::error;

use crate::hash::cryptographic::CryptographicHash;
use crate::hash::perceptual::PerceptualHash;
use crate::hash::HashError::{
    ImageDecodeError, ImageHashError, ImageTypeUnknown, ImageTypeUnsupported,
};

pub(crate) mod cryptographic;
pub(crate) mod perceptual;

#[derive(Default, Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct VeracityHash {
    pub perceptual_hash: PerceptualHash,
    pub crypto_hash: CryptographicHash,
}

#[inline]
pub fn hash_image(buffer: &[u8]) -> Result<VeracityHash, HashError> {
    let reader = Reader::new(Cursor::new(buffer))
        .with_guessed_format()
        .map_err(|_| ImageDecodeError)?;
    match reader.format() {
        Some(ImageFormat::Jpeg | ImageFormat::Png) => match reader.decode() {
            Ok(image) => {
                let perceptual_hash = blockhash256(&image).into();
                let crypto_hash = crypto_image(&image)
                    .try_into()
                    .map_err(|_| ImageHashError)?;
                Ok(VeracityHash {
                    perceptual_hash,
                    crypto_hash,
                })
            }
            Err(e) => {
                error!("{}", e.to_string());
                Err(ImageDecodeError)
            }
        },
        Some(format) => Err(ImageTypeUnsupported(format)),
        None => Err(ImageTypeUnknown),
    }
}

fn crypto_image(image: &DynamicImage) -> Digest {
    let pixels = image.as_bytes();
    default_crypto_hash(pixels)
}

fn default_crypto_hash(pixels: &[u8]) -> Digest {
    digest(&SHA256, pixels)
}

#[derive(Error, Debug)]
pub enum HashError {
    #[error("did not recognize image type")]
    ImageTypeUnknown,
    #[error("image format unsupported")]
    ImageTypeUnsupported(ImageFormat),
    #[error("could not decode image from format")]
    ImageDecodeError,
    #[error("could not hash image")]
    ImageHashError,
    #[error("hash string was not valid base64")]
    InvalidBase64,
    #[error("hash bytes length not 32")]
    InvalidLength,
    #[error("hash string was not valid hex characters")]
    InvalidHexCharacters,
}

#[cfg(test)]
mod tests {
    use blockhash::Blockhash256;
    use eyre::Result;
    use image::EncodableLayout;
    use ring::test;
    use std::fs;
    use std::path::PathBuf;

    use super::*;

    const IMAGE_PATH: &str = "resources/test";

    use std::{env, process::Command};

    /// Get the absolute path from the Cargo workspace root for project-level resources
    pub fn get_workspace_root() -> Result<PathBuf> {
        let current_dir = env::current_dir()?;
        let cmd_output = Command::new("cargo")
            .args(["metadata", "--format-version=1"])
            .output()?;

        if !cmd_output.status.success() {
            return Ok(current_dir);
        }

        let json = serde_json::from_str::<serde_json::Value>(
            String::from_utf8(cmd_output.stdout)?.as_str(),
        )?;
        let path = match json.get("workspace_root") {
            Some(val) => match val.as_str() {
                Some(val) => val,
                None => return Ok(current_dir),
            },
            None => return Ok(current_dir),
        };
        Ok(fs::canonicalize(PathBuf::from(path)).unwrap())
    }

    fn get_test_image(image_name: &str) -> DynamicImage {
        let image_path = get_workspace_root()
            .expect("workspace should have a root")
            .join(format!("{}/{}", IMAGE_PATH, image_name));

        match image::open(&image_path) {
            Ok(img) => img,
            Err(err) => {
                panic!("could not get image from {:?} err: {:?}", &image_path, err)
            }
        }
    }

    #[test]
    fn blockhash_persistent_hash() {
        let known_hash = Blockhash256::from([
            0x9c, 0xfd, 0xe0, 0x3d, 0xc4, 0x19, 0x84, 0x67, 0xad, 0x67, 0x1d, 0x17, 0x1c, 0x07,
            0x1c, 0x5b, 0x1f, 0xf8, 0x1b, 0xf9, 0x19, 0xd9, 0x18, 0x18, 0x38, 0xf8, 0xf8, 0x90,
            0xf8, 0x07, 0xff, 0x01,
        ]);
        let known_hex = "9cfde03dc4198467ad671d171c071c5b1ff81bf919d9181838f8f890f807ff01";

        let img = get_test_image("test_495kb.png");
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
        let img_png = get_test_image("test_495kb.png");
        let hash1 = blockhash256(&img_png);

        let img_jpg = get_test_image("test_from_495kb_png.jpg");
        let hash2 = blockhash256(&img_jpg);

        assert_eq!(hash1, hash2);

        // monochrome image
        let img_png = get_test_image("test_1050kb.png");
        let hash1 = blockhash256(&img_png);

        let img_jpg = get_test_image("test_from_1050kb_png.jpg");
        let hash2 = blockhash256(&img_jpg);

        assert_eq!(hash1, hash2);

        // large jpg -> larger png
        let large_png = get_test_image("test_from_2890kb_jpg.png");
        let hash_large_png = blockhash256(&large_png);

        let large_jpg = get_test_image("test_2890kb.jpg");
        let hash_large_jpg = blockhash256(&large_jpg);

        assert_eq!(hash_large_png, hash_large_jpg);
    }

    #[test]
    /// Test hashing output does not change across versions
    fn crypto_persistent_hash() {
        let known_hash = "oY1OmtqoZ32_nUVGgKzmAAdn6Bo0ndvr-YhnDRYju4U";

        let img = get_test_image("test_495kb.png");
        let crypt_hash: CryptographicHash = crypto_image(&img)
            .try_into()
            .expect("valid, decodable image");
        assert_eq!(crypt_hash.to_b64(), known_hash)
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

        let image = get_test_image("test_495kb.png");

        // Expected hash created from the test_495kb.png using Golang:
        // ```golang
        // package main
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
        //	file1, _ := os.Open("test_495kb.png")
        //	defer func(file1 *os.File) {
        //		_ = file1.Close()
        //	}(file1)
        //
        // 	img, _ := png.Decode(file1)
        // 	rgba, _ := img.(*image.RGBA)
        // 	entry := createEntry(*rgba)
        // 	println(hex.EncodeToString(entry))
        // }
        // ```
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
