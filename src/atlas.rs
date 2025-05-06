use std::char::MAX;

use crate::text::{self, freetype::get_char_glyph};
use freetype::freetype;
use ::freetype::freetype::{FT_BBox, FT_Face};
use metal::*;

pub const ASCII_START: u8 = 97;
pub const NUM_ASCII_CHARS: u8 = 26;

pub fn create_texture_atlas(
    face: FT_Face,
    device: &DeviceRef,
) -> Result<(metal::Texture, u64, u64), freetype::FT_Error> {
    let mut all_char_bitmaps = Vec::new();
    let mut all_char_widths = Vec::new();
    let mut all_char_heights = Vec::new();

    let mut max_width = 0;
    let mut max_height = 0;
    for i in ASCII_START..(ASCII_START + NUM_ASCII_CHARS) {
        // println!("{i}");
        let (bitmap, width, height) = get_char_glyph(face, i.into())?;
        max_width = max_width.max(width);
        max_height = max_height.max(height);
        all_char_bitmaps.push(bitmap);
        all_char_widths.push(width);
        all_char_heights.push(height);
    }
    let atlas_descriptor = TextureDescriptor::new();
    atlas_descriptor.set_pixel_format(MTLPixelFormat::R8Unorm);
    atlas_descriptor.set_width(max_width);
    atlas_descriptor.set_height(NUM_ASCII_CHARS as u64 * max_height);

    let text = device.new_texture(&atlas_descriptor);

    for i in 0..NUM_ASCII_CHARS as usize {
        let region = MTLRegion::new_2d(
            0,
            max_height * i as u64,
            all_char_widths[i],
            all_char_heights[i],
        );
        if all_char_widths[i] <= 0 || all_char_heights[i] <= 0 {
            continue;
        }
        text.replace_region(region, 0, all_char_bitmaps[i].as_ptr() as *const _, all_char_widths[i] * 1);
    }

    Ok((text, max_width, max_height))
}
