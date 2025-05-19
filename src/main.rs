use std::str::FromStr;

use atlas::{ASCII_START, Atlas};
use freetype::{Face, Library, ffi::FT_Vector};
use maths::{Float2, Float4, apply_rotation_float2, float2_add, float2_subtract};
use objc2::rc::autoreleasepool;
use objc2_app_kit::{NSAnyEventMask, NSEventType};
use objc2_foundation::{NSComparisonResult, NSDate, NSDefaultRunLoopMode};
use text::freetype::init_typeface_with_size;
use utils::{
    copy_to_buf, get_library, get_next_frame, init_render_with_bufs, make_buf,
    make_buf_with_capacity, new_render_pass_descriptor, prepare_pipeline_state, simple_app,
};

mod atlas;
mod maths;
mod text;
mod utils;

/*
Things to do:
    Add file i/o,
    Add cursor and ability to edit from anywhere in the text
    Add newline, tab indent, etc
    Maybe: reorganize project files

Much later things to do:
    Fix kerning / implement more sophisticated kerning
    Update backing data structure for text
    Add text search (then cross-file search)
    Add markdown support
    Add link support
    Add GUI and mouse interactivity
*/

fn main() {
    let view_width = 1024.0;
    let view_height = 768.0;
    let (app, _window, device, layer) = simple_app(view_width, view_height, "Texter");

    let shaderlib = get_library(&device);

    let render_pipeline = prepare_pipeline_state(&device, "box_vertex", "box_fragment", &shaderlib);
    let command_queue = device.new_command_queue();

    let ft_lib = Library::init().unwrap();
    let ft_face = init_typeface_with_size(&ft_lib, "Arial.ttf", 100).unwrap();
    let atlas = Atlas::new(&ft_face, &device).unwrap();

    let max_char_count = 1000;
    let text_box_size = 2000.0;

    let mut word = String::new();
    let cursor_start = Float2(-1000.0, 700.0);
    let mut cursor = cursor_start.clone();
    let unis = Uniforms {
        screen_size: Float2(view_width as f32, view_height as f32),
    };
    let uni_buf = make_buf(&vec![unis], &device);

    //initialize with dummy word so mem region isn't empty, otherwise segfaults
    let init_string = String::from_str("initial").unwrap();
    let (verts, texs) = verts_from_word(&mut cursor, &init_string, text_box_size, &atlas, &ft_face);
    let vert_buf = make_buf_with_capacity(&verts, max_char_count * 6, &device);
    let tex_buf = make_buf_with_capacity(&texs, max_char_count, &device);

    //empty afterwards
    let (verts, texs) = verts_from_word(&mut cursor, &word, text_box_size, &atlas, &ft_face);
    copy_to_buf(&verts, &vert_buf);
    copy_to_buf(&texs, &tex_buf);

    let fps = 60.0f32;
    let mut frames = 0;
    let mut frame_time = get_next_frame(fps as f64);

    loop {
        autoreleasepool(|_| {
            if app.windows().is_empty() {
                unsafe {
                    app.terminate(None);
                }
            }
            if unsafe { frame_time.compare(&NSDate::now()) } == NSComparisonResult::Ascending {
                frame_time = get_next_frame(fps as f64);
                frames += 1;

                let command_buffer = command_queue.new_command_buffer();

                let drawable = layer.next_drawable().unwrap();
                let texture = drawable.texture();
                let render_descriptor = new_render_pass_descriptor(&texture);

                let encoder = init_render_with_bufs(
                    &[&uni_buf, &vert_buf, &tex_buf],
                    &render_descriptor,
                    &render_pipeline,
                    command_buffer,
                );
                encoder.set_fragment_texture(0, Some(&atlas.texture));
                encoder.draw_primitives_instanced(
                    metal::MTLPrimitiveType::Triangle,
                    0,
                    6,                 //six verts per char
                    word.len() as u64, //num of chars
                );
                encoder.end_encoding();

                command_buffer.present_drawable(drawable);
                command_buffer.commit();
            }

            loop {
                unsafe {
                    let e = app.nextEventMatchingMask_untilDate_inMode_dequeue(
                        NSAnyEventMask,
                        None,
                        NSDefaultRunLoopMode,
                        true,
                    );
                    match e {
                        Some(ref e) => match e.r#type() {
                            NSEventType::KeyDown => {
                                let in_chars = &e.characters();
                                match in_chars {
                                    Some(str) => {
                                        cursor = cursor_start;
                                        let char =
                                            char::decode_utf16(Some(str.characterAtIndex(0)))
                                                .map(|r| r.unwrap())
                                                .collect::<Vec<char>>()[0];
                                        if let Some(index) = char_to_index_checked(char) {
                                            //don't love hardcoded backspace index
                                            if index == 95 {
                                                word.pop();
                                            } else {
                                                word.push(char);
                                            }
                                            let (verts, texs) = verts_from_word(
                                                &mut cursor,
                                                &word,
                                                text_box_size,
                                                &atlas,
                                                &ft_face,
                                            );
                                            copy_to_buf(&verts, &vert_buf);
                                            copy_to_buf(&texs, &tex_buf);
                                        }
                                    }
                                    None => {
                                        println!("Huh? : {}", e.keyCode());
                                        panic!()
                                    }
                                }
                            }
                            _ => app.sendEvent(e),
                        },
                        None => {
                            break;
                        }
                    }
                }
            }
        })
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct vertex_t {
    position: Float4,
    uv: Float4,
    color: Float4,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct Uniforms {
    screen_size: Float2,
}

//bottom left corner rect
fn build_rect(x: f32, y: f32, width: f32, height: f32, rot: f32) -> Vec<vertex_t> {
    let mut verts = Vec::new();

    let origin = Float2(x, y - height);
    let v1_pos = origin;
    let v1_rot_pos = float2_add(
        apply_rotation_float2(float2_subtract(v1_pos, origin), rot),
        origin,
    );
    let vert1 = vertex_t {
        position: Float4(v1_rot_pos.0, v1_rot_pos.1, 0.0, 1.0),
        uv: Float4(0.0, height, 0.0, 0.0),
        color: Float4(0.3, 0.3, 0.3, 1.0),
    };

    let v2_pos = Float2(x + width, y - height);
    let v2_rot_pos = float2_add(
        apply_rotation_float2(float2_subtract(v2_pos, origin), rot),
        origin,
    );
    let vert2 = vertex_t {
        position: Float4(v2_rot_pos.0, v2_rot_pos.1, 0.0, 1.0),
        uv: Float4(width, height, 0.0, 0.0),
        color: Float4(0.3, 0.3, 0.3, 1.0),
    };

    let v3_pos = Float2(x, y);
    let v3_rot_pos = float2_add(
        apply_rotation_float2(float2_subtract(v3_pos, origin), rot),
        origin,
    );
    let vert3 = vertex_t {
        position: Float4(v3_rot_pos.0, v3_rot_pos.1, 0.0, 1.0),
        uv: Float4(0.0, 0.0, 0.0, 0.0),
        color: Float4(0.3, 0.3, 0.3, 1.0),
    };

    let v4_pos = Float2(x + width, y);
    let v4_rot_pos = float2_add(
        apply_rotation_float2(float2_subtract(v4_pos, origin), rot),
        origin,
    );
    let vert4 = vertex_t {
        position: Float4(v4_rot_pos.0, v4_rot_pos.1, 0.0, 1.0),
        uv: Float4(width, 0.0, 0.0, 0.0),
        color: Float4(0.3, 0.3, 0.3, 1.0),
    };

    verts.push(vert1);
    verts.push(vert2);
    verts.push(vert3);
    verts.push(vert2);
    verts.push(vert3);
    verts.push(vert4);

    verts
}

//Realized that I'm recalculating text wrapping every character draw
//might not be necessary for simple rendering system I currently have
fn verts_from_word(
    cursor: &mut Float2,
    word: &str,
    text_box_width: f32,
    atlas: &Atlas,
    face: &Face,
) -> (Vec<vertex_t>, Vec<Float2>) {
    let mut all_verts = Vec::new();
    let mut all_tex_pointers = Vec::new();

    let initial_cursor_pos = cursor.clone();
    let mut char_positions: Vec<Float2> = Vec::new();
    for i in 0..word.len() {
        char_positions.push(cursor.clone());
        let current_char_index = char_to_index(word.chars().nth(i).unwrap());
        let next_char = word.chars().nth(i + 1);
        cursor.0 += atlas.get_advance(current_char_index, cursor);
        let kerning = match next_char {
            Some(char) => face
                .get_kerning(
                    current_char_index as u32,
                    char_to_index(char) as u32,
                    freetype::face::KerningMode::KerningDefault,
                )
                .unwrap_or_default()
                .into(),
            None => Float2(0.0, 0.0),
        };
        *cursor = *cursor + Float2(kerning.0 / 64.0, 0.0);
        if cursor.0 - initial_cursor_pos.0 >= text_box_width {
            let (str, _) = word.split_at(i);
            let index = str.rfind(char::is_whitespace).unwrap_or(str.len() - 1) + 1;

            let cursor_diff = char_positions[index].0 - initial_cursor_pos.0;
            if cursor_diff <= 0.0 {
                char_positions[i].0 = initial_cursor_pos.0;
                char_positions[i].1 -= atlas.max_height as f32;
            } else {
                let height_diff = atlas.max_height as f32;
                for j in index..=str.len() {
                    char_positions[j].0 -= cursor_diff;
                    char_positions[j].1 -= height_diff;
                }
            }
            cursor.0 = char_positions[i].0 + atlas.get_advance(current_char_index, cursor);
            cursor.1 -= atlas.max_height as f32;
        }
    }
    for i in 0..word.len() {
        let current_char_index = char_to_index(word.chars().nth(i).unwrap());
        all_verts.append(&mut build_rect(
            char_positions[i].0 + atlas.cboxes[current_char_index].xMin as f32,
            char_positions[i].1 + atlas.cboxes[current_char_index].yMin as f32,
            atlas.max_width as f32,
            atlas.max_height as f32,
            0.0,
        ));
        all_tex_pointers.push(Float2(
            0.0,
            (current_char_index as u64 * atlas.max_height) as f32,
        ));
    }
    (all_verts, all_tex_pointers)
}

fn newline(initial_cursor_pos: Float2, cursor: &mut Float2, line_height: f32) {
    *cursor = initial_cursor_pos;
    cursor.1 -= line_height;
}

fn char_to_index(char: char) -> usize {
    (char as u8 - ASCII_START) as usize
}

fn char_to_index_checked(char: char) -> Option<usize> {
    ((char as u8).checked_sub(ASCII_START)).and_then(|index| Some(index as usize))
}

impl From<FT_Vector> for Float2 {
    fn from(value: FT_Vector) -> Self {
        Float2(value.x as f32, value.y as f32)
    }
}
