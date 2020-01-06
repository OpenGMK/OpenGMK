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
    render::{Renderer, RendererOptions, Texture},
};
use glfw::Context;
use rect_packer::DensePacker;
use std::{
    fs,
    io::{self, BufWriter},
    mem::size_of,
    ops::Drop,
    os::raw::c_char,
    path::PathBuf,
    ptr,
};

// OpenGL typedefs
use gl::types::{GLchar, GLfloat, GLint, GLsizei, GLsizeiptr, GLuint};

pub struct OpenGLRenderer {
    // GLFW
    window: glfw::Window,

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
    pub texture: usize,
    pub projection_matrix: [f32; 16],
    pub colour: i32,
    pub alpha: f64,
}

// Vertex shader
const VERTEX_SHADER_SOURCE: &[u8] = br#"
    #version 330 core
    layout (location = 0) in vec3 aPos;
    in mat4 project;
    void main() {
       gl_Position = project * vec4(aPos.x, aPos.y, aPos.z, 1.0);
    }
\0"#;

// Fragment shader
const FRAGMENT_SHADER_SOURCE: &[u8] = br#"
    #version 330 core
    out vec4 FragColour;
    void main() {
       FragColour = vec4(1.0f, 0.0f, 0.0f, 0.2f);
    }
\0"#;

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
            gl::ShaderSource(vertex_shader, 1, &(VERTEX_SHADER_SOURCE.as_ptr() as *const c_char), ptr::null());
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
                return Err(format!(
                    "Failed to compile vertex shader, compiler output:\n{}",
                    std::str::from_utf8(&info).unwrap_or("<INVALID UTF-8>")
                ));
            }

            // Compile fragment shader
            let fragment_shader = gl::CreateShader(gl::FRAGMENT_SHADER);
            gl::ShaderSource(fragment_shader, 1, &(FRAGMENT_SHADER_SOURCE.as_ptr() as *const c_char), ptr::null());
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
                &vertices[0] as *const f32 as *const std::os::raw::c_void,
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
            gl::Enable(gl::TEXTURE_2D);
            gl::Enable(gl::CULL_FACE);
            gl::Enable(gl::BLEND);
            gl::Disable(gl::DEPTH_TEST);

            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);

            // Unbind VBO
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);

            (shader_program, vao, vbo)
        };

        Ok(Self {
            window,

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
        texture: &Texture,
        _x: f64,
        _y: f64,
        _xscale: f64,
        _yscale: f64,
        _angle: f64,
        colour: i32,
        alpha: f64,
    ) {
        let projection_matrix: [f32; 16] = [
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0,
        ];

        self.draw_commands.push(DrawCommand {
            texture: texture.0,
            projection_matrix,
            colour,
            alpha,
        });
    }

    fn draw(&mut self) {
        unsafe {
            gl::ClearColor(0.2, 0.3, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
            gl::UseProgram(self.program);

            let mut commands_vbo: GLuint = 0;
            gl::GenBuffers(1, &mut commands_vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, commands_vbo);
            gl::BufferData(gl::ARRAY_BUFFER, (size_of::<DrawCommand>() * self.draw_commands.len()) as _, self.draw_commands.as_ptr() as _, gl::STATIC_DRAW);

            let project = gl::GetAttribLocation(self.program, b"project\0".as_ptr() as *const c_char) as u32;
            gl::EnableVertexAttribArray(project);
            gl::VertexAttribPointer(project, 4, gl::FLOAT, gl::FALSE, size_of::<DrawCommand>() as i32, offset_of!(DrawCommand, projection_matrix) as *const _);
            gl::EnableVertexAttribArray(project + 1);
            gl::VertexAttribPointer(project + 1, 4, gl::FLOAT, gl::FALSE, size_of::<DrawCommand>() as i32, (offset_of!(DrawCommand, projection_matrix) + (4  * size_of::<f32>())) as *const _);
            gl::EnableVertexAttribArray(project + 2);
            gl::VertexAttribPointer(project + 2, 4, gl::FLOAT, gl::FALSE, size_of::<DrawCommand>() as i32, (offset_of!(DrawCommand, projection_matrix) + (8  * size_of::<f32>())) as *const _);
            gl::EnableVertexAttribArray(project + 3);
            gl::VertexAttribPointer(project + 3, 4, gl::FLOAT, gl::FALSE, size_of::<DrawCommand>() as i32, (offset_of!(DrawCommand, projection_matrix) + (12 * size_of::<f32>())) as *const _);
            gl::VertexAttribDivisor(project, 1);
            gl::VertexAttribDivisor(project + 1, 1);
            gl::VertexAttribDivisor(project + 2, 1);
            gl::VertexAttribDivisor(project + 3, 1);

            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
            gl::DrawArraysInstanced(gl::TRIANGLE_STRIP, 0, 4, self.draw_commands.len() as i32);

            gl::DeleteBuffers(1, &commands_vbo);
        }

        self.draw_commands.clear();
        self.window.swap_buffers();
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

    fn set_viewport(&self, width: i32, height: i32) {
        unsafe {
            gl::Viewport(0, 0, width, height);
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
