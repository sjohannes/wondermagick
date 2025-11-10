use std::io::Write;

use crate::encoders::common::to_8bit_rgb_maybe_a;
use crate::{error::MagickError, image::Image, plan::Modifiers, wm_err, wm_try};
use image::{codecs::webp::WebPEncoder, ImageEncoder};
use webp::{Encoder, WebPMemory};

pub fn encode<W: Write>(
    image: &Image,
    writer: &mut W,
    modifiers: &Modifiers,
) -> Result<(), MagickError> {
    if !modifiers
        .definitions
        .get("webp:use-libwebp")
        .map(|value| match value.as_encoded_bytes() {
            b"0" | b"false" => Ok(false),
            b"1" | b"true" => Ok(true),
            _ => Err(wm_err!("webp:use-libwebp: invalid value")),
        })
        .transpose()?
        .unwrap_or(true)
    {
        let mut encoder = WebPEncoder::new_lossless(writer);
        if let Some(icc) = image.icc.clone() {
            let _ = encoder.set_icc_profile(icc); // ignore UnsupportedError
        };
        wm_try!(image.pixels.write_with_encoder(encoder));
        return Ok(());
    }

    // Convert the image to Rgb(a)8, because those are the only formats the encoder supports
    let pixels = to_8bit_rgb_maybe_a(&image.pixels);

    // https://imagemagick.org/script/webp.php
    let mut config = webp::WebPConfig::new().unwrap();
    let defs = &modifiers.definitions;
    config.lossless = match defs.get("webp:lossless").map(|s| s.as_encoded_bytes()) {
        // If webp:lossless is unset, ImageMagick uses lossless when quality=100
        None => i32::from(modifiers.quality == Some(100.0)),
        Some(b"0" | b"false") => 0,
        Some(b"1" | b"true") => 1,
        _ => return Err(wm_err!("webp:lossless must be true, false, 1, or 0")),
    };
    if let Some(value) = defs.get("webp:method") {
        config.method = value
            .to_str()
            .and_then(|s| s.parse().ok())
            .ok_or_else(|| wm_err!("invalid webp:method value"))?;
    }
    // default quality is not documented, was determined experimentally
    // (75 is also libwebp default)
    config.quality = modifiers.quality.unwrap_or(75.0) as f32;

    // Encode the image with the specified config
    let encoder: Encoder = Encoder::from_image(&pixels).unwrap();
    let webp: WebPMemory = encoder
        .encode_advanced(&config)
        .map_err(|e| wm_err!("WebP encoding failed: {e:?}"))?;
    // TODO: `webp` crate doesn't support setting the ICC profile:
    // https://github.com/jaredforth/webp/issues/41
    wm_try!(writer.write_all(&webp));
    Ok(())
}
