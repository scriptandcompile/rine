use rine_types::errors::WinBool;

use tracing::warn;

#[repr(u32)]
pub enum ClipboardFormat {
    /// Text Format. Each line ends with a carriage return and linefeed character combination.
    /// A null character signals the end of the data.
    /// USed for ANSI text.
    Text = 1,
    /// A handle to a bitmap (HBITMAP) in the clipboard.
    Bitmap = 2,
    /// A handle to a metafile picture (HMETAFILEPICT) in the clipboard as defined by the METAFILEPICT structure.
    /// When passing a metafilePict handle by means of DDE, the application responsible for delete the hMem should
    /// also free the metafile referred to by the MetafilePict handle.
    MetafilePict = 3,
    /// Microsoft Symbolic Link (SYLK) format.
    Sylk = 4,
    /// Software Arts' Data Interchange Format.
    Dif = 5,
    /// Tagged Image File Format.
    Tiff = 6,
    /// Text format containing characters in the OEM character set.
    /// Each line ends with a carriage return and linefeed character combination.
    /// A null character signals the end of the data.
    OemText = 7,
    /// A memory object containing a BITMAPVSHEADER structure followed by the bitmap color space information and the bitmap bits.
    DIB = 8,
    /// Handle to a color palette.
    /// Whenever an application places data in the clipboard that depends on or assumes a color palette, it should place
    /// the palette on the clipboard as well.
    /// If the clipboard contains data in the CF_PALETTE (logical color palette) format, the application should use the
    /// SelectPalette and RealizePalette functions to realize (compare) any other data in the clipboard against that logical palette.
    ///When displaying clipboard data, the clipboard always uses as its current palette any object on the clipboard that is in
    /// the CF_PALETTE format.
    Palette = 9,
    /// Data for the pen extensions to the Microsoft Windows for Pen Computing.
    Pendata = 10,
    /// Represents audio data more complex than can be representing a CF_WAVE format.
    /// The data is in the form of a RIFF chunk with a WAVE form type.
    Riff = 11,
    /// Audio in one of the standard wave formats.
    Wave = 12,
    /// Unicode text format.
    /// Each line ends with a carriage return/linefeed character combination.
    /// A null character signals the end of the data.
    UnicodeText = 13,
    /// A handle to an enhanced metafile (HENHMETAFILE) in the clipboard.
    EnhancedMetafile = 14,
    /// A handle to type HDROP that identifies a list of files.
    /// More information can be obtained by passing this handle to the `DragQueryFile` function.
    HDrop = 15,
    /// A Global handle to a locale identifier (LCID).
    Locale = 16,
}

impl TryFrom<u32> for ClipboardFormat {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::Text),
            2 => Ok(Self::Bitmap),
            3 => Ok(Self::MetafilePict),
            4 => Ok(Self::Sylk),
            5 => Ok(Self::Dif),
            6 => Ok(Self::Tiff),
            7 => Ok(Self::OemText),
            8 => Ok(Self::DIB),
            9 => Ok(Self::Palette),
            10 => Ok(Self::Pendata),
            11 => Ok(Self::Riff),
            12 => Ok(Self::Wave),
            13 => Ok(Self::UnicodeText),
            14 => Ok(Self::EnhancedMetafile),
            15 => Ok(Self::HDrop),
            16 => Ok(Self::Locale),
            _ => Err(()),
        }
    }
}

/// Checks if the specified clipboard format is available.
///
/// # Arguments
/// * `_format` - The clipboard format to check for availability.
///
/// # Returns
/// * `WinBool::TRUE` if the specified clipboard format is available, `WinBool::FALSE` otherwise.
pub fn is_clipboard_format_available(_format: ClipboardFormat) -> WinBool {
    warn!("is_clipboard_format_available is not implemented yet, returning false for all formats");

    WinBool::FALSE
}
