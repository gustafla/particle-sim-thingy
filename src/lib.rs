#![allow(dead_code)]

mod glesv2_raii;

use glesv2_raii::{Buffer, Framebuffer, Program, Texture, TextureAttachment};
use log::info;
use opengles::glesv2::{self, constants::*, types::*};
use std::ffi::{c_void, CString};
use std::os::raw::c_char;

struct Scene {
    sync_get_raw: extern "C" fn(*const c_char) -> f64,
    resolution: (i32, i32),
    program: Program,
    buffer: Buffer,
    post_fbo: Framebuffer,
    post_program: Program,
    post_buffer: Buffer,
}

impl Scene {
    fn sync_get(&self, name: &str) -> f64 {
        let string = CString::new(name).unwrap();
        (self.sync_get_raw)(string.as_c_str().as_ptr())
    }
}

#[no_mangle]
extern "C" fn scene_init(w: i32, h: i32, get: extern "C" fn(*const c_char) -> f64) -> *mut c_void {
    simple_logger::init().unwrap();
    glesv2::viewport(0, 0, w, h);

    // Create a buffer for post processing pass quad
    let post_buffer = Buffer::new();
    glesv2::bind_buffer(GL_ARRAY_BUFFER, post_buffer.handle());
    glesv2::buffer_data(
        GL_ARRAY_BUFFER,
        &[
            -1f32, -1., 0., 0., 0., 1., -1., 0., 1., 0., 1., 1., 0., 1., 1., -1., -1., 0., 0., 0.,
            1., 1., 0., 1., 1., -1., 1., 0., 0., 1.,
        ],
        GL_STATIC_DRAW,
    );

    // Create a buffer for test triangle
    let buffer = Buffer::new();
    glesv2::bind_buffer(GL_ARRAY_BUFFER, buffer.handle());
    glesv2::buffer_data(
        GL_ARRAY_BUFFER,
        &[-0.5f32, -0.5, 0.0, 0.5, -0.5, 0.0, 0.0, 0.5, 0.0],
        GL_STATIC_DRAW,
    );

    // Create an FBO for post processing
    let fbo_texture = Texture::new();
    glesv2::bind_texture(GL_TEXTURE_2D, fbo_texture.handle());
    Texture::image::<u8>(GL_TEXTURE_2D, 0, GL_RGB, w, h, GL_UNSIGNED_BYTE, &[]);
    Texture::set_filters(GL_TEXTURE_2D, GL_NEAREST);
    let post_fbo = Framebuffer::new(
        Some(vec![(
            GL_COLOR_ATTACHMENT0,
            TextureAttachment {
                target: GL_TEXTURE_2D,
                texture: fbo_texture,
                mipmap_level: 0,
            },
        )]),
        None,
    )
    .unwrap();

    let scene = Box::new(Scene {
        sync_get_raw: get,
        resolution: (w, h),
        program: Program::from_sources(&["shader.vert", "shader.frag"]).unwrap(),
        buffer,
        post_fbo,
        post_program: Program::from_sources(&["shader.vert", "post.frag"]).unwrap(),
        post_buffer,
    });

    info!("scene created");

    Box::into_raw(scene) as *mut c_void
}

#[no_mangle]
extern "C" fn scene_deinit(data: *mut c_void) {
    info!("scene dropped");
    let _scene = unsafe { Box::from_raw(data as *mut Scene) };
}

#[no_mangle]
extern "C" fn scene_render(time: f64, data: *mut c_void) {
    let scene = Box::leak(unsafe { Box::from_raw(data as *mut Scene) });

    // Test picture -------------------------------------------------------------------------------

    glesv2::bind_framebuffer(GL_FRAMEBUFFER, scene.post_fbo.handle());
    glesv2::clear_color(f32::sin(time as f32), 1., 0., 1.);
    glesv2::clear(GL_COLOR_BUFFER_BIT);

    glesv2::bind_buffer(GL_ARRAY_BUFFER, scene.buffer.handle());
    let index_pos = scene.program.attrib_location("a_Pos");
    glesv2::enable_vertex_attrib_array(index_pos);
    glesv2::vertex_attrib_pointer_offset(index_pos, 3, GL_FLOAT, false, 0, 0);

    glesv2::use_program(scene.program.handle());

    glesv2::draw_arrays(GL_TRIANGLES, 0, 3);

    // Post pass ----------------------------------------------------------------------------------

    glesv2::bind_framebuffer(GL_FRAMEBUFFER, 0);
    glesv2::active_texture(GL_TEXTURE0);
    glesv2::bind_texture(
        GL_TEXTURE_2D,
        scene.post_fbo.texture_handle(GL_COLOR_ATTACHMENT0).unwrap(),
    );

    glesv2::bind_buffer(GL_ARRAY_BUFFER, scene.post_buffer.handle());
    let index_pos = scene.post_program.attrib_location("a_Pos");
    let index_tex_coord = scene.post_program.attrib_location("a_TexCoord");
    let stride = (std::mem::size_of::<f32>() * 5) as GLint;
    glesv2::enable_vertex_attrib_array(index_pos);
    glesv2::vertex_attrib_pointer_offset(index_pos, 3, GL_FLOAT, false, stride, 0);
    glesv2::enable_vertex_attrib_array(index_tex_coord);
    glesv2::vertex_attrib_pointer_offset(
        index_tex_coord,
        2,
        GL_FLOAT,
        false,
        stride,
        std::mem::size_of::<f32>() as GLuint * 3,
    );

    glesv2::use_program(scene.post_program.handle());
    glesv2::uniform1f(
        scene.post_program.uniform_location("u_NoiseTime"),
        scene.sync_get("noise_time") as f32,
    );
    glesv2::uniform1f(
        scene.post_program.uniform_location("u_NoiseAmount"),
        scene.sync_get("noise_amount") as f32,
    );
    glesv2::uniform1i(scene.post_program.uniform_location("u_InputSampler"), 0);
    glesv2::uniform2f(
        scene.post_program.uniform_location("u_Resolution"),
        scene.resolution.0 as f32,
        scene.resolution.1 as f32,
    );

    glesv2::draw_arrays(GL_TRIANGLES, 0, 6);

    glesv2_raii::check().unwrap();
}
