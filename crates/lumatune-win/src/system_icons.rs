use windows::Win32::Graphics::Gdi::{
    CLIP_DEFAULT_PRECIS, CreateCompatibleDC, CreateFontW, DEFAULT_CHARSET, DEFAULT_QUALITY,
    DeleteDC, DeleteObject, FIXED, FW_NORMAL, GDI_ERROR, GET_GLYPH_OUTLINE_FORMAT,
    GGI_MARK_NONEXISTING_GLYPHS, GGO_GLYPH_INDEX, GGO_GRAY8_BITMAP, GLYPHMETRICS, GetGlyphIndicesW,
    GetGlyphOutlineW, GetTextFaceW, HDC, MAT2, OUT_TT_ONLY_PRECIS, SelectObject,
};
use windows::core::{PCWSTR, w};

pub const SIZE: usize = 64;

/// Rasterize system symbol glyphs onto square alpha masks.
pub fn render<const N: usize>(codes: [u16; N]) -> [Option<Vec<u8>>; N] {
    let mut icons = std::array::from_fn(|_| None);
    // SAFETY: the memory DC and fonts are local to this thread. Restore the selected
    // object before deleting each font, and release the DC before returning.
    unsafe {
        let dc = CreateCompatibleDC(None);
        if dc.0.is_null() {
            return icons;
        }
        for (family, name) in [
            (w!("Segoe Fluent Icons"), "Segoe Fluent Icons"),
            (w!("Segoe MDL2 Assets"), "Segoe MDL2 Assets"),
        ] {
            let font = CreateFontW(
                -(SIZE as i32),
                0,
                0,
                0,
                FW_NORMAL.0 as i32,
                0,
                0,
                0,
                DEFAULT_CHARSET,
                OUT_TT_ONLY_PRECIS,
                CLIP_DEFAULT_PRECIS,
                DEFAULT_QUALITY,
                0,
                family,
            );
            if font.0.is_null() {
                continue;
            }
            let previous = SelectObject(dc, font.into());
            if !previous.0.is_null() {
                let mut face = [0; 64];
                if GetTextFaceW(dc, Some(&mut face)) > 0 {
                    let length = face
                        .iter()
                        .position(|&unit| unit == 0)
                        .unwrap_or(face.len());
                    if String::from_utf16_lossy(&face[..length]).eq_ignore_ascii_case(name) {
                        for (code, icon) in codes.iter().zip(&mut icons) {
                            if icon.is_none() {
                                *icon = render_glyph(dc, code);
                            }
                        }
                    }
                }
                SelectObject(dc, previous);
            }
            let _ = DeleteObject(font.into());
            if icons.iter().all(Option::is_some) {
                break;
            }
        }
        let _ = DeleteDC(dc);
    }
    icons
}

fn render_glyph(dc: HDC, code: &u16) -> Option<Vec<u8>> {
    let mut glyph = 0u16;
    // SAFETY: the DC has a live system symbol font selected. One UTF-16 unit
    // and one output glyph index are supplied for the requested length of one.
    let found =
        unsafe { GetGlyphIndicesW(dc, PCWSTR(code), 1, &mut glyph, GGI_MARK_NONEXISTING_GLYPHS) };
    if found != 1 || glyph == u16::MAX {
        return None;
    }
    let mut metrics = GLYPHMETRICS::default();
    let identity = MAT2 {
        eM11: FIXED { value: 1, fract: 0 },
        eM22: FIXED { value: 1, fract: 0 },
        ..MAT2::default()
    };
    let format = GET_GLYPH_OUTLINE_FORMAT(GGO_GRAY8_BITMAP.0 | GGO_GLYPH_INDEX.0);
    // SAFETY: the first call queries the byte count. The second passes a
    // buffer of that exact size using the same font, glyph and transform.
    let length =
        unsafe { GetGlyphOutlineW(dc, glyph.into(), format, &mut metrics, 0, None, &identity) };
    if length == 0 || length == GDI_ERROR as u32 {
        return None;
    }
    let mut grayscale = vec![0u8; length as usize];
    let written = unsafe {
        GetGlyphOutlineW(
            dc,
            glyph.into(),
            format,
            &mut metrics,
            length,
            Some(grayscale.as_mut_ptr().cast()),
            &identity,
        )
    };
    let width = metrics.gmBlackBoxX as usize;
    let height = metrics.gmBlackBoxY as usize;
    if width == 0 || height == 0 || width > SIZE || height > SIZE {
        return None;
    }
    // Gray8 rows occupy a multiple of four bytes and contain coverage from 0 to 64.
    let stride = width.next_multiple_of(4);
    if written == GDI_ERROR as u32 || grayscale.len() < stride * height {
        return None;
    }
    let mut mask = vec![0; SIZE * SIZE];
    let x = (SIZE - width) / 2;
    let y = (SIZE - height) / 2;
    for (row, source) in grayscale.chunks_exact(stride).take(height).enumerate() {
        let start = (y + row) * SIZE + x;
        for (alpha, coverage) in mask[start..start + width].iter_mut().zip(source) {
            *alpha = (u16::from(*coverage) * 255 / 64) as u8;
        }
    }
    Some(mask)
}
