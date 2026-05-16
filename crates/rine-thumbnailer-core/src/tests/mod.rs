//! Integration-level tests that construct minimal synthetic PE binaries and
//! exercise the full extraction pipeline.

use crate::{generate_png_thumbnail_from_bytes, ThumbnailError};

mod pe_builder;

// ── Helpers ──────────────────────────────────────────────────────────────────

fn make_2x2_32bpp_dib() -> Vec<u8> {
    let mut dib = Vec::new();
    dib.extend_from_slice(&40u32.to_le_bytes()); // biSize
    dib.extend_from_slice(&2i32.to_le_bytes()); // biWidth
    dib.extend_from_slice(&4i32.to_le_bytes()); // biHeight (×2 for AND mask)
    dib.extend_from_slice(&1u16.to_le_bytes()); // biPlanes
    dib.extend_from_slice(&32u16.to_le_bytes()); // biBitCount
    dib.extend_from_slice(&[0u8; 24]); // rest of BITMAPINFOHEADER
    // 4 BGRA pixels
    for _ in 0..4 {
        dib.extend_from_slice(&[0u8, 128u8, 255u8, 200u8]); // B G R A
    }
    // AND mask: 2 rows × 4-byte-aligned
    dib.extend_from_slice(&[0u8; 8]);
    dib
}

fn make_16x16_32bpp_dib() -> Vec<u8> {
    let w: usize = 16;
    let h: usize = 16;
    let mut dib = Vec::new();
    dib.extend_from_slice(&40u32.to_le_bytes());
    dib.extend_from_slice(&(w as i32).to_le_bytes());
    dib.extend_from_slice(&((h * 2) as i32).to_le_bytes());
    dib.extend_from_slice(&1u16.to_le_bytes());
    dib.extend_from_slice(&32u16.to_le_bytes());
    dib.extend_from_slice(&[0u8; 24]);
    for i in 0..(w * h) {
        let v = (i & 0xFF) as u8;
        dib.extend_from_slice(&[v, v, v, 255u8]);
    }
    // AND mask: h rows × align4(ceil(w/8)) bytes
    let mask_stride = (w.div_ceil(8) + 3) & !3;
    dib.extend_from_slice(&vec![0u8; mask_stride * h]);
    dib
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[test]
fn full_pipeline_2x2_32bpp() {
    let icon_data = make_2x2_32bpp_dib();
    // Build a PE with one group icon (2×2, 32bpp) referencing RT_ICON id=1.
    let pe_bytes = pe_builder::build_pe_with_single_icon(2, 2, 32, &icon_data);
    let png = generate_png_thumbnail_from_bytes(&pe_bytes, 2).expect("thumbnail failed");

    // Verify it is a valid PNG.
    assert!(png.starts_with(b"\x89PNG"), "output should be a PNG");
}

#[test]
fn full_pipeline_16x16_scaled_to_32() {
    let icon_data = make_16x16_32bpp_dib();
    let pe_bytes = pe_builder::build_pe_with_single_icon(16, 16, 32, &icon_data);
    let png = generate_png_thumbnail_from_bytes(&pe_bytes, 32).expect("thumbnail failed");
    assert!(png.starts_with(b"\x89PNG"));
}

#[test]
fn no_icon_resource_returns_error() {
    // A minimal valid PE with no resource section at all.
    let pe_bytes = pe_builder::build_pe_no_resources();
    assert!(matches!(
        generate_png_thumbnail_from_bytes(&pe_bytes, 32),
        Err(ThumbnailError::NoIconResource)
    ));
}

#[test]
fn malformed_bytes_return_error() {
    let garbage = b"This is not a PE file at all!!";
    assert!(matches!(
        generate_png_thumbnail_from_bytes(garbage, 32),
        Err(ThumbnailError::MalformedResource)
    ));
}
