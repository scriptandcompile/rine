//! Core PE icon extraction and PNG thumbnail generation.
//!
//! Parses Windows PE resource tables to extract embedded icons and renders
//! them as PNG thumbnails at a requested pixel size.

mod icon_decode;
mod icon_select;
mod pe_resources;

#[cfg(test)]
mod tests;

use thiserror::Error;

/// Maximum decoded image dimension (width or height) in pixels.
pub const MAX_IMAGE_DIM: u32 = 1024;

/// Input to thumbnail generation.
pub struct ThumbnailRequest<'a> {
    pub input_path: &'a std::path::Path,
    /// Requested output size in pixels (square).
    pub size_px: u32,
}

#[derive(Debug, Error)]
pub enum ThumbnailError {
    #[error("unsupported format")]
    UnsupportedFormat,
    #[error("PE has no icon resource")]
    NoIconResource,
    #[error("malformed PE resource data")]
    MalformedResource,
    #[error("icon decode failure: {0}")]
    DecodeFailure(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// Extract the best available icon from `path` and render it as a PNG at
/// `size_px × size_px`, returning the PNG bytes.
pub fn generate_png_thumbnail(req: &ThumbnailRequest) -> Result<Vec<u8>, ThumbnailError> {
    let file_bytes = std::fs::read(req.input_path)?;
    generate_png_thumbnail_from_bytes(&file_bytes, req.size_px)
}

/// Same as [`generate_png_thumbnail`] but operates on already-loaded bytes.
/// Useful for testing.
pub fn generate_png_thumbnail_from_bytes(
    file_bytes: &[u8],
    size_px: u32,
) -> Result<Vec<u8>, ThumbnailError> {
    let extracted = pe_resources::extract_icons(file_bytes)?;

    let entry = icon_select::select_best(&extracted.group_entries, size_px)
        .ok_or(ThumbnailError::NoIconResource)?;

    let icon_data = extracted
        .icons
        .iter()
        .find(|i| i.id == entry.id)
        .ok_or(ThumbnailError::MalformedResource)?;

    let rgba = icon_decode::decode_icon_to_rgba(icon_data, entry.width, entry.height)?;

    let resized = resize_to_square(rgba, size_px);
    encode_png(resized)
}

fn resize_to_square(img: image::RgbaImage, size_px: u32) -> image::RgbaImage {
    if img.width() == size_px && img.height() == size_px {
        return img;
    }
    let dyn_img = image::DynamicImage::ImageRgba8(img);
    dyn_img
        .resize(size_px, size_px, image::imageops::FilterType::Lanczos3)
        .to_rgba8()
}

fn encode_png(img: image::RgbaImage) -> Result<Vec<u8>, ThumbnailError> {
    use image::ImageEncoder;
    let mut buf = Vec::new();
    let encoder = image::codecs::png::PngEncoder::new(&mut buf);
    encoder
        .write_image(
            img.as_raw(),
            img.width(),
            img.height(),
            image::ExtendedColorType::Rgba8,
        )
        .map_err(|e| ThumbnailError::DecodeFailure(e.to_string()))?;
    Ok(buf)
}
