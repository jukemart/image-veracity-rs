use image::io::Reader;
use std::io::Cursor;
use image::{DynamicImage, EncodableLayout, ImageError, ImageFormat};
use blockhash::{blockhash256};
use image::error::{ImageFormatHint, UnsupportedError};
use tracing::error;
use ring::digest::{digest, Digest, SHA256};
use core::fmt::Write;
use base64::Engine;
use base64::engine::general_purpose;

pub struct VeracityHash {
    perceptual_hash: String,
    crypto_hash: String,
}

#[inline]
pub fn hash_image(buffer: &[u8]) -> Result<VeracityHash, ImageError> {
    let reader = Reader::new(Cursor::new(buffer)).with_guessed_format()?;
    match reader.format() {
        Some(ImageFormat::Jpeg | ImageFormat::Png) => {
            match reader.decode() {
                Ok(image) => {
                    let perceptual_hash = blockhash256(&image).to_string();
                    let crypto_hash = crypto_image(&image);
                    Ok(VeracityHash{
                        perceptual_hash,
                        crypto_hash
                    })
                }
                Err(e) => {
                    error!("{}", e.to_string());
                    Err(e)
                }
            }
        },
        Some(format) => {
            Err(ImageError::Unsupported(UnsupportedError::from(ImageFormatHint::Exact(format))))
        }
        None => Err(ImageError::Unsupported(UnsupportedError::from(ImageFormatHint::Unknown)))
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
    use base64::Engine;
    use base64::engine::general_purpose;
    use super::*;
    use blockhash::Blockhash256;

    #[test]
    fn blockhash_persistent_hash() {

        let known_hash = Blockhash256::from([
            0x9c, 0xfd, 0xe0, 0x3d, 0xc4, 0x19, 0x84, 0x67,
            0xad, 0x67, 0x1d, 0x17, 0x1c, 0x07, 0x1c, 0x5b,
            0x1f, 0xf8, 0x1b, 0xf9, 0x19, 0xd9, 0x18, 0x18,
            0x38, 0xf8, 0xf8, 0x90, 0xf8, 0x07, 0xff, 0x01,
        ]);

        let img = image::open("resources/test/test_495kb.png").unwrap();
        let hash = blockhash256(&img);

        assert_eq!(
            hash,
            known_hash,
        );
    }

    #[test]
    fn blockhash_same_between_formats() {

        // baseline
        let img_png = image::open("resources/test/test_495kb.png").unwrap();
        let hash1 = blockhash256(&img_png);

        let img_jpg = image::open("resources/test/test_from_495kb_png.jpg").unwrap();
        let hash2 = blockhash256(&img_jpg);

        assert_eq!(
            hash1,
            hash2,
        );

        // monochrome image
        let img_png = image::open("resources/test/test_1050kb.png").unwrap();
        let hash1 = blockhash256(&img_png);

        let img_jpg = image::open("resources/test/test_from_1050kb_png.jpg").unwrap();
        let hash2 = blockhash256(&img_jpg);

        assert_eq!(
            hash1,
            hash2,
        );

        // large jpg -> larger png
        let large_png = image::open("resources/test/test_from_2890kb_jpg.png").unwrap();
        let hash_large_png = blockhash256(&large_png);

        let large_jpg = image::open("resources/test/test_2890kb.jpg").unwrap();
        let hash_large_jpg = blockhash256(&large_jpg);

        assert_eq!(
            hash_large_png,
            hash_large_jpg,
        );

    }

    #[test]
    fn test_crypto_hash() {
        let img = image::open("resources/test/test_22kb.jpg").unwrap();

        let digest = crypto_image(&img);

        assert_eq!(digest.as_str(), "8Use8SlvsNQYnn1N68nybcgTSizRNdy6LtFuOQKdLJk")
    }

    fn write_hex(bytes: &[u8]) -> String {
        let mut s = String::with_capacity(2 * bytes.len());
        for byte in bytes {
            write!(s, "{:02X}", byte).expect("string should be writeable");
        }
        s
    }
}
