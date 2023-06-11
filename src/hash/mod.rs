use base64::engine::general_purpose;
use base64::Engine;
use blockhash::blockhash256;
use image::error::{ImageFormatHint, UnsupportedError};
use image::io::Reader;
use image::{DynamicImage, ImageError, ImageFormat};
use ring::digest::{digest, SHA256};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::io::Cursor;
use tracing::error;

#[derive(Default, Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct VeracityHash {
    pub perceptual_hash: String,
    pub crypto_hash: String,
}

#[inline]
pub fn hash_image(buffer: &[u8]) -> Result<VeracityHash, ImageError> {
    let reader = Reader::new(Cursor::new(buffer)).with_guessed_format()?;
    match reader.format() {
        Some(ImageFormat::Jpeg | ImageFormat::Png) => match reader.decode() {
            Ok(image) => {
                let perceptual_hash = blockhash256(&image).to_string();
                let crypto_hash = crypto_image(&image);
                Ok(VeracityHash {
                    perceptual_hash,
                    crypto_hash,
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
pub fn crypto_image(image: &DynamicImage) -> String {
    let pixels = image.as_bytes();
    let digest = digest(&SHA256, pixels);
    general_purpose::URL_SAFE_NO_PAD.encode(digest)
}

#[cfg(test)]
mod tests {
    use super::*;
    use blockhash::Blockhash256;
    use image::EncodableLayout;
    use ring::test;

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
    fn crypto_persistent_hash() {
        let known_hash = "oY1OmtqoZ32_nUVGgKzmAAdn6Bo0ndvr-YhnDRYju4U";

        let img = image::open("resources/test/test_495kb.png").unwrap();
        let digest = crypto_image(&img);

        assert_eq!(digest.as_str(), known_hash)
    }
}
