mod wgl;

use crate::{
    atlas::{AtlasBuilder, AtlasRef},
    render::{mat4mult, BlendType, RendererOptions, RendererTrait, SavedTexture},
    window::Window,
};
use cfg_if::cfg_if;
use memoffset::offset_of;
use rect_packer::DensePacker;
use shared::types::Colour;
use std::{any::Any, mem::size_of, ptr};

/// Auto-generated OpenGL bindings from gl_generator
pub mod gl {
    #![allow(clippy::all)]
    include!(concat!(env!("OUT_DIR"), "/gl_bindings.rs"));
}
use gl::types::{GLchar, GLenum, GLfloat, GLint, GLsizei, GLsizeiptr, GLuint};

cfg_if! {
    if #[cfg(target_os = "windows")] {
        use crate::window::win32 as w_imp;
        use wgl as imp;
    } else {
        // TODO: This won't work when Wayland but that's okay just make a function for it.
        use crate::window::xorg as w_imp;
    }
}

macro_rules! shader_file {
    ($path: expr) => {
        concat!(include_str!($path), "\0").as_bytes()
    };
}

pub struct RendererImpl {
    imp: imp::PlatformImpl,
    //program: GLuint,
    //vao: GLuint,
    vbo: GLuint,

    atlas_packers: Vec<DensePacker>,
    texture_ids: Vec<Option<GLuint>>,
    fbo_ids: Vec<Option<GLuint>>,
    stock_atlas_count: u32,
    current_atlas: GLuint,
    white_pixel: AtlasRef,
    draw_queue: Vec<DrawCommand>,

    loc_tex: GLint,  // uniform sampler2D tex
    loc_proj: GLint, // uniform mat4 projection
}

static VERTEX_SHADER_SOURCE: &[u8] = shader_file!("glsl/vertex.glsl");
static FRAGMENT_SHADER_SOURCE: &[u8] = shader_file!("glsl/fragment.glsl");

/// A command to draw a sprite or section of a sprite.
/// These are queued and executed (instanced if possible).
pub struct DrawCommand {
    pub atlas_ref: AtlasRef,
    pub model_view_matrix: [f32; 16],
    pub blend: (f32, f32, f32),
    pub alpha: f32,
}

unsafe fn shader_info_log(name: &str, id: GLuint) -> String {
    let mut info_len: GLint = 0;
    gl::GetShaderiv(id, gl::INFO_LOG_LENGTH, &mut info_len);
    let mut info = vec![0u8; info_len as usize];
    gl::GetShaderInfoLog(id, info_len as GLsizei, ptr::null_mut(), info.as_mut_ptr() as *mut GLchar);
    info.set_len((info_len - 1) as usize); // ignore null for str::from_utf8
    format!(
        "Failed to compile {} shader, compiler output:\n{}",
        name,
        std::str::from_utf8(&info).unwrap_or("<INVALID UTF-8>")
    )
}

impl From<BlendType> for GLenum {
    fn from(bt: BlendType) -> Self {
        match bt {
            BlendType::Zero => gl::ZERO,
            BlendType::One => gl::ONE,
            BlendType::SrcColour => gl::SRC_COLOR,
            BlendType::InvSrcColour => gl::ONE_MINUS_SRC_COLOR,
            BlendType::SrcAlpha => gl::SRC_ALPHA,
            BlendType::InvSrcAlpha => gl::ONE_MINUS_SRC_ALPHA,
            BlendType::DestAlpha => gl::DST_ALPHA,
            BlendType::InvDestAlpha => gl::ONE_MINUS_DST_ALPHA,
            BlendType::DestColour => gl::DST_COLOR,
            BlendType::InvDestColour => gl::ONE_MINUS_DST_COLOR,
            BlendType::SrcAlphaSaturate => gl::SRC_ALPHA_SATURATE,
        }
    }
}

impl From<GLenum> for BlendType {
    fn from(bt: GLenum) -> Self {
        match bt {
            gl::ZERO => BlendType::Zero,
            gl::ONE => BlendType::One,
            gl::SRC_COLOR => BlendType::SrcColour,
            gl::ONE_MINUS_SRC_COLOR => BlendType::InvSrcColour,
            gl::SRC_ALPHA => BlendType::SrcAlpha,
            gl::ONE_MINUS_SRC_ALPHA => BlendType::InvSrcAlpha,
            gl::DST_ALPHA => BlendType::DestAlpha,
            gl::ONE_MINUS_DST_ALPHA => BlendType::InvDestAlpha,
            gl::DST_COLOR => BlendType::DestColour,
            gl::ONE_MINUS_DST_COLOR => BlendType::InvDestColour,
            gl::SRC_ALPHA_SATURATE => BlendType::SrcAlphaSaturate,
            _ => unreachable!(),
        }
    }
}

impl RendererImpl {
    pub fn new(options: &RendererOptions, window: &Window, clear_colour: Colour) -> Result<Self, String> {
        let window_impl: &w_imp::WindowImpl = match window.as_any().downcast_ref() {
            Some(x) => x,
            None => return Err("Wrong backend provided to OpenGLRenderer::new()".into()),
        };

        unsafe {
            let imp = imp::PlatformImpl::new(window_impl)?;

            if options.vsync {
                imp.set_swap_interval(1);
            } else {
                imp.set_swap_interval(0);
            }

            let (v_maj, v_min) = imp.version();
            assert!((v_maj == 3 && v_min >= 3) || v_maj > 3);

            // Compile vertex shader
            let vertex_shader = gl::CreateShader(gl::VERTEX_SHADER);
            gl::ShaderSource(vertex_shader, 1, &(VERTEX_SHADER_SOURCE.as_ptr().cast()), ptr::null());
            gl::CompileShader(vertex_shader);

            // Check for vertex shader compile errors
            let mut success = gl::FALSE as GLint;
            gl::GetShaderiv(vertex_shader, gl::COMPILE_STATUS, &mut success);
            if success != gl::TRUE as GLint {
                return Err(shader_info_log("vertex", vertex_shader))
            }

            // Compile fragment shader
            let fragment_shader = gl::CreateShader(gl::FRAGMENT_SHADER);
            gl::ShaderSource(fragment_shader, 1, &(FRAGMENT_SHADER_SOURCE.as_ptr().cast()), ptr::null());
            gl::CompileShader(fragment_shader);

            // Check for fragment shader compile errors
            gl::GetShaderiv(fragment_shader, gl::COMPILE_STATUS, &mut success);
            if success != gl::TRUE as GLint {
                return Err(shader_info_log("fragment", fragment_shader))
            }

            // Link shaders
            let program = gl::CreateProgram();
            gl::AttachShader(program, vertex_shader);
            gl::AttachShader(program, fragment_shader);
            gl::LinkProgram(program);

            // Check for linking errors
            // TODO: generalize this like with shader info logs!! please!!!
            gl::GetProgramiv(program, gl::LINK_STATUS, &mut success);
            if success != gl::TRUE as GLint {
                let mut info_len: GLint = 0;
                gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut info_len);
                let mut info = vec![0u8; info_len as usize];
                gl::GetProgramInfoLog(program, info_len as GLsizei, ptr::null_mut(), info.as_mut_ptr() as *mut GLchar);
                info.set_len((info_len - 1) as usize); // ignore null for str::from_utf8
                return Err(format!(
                    "Failed to link shaders, compiler output:\n{}",
                    std::str::from_utf8(&info).unwrap_or("<INVALID UTF-8>")
                ))
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
                vertices.as_ptr().cast(),
                gl::STATIC_DRAW,
            );

            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 3 * size_of::<GLfloat>() as GLsizei, ptr::null());
            gl::EnableVertexAttribArray(0);

            // Enable and disable GL features
            gl::Enable(gl::SCISSOR_TEST);
            // gl::Enable(gl::TEXTURE_2D);
            gl::Disable(gl::CULL_FACE);
            gl::Enable(gl::BLEND);
            gl::Disable(gl::DEPTH_TEST);

            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);

            // Unbind VBO
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);

            // Use program
            gl::UseProgram(program);

            // Configure gl::ReadPixels() to read from the back buffer
            gl::ReadBuffer(gl::BACK);

            // Configure gl::ReadPixels() to align to 1 byte
            gl::PixelStorei(gl::PACK_ALIGNMENT, 1);

            // Create Renderer
            let mut renderer = Self {
                imp,
                //program,
                //vao,
                vbo,

                atlas_packers: vec![],
                texture_ids: vec![],
                fbo_ids: vec![],
                stock_atlas_count: 0,
                current_atlas: 0,
                white_pixel: Default::default(),
                draw_queue: Vec::with_capacity(256),

                loc_tex: gl::GetUniformLocation(program, b"tex\0".as_ptr().cast()),
                loc_proj: gl::GetUniformLocation(program, b"projection\0".as_ptr().cast()),
            };

            // Start first frame
            renderer.setup_frame(options.size.0, options.size.1, clear_colour);

            Ok(renderer)
        }
    }

    fn setup_frame(&mut self, width: u32, height: u32, clear_colour: Colour) {
        unsafe {
            gl::Viewport(0, 0, width as _, height as _);
            gl::Scissor(0, 0, width as _, height as _);
            gl::ClearColor(clear_colour.r as f32, clear_colour.g as f32, clear_colour.b as f32, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
    }
}

impl RendererTrait for RendererImpl {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn max_texture_size(&self) -> u32 {
        let mut size: GLint = 0;
        unsafe {
            gl::GetIntegerv(gl::MAX_TEXTURE_SIZE, &mut size);
        }
        size.max(0) as u32
    }

    fn push_atlases(&mut self, mut atl: AtlasBuilder) -> Result<(), String> {
        assert!(self.atlas_packers.is_empty(), "atlases should be initialized only once");
        self.white_pixel =
            atl.texture(1, 1, 0, 0, Box::new([0xFF, 0xFF, 0xFF, 0xFF])).ok_or("Couldn't pack white_pixel")?;
        let (packers, sprites) = atl.into_inner();

        unsafe {
            let textures: Vec<GLuint> = {
                let mut buf = vec![0 as GLuint; packers.len()];
                gl::GenTextures(buf.len() as _, buf.as_mut_ptr());
                for (i, (tex_id, packer)) in buf.iter().copied().zip(&packers).enumerate() {
                    let (width, height) = packer.size();

                    gl::BindTexture(gl::TEXTURE_2D, tex_id);
                    self.current_atlas = i as u32;

                    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as _);
                    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as _);
                    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as _);
                    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as _);
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

            // verify it actually worked
            match gl::GetError() {
                0 => (),
                err => return Err(format!("Failed to allocate texture on GPU! (OpenGL code {})", err)),
            }

            // upload textures
            for (atl_ref, pixels) in &sprites {
                if self.current_atlas != atl_ref.atlas_id {
                    gl::BindTexture(gl::TEXTURE_2D, textures[atl_ref.atlas_id as usize]);
                    self.current_atlas = atl_ref.atlas_id;
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

            // generate framebuffers
            let mut fbo_ids = Vec::with_capacity(textures.len());
            for tex_id in &textures {
                let mut fbo = 0;
                gl::GenFramebuffers(1, &mut fbo);
                gl::BindFramebuffer(gl::READ_FRAMEBUFFER, fbo);
                gl::FramebufferTexture2D(gl::READ_FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, *tex_id, 0);
                fbo_ids.push(Some(fbo));
            }
            gl::BindFramebuffer(gl::READ_FRAMEBUFFER, 0);

            // store opengl texture handles
            self.texture_ids = textures.iter().map(|t| Some(*t)).collect();
            self.fbo_ids = fbo_ids;
            self.stock_atlas_count = textures.len() as u32;
        }

        // store packers, discard pixeldata
        self.atlas_packers = packers;

        Ok(())
    }

    fn upload_sprite(
        &mut self,
        data: Box<[u8]>,
        width: i32,
        height: i32,
        origin_x: i32,
        origin_y: i32,
    ) -> Result<AtlasRef, String> {
        let atlas_ref = AtlasRef {
            origin_x: origin_x as f32 / width as f32,
            origin_y: origin_y as f32 / height as f32,
            ..self.create_surface(width, height)?
        };
        unsafe {
            // store previous
            let mut prev_tex2d = 0;
            gl::GetIntegerv(gl::TEXTURE_BINDING_2D, &mut prev_tex2d);

            // upload texture
            gl::BindTexture(gl::TEXTURE_2D, self.texture_ids[atlas_ref.atlas_id as usize].unwrap());
            gl::TexSubImage2D(
                gl::TEXTURE_2D,     // target
                0,                  // level
                atlas_ref.x as _,   // xoffset
                atlas_ref.y as _,   // yoffset
                atlas_ref.w as _,   // width
                atlas_ref.h as _,   // height
                gl::RGBA,           // format
                gl::UNSIGNED_BYTE,  // type
                data.as_ptr() as _, // pixels
            );

            // verify it actually worked
            match gl::GetError() {
                0 => (),
                err => return Err(format!("Failed to upload texture to GPU! (OpenGL code {})", err)),
            }

            // cleanup
            gl::BindTexture(gl::TEXTURE_2D, prev_tex2d as _);
        }
        Ok(atlas_ref)
    }

    fn duplicate_sprite(&mut self, atlas_ref: AtlasRef) -> Result<AtlasRef, String> {
        let new_sprite = self.create_surface(atlas_ref.w, atlas_ref.h)?;
        unsafe {
            // store previous
            let mut prev_read_fbo = 0;
            gl::GetIntegerv(gl::READ_FRAMEBUFFER_BINDING, &mut prev_read_fbo);
            let mut prev_draw_fbo = 0;
            gl::GetIntegerv(gl::DRAW_FRAMEBUFFER_BINDING, &mut prev_draw_fbo);

            gl::BindFramebuffer(gl::READ_FRAMEBUFFER, self.fbo_ids[atlas_ref.atlas_id as usize].unwrap());
            gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, self.fbo_ids[new_sprite.atlas_id as usize].unwrap());

            gl::BlitFramebuffer(
                0,
                0,
                atlas_ref.w,
                atlas_ref.h,
                0,
                0,
                atlas_ref.w,
                atlas_ref.h,
                gl::COLOR_BUFFER_BIT,
                gl::NEAREST,
            );

            gl::BindFramebuffer(gl::READ_FRAMEBUFFER, prev_read_fbo as _);
            gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, prev_draw_fbo as _);
        }
        Ok(new_sprite)
    }

    fn delete_sprite(&mut self, atlas_ref: AtlasRef) {
        // this only deletes sprites created with upload_sprite
        if atlas_ref.atlas_id >= self.stock_atlas_count {
            let tex_id = self.texture_ids[atlas_ref.atlas_id as usize].unwrap();
            unsafe {
                gl::DeleteTextures(1, &tex_id);
            }
            self.texture_ids[atlas_ref.atlas_id as usize] = None;
            if let Some(Some(fbo)) = self.fbo_ids.get(atlas_ref.atlas_id as usize) {
                unsafe {
                    gl::DeleteFramebuffers(1, fbo);
                }
                self.fbo_ids[atlas_ref.atlas_id as usize] = None;
            }
        }
    }

    fn set_swap_interval(&self, n: Option<u32>) -> bool {
        unsafe { self.imp.set_swap_interval(n.unwrap_or(0)) }
    }

    fn create_surface(&mut self, width: i32, height: i32) -> Result<AtlasRef, String> {
        let atlas_id = if let Some(id) = self.texture_ids.iter().position(|x| x.is_none()) {
            id as u32
        } else {
            self.texture_ids.push(None);
            self.fbo_ids.push(None);
            self.texture_ids.len() as u32 - 1
        };
        unsafe {
            // store previous
            let mut prev_tex2d = 0;
            gl::GetIntegerv(gl::TEXTURE_BINDING_2D, &mut prev_tex2d);
            let mut prev_fbo = 0;
            gl::GetIntegerv(gl::READ_FRAMEBUFFER_BINDING, &mut prev_fbo);

            // generate new
            let mut tex_id: GLuint = 0;
            gl::GenTextures(1, &mut tex_id);
            gl::BindTexture(gl::TEXTURE_2D, tex_id);

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as _);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as _);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as _);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as _);

            gl::TexImage2D(
                gl::TEXTURE_2D,    // target
                0,                 // level
                gl::RGBA as _,     // internalformat
                width as _,        // width
                height as _,       // height
                0,                 // border ("must be 0")
                gl::RGBA,          // format
                gl::UNSIGNED_BYTE, // type
                ptr::null(),       // data
            );

            // verify it actually worked
            match gl::GetError() {
                0 => (),
                err => return Err(format!("Failed to allocate texture on GPU! (OpenGL code {})", err)),
            }

            // generate fbo
            let mut fbo = 0;
            gl::GenFramebuffers(1, &mut fbo);
            gl::BindFramebuffer(gl::READ_FRAMEBUFFER, fbo);
            gl::FramebufferTexture2D(gl::READ_FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, tex_id, 0);

            // store opengl texture handles
            self.texture_ids[atlas_id as usize] = Some(tex_id);
            self.fbo_ids[atlas_id as usize] = Some(fbo);

            // cleanup
            gl::BindTexture(gl::TEXTURE_2D, prev_tex2d as _);
            gl::BindFramebuffer(gl::READ_FRAMEBUFFER, prev_fbo as _);
        }
        Ok(AtlasRef { atlas_id, x: 0, y: 0, w: width, h: height, origin_x: 0.0, origin_y: 0.0 })
    }

    fn set_target(&mut self, atlas_ref: &AtlasRef) {
        self.flush_queue();
        if let Some(Some(fbo_id)) = self.fbo_ids.get(atlas_ref.atlas_id as usize) {
            unsafe {
                gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, *fbo_id);
            }
            self.set_view(
                atlas_ref.w as _,
                atlas_ref.h as _,
                atlas_ref.w as _,
                atlas_ref.h as _,
                atlas_ref.x,
                atlas_ref.y, // set_view flips upside down, so unflip for surfaces
                atlas_ref.w,
                atlas_ref.h,
                0.0,
                atlas_ref.x,
                atlas_ref.y,
                atlas_ref.w,
                atlas_ref.h,
            );
        }
    }

    fn reset_target(&mut self, w: i32, h: i32, unscaled_w: i32, unscaled_h: i32) {
        unsafe {
            self.flush_queue();
            gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, 0);
        }
        self.set_view(w as _, h as _, unscaled_w as _, unscaled_h as _, 0, 0, w, h, 0.0, 0, 0, w, h);
    }

    fn dump_sprite(&self, atlas_ref: &AtlasRef) -> Box<[u8]> {
        unsafe {
            // store read fbo
            let mut prev_read_fbo: GLint = 0;
            gl::GetIntegerv(gl::READ_FRAMEBUFFER_BINDING, &mut prev_read_fbo);

            // bind texture fbo
            gl::BindFramebuffer(
                gl::READ_FRAMEBUFFER,
                self.fbo_ids[atlas_ref.atlas_id as usize].expect("Trying to dump nonexistent sprite"),
            );

            // read data
            let len = (atlas_ref.w * atlas_ref.h * 4) as usize;
            let mut data: Vec<u8> = Vec::with_capacity(len);
            data.set_len(len);
            gl::ReadPixels(
                atlas_ref.x,
                atlas_ref.y,
                atlas_ref.w,
                atlas_ref.h,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                data.as_mut_ptr().cast(),
            );

            assert_eq!(gl::GetError(), 0);

            // cleanup
            gl::BindFramebuffer(gl::READ_FRAMEBUFFER, prev_read_fbo as GLuint);

            data.into_boxed_slice()
        }
    }

    fn get_pixels(&self, x: i32, y: i32, w: i32, h: i32) -> Box<[u8]> {
        unsafe {
            let len = (w * h * 3) as usize;
            let mut data: Vec<u8> = Vec::with_capacity(len);
            data.set_len(len);
            gl::ReadPixels(x, y, w, h, gl::RGB, gl::UNSIGNED_BYTE, data.as_mut_ptr().cast());
            data.into_boxed_slice()
        }
    }

    fn draw_raw_frame(&mut self, rgb: Box<[u8]>, w: i32, h: i32, clear_colour: Colour) {
        unsafe {
            // store previous texture, upload new texture to gpu
            let mut prev_tex2d = 0;
            gl::GetIntegerv(gl::TEXTURE_BINDING_2D, &mut prev_tex2d);
            let mut tex: GLuint = 0;
            gl::GenTextures(1, &mut tex);
            gl::BindTexture(gl::TEXTURE_2D, tex);
            gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGB as _, w, h, 0, gl::RGB, gl::UNSIGNED_BYTE, rgb.as_ptr().cast());

            assert_eq!(gl::GetError(), 0);

            // store read fbo
            let mut prev_read_fbo: GLint = 0;
            gl::GetIntegerv(gl::READ_FRAMEBUFFER_BINDING, &mut prev_read_fbo);

            // setup temp fbo
            let mut fbo: GLuint = 0;
            gl::GenFramebuffers(1, &mut fbo);
            gl::BindFramebuffer(gl::READ_FRAMEBUFFER, fbo);

            // bind texture to fbo
            gl::FramebufferTexture2D(gl::READ_FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, tex, 0);

            // draw the damn thing
            gl::BlitFramebuffer(0, 0, w, h, 0, 0, w, h, gl::COLOR_BUFFER_BIT, gl::NEAREST); // <- TODO applies here

            assert_eq!(gl::GetError(), 0);

            // cleanup
            gl::BindFramebuffer(gl::READ_FRAMEBUFFER, prev_read_fbo as GLuint);
            gl::BindTexture(gl::TEXTURE_2D, prev_tex2d as GLuint);
            gl::DeleteFramebuffers(1, &fbo);
            gl::DeleteTextures(1, &tex);

            self.imp.swap_buffers();
        }
        self.draw_queue.clear();
        self.setup_frame(w as _, h as _, clear_colour);
    }

    fn dump_dynamic_textures(&self) -> Vec<Option<SavedTexture>> {
        unsafe {
            // store previous
            let mut prev_tex2d = 0;
            gl::GetIntegerv(gl::TEXTURE_BINDING_2D, &mut prev_tex2d);

            let mut textures = Vec::with_capacity(self.texture_ids.len() - self.stock_atlas_count as usize);
            for tex_id in self.texture_ids.iter().skip(self.stock_atlas_count as usize).copied() {
                textures.push(match tex_id {
                    Some(tex_id) => {
                        gl::BindTexture(gl::TEXTURE_2D, tex_id);
                        let mut width = 0;
                        let mut height = 0;
                        gl::GetTexLevelParameteriv(gl::TEXTURE_2D, 0, gl::TEXTURE_WIDTH, &mut width);
                        gl::GetTexLevelParameteriv(gl::TEXTURE_2D, 0, gl::TEXTURE_HEIGHT, &mut height);
                        let mut pixels = vec![0; width as usize * height as usize * 4];
                        gl::GetTexImage(gl::TEXTURE_2D, 0, gl::RGBA, gl::UNSIGNED_BYTE, pixels.as_mut_ptr().cast());
                        Some(SavedTexture { width, height, pixels: pixels.into_boxed_slice() })
                    },
                    None => None,
                });
            }

            gl::BindTexture(gl::TEXTURE_2D, prev_tex2d as _);
            assert_eq!(gl::GetError(), 0);

            textures
        }
    }

    fn upload_dynamic_textures(&mut self, textures: &[Option<SavedTexture>]) {
        unsafe {
            for tex_id in self.texture_ids.iter_mut().skip(self.stock_atlas_count as usize) {
                if let Some(tex_id) = tex_id.as_ref() {
                    gl::DeleteTextures(1, tex_id);
                }
                *tex_id = None;
            }
            self.texture_ids.resize(self.stock_atlas_count as usize + textures.len(), None);
            for fbo_id in self.fbo_ids.iter_mut().skip(self.stock_atlas_count as usize) {
                if let Some(fbo_id) = fbo_id.as_ref() {
                    gl::DeleteFramebuffers(1, fbo_id);
                }
                *fbo_id = None;
            }
            self.fbo_ids.resize(self.stock_atlas_count as usize + textures.len(), None);
            for (i, tex) in textures.iter().enumerate() {
                let i = i + self.stock_atlas_count as usize;
                if let Some(tex) = tex.as_ref() {
                    let mut tex_id = 0;
                    gl::GenTextures(1, &mut tex_id);
                    gl::BindTexture(gl::TEXTURE_2D, tex_id);

                    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as _);
                    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as _);
                    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as _);
                    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as _);

                    gl::TexImage2D(
                        gl::TEXTURE_2D,
                        0,
                        gl::RGBA as _,
                        tex.width,
                        tex.height,
                        0,
                        gl::RGBA,
                        gl::UNSIGNED_BYTE,
                        tex.pixels.as_ptr().cast(),
                    );
                    self.texture_ids[i] = Some(tex_id);

                    let mut fbo_id = 0;
                    gl::GenFramebuffers(1, &mut fbo_id);
                    gl::BindFramebuffer(gl::READ_FRAMEBUFFER, fbo_id);
                    gl::FramebufferTexture2D(gl::READ_FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, tex_id, 0);
                    self.fbo_ids[i] = Some(fbo_id);
                }
            }
            gl::BindFramebuffer(gl::READ_FRAMEBUFFER, 0);
            assert_eq!(gl::GetError(), 0);
        }
    }

    fn draw_sprite(
        &mut self,
        texture: &AtlasRef,
        x: f64,
        y: f64,
        xscale: f64,
        yscale: f64,
        angle: f64,
        colour: i32,
        alpha: f64,
    ) {
        let atlas_ref = texture.clone();

        if atlas_ref.atlas_id != self.current_atlas {
            if self.texture_ids[atlas_ref.atlas_id as usize].is_none() {
                return
            } // fail silently when drawing deleted sprite fonts
            self.flush_queue();
            self.current_atlas = atlas_ref.atlas_id;
        }

        let angle = -angle.to_radians();
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

        self.draw_queue.push(DrawCommand {
            atlas_ref,
            model_view_matrix,
            blend: (
                ((colour & 0xFF) as f32) / 255.0,
                (((colour >> 8) & 0xFF) as f32) / 255.0,
                (((colour >> 16) & 0xFF) as f32) / 255.0,
            ),
            alpha: alpha as f32,
        });
    }

    fn draw_rectangle(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, colour: i32, alpha: f64) {
        let copied_pixel = self.white_pixel;
        self.draw_sprite(&copied_pixel, x1, y1, x2 + 1.0 - x1, y2 + 1.0 - y1, 0.0, colour, alpha)
    }

    fn draw_rectangle_outline(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, colour: i32, alpha: f64) {
        let copied_pixel = self.white_pixel;
        self.draw_sprite(&copied_pixel, x1, y1, x2 + 1.0 - x1, 1.0, 0.0, colour, alpha); // top line
        self.draw_sprite(&copied_pixel, x1, y2, x2 + 1.0 - x1, 1.0, 0.0, colour, alpha); // bottom line
        self.draw_sprite(&copied_pixel, x1, y1, 1.0, y2 + 1.0 - y1, 0.0, colour, alpha); // left line
        self.draw_sprite(&copied_pixel, x2, y1, 1.0, y2 + 1.0 - y1, 0.0, colour, alpha); // right line
    }

    fn get_blend_mode(&self) -> (BlendType, BlendType) {
        let mut src: GLint = 0;
        let mut dst: GLint = 0;
        unsafe {
            gl::GetIntegerv(gl::BLEND_SRC_RGB, &mut src);
            gl::GetIntegerv(gl::BLEND_DST_RGB, &mut dst);
        }
        ((src as GLenum).into(), (dst as GLenum).into())
    }

    fn set_blend_mode(&mut self, src: BlendType, dst: BlendType) {
        self.flush_queue();
        unsafe {
            gl::BlendFunc(src.into(), dst.into());
        }
    }

    /// Does anything that's queued to be done.
    fn flush_queue(&mut self) {
        if self.draw_queue.is_empty() {
            return
        }

        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.texture_ids[self.current_atlas as usize].unwrap());

            let mut commands_vbo: GLuint = 0;
            gl::GenBuffers(1, &mut commands_vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, commands_vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (size_of::<DrawCommand>() * self.draw_queue.len()) as _,
                self.draw_queue.as_ptr().cast(),
                gl::STATIC_DRAW,
            );

            gl::Uniform1i(self.loc_tex, 0 as _);

            // layout (location = 1) in mat4 model_view;
            // layout (location = 6) in vec4 atlas_xywh;
            // layout (location = 7) in vec3 blend;
            // layout (location = 8) in float alpha;
            gl::EnableVertexAttribArray(1);
            gl::VertexAttribPointer(
                1,
                4,
                gl::FLOAT,
                gl::FALSE,
                size_of::<DrawCommand>() as i32,
                offset_of!(DrawCommand, model_view_matrix) as *const _,
            );
            gl::EnableVertexAttribArray(2);
            gl::VertexAttribPointer(
                2,
                4,
                gl::FLOAT,
                gl::FALSE,
                size_of::<DrawCommand>() as i32,
                (offset_of!(DrawCommand, model_view_matrix) + (4 * size_of::<f32>())) as *const _,
            );
            gl::EnableVertexAttribArray(3);
            gl::VertexAttribPointer(
                3,
                4,
                gl::FLOAT,
                gl::FALSE,
                size_of::<DrawCommand>() as i32,
                (offset_of!(DrawCommand, model_view_matrix) + (8 * size_of::<f32>())) as *const _,
            );
            gl::EnableVertexAttribArray(4);
            gl::VertexAttribPointer(
                4,
                4,
                gl::FLOAT,
                gl::FALSE,
                size_of::<DrawCommand>() as i32,
                (offset_of!(DrawCommand, model_view_matrix) + (12 * size_of::<f32>())) as *const _,
            );
            gl::EnableVertexAttribArray(6);
            gl::VertexAttribPointer(
                6,
                4,
                gl::INT,
                gl::FALSE,
                size_of::<DrawCommand>() as i32,
                (offset_of!(DrawCommand, atlas_ref) + offset_of!(AtlasRef, x)) as *const _,
            );
            gl::EnableVertexAttribArray(7);
            gl::VertexAttribPointer(
                7,
                3,
                gl::FLOAT,
                gl::FALSE,
                size_of::<DrawCommand>() as i32,
                offset_of!(DrawCommand, blend) as *const _,
            );
            gl::EnableVertexAttribArray(8);
            gl::VertexAttribPointer(
                8,
                1,
                gl::FLOAT,
                gl::FALSE,
                size_of::<DrawCommand>() as i32,
                offset_of!(DrawCommand, alpha) as *const _,
            );
            gl::VertexAttribDivisor(1, 1);
            gl::VertexAttribDivisor(2, 1);
            gl::VertexAttribDivisor(3, 1);
            gl::VertexAttribDivisor(4, 1);
            gl::VertexAttribDivisor(6, 1);
            gl::VertexAttribDivisor(7, 1);
            gl::VertexAttribDivisor(8, 1);

            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);

            // layout (location = 5) in vec2 tex_coord;
            gl::EnableVertexAttribArray(5);
            gl::VertexAttribPointer(5, 2, gl::FLOAT, gl::FALSE, (3 * size_of::<f32>()) as _, 0 as _);

            gl::DrawArraysInstanced(gl::TRIANGLE_STRIP, 0, 4, self.draw_queue.len() as i32);

            gl::DeleteBuffers(1, &commands_vbo);
        }

        self.draw_queue.clear();
    }

    fn set_view(
        &mut self,
        width: u32,
        height: u32,
        unscaled_width: u32,
        unscaled_height: u32,
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
        self.flush_queue();

        // Make projection matrix for new view
        // Note: sin is negated because it's the same as negating the angle, which is how GM8 does view angles
        let sin_angle = -src_angle.sin() as f32;
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
        let (width, height) = (width as i32, height as i32);
        let port_w = ((port_w * width) as f64 / unscaled_width as f64) as i32;
        let port_h = ((port_h * height) as f64 / unscaled_height as f64) as i32;
        let port_x = ((port_x * width) as f64 / unscaled_width as f64) as i32;
        let port_y = height - (((port_y * height) as f64 / unscaled_height as f64) as i32 + port_h);

        // Set viewport (gl::Viewport, gl::Scissor) and projection matrix (shader uniform)
        unsafe {
            gl::Viewport(port_x, port_y, port_w, port_h);
            gl::Scissor(port_x, port_y, port_w, port_h);

            gl::UniformMatrix4fv(self.loc_proj, 1, gl::FALSE, projection.as_ptr());
        }
    }

    fn clear_view(&mut self, colour: Colour, alpha: f64) {
        self.flush_queue();
        unsafe {
            gl::ClearColor(colour.r as f32, colour.g as f32, colour.b as f32, alpha as f32);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
    }

    fn present(&mut self) {
        // Finish drawing frame
        self.flush_queue();

        // Swap buffers
        unsafe {
            self.imp.swap_buffers();
        }
    }

    fn finish(&mut self, width: u32, height: u32, clear_colour: Colour) {
        // Present screen
        self.present();

        // Start next frame
        self.setup_frame(width, height, clear_colour)
    }
}
