use freetype::{
    Face, FtResult,
    ffi::{FT_BBox, FT_Vector},
};
use metal::*;

use crate::{maths::Float2, text::freetype::get_char_glyph};

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

//current texture atlas impl doesn't work for large fonts, max mtltexture size is 16384
impl Atlas {
    pub fn new(face: &Face, device: &DeviceRef) -> FtResult<Atlas> {
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
        let padded_height = max_height + 1;
        let tex_height = (NUM_ASCII_CHARS as u64) * padded_height;
        atlas_descriptor.set_pixel_format(MTLPixelFormat::R8Unorm);
        atlas_descriptor.set_width(max_width);
        atlas_descriptor.set_height(tex_height);

        let texture = device.new_texture(&atlas_descriptor);

        for i in 0..NUM_ASCII_CHARS as usize {
            let height_diff = max_height - all_char_heights[i];
            let region = MTLRegion::new_2d(
                0,
                padded_height * i as u64 + height_diff, //if char height = max height the replaced region has 1 row of padding
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
            max_height: padded_height, //max height is a bad label, should rename to slot_height maybe
            advances: all_char_advances,
            cboxes: all_char_cboxes,
        };
        Ok(atlas)
    }

    pub fn get_advance(&self, index: usize, cursor: &mut Float2) -> f32 {
        self.advances[index].x as f32 / 64.0
    }
}
