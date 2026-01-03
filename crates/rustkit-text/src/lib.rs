//! # RustKit Text
//!
//! RustKit-owned access to fonts, metrics, and glyph indices.
//!
//! Bravo 2 goal: remove the external `dwrote` crate usage by using DirectWrite via the `windows` crate.
//!
//! Current scope:
//! - System font collection lookup by family name
//! - Match a font by weight/stretch/style
//! - Create font face
//! - Read font metrics (design units)
//! - Map Unicode codepoints -> glyph indices
//! - Read design glyph metrics (advance widths)

use thiserror::Error;

/// Errors for rustkit-text operations.
#[derive(Error, Debug, Clone)]
pub enum TextBackendError {
    #[error("Not implemented on this platform")]
    NotImplemented,

    #[error("DirectWrite error: {0}")]
    DirectWrite(String),

    #[error("Font not found: {0}")]
    FontNotFound(String),
}

/// Font style.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontStyle {
    Normal,
    Italic,
    Oblique,
}

/// Font weight (DirectWrite-compatible numeric weight).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FontWeight(pub u32);

impl FontWeight {
    pub fn from_u32(v: u32) -> Self {
        Self(v)
    }
}

/// Font stretch (DirectWrite-compatible numeric stretch).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FontStretch(pub u32);

impl FontStretch {
    pub fn from_u32(v: u32) -> Self {
        Self(v)
    }
}

/// Font metrics in design units.
#[derive(Debug, Clone, Copy)]
pub struct FontMetrics {
    pub design_units_per_em: u16,
    pub ascent: u16,
    pub descent: u16,
    pub line_gap: i16,
    pub underline_position: i16,
    pub underline_thickness: u16,
    pub strikethrough_position: i16,
    pub strikethrough_thickness: u16,
}

/// Glyph metrics in design units.
#[derive(Debug, Clone, Copy)]
pub struct GlyphMetrics {
    pub advance_width: i32,
}

#[cfg(windows)]
mod win;

#[cfg(windows)]
pub use win::{FontCollection, FontFace, FontFamily, Font};

#[cfg(not(windows))]
mod nowin {
    use super::*;

    #[derive(Clone)]
    pub struct FontCollection;
    pub struct FontFamily;
    pub struct Font;
    #[derive(Clone)]
    pub struct FontFace;

    impl FontCollection {
        pub fn system() -> Result<Self, TextBackendError> {
            Err(TextBackendError::NotImplemented)
        }

        pub fn font_family_by_name(&self, _name: &str) -> Result<Option<FontFamily>, TextBackendError> {
            Err(TextBackendError::NotImplemented)
        }
    }

    impl FontFamily {
        pub fn first_matching_font(
            &self,
            _weight: FontWeight,
            _stretch: FontStretch,
            _style: FontStyle,
        ) -> Result<Font, TextBackendError> {
            Err(TextBackendError::NotImplemented)
        }
    }

    impl Font {
        pub fn create_font_face(&self) -> Result<FontFace, TextBackendError> {
            Err(TextBackendError::NotImplemented)
        }
    }

    impl FontFace {
        pub fn metrics(&self) -> Result<FontMetrics, TextBackendError> {
            Err(TextBackendError::NotImplemented)
        }

        pub fn glyph_indices(&self, _codepoints: &[u32]) -> Result<Vec<u16>, TextBackendError> {
            Err(TextBackendError::NotImplemented)
        }

        pub fn design_glyph_metrics(
            &self,
            _glyph_indices: &[u16],
            _is_sideways: bool,
        ) -> Result<Vec<GlyphMetrics>, TextBackendError> {
            Err(TextBackendError::NotImplemented)
        }
    }
}

#[cfg(not(windows))]
pub use nowin::{FontCollection, FontFace, FontFamily, Font};


