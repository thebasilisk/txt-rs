use atlas::{ASCII_START, Atlas, NUM_ASCII_CHARS, create_texture_atlas};
use freetype::freetype::FT_Vector_;
use maths::{Float2, Float4, apply_rotation_float2, float2_add, float2_subtract};
use metal::{Buffer, DeviceRef};
use objc2::rc::autoreleasepool;
use objc2_app_kit::{NSAnyEventMask, NSEventType};
use objc2_foundation::{NSComparisonResult, NSDate, NSDefaultRunLoopMode};
use text::freetype::{init_ft_lib, load_typeface};
use utils::{
    get_library, get_next_frame, init_render_with_bufs, make_buf, new_render_pass_descriptor,
    prepare_pipeline_state, simple_app,
};

mod atlas;
mod maths;
mod text;
mod utils;

fn main() {
    let view_width = 1024.0;
    let view_height = 768.0;
    let (app, window, device, layer) = simple_app(view_width, view_height, "Texter");

    let shaderlib = get_library(&device);

    let render_pipeline = prepare_pipeline_state(&device, "box_vertex", "box_fragment", &shaderlib);
    let command_queue = device.new_command_queue();

    // let x = 0.0;
    // let y = 0.0;
    // let width = 300.0;
    // let height = width;

    let ft_lib = init_ft_lib().unwrap();
    let ft_face = load_typeface(ft_lib, "Arial").unwrap();
    // let (bitmap, char_width, char_height) = get_char_glyph(ft_face, 'z').unwrap();
    // let char_width = 153;
    // let char_height = 153;
    // let (texture_atlas, width, height) = create_texture_atlas(ft_face, &device).unwrap();
    let atlas = create_texture_atlas(ft_face, &device).unwrap();

    // let tex_descriptor = TextureDescriptor::new();
    // tex_descriptor.set_pixel_format(MTLPixelFormat::R8Unorm);
    // tex_descriptor.set_width(char_height);
    // tex_descriptor.set_height(char_height);

    // let char_tex = device.new_texture(&tex_descriptor);
    // let region = MTLRegion::new_2d(0, 0, char_width, char_height);
    // char_tex.replace_region(region, 0, bitmap as *const _, char_width * 1); //pixels per row * bytes per pixel

    // let vertex_data = build_rect(
    //     x,
    //     y,
    //     width as f32,
    //     // (height * NUM_ASCII_CHARS as u64) as f32,
    //     height as f32,
    //     0.0,
    // );
    // let vertex_data = make_buf(&text_rect, &device);

    let word = "hello world";

    let mut cursor = Float2(-650.0, 150.0);
    let unis = Uniforms {
        screen_size: Float2(view_width as f32, view_height as f32),
    };
    let uni_buf = make_buf(&vec![unis], &device);
    let (vert_buf, tex_buf) = verts_from_word(&mut cursor, word, &atlas, &device);
    let buffers = vec![&uni_buf, &vert_buf, &tex_buf];
    // let vertex_data = make_buf(data, device)

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
                // encoder.set_fragment_texture(0, Some(&char_tex));
                encoder.draw_primitives(
                    metal::MTLPrimitiveType::Triangle,
                    0,
                    word.len() as u64 * 6, //six verts per char
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

fn verts_from_word(
    cursor: &mut Float2,
    word: &str,
    atlas: &Atlas,
    device: &DeviceRef,
) -> (Buffer, Buffer) {
    let mut all_verts = Vec::new();
    let mut all_tex_pointers = Vec::new();

    for char in word.chars() {
        let i = char_to_index(char);
        all_verts.append(&mut build_rect(
            cursor.0,
            cursor.1,
            atlas.max_width as f32,
            atlas.max_height as f32,
            0.0,
        ));
        all_tex_pointers.push(Float2(0.0, (i as u64 * atlas.max_height) as f32));
        *cursor = *cursor + Float2(atlas.advances[i].x as f32 / 64.0, 0.0);
    }

    let vertex_buffer = make_buf(&all_verts, &device);
    let tex_pointer_buffer = make_buf(&all_tex_pointers, &device);
    (vertex_buffer, tex_pointer_buffer)
}

fn char_to_index(char: char) -> usize {
    if char == ' ' {
        (NUM_ASCII_CHARS) as usize
    } else {
        (char as u8 - ASCII_START) as usize
    }
}

impl From<FT_Vector_> for Float2 {
    fn from(value: FT_Vector_) -> Self {
        Float2(value.x as f32, value.y as f32)
    }
}
