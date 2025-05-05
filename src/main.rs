use maths::{Float2, Float4, apply_rotation_float2, float2_add, float2_subtract};
use metal::{MTLPixelFormat, MTLRegion, TextureDescriptor};
use objc2::rc::autoreleasepool;
use objc2_app_kit::{NSAnyEventMask, NSEventType};
use objc2_foundation::{NSComparisonResult, NSDate, NSDefaultRunLoopMode};
use text::freetype::get_char_glyph;
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
    let (app, window, device, layer) = simple_app(view_width, view_height, "Colorstep");

    let shaderlib = get_library(&device);

    let render_pipeline = prepare_pipeline_state(&device, "box_vertex", "box_fragment", &shaderlib);
    let command_queue = device.new_command_queue();

    let x = 0.0;
    let y = 0.0;
    let width = 300.0;
    let height = width;

    let (bitmap, char_width, char_height) = get_char_glyph("Arial", 'a').unwrap();
    // let char_width = 153;
    // let char_height = 153;

    let tex_descriptor = TextureDescriptor::new();
    tex_descriptor.set_pixel_format(MTLPixelFormat::R8Unorm);
    tex_descriptor.set_width(char_height);
    tex_descriptor.set_height(char_height);

    let char_tex = device.new_texture(&tex_descriptor);
    let region = MTLRegion::new_2d(0, 0, char_width, char_height);
    char_tex.replace_region(region, 0, bitmap as *const _, char_width * 1); //pixels per row * bytes per pixel

    let vertex_data = build_rect(x, y, char_width as f32, char_height as f32, 0.0);
    // let vertex_data = make_buf(&text_rect, &device);

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
                    &vec![],
                    &render_descriptor,
                    &render_pipeline,
                    command_buffer,
                );
                //second uniform here is texture width, need to define uniforms object
                encoder.set_vertex_bytes(
                    0,
                    (size_of::<Float4>()) as u64 * 2,
                    vec![
                        Float4(view_width as f32, view_height as f32, 0.0, 0.0),
                        Float4(153.0, 0.0, 0.0, 0.0),
                    ]
                    .as_ptr() as *const _,
                );
                encoder.set_vertex_bytes(
                    1,
                    (size_of::<vertex_t>() * vertex_data.len()) as u64,
                    vertex_data.as_ptr() as *const _,
                );
                encoder.set_vertex_bytes(
                    2,
                    (size_of::<Float2>()) as u64,
                    vec![Float2(0.0, 0.0)].as_ptr() as *const _,
                );
                encoder.set_fragment_texture(0, Some(&char_tex));
                encoder.draw_primitives(
                    metal::MTLPrimitiveType::Triangle,
                    0,
                    vertex_data.len() as u64,
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

fn build_rect(x: f32, y: f32, width: f32, height: f32, rot: f32) -> Vec<vertex_t> {
    let mut verts = Vec::new();

    let origin = Float2(x - width / 2.0, y - height / 2.0);
    let v1_pos = origin;
    let v1_rot_pos = float2_add(
        apply_rotation_float2(float2_subtract(v1_pos, origin), rot),
        origin,
    );
    let vert1 = vertex_t {
        position: Float4(v1_rot_pos.0, v1_rot_pos.1, 0.0, 1.0),
        uv: Float4(0.0, 1.0, 0.0, 0.0),
        color: Float4(0.3, 0.3, 0.3, 1.0),
    };

    let v2_pos = Float2(x + width / 2.0, y - height / 2.0);
    let v2_rot_pos = float2_add(
        apply_rotation_float2(float2_subtract(v2_pos, origin), rot),
        origin,
    );
    let vert2 = vertex_t {
        position: Float4(v2_rot_pos.0, v2_rot_pos.1, 0.0, 1.0),
        uv: Float4(1.0, 1.0, 0.0, 0.0),
        color: Float4(0.3, 0.3, 0.3, 1.0),
    };

    let v3_pos = Float2(x - width / 2.0, y + height / 2.0);
    let v3_rot_pos = float2_add(
        apply_rotation_float2(float2_subtract(v3_pos, origin), rot),
        origin,
    );
    let vert3 = vertex_t {
        position: Float4(v3_rot_pos.0, v3_rot_pos.1, 0.0, 1.0),
        uv: Float4(0.0, 0.0, 0.0, 0.0),
        color: Float4(0.3, 0.3, 0.3, 1.0),
    };

    let v4_pos = Float2(x + width / 2.0, y + height / 2.0);
    let v4_rot_pos = float2_add(
        apply_rotation_float2(float2_subtract(v4_pos, origin), rot),
        origin,
    );
    let vert4 = vertex_t {
        position: Float4(v4_rot_pos.0, v4_rot_pos.1, 0.0, 1.0),
        uv: Float4(1.0, 0.0, 0.0, 0.0),
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
