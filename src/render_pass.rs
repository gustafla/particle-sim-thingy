use crate::glesv2_raii::{Framebuffer, Texture, TextureAttachment, UniformValue};
use crate::Scene;
use opengles::glesv2::{self, constants::*, types::*};

pub struct RenderPass {
    pub fbo: Framebuffer,
    shader_path: String,
}

impl RenderPass {
    pub fn new(w: i32, h: i32, frag_path: &str) -> Self {
        let fbo_texture = Texture::new();
        glesv2::bind_texture(GL_TEXTURE_2D, fbo_texture.handle());
        Texture::image::<u8>(GL_TEXTURE_2D, 0, GL_RGB, w, h, GL_UNSIGNED_BYTE, &[]);
        glesv2::tex_parameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_NEAREST as GLint);
        glesv2::tex_parameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_NEAREST as GLint);
        glesv2::tex_parameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_CLAMP_TO_EDGE as GLint);
        glesv2::tex_parameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_CLAMP_TO_EDGE as GLint);

        let fbo = Framebuffer::new(
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

        Self {
            fbo,
            shader_path: format!("./shader.vert {}", frag_path),
        }
    }

    pub fn render(&self, scene: &Scene, textures: &[GLuint], uniforms: &[(&str, UniformValue)]) {
        let program = scene.resources.program(&self.shader_path).unwrap();
        glesv2::use_program(program.handle());

        glesv2::active_texture(GL_TEXTURE0);
        glesv2::bind_texture(
            GL_TEXTURE_2D,
            self.fbo.texture_handle(GL_COLOR_ATTACHMENT0).unwrap(),
        );
        glesv2::uniform1i(program.uniform_location("u_InputSampler0").unwrap(), 0);

        for (i, texture) in textures.iter().enumerate() {
            glesv2::active_texture(GL_TEXTURE1 + i as GLuint);
            glesv2::bind_texture(GL_TEXTURE_2D, *texture);
            let i = i as GLint + 1;
            glesv2::uniform1i(
                program
                    .uniform_location(&format!("u_InputSampler{}", i))
                    .unwrap(),
                i,
            );
        }

        if let Some(loc) = program.uniform_location("u_Resolution") {
            glesv2::uniform2f(loc, scene.resolution.0 as f32, scene.resolution.1 as f32);
        }

        for ufm in uniforms {
            let loc = program.uniform_location(ufm.0).unwrap();
            match ufm.1 {
                UniformValue::Float(x) => glesv2::uniform1f(loc, x),
                UniformValue::Vec2(x, y) => glesv2::uniform2f(loc, x, y),
                UniformValue::Vec3(x, y, z) => glesv2::uniform3f(loc, x, y, z),
                UniformValue::Vec4(x, y, z, w) => glesv2::uniform4f(loc, x, y, z, w),
            }
        }

        scene.resources.buffer("./quad.abuf").unwrap().bind();
        let index_pos = program.attrib_location("a_Pos").unwrap() as GLuint;
        let index_tex_coord = program.attrib_location("a_TexCoord").unwrap() as GLuint;
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

        glesv2::draw_arrays(GL_TRIANGLES, 0, 6);
    }
}
