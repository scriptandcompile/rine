//! Decode a raw icon payload (RT_ICON data) into an RGBA image.
//!
//! An RT_ICON entry contains either:
//! - A PNG file (detected by the PNG magic bytes `\x89PNG`).
//! - A DIB (device-independent bitmap) starting with BITMAPINFOHEADER.
//!   The height in the header is `2 × actual_height` to accommodate the
//!   XOR+AND masks; we only decode the XOR portion.
//!
//! Supported DIB bit depths: 32, 24, 8, 4.
//! 32bpp icons carry per-pixel alpha in the BGRA bytes; lower depths use the
//! AND mask for transparency.

use image::RgbaImage;

use crate::ThumbnailError;

const PNG_MAGIC: &[u8; 8] = b"\x89PNG\r\n\x1a\n";
const BIH_SIZE: usize = 40; // BITMAPINFOHEADER

pub fn decode_icon_to_rgba(
    icon: &crate::pe_resources::IconData,
    expected_w: u32,
    expected_h: u32,
) -> Result<RgbaImage, ThumbnailError> {
    let data = &icon.data;
    if data.len() < 4 {
        return Err(ThumbnailError::MalformedResource);
    }

    if data.starts_with(PNG_MAGIC) {
        return decode_png(data);
    }

    decode_dib(data, expected_w, expected_h)
}

// ── PNG ──────────────────────────────────────────────────────────────────────

fn decode_png(data: &[u8]) -> Result<RgbaImage, ThumbnailError> {
    let img = image::load_from_memory_with_format(data, image::ImageFormat::Png)
        .map_err(|e| ThumbnailError::DecodeFailure(e.to_string()))?;

    let w = img.width();
    let h = img.height();
    if w > crate::MAX_IMAGE_DIM || h > crate::MAX_IMAGE_DIM {
        return Err(ThumbnailError::DecodeFailure(format!(
            "PNG icon dimensions {w}×{h} exceed maximum allowed {}",
            crate::MAX_IMAGE_DIM
        )));
    }

    Ok(img.to_rgba8())
}

// ── DIB ──────────────────────────────────────────────────────────────────────

fn read_u16_le(data: &[u8], off: usize) -> Option<u16> {
    let b = data.get(off..off + 2)?;
    Some(u16::from_le_bytes([b[0], b[1]]))
}

fn read_i32_le(data: &[u8], off: usize) -> Option<i32> {
    let b = data.get(off..off + 4)?;
    Some(i32::from_le_bytes([b[0], b[1], b[2], b[3]]))
}

fn read_u32_le(data: &[u8], off: usize) -> Option<u32> {
    let b = data.get(off..off + 4)?;
    Some(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
}

/// Byte-align a value upward to a multiple of 4.
fn align4(n: usize) -> Option<usize> {
    n.checked_add(3).map(|v| v & !3)
}

fn decode_dib(
    data: &[u8],
    _expected_w: u32,
    _expected_h: u32,
) -> Result<RgbaImage, ThumbnailError> {
    if data.len() < BIH_SIZE {
        return Err(ThumbnailError::MalformedResource);
    }

    let bi_size = read_u32_le(data, 0).ok_or(ThumbnailError::MalformedResource)?;
    if bi_size < 40 {
        return Err(ThumbnailError::UnsupportedFormat);
    }

    let width = read_i32_le(data, 4).ok_or(ThumbnailError::MalformedResource)?;
    let raw_height = read_i32_le(data, 8).ok_or(ThumbnailError::MalformedResource)?;
    let bit_count = read_u16_le(data, 14).ok_or(ThumbnailError::MalformedResource)?;
    let compression = read_u32_le(data, 16).ok_or(ThumbnailError::MalformedResource)?;
    let clr_used = read_u32_le(data, 32).ok_or(ThumbnailError::MalformedResource)?;

    if width <= 0 || raw_height == 0 {
        return Err(ThumbnailError::MalformedResource);
    }
    // DIB icons store height * 2 to include AND mask.
    let height = (raw_height.unsigned_abs() / 2) as usize;
    let width = width.unsigned_abs() as usize;

    if width > crate::MAX_IMAGE_DIM as usize || height > crate::MAX_IMAGE_DIM as usize {
        return Err(ThumbnailError::DecodeFailure(format!(
            "DIB icon dimensions {width}×{height} exceed maximum allowed {}",
            crate::MAX_IMAGE_DIM
        )));
    }

    // Only BI_RGB (0) is supported.
    if compression != 0 {
        return Err(ThumbnailError::UnsupportedFormat);
    }

    let header_size = bi_size as usize;

    match bit_count {
        32 => decode_dib_32(data, header_size, width, height),
        24 => decode_dib_24(data, header_size, width, height),
        8 => {
            let palette_count = if clr_used == 0 {
                256
            } else {
                clr_used as usize
            };
            decode_dib_8(data, header_size, width, height, palette_count)
        }
        4 => {
            let palette_count = if clr_used == 0 { 16 } else { clr_used as usize };
            decode_dib_4(data, header_size, width, height, palette_count)
        }
        _ => Err(ThumbnailError::UnsupportedFormat),
    }
}

/// 32bpp BGRA – alpha is embedded in the pixel data.
fn decode_dib_32(
    data: &[u8],
    header_size: usize,
    width: usize,
    height: usize,
) -> Result<RgbaImage, ThumbnailError> {
    let stride = width * 4;
    let pixel_bytes = stride * height;
    let pixel_start = header_size;
    let pixel_end = pixel_start
        .checked_add(pixel_bytes)
        .ok_or(ThumbnailError::MalformedResource)?;

    if pixel_end > data.len() {
        return Err(ThumbnailError::MalformedResource);
    }

    let pixels = &data[pixel_start..pixel_end];
    let mut img = RgbaImage::new(width as u32, height as u32);

    for row in 0..height {
        // DIB rows are stored bottom-up.
        let src_row = height - 1 - row;
        let row_start = src_row * stride;
        for col in 0..width {
            let off = row_start + col * 4;
            let b = pixels[off];
            let g = pixels[off + 1];
            let r = pixels[off + 2];
            let a = pixels[off + 3];
            img.put_pixel(col as u32, row as u32, image::Rgba([r, g, b, a]));
        }
    }

    // If all alpha values are zero the icon stores opacity in the AND mask
    // rather than the alpha channel; treat as fully opaque.
    let all_zero_alpha = img.pixels().all(|p| p.0[3] == 0);
    if all_zero_alpha {
        for pixel in img.pixels_mut() {
            pixel.0[3] = 255;
        }
    }

    Ok(img)
}

/// 24bpp BGR, no alpha; AND mask provides transparency.
fn decode_dib_24(
    data: &[u8],
    header_size: usize,
    width: usize,
    height: usize,
) -> Result<RgbaImage, ThumbnailError> {
    let stride = align4(width * 3).ok_or(ThumbnailError::MalformedResource)?;
    let pixel_bytes = stride * height;
    let pixel_start = header_size;
    let pixel_end = pixel_start
        .checked_add(pixel_bytes)
        .ok_or(ThumbnailError::MalformedResource)?;

    if pixel_end > data.len() {
        return Err(ThumbnailError::MalformedResource);
    }

    let pixels = &data[pixel_start..pixel_end];

    // AND mask immediately follows XOR data; 1bpp, stride = ceil(width/8) aligned to 4.
    let mask_row_stride = align4(width.div_ceil(8)).ok_or(ThumbnailError::MalformedResource)?;
    let mask_bytes = mask_row_stride * height;
    let mask_end = pixel_end
        .checked_add(mask_bytes)
        .ok_or(ThumbnailError::MalformedResource)?;
    let mask = if mask_end <= data.len() {
        Some(&data[pixel_end..pixel_end + mask_bytes])
    } else {
        None
    };

    let mut img = RgbaImage::new(width as u32, height as u32);
    for row in 0..height {
        let src_row = height - 1 - row;
        let row_start = src_row * stride;
        let mask_row_start = src_row * mask_row_stride;
        for col in 0..width {
            let off = row_start + col * 3;
            let b = pixels[off];
            let g = pixels[off + 1];
            let r = pixels[off + 2];
            let alpha = if let Some(m) = mask {
                let byte = m[mask_row_start + col / 8];
                let bit = 7 - (col % 8);
                if (byte >> bit) & 1 == 0 { 255 } else { 0 }
            } else {
                255
            };
            img.put_pixel(col as u32, row as u32, image::Rgba([r, g, b, alpha]));
        }
    }
    Ok(img)
}

/// 8bpp with 256-entry RGBQUAD palette.
fn decode_dib_8(
    data: &[u8],
    header_size: usize,
    width: usize,
    height: usize,
    palette_count: usize,
) -> Result<RgbaImage, ThumbnailError> {
    let palette_bytes = palette_count * 4;
    let palette_start = header_size;
    let palette_end = palette_start
        .checked_add(palette_bytes)
        .ok_or(ThumbnailError::MalformedResource)?;
    if palette_end > data.len() {
        return Err(ThumbnailError::MalformedResource);
    }
    let palette = &data[palette_start..palette_end];

    let stride = align4(width).ok_or(ThumbnailError::MalformedResource)?;
    let pixel_bytes = stride * height;
    let pixel_start = palette_end;
    let pixel_end = pixel_start
        .checked_add(pixel_bytes)
        .ok_or(ThumbnailError::MalformedResource)?;
    if pixel_end > data.len() {
        return Err(ThumbnailError::MalformedResource);
    }
    let pixels = &data[pixel_start..pixel_end];

    let mask_row_stride = align4(width.div_ceil(8)).ok_or(ThumbnailError::MalformedResource)?;
    let mask_bytes = mask_row_stride * height;
    let mask_end = pixel_end
        .checked_add(mask_bytes)
        .ok_or(ThumbnailError::MalformedResource)?;
    let mask = if mask_end <= data.len() {
        Some(&data[pixel_end..pixel_end + mask_bytes])
    } else {
        None
    };

    let mut img = RgbaImage::new(width as u32, height as u32);
    for row in 0..height {
        let src_row = height - 1 - row;
        let row_start = src_row * stride;
        let mask_row_start = src_row * mask_row_stride;
        for col in 0..width {
            let idx = pixels[row_start + col] as usize;
            if idx >= palette_count {
                return Err(ThumbnailError::MalformedResource);
            }
            let pe = &palette[idx * 4..idx * 4 + 4];
            let (b, g, r) = (pe[0], pe[1], pe[2]);
            let alpha = if let Some(m) = mask {
                let byte = m[mask_row_start + col / 8];
                let bit = 7 - (col % 8);
                if (byte >> bit) & 1 == 0 { 255 } else { 0 }
            } else {
                255
            };
            img.put_pixel(col as u32, row as u32, image::Rgba([r, g, b, alpha]));
        }
    }
    Ok(img)
}

/// 4bpp with 16-entry RGBQUAD palette.
fn decode_dib_4(
    data: &[u8],
    header_size: usize,
    width: usize,
    height: usize,
    palette_count: usize,
) -> Result<RgbaImage, ThumbnailError> {
    let palette_bytes = palette_count * 4;
    let palette_start = header_size;
    let palette_end = palette_start
        .checked_add(palette_bytes)
        .ok_or(ThumbnailError::MalformedResource)?;
    if palette_end > data.len() {
        return Err(ThumbnailError::MalformedResource);
    }
    let palette = &data[palette_start..palette_end];

    // Each row: ceil(width/2) bytes, aligned to 4.
    let row_pixels = width.div_ceil(2);
    let stride = align4(row_pixels).ok_or(ThumbnailError::MalformedResource)?;
    let pixel_bytes = stride * height;
    let pixel_start = palette_end;
    let pixel_end = pixel_start
        .checked_add(pixel_bytes)
        .ok_or(ThumbnailError::MalformedResource)?;
    if pixel_end > data.len() {
        return Err(ThumbnailError::MalformedResource);
    }
    let pixels = &data[pixel_start..pixel_end];

    let mask_row_stride = align4(width.div_ceil(8)).ok_or(ThumbnailError::MalformedResource)?;
    let mask_bytes = mask_row_stride * height;
    let mask_end = pixel_end
        .checked_add(mask_bytes)
        .ok_or(ThumbnailError::MalformedResource)?;
    let mask = if mask_end <= data.len() {
        Some(&data[pixel_end..pixel_end + mask_bytes])
    } else {
        None
    };

    let mut img = RgbaImage::new(width as u32, height as u32);
    for row in 0..height {
        let src_row = height - 1 - row;
        let row_start = src_row * stride;
        let mask_row_start = src_row * mask_row_stride;
        for col in 0..width {
            let byte = pixels[row_start + col / 2];
            let nibble = if col % 2 == 0 {
                (byte >> 4) & 0xF
            } else {
                byte & 0xF
            } as usize;
            if nibble >= palette_count {
                return Err(ThumbnailError::MalformedResource);
            }
            let pe = &palette[nibble * 4..nibble * 4 + 4];
            let (b, g, r) = (pe[0], pe[1], pe[2]);
            let alpha = if let Some(m) = mask {
                let mbyte = m[mask_row_start + col / 8];
                let bit = 7 - (col % 8);
                if (mbyte >> bit) & 1 == 0 { 255 } else { 0 }
            } else {
                255
            };
            img.put_pixel(col as u32, row as u32, image::Rgba([r, g, b, alpha]));
        }
    }
    Ok(img)
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pe_resources::IconData;

    fn make_icon(data: Vec<u8>, id: u16) -> IconData {
        IconData { id, data }
    }

    /// Build a minimal 2×2 32bpp DIB icon blob.
    fn minimal_32bpp_dib(pixels: &[[u8; 4]; 4]) -> Vec<u8> {
        let mut dib = Vec::new();
        // BITMAPINFOHEADER (40 bytes)
        dib.extend_from_slice(&40u32.to_le_bytes()); // biSize
        dib.extend_from_slice(&2i32.to_le_bytes()); // biWidth
        dib.extend_from_slice(&4i32.to_le_bytes()); // biHeight = 2*actual (XOR+AND)
        dib.extend_from_slice(&1u16.to_le_bytes()); // biPlanes
        dib.extend_from_slice(&32u16.to_le_bytes()); // biBitCount
        dib.extend_from_slice(&0u32.to_le_bytes()); // biCompression
        dib.extend_from_slice(&0u32.to_le_bytes()); // biSizeImage
        dib.extend_from_slice(&0i32.to_le_bytes()); // biXPelsPerMeter
        dib.extend_from_slice(&0i32.to_le_bytes()); // biYPelsPerMeter
        dib.extend_from_slice(&0u32.to_le_bytes()); // biClrUsed
        dib.extend_from_slice(&0u32.to_le_bytes()); // biClrImportant
        // XOR data: 4 pixels × 4 bytes (BGRA), bottom row first
        for px in pixels.iter() {
            dib.extend_from_slice(px); // stored as BGRA
        }
        // AND mask: 2×2 → 1bpp → 1 byte per row, padded to 4 bytes = 8 bytes total
        dib.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // row 0 (bottom)
        dib.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // row 1 (top)
        dib
    }

    #[test]
    fn decode_32bpp_bgra_to_rgba() {
        // Bottom row: pixel(0,1)=[B=0,G=128,R=255,A=200], pixel(1,1)=[B=0,G=0,R=0,A=255]
        // Top row:    pixel(0,0)=[...], pixel(1,0)=[...]
        let pixels: [[u8; 4]; 4] = [
            [0, 128, 255, 200], // bottom-left stored first → displayed as (col=0,row=1)
            [0, 0, 0, 255],     // bottom-right
            [10, 20, 30, 128],  // top-left
            [40, 50, 60, 64],   // top-right
        ];
        let dib = minimal_32bpp_dib(&pixels);
        let icon = make_icon(dib, 1);
        let img = decode_icon_to_rgba(&icon, 2, 2).expect("decode failed");
        assert_eq!(img.dimensions(), (2, 2));
        // Bottom-left pixel stored first in DIB → row=1 in top-down image.
        // BGRA [0,128,255,200] → RGBA [255,128,0,200]
        let p = img.get_pixel(0, 1);
        assert_eq!(p.0, [255, 128, 0, 200]);
    }

    #[test]
    fn decode_32bpp_zero_alpha_treated_as_opaque() {
        let pixels: [[u8; 4]; 4] = [
            [255, 0, 0, 0], // BGRA blue, alpha=0
            [0, 255, 0, 0],
            [0, 0, 255, 0],
            [128, 128, 128, 0],
        ];
        let dib = minimal_32bpp_dib(&pixels);
        let icon = make_icon(dib, 1);
        let img = decode_icon_to_rgba(&icon, 2, 2).expect("decode failed");
        // All alpha values should be 255 since input was all-zero.
        for px in img.pixels() {
            assert_eq!(px.0[3], 255, "expected opaque pixel");
        }
    }

    #[test]
    fn rejects_oversized_dib() {
        let mut dib = Vec::new();
        dib.extend_from_slice(&40u32.to_le_bytes()); // biSize
        dib.extend_from_slice(&2048i32.to_le_bytes()); // width > MAX_IMAGE_DIM
        dib.extend_from_slice(&4096i32.to_le_bytes()); // raw_height
        dib.extend_from_slice(&1u16.to_le_bytes()); // planes
        dib.extend_from_slice(&32u16.to_le_bytes()); // bits
        dib.extend_from_slice(&[0u8; 24]); // rest of header
        let icon = make_icon(dib, 1);
        assert!(matches!(
            decode_icon_to_rgba(&icon, 2048, 2048),
            Err(ThumbnailError::DecodeFailure(_))
        ));
    }

    #[test]
    fn rejects_truncated_data() {
        let mut dib = minimal_32bpp_dib(&[[255, 0, 0, 255]; 4]);
        dib.truncate(30); // truncate mid-header
        let icon = make_icon(dib, 1);
        assert!(matches!(
            decode_icon_to_rgba(&icon, 2, 2),
            Err(ThumbnailError::MalformedResource)
        ));
    }
}
