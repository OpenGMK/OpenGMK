//! OpenGL bindings & functions
//!
//! The raw bindings are generated at build time, see build.rs

/// Auto-generated OpenGL bindings from gl_generator
#[allow(clippy::all)]
mod gl {
    include!(concat!(env!("OUT_DIR"), "/gl_bindings.rs"));
}

use memoffset::offset_of;

use crate::{
    atlas::{AtlasBuilder, AtlasRef},
    render::{Renderer, RendererOptions},
};
use glfw::Context;
use rect_packer::DensePacker;
use std::{
    fs,
    io::{self, BufWriter},
    mem::size_of,
    ops::Drop,
    os::raw::{c_char, c_void},
    path::PathBuf,
    ptr,
};

// OpenGL typedefs
use gl::types::{GLchar, GLfloat, GLint, GLsizei, GLsizeiptr, GLuint};

pub struct OpenGLRenderer {
    // GLFW
    window: glfw::Window,

    // Width the window is supposed to have, assuming it hasn't been resized by the user
    unscaled_width: u32,
    // Height the window is supposed to have, assuming it hasn't been resized by the user
    unscaled_height: u32,

    // Draw command queue
    draw_commands: Vec<DrawCommand>,

    // Shaders and OpenGL objects
    program: u32,
    vao: u32,
    vbo: u32,

    // -- TEXTURE ATLASES --
    /// Whether the initial atlases have been uploaded (see upload_atlases).
    atlases_initialized: bool,
    /// Atlases' rectangle packers to be reused for dynamic sprite loading.
    atlas_packers: Vec<DensePacker>,
    /// Atlas references (xywh + idx) to be indexed by `Texture`s.
    atlas_refs: Vec<AtlasRef>,
    /// OpenGL's texture handles in identical order to the atlases.
    texture_ids: Vec<GLuint>,
}

// A command to draw a sprite or section of a sprite. These are queued and executed
pub struct DrawCommand {
    pub atlas_ref: AtlasRef,
    pub model_view_matrix: [f32; 16],
    pub colour: i32,
    pub alpha: f64,
}

macro_rules! shader_file {
    ($path: expr) => {
        concat!(include_str!($path), "\0").as_bytes()
    };
}

const VERTEX_SHADER_SOURCE: &[u8] = shader_file!("glsl/vertex.glsl");
const FRAGMENT_SHADER_SOURCE: &[u8] = shader_file!("glsl/fragment.glsl");

impl OpenGLRenderer {
    pub fn new(options: RendererOptions, mut window: glfw::Window) -> Result<Self, String> {
        window.set_icon_from_pixels(
            options
                .icons
                .iter()
                .map(|x| glfw::PixelImage {
                    width: x.1,
                    height: x.2,
                    pixels: x
                        .0
                        .rchunks_exact(x.1 as usize * 4)
                        .flat_map(|x| x.chunks_exact(4).map(|r| u32::from_le_bytes([r[2], r[1], r[0], r[3]])))
                        .collect::<Vec<_>>(),
                })
                .collect(),
        );

        window.set_key_polling(true);
        window.set_framebuffer_size_polling(true);

        gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);

        let mut render_context = window.render_context();
        render_context.make_current();

        let (program, vao, vbo) = unsafe {
            // Compile vertex shader
            let vertex_shader = gl::CreateShader(gl::VERTEX_SHADER);
            gl::ShaderSource(
                vertex_shader,
                1,
                &(VERTEX_SHADER_SOURCE.as_ptr() as *const c_char),
                ptr::null(),
            );
            gl::CompileShader(vertex_shader);

            // Check for vertex shader compile errors
            let mut success = gl::FALSE as GLint;
            gl::GetShaderiv(vertex_shader, gl::COMPILE_STATUS, &mut success);
            if success != gl::TRUE as GLint {
                let mut info_len: GLint = 0;
                gl::GetShaderiv(vertex_shader, gl::INFO_LOG_LENGTH, &mut info_len);
                let mut info = vec![0u8; info_len as usize];
                gl::GetShaderInfoLog(
                    vertex_shader,
                    info_len as GLsizei,
                    ptr::null_mut(),
                    info.as_mut_ptr() as *mut GLchar,
                );
                info.set_len((info_len - 1) as usize); // ignore null for str::from_utf8
                return Err(format!(
                    "Failed to compile vertex shader, compiler output:\n{}",
                    std::str::from_utf8(&info).unwrap_or("<INVALID UTF-8>")
                ));
            }

            // Compile fragment shader
            let fragment_shader = gl::CreateShader(gl::FRAGMENT_SHADER);
            gl::ShaderSource(
                fragment_shader,
                1,
                &(FRAGMENT_SHADER_SOURCE.as_ptr() as *const c_char),
                ptr::null(),
            );
            gl::CompileShader(fragment_shader);

            // Check for fragment shader compile errors
            gl::GetShaderiv(fragment_shader, gl::COMPILE_STATUS, &mut success);
            if success != gl::TRUE as GLint {
                let mut info_len: GLint = 0;
                gl::GetShaderiv(fragment_shader, gl::INFO_LOG_LENGTH, &mut info_len);
                let mut info = vec![0u8; info_len as usize];
                gl::GetShaderInfoLog(
                    fragment_shader,
                    info_len as GLsizei,
                    ptr::null_mut(),
                    info.as_mut_ptr() as *mut GLchar,
                );
                info.set_len((info_len - 1) as usize); // ignore null for str::from_utf8
                return Err(format!(
                    "Failed to compile fragment shader, compiler output:\n{}",
                    std::str::from_utf8(&info).unwrap_or("<INVALID UTF-8>")
                ));
            }

            // Link shaders
            let shader_program = gl::CreateProgram();
            gl::AttachShader(shader_program, vertex_shader);
            gl::AttachShader(shader_program, fragment_shader);
            gl::LinkProgram(shader_program);

            // Check for linking errors
            gl::GetProgramiv(shader_program, gl::LINK_STATUS, &mut success);
            if success != gl::TRUE as GLint {
                let mut info_len: GLint = 0;
                gl::GetProgramiv(shader_program, gl::INFO_LOG_LENGTH, &mut info_len);
                let mut info = vec![0u8; info_len as usize];
                gl::GetProgramInfoLog(
                    shader_program,
                    info_len as GLsizei,
                    ptr::null_mut(),
                    info.as_mut_ptr() as *mut GLchar,
                );
                info.set_len((info_len - 1) as usize); // ignore null for str::from_utf8
                return Err(format!(
                    "Failed to link shaders, compiler output:\n{}",
                    std::str::from_utf8(&info).unwrap_or("<INVALID UTF-8>")
                ));
            }
            gl::DeleteShader(vertex_shader);
            gl::DeleteShader(fragment_shader);

            // set up vertex data and configure vertex attributes
            let vertices: [f32; 12] = [
                0.0, 0.0, 0.0, // bottom left
                1.0, 0.0, 0.0, // bottom right
                0.0, 1.0, 0.0, // top left
                1.0, 1.0, 0.0, // top right
            ];
            let (mut vbo, mut vao) = (0, 0);
            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut vbo);
            gl::BindVertexArray(vao);

            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (vertices.len() * size_of::<GLfloat>()) as GLsizeiptr,
                vertices.as_ptr() as *const c_void,
                gl::STATIC_DRAW,
            );

            gl::VertexAttribPointer(
                0,
                3,
                gl::FLOAT,
                gl::FALSE,
                3 * size_of::<GLfloat>() as GLsizei,
                ptr::null(),
            );
            gl::EnableVertexAttribArray(0);

            // Enable and disable GL features
            gl::Enable(gl::SCISSOR_TEST);
            gl::Enable(gl::TEXTURE_2D);
            gl::Disable(gl::CULL_FACE);
            gl::Enable(gl::BLEND);
            gl::Disable(gl::DEPTH_TEST);

            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);

            // Unbind VBO
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);

            (shader_program, vao, vbo)
        };

        Ok(Self {
            window,

            unscaled_width: options.size.0,
            unscaled_height: options.size.1,

            draw_commands: Vec::with_capacity(256),

            program,
            vao,
            vbo,

            atlases_initialized: false,
            atlas_packers: Vec::new(),
            atlas_refs: Vec::new(),
            texture_ids: Vec::new(),
        })
    }

    /// Does anything that's queued to be done.
    fn flush(&mut self) {
        unsafe {
            let mut commands_vbo: GLuint = 0;
            gl::GenBuffers(1, &mut commands_vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, commands_vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (size_of::<DrawCommand>() * self.draw_commands.len()) as _,
                self.draw_commands.as_ptr() as _,
                gl::STATIC_DRAW,
            );

            let glsl_model_view = gl::GetAttribLocation(self.program, b"model_view\0".as_ptr() as *const c_char) as u32;
            gl::EnableVertexAttribArray(glsl_model_view);
            gl::VertexAttribPointer(
                glsl_model_view,
                4,
                gl::FLOAT,
                gl::FALSE,
                size_of::<DrawCommand>() as i32,
                offset_of!(DrawCommand, model_view_matrix) as *const _,
            );
            gl::EnableVertexAttribArray(glsl_model_view + 1);
            gl::VertexAttribPointer(
                glsl_model_view + 1,
                4,
                gl::FLOAT,
                gl::FALSE,
                size_of::<DrawCommand>() as i32,
                (offset_of!(DrawCommand, model_view_matrix) + (4 * size_of::<f32>())) as *const _,
            );
            gl::EnableVertexAttribArray(glsl_model_view + 2);
            gl::VertexAttribPointer(
                glsl_model_view + 2,
                4,
                gl::FLOAT,
                gl::FALSE,
                size_of::<DrawCommand>() as i32,
                (offset_of!(DrawCommand, model_view_matrix) + (8 * size_of::<f32>())) as *const _,
            );
            gl::EnableVertexAttribArray(glsl_model_view + 3);
            gl::VertexAttribPointer(
                glsl_model_view + 3,
                4,
                gl::FLOAT,
                gl::FALSE,
                size_of::<DrawCommand>() as i32,
                (offset_of!(DrawCommand, model_view_matrix) + (12 * size_of::<f32>())) as *const _,
            );
            gl::VertexAttribDivisor(glsl_model_view, 1);
            gl::VertexAttribDivisor(glsl_model_view + 1, 1);
            gl::VertexAttribDivisor(glsl_model_view + 2, 1);
            gl::VertexAttribDivisor(glsl_model_view + 3, 1);

            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);

            gl::DrawArraysInstanced(gl::TRIANGLE_STRIP, 0, 4, self.draw_commands.len() as i32);

            gl::DeleteBuffers(1, &commands_vbo);
        }

        self.draw_commands.clear();
    }
}

impl Renderer for OpenGLRenderer {
    fn max_gpu_texture_size(&self) -> usize {
        unsafe {
            let mut v: GLint = 0;
            gl::GetIntegerv(gl::MAX_TEXTURE_SIZE, &mut v as _);
            v as _
        }
    }

    fn upload_atlases(&mut self, atl: AtlasBuilder) -> Result<(), String> {
        assert!(!self.atlases_initialized, "atlases should be initialized only once");

        let (packers, sprites) = atl.into_inner();

        unsafe {
            let textures: Vec<GLuint> = {
                let mut buf = vec![0 as GLuint; packers.len()];
                gl::GenTextures(buf.len() as _, buf.as_mut_ptr());
                for (tex_id, packer) in buf.iter().copied().zip(&packers) {
                    let (width, height) = packer.size();

                    gl::BindTexture(gl::TEXTURE_2D, tex_id);
                    gl::TexImage2D(
                        gl::TEXTURE_2D,    // target
                        0,                 // level
                        gl::RGBA as _,     // internalformat
                        width as _,        // width
                        height as _,       // height
                        0,                 // border ("must be 0")
                        gl::BGRA,          // format
                        gl::UNSIGNED_BYTE, // type
                        ptr::null(),       // data
                    );
                }
                buf
            };

            // upload textures
            let mut current_texture: GLint = 0;
            for (atl_ref, pixels) in &sprites {
                if current_texture != atl_ref.atlas_id as _ {
                    gl::BindTexture(gl::TEXTURE_2D, textures[atl_ref.atlas_id as usize]);
                    current_texture = atl_ref.atlas_id as _;
                }

                gl::TexSubImage2D(
                    gl::TEXTURE_2D,       // target
                    0,                    // level
                    atl_ref.x as _,       // xoffset
                    atl_ref.y as _,       // yoffset
                    atl_ref.w as _,       // width
                    atl_ref.h as _,       // height
                    gl::BGRA,             // format
                    gl::UNSIGNED_BYTE,    // type
                    pixels.as_ptr() as _, // pixels
                );
            }

            // verify it actually worked
            match gl::GetError() {
                0 => (),
                err => return Err(format!("Failed to upload textures to GPU! (OpenGL code {})", err)),
            }

            // store opengl texture handles
            self.texture_ids = textures;
        }

        // store packers, discard pixeldata
        self.atlas_packers = packers;
        self.atlas_refs = sprites.into_iter().map(|(x, _)| x).collect();

        // generate texture handles
        self.atlases_initialized = true;
        Ok(())
    }

    fn draw_sprite(
        &mut self,
        atlas_ref: &AtlasRef,
        x: f64,
        y: f64,
        xscale: f64,
        yscale: f64,
        angle: f64,
        colour: i32,
        alpha: f64,
    ) {
        let atlas_ref = atlas_ref.clone();
        let angle_sin = angle.sin() as f32;
        let angle_cos = angle.cos() as f32;

        #[rustfmt::skip]
        let model_view_matrix = mat4mult(
            mat4mult(
                mat4mult(
                    // Translate so sprite origin is at [0,0]
                    [
                        1.0, 0.0, 0.0, 0.0,
                        0.0, 1.0, 0.0, 0.0,
                        0.0, 0.0, 1.0, 0.0,
                        -atlas_ref.origin_x, -atlas_ref.origin_y, 0.0, 1.0,
                    ],
                    // Scale according to image size and xscale/yscale
                    [
                        xscale as f32 * atlas_ref.w as f32, 0.0, 0.0, 0.0,
                        0.0, yscale as f32 * atlas_ref.h as f32, 0.0, 0.0,
                        0.0, 0.0, 1.0, 0.0,
                        0.0, 0.0, 0.0, 1.0,
                    ]
                ),
                // Rotate by image_angle
                [
                    angle_cos,  angle_sin, 0.0, 0.0,
                    -angle_sin, angle_cos, 0.0, 0.0,
                    0.0,        0.0,       1.0, 0.0,
                    0.0,        0.0,       0.0, 1.0,
                ]
            ),
            // Move the image into "world coordinates"
            [
                1.0,      0.0,      0.0, 0.0,
                0.0,      1.0,      0.0, 0.0,
                0.0,      0.0,      1.0, 0.0,
                x as f32, y as f32, 0.0, 1.0,
            ]
        );

        self.draw_commands.push(DrawCommand {
            atlas_ref,
            model_view_matrix,
            colour,
            alpha,
        });
    }

    fn clear(&self, red: f32, green: f32, blue: f32) {
        unsafe {
            gl::ClearColor(red, green, blue, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
    }

    fn set_view(
        &mut self,
        src_x: i32,
        src_y: i32,
        src_w: i32,
        src_h: i32,
        src_angle: f64,
        port_x: i32,
        port_y: i32,
        port_w: i32,
        port_h: i32,
    ) {
        // Draw anything that was meant to be drawn with the old view first
        self.flush();

        // Make projection matrix for new view
        let sin_angle = src_angle.sin() as f32;
        let cos_angle = src_angle.cos() as f32;

        #[rustfmt::skip]
        let projection: [f32; 16] = {
            // source rectangle's center coordinates aka -(x + w/2) and -(y + h/2)
            let scx = -((src_x as f32) + (src_w as f32 / 2.0));
            let scy = -((src_y as f32) + (src_h as f32 / 2.0));
            mat4mult(
                mat4mult(
                    // Translate world so center of view is at [0,0]
                    [
                        1.0, 0.0, 0.0, 0.0,
                        0.0, 1.0, 0.0, 0.0,
                        0.0, 0.0, 1.0, 0.0,
                        scx, scy, 0.0, 1.0,
                    ],
                    // Rotate to view_angle
                    [
                        cos_angle,  sin_angle, 0.0, 0.0,
                        -sin_angle, cos_angle, 0.0, 0.0,
                        0.0,        0.0,       1.0, 0.0,
                        0.0,        0.0,       0.0, 1.0,
                    ]
                ),
                // Squish to screen (and flip upside down)
                [
                    2.0 / src_w as f32, 0.0,                 0.0, 0.0,
                    0.0,                -2.0 / src_h as f32, 0.0, 0.0,
                    0.0,                0.0,                 1.0, 0.0,
                    0.0,                0.0,                 0.0, 1.0,
                ]
            )
        };

        // Do scaling by comparing unscaled window size to actual size
        // TODO: use the scaling setting correctly
        let (width, height) = self.window.get_size();
        let port_w = ((port_w * width) as f64 / self.unscaled_width as f64) as i32;
        let port_h = ((port_h * height) as f64 / self.unscaled_height as f64) as i32;
        let port_x = ((port_x * width) as f64 / self.unscaled_width as f64) as i32;
        let port_y = height - (((port_y * height) as f64 / self.unscaled_height as f64) as i32 + port_h);

        // Set viewport (gl::Viewport, gl::Scissor) and projection matrix (shader uniform)
        unsafe {
            gl::Viewport(port_x, port_y, port_w, port_h);
            gl::Scissor(port_x, port_y, port_w, port_h);
            gl::UniformMatrix4fv(
                gl::GetUniformLocation(self.program, b"projection\0".as_ptr() as *const c_char),
                1,
                gl::FALSE,
                &projection as _,
            );
        }
    }

    fn finish(&mut self) {
        // Finish drawing frame
        self.flush();
        self.window.swap_buffers();

        // Start next frame
        let (window_w, window_h) = self.window.get_size();
        unsafe {
            gl::Viewport(0, 0, window_w, window_h);
            gl::Scissor(0, 0, window_w, window_h);
            gl::ClearColor(0.2, 0.3, 0.3, 1.0); // TODO: use clear_colour setting
            gl::Clear(gl::COLOR_BUFFER_BIT);
            gl::UseProgram(self.program);
        }
    }

    fn dump_atlases(&self, path: impl Fn(usize) -> PathBuf) -> io::Result<()> {
        for ((i, texture), packer) in self.texture_ids.iter().enumerate().zip(self.atlas_packers.iter()) {
            let w = BufWriter::new(fs::File::create(&path(i))?);
            let (width, height) = packer.size();
            let mut encoder = png::Encoder::new(w, width as _, height as _);
            encoder.set_color(png::ColorType::RGBA);
            encoder.set_depth(png::BitDepth::Eight);
            let mut writer = encoder.write_header().unwrap();
            let mut buf = vec![0u8; width as usize * height as usize * 4];
            unsafe {
                gl::BindTexture(gl::TEXTURE_2D, *texture);
                gl::GetTexImage(
                    gl::TEXTURE_2D,
                    0,
                    gl::RGBA,
                    gl::UNSIGNED_BYTE,
                    buf.as_mut_ptr() as *mut _,
                );
            }
            writer.write_image_data(&buf).unwrap();
        }

        Ok(())
    }

    fn should_close(&self) -> bool {
        self.window.should_close()
    }

    fn set_should_close(&mut self, b: bool) {
        self.window.set_should_close(b)
    }

    fn show_window(&mut self) {
        self.window.show()
    }

    fn resize_window(&mut self, width: u32, height: u32) {
        // GameMaker only actually resizes the window if the expected (unscaled) size is changing.
        if self.unscaled_width != width || self.unscaled_height != height {
            self.unscaled_width = width;
            self.unscaled_height = height;
            self.window.set_size(width as _, height as _);
        }
    }
}

impl Drop for OpenGLRenderer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(self.texture_ids.len() as _, self.texture_ids.as_mut_ptr() as *mut _);
        }
    }
}

// Helper fn - multiply two mat4s together
fn mat4mult(m1: [f32; 16], m2: [f32; 16]) -> [f32; 16] {
    [
        (m1[0] * m2[0]) + (m1[1] * m2[4]) + (m1[2] * m2[8]) + (m1[3] * m2[12]),
        (m1[0] * m2[1]) + (m1[1] * m2[5]) + (m1[2] * m2[9]) + (m1[3] * m2[13]),
        (m1[0] * m2[2]) + (m1[1] * m2[6]) + (m1[2] * m2[10]) + (m1[3] * m2[14]),
        (m1[0] * m2[3]) + (m1[1] * m2[7]) + (m1[2] * m2[11]) + (m1[3] * m2[15]),
        (m1[4] * m2[0]) + (m1[5] * m2[4]) + (m1[6] * m2[8]) + (m1[7] * m2[12]),
        (m1[4] * m2[1]) + (m1[5] * m2[5]) + (m1[6] * m2[9]) + (m1[7] * m2[13]),
        (m1[4] * m2[2]) + (m1[5] * m2[6]) + (m1[6] * m2[10]) + (m1[7] * m2[14]),
        (m1[4] * m2[3]) + (m1[5] * m2[7]) + (m1[6] * m2[11]) + (m1[7] * m2[15]),
        (m1[8] * m2[0]) + (m1[9] * m2[4]) + (m1[10] * m2[8]) + (m1[11] * m2[12]),
        (m1[8] * m2[1]) + (m1[9] * m2[5]) + (m1[10] * m2[9]) + (m1[11] * m2[13]),
        (m1[8] * m2[2]) + (m1[9] * m2[6]) + (m1[10] * m2[10]) + (m1[11] * m2[14]),
        (m1[8] * m2[3]) + (m1[9] * m2[7]) + (m1[10] * m2[11]) + (m1[11] * m2[15]),
        (m1[12] * m2[0]) + (m1[13] * m2[4]) + (m1[14] * m2[8]) + (m1[15] * m2[12]),
        (m1[12] * m2[1]) + (m1[13] * m2[5]) + (m1[14] * m2[9]) + (m1[15] * m2[13]),
        (m1[12] * m2[2]) + (m1[13] * m2[6]) + (m1[14] * m2[10]) + (m1[15] * m2[14]),
        (m1[12] * m2[3]) + (m1[13] * m2[7]) + (m1[14] * m2[11]) + (m1[15] * m2[15]),
    ]
}
