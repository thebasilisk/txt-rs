use freetype::{
    FT_Error,
    freetype::{
        FT_Face, FT_Init_FreeType, FT_LOAD_RENDER, FT_Library, FT_Load_Char, FT_New_Face,
        FT_Set_Char_Size, FT_Set_Pixel_Sizes,
    },
    succeeded,
};
use std::{ptr, slice};

pub fn init_ft_lib() -> Result<FT_Library, FT_Error> {
    let mut lib = ptr::null_mut();
    let result = unsafe { FT_Init_FreeType(&mut lib) };

    if succeeded(result) {
        Ok(lib)
    } else {
        Err(result)
    }
}

pub fn load_typeface(lib: FT_Library, name: &str) -> Result<FT_Face, FT_Error> {
    let mut face: FT_Face = ptr::null_mut();
    let filepath = format!("./resources/{name}.ttf");
    // let filepath = format!("./resources/Arial.ttf");
    let result = unsafe { FT_New_Face(lib, filepath.as_ptr() as *const _, 0, &mut face) };
    if succeeded(result) {
        Ok(face)
    } else {
        println!("{result}");
        println!("Error loading typeface");
        Err(result)
    }
}

pub fn get_char_glyph(font: &str, character: char) -> Result<(*mut u8, u64, u64), FT_Error> {
    let lib = init_ft_lib()?;
    let face = load_typeface(lib.clone(), font)?;
    let result = unsafe { FT_Set_Pixel_Sizes(face, 300, 300) };
    if !succeeded(result) {
        return Err(result);
    }

    let result = unsafe { FT_Load_Char(face, character as u64, FT_LOAD_RENDER as i32) };

    if !succeeded(result) {
        return Err(result);
    }

    let slot = unsafe { (*face).glyph };
    let buffer = unsafe { (*slot).bitmap.buffer };
    let width = unsafe { (*slot).bitmap.width };
    let height = unsafe { (*slot).bitmap.rows };
    println!("{width}, {height}");

    // let vector = unsafe { slice::from_raw_parts_mut(buffer, size as usize) }.to_vec();
    Ok((buffer, width as u64, height as u64))
}
