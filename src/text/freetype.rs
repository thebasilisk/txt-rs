use freetype::{
    Face, FtResult, Library,
    face::LoadFlag,
    ffi::{FT_BBox, FT_Vector},
};

pub struct GlyphData {
    pub bitmap: Vec<u8>,
    pub width: u64,
    pub height: u64,
    pub advance: FT_Vector,
    pub cbox: FT_BBox,
}

pub fn init_typeface_with_size(lib: &Library, name: &str, size: u32) -> FtResult<Face> {
    // let filepath = format!("./resources/{name}.ttf");
    let filepath = format!("/Users/basil/rust-projects/txt-rs/resources/{name}");
    let face = lib.new_face(filepath, 0)?;
    face.set_pixel_sizes(size, size)?;
    Ok(face)
}

pub fn get_char_glyph(face: &Face, character: char) -> FtResult<GlyphData> {
    face.load_char(character as usize, LoadFlag::RENDER)?;

    let slot = face.glyph();
    let bitmap = slot.bitmap();
    let width = slot.bitmap().width();
    let height = slot.bitmap().rows();
    let advance = slot.advance();

    let vec = if width == 0 || height == 0 {
        vec![]
    } else {
        bitmap.buffer().to_vec()
    };

    let cbox = slot.get_glyph().unwrap().get_cbox(3); //3 is FT_GLYPH_BBOX_PIXELS
    Ok(GlyphData {
        bitmap: vec,
        width: width as u64,
        height: height as u64,
        advance,
        cbox,
    })
}
