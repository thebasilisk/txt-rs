use freetype::{
    Face, FtResult,
    ffi::{FT_BBox, FT_Vector},
};
use metal::*;

use crate::text::freetype::get_char_glyph;

pub const ASCII_START: u8 = 32;
pub const NUM_ASCII_CHARS: u8 = 96;

pub struct Atlas {
    pub texture: Texture,
    pub max_width: u64,
    pub max_height: u64,
    // bitmaps: Vec<Vec<u8>>,
    pub advances: Vec<FT_Vector>,
    pub cboxes: Vec<FT_BBox>,
}

//should maybe be an impl on atlas
pub fn create_texture_atlas(face: &Face, device: &DeviceRef) -> FtResult<Atlas> {
    let mut all_char_bitmaps = Vec::new();
    let mut all_char_widths = Vec::new();
    let mut all_char_heights = Vec::new();
    let mut all_char_advances = Vec::new();
    let mut all_char_cboxes = Vec::new();

    let mut max_width = 0;
    let mut max_height = 0;
    for i in ASCII_START..ASCII_START + NUM_ASCII_CHARS {
        // println!("{i}");
        let glyph_data = get_char_glyph(face, i.into())?;
        max_width = max_width.max(glyph_data.width);
        max_height = max_height.max(glyph_data.height);
        all_char_bitmaps.push(glyph_data.bitmap);
        all_char_widths.push(glyph_data.width);
        all_char_heights.push(glyph_data.height);
        all_char_advances.push(glyph_data.advance);
        all_char_cboxes.push(glyph_data.cbox);
    }
    let atlas_descriptor = TextureDescriptor::new();
    let tex_height = (NUM_ASCII_CHARS as u64 + 1) * max_height; // +1 is a bad way to add space character maybe
    atlas_descriptor.set_pixel_format(MTLPixelFormat::R8Unorm);
    atlas_descriptor.set_width(max_width);
    atlas_descriptor.set_height(tex_height);

    let texture = device.new_texture(&atlas_descriptor);

    for i in 0..NUM_ASCII_CHARS as usize {
        let height_diff = max_height - all_char_heights[i];
        let region = MTLRegion::new_2d(
            0,
            max_height * i as u64 + height_diff,
            all_char_widths[i],
            all_char_heights[i],
        );
        if all_char_widths[i] <= 0 || all_char_heights[i] <= 0 {
            continue;
        }
        texture.replace_region(
            region,
            0,
            all_char_bitmaps[i].as_ptr() as *const _,
            all_char_widths[i] * 1,
        );
    }

    // let atlas = Atlas{ texture, max_width, max_height, bitmaps: all_char_bitmaps, advances: all_char_advances };
    let atlas = Atlas {
        texture,
        max_width,
        max_height,
        advances: all_char_advances,
        cboxes: all_char_cboxes,
    };
    Ok(atlas)
}
