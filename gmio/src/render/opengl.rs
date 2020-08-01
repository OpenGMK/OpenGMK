mod wgl;

use crate::{
    atlas::{AtlasBuilder, AtlasRef},
    render::{mat4mult, BlendType, RendererOptions, RendererTrait, SavedTexture, Scaling},
    window::Window,
};
use cfg_if::cfg_if;
use memoffset::offset_of;
use rect_packer::DensePacker;
use shared::types::Colour;
use std::{any::Any, ffi::CStr, mem::size_of, ptr};

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
    gl: gl::Gl,
    //program: GLuint,
    //vao: GLuint,
    vbo: GLuint,

    atlas_packers: Vec<DensePacker>,
    texture_ids: Vec<Option<GLuint>>,
    fbo_ids: Vec<Option<GLuint>>,
    stock_atlas_count: u32,
    current_atlas: GLuint,
    framebuffer_texture: GLuint,
    framebuffer_fbo: GLuint,
    white_pixel: AtlasRef,
    draw_queue: Vec<DrawCommand>,
    interpolate_pixels: bool,

    model_matrix: [f32; 16],
    view_matrix: [f32; 16],
    proj_matrix: [f32; 16],

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

unsafe fn shader_info_log(gl: &gl::Gl, name: &str, id: GLuint) -> String {
    let mut info_len: GLint = 0;
    gl.GetShaderiv(id, gl::INFO_LOG_LENGTH, &mut info_len);
    let mut info = vec![0u8; info_len as usize];
    gl.GetShaderInfoLog(id, info_len as GLsizei, ptr::null_mut(), info.as_mut_ptr() as *mut GLchar);
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

            // gl function pointers
            let gl = gl::Gl::load_with(imp::PlatformImpl::get_function_loader()?);
            imp::PlatformImpl::clean_function_loader();

            // debug print
            let ver_str = CStr::from_ptr(gl.GetString(gl::VERSION).cast()).to_str().unwrap();
            println!("OpenGL Version: {}", ver_str);
            let vendor_str = CStr::from_ptr(gl.GetString(gl::VENDOR).cast()).to_str().unwrap();
            println!("OpenGL Vendor: {}", vendor_str);

            // requires at least GL 3.3
            let mut v_maj: GLint = 0;
            gl.GetIntegerv(gl::MAJOR_VERSION, &mut v_maj);
            let mut v_min: GLint = 0;
            gl.GetIntegerv(gl::MINOR_VERSION, &mut v_min);
            assert!(
                (v_maj == 3 && v_min >= 3) || v_maj > 3,
                "OpenGL version 3.3 or later is required, but found version {}.{}",
                v_maj,
                v_min
            );

            if options.vsync {
                imp.set_swap_interval(1);
            } else {
                imp.set_swap_interval(0);
            }

            // Compile vertex shader
            let vertex_shader = gl.CreateShader(gl::VERTEX_SHADER);
            gl.ShaderSource(vertex_shader, 1, &(VERTEX_SHADER_SOURCE.as_ptr().cast()), ptr::null());
            gl.CompileShader(vertex_shader);

            // Check for vertex shader compile errors
            let mut success = gl::FALSE as GLint;
            gl.GetShaderiv(vertex_shader, gl::COMPILE_STATUS, &mut success);
            if success != gl::TRUE as GLint {
                return Err(shader_info_log(&gl, "vertex", vertex_shader))
            }

            // Compile fragment shader
            let fragment_shader = gl.CreateShader(gl::FRAGMENT_SHADER);
            gl.ShaderSource(fragment_shader, 1, &(FRAGMENT_SHADER_SOURCE.as_ptr().cast()), ptr::null());
            gl.CompileShader(fragment_shader);

            // Check for fragment shader compile errors
            gl.GetShaderiv(fragment_shader, gl::COMPILE_STATUS, &mut success);
            if success != gl::TRUE as GLint {
                return Err(shader_info_log(&gl, "fragment", fragment_shader))
            }

            // Link shaders
            let program = gl.CreateProgram();
            gl.AttachShader(program, vertex_shader);
            gl.AttachShader(program, fragment_shader);
            gl.LinkProgram(program);

            // Check for linking errors
            // TODO: generalize this like with shader info logs!! please!!!
            gl.GetProgramiv(program, gl::LINK_STATUS, &mut success);
            if success != gl::TRUE as GLint {
                let mut info_len: GLint = 0;
                gl.GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut info_len);
                let mut info = vec![0u8; info_len as usize];
                gl.GetProgramInfoLog(program, info_len as GLsizei, ptr::null_mut(), info.as_mut_ptr() as *mut GLchar);
                info.set_len((info_len - 1) as usize); // ignore null for str::from_utf8
                return Err(format!(
                    "Failed to link shaders, compiler output:\n{}",
                    std::str::from_utf8(&info).unwrap_or("<INVALID UTF-8>")
                ))
            }
            gl.DeleteShader(vertex_shader);
            gl.DeleteShader(fragment_shader);

            // set up vertex data and configure vertex attributes
            let vertices: [f32; 12] = [
                0.0, 0.0, 0.0, // bottom left
                1.0, 0.0, 0.0, // bottom right
                0.0, 1.0, 0.0, // top left
                1.0, 1.0, 0.0, // top right
            ];
            let (mut vbo, mut vao) = (0, 0);
            gl.GenVertexArrays(1, &mut vao);
            gl.GenBuffers(1, &mut vbo);
            gl.BindVertexArray(vao);

            gl.BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl.BufferData(
                gl::ARRAY_BUFFER,
                (vertices.len() * size_of::<GLfloat>()) as GLsizeiptr,
                vertices.as_ptr().cast(),
                gl::STATIC_DRAW,
            );

            gl.VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 3 * size_of::<GLfloat>() as GLsizei, ptr::null());
            gl.EnableVertexAttribArray(0);

            // Enable and disable GL features
            gl.Enable(gl::SCISSOR_TEST);
            // gl::Enable(gl::TEXTURE_2D);
            gl.Disable(gl::CULL_FACE);
            gl.Enable(gl::BLEND);
            gl.Disable(gl::DEPTH_TEST);

            gl.BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);

            // Unbind VBO
            gl.BindBuffer(gl::ARRAY_BUFFER, 0);

            // Use program
            gl.UseProgram(program);

            // Configure gl::ReadPixels() to read from the back buffer
            gl.ReadBuffer(gl::BACK);

            // Configure gl::ReadPixels() to align to 1 byte
            gl.PixelStorei(gl::PACK_ALIGNMENT, 1);

            // Create framebuffer
            let (mut framebuffer_texture, mut framebuffer_fbo) = (0, 0);
            gl.GenTextures(1, &mut framebuffer_texture);
            gl.BindTexture(gl::TEXTURE_2D, framebuffer_texture);
            gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as _);
            gl.TexImage2D(
                gl::TEXTURE_2D,      // target
                0,                   // level
                gl::RGBA as _,       // internalformat
                options.size.0 as _, // width
                options.size.1 as _, // height
                0,                   // border ("must be 0")
                gl::RGBA,            // format
                gl::UNSIGNED_BYTE,   // type
                ptr::null(),         // data
            );
            gl.GenFramebuffers(1, &mut framebuffer_fbo);
            gl.BindFramebuffer(gl::FRAMEBUFFER, framebuffer_fbo);
            gl.FramebufferTexture2D(
                gl::READ_FRAMEBUFFER,
                gl::COLOR_ATTACHMENT0,
                gl::TEXTURE_2D,
                framebuffer_texture,
                0,
            );

            // Create identity matrix to initialize MVP matrices with
            #[rustfmt::skip]
            let identity_matrix: [f32; 16] = [
                1.0, 0.0, 0.0, 0.0,
                0.0, 1.0, 0.0, 0.0,
                0.0, 0.0, 1.0, 0.0,
                0.0, 0.0, 0.0, 1.0,
            ];

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
                framebuffer_texture,
                framebuffer_fbo,
                white_pixel: Default::default(),
                draw_queue: Vec::with_capacity(256),
                interpolate_pixels: options.interpolate_pixels,

                model_matrix: identity_matrix.clone(),
                view_matrix: identity_matrix.clone(),
                proj_matrix: identity_matrix.clone(),

                loc_tex: gl.GetUniformLocation(program, b"tex\0".as_ptr().cast()),
                loc_proj: gl.GetUniformLocation(program, b"projection\0".as_ptr().cast()),

                gl,
            };

            // Start first frame
            renderer.setup_frame(clear_colour);

            // verify it actually worked
            match renderer.gl.GetError() {
                0 => Ok(renderer),
                err => Err(format!("Failed to fully initialize OpenGL! (OpenGL code {})", err)),
            }
        }
    }

    fn setup_frame(&mut self, clear_colour: Colour) {
        unsafe {
            // get framebuffer size
            let (mut width, mut height) = (0, 0);
            self.gl.BindTexture(gl::TEXTURE_2D, self.framebuffer_texture);
            self.gl.GetTexLevelParameteriv(gl::TEXTURE_2D, 0, gl::TEXTURE_WIDTH, &mut width);
            self.gl.GetTexLevelParameteriv(gl::TEXTURE_2D, 0, gl::TEXTURE_HEIGHT, &mut height);
            // setup proj and viewport
            self.set_projection_ortho(0.0, 0.0, width.into(), height.into(), 0.0);
            self.gl.Viewport(0, 0, width as _, height as _);
            self.gl.Scissor(0, 0, width as _, height as _);
            // clear screen
            self.gl.ClearColor(clear_colour.r as f32, clear_colour.g as f32, clear_colour.b as f32, 1.0);
            self.gl.Clear(gl::COLOR_BUFFER_BIT);
        }
    }

    fn update_matrix(&mut self) {
        let viewproj = mat4mult(self.model_matrix, mat4mult(self.view_matrix, self.proj_matrix));
        unsafe {
            self.gl.UniformMatrix4fv(self.loc_proj, 1, gl::FALSE, viewproj.as_ptr());
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
            self.gl.GetIntegerv(gl::MAX_TEXTURE_SIZE, &mut size);
            if size == 16384 && CStr::from_ptr(self.gl.GetString(gl::VENDOR).cast()).to_bytes() == b"Intel" {
                // Intel driver bug throws GL_OUT_OF_MEMORY when allocating 16384x16384 texture
                // TODO: find proper fix or maybe allow allocating 16384x8192?
                size = 8192;
            }
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
                self.gl.GenTextures(buf.len() as _, buf.as_mut_ptr());
                for (i, (tex_id, packer)) in buf.iter().copied().zip(&packers).enumerate() {
                    let (width, height) = packer.size();

                    self.gl.BindTexture(gl::TEXTURE_2D, tex_id);
                    self.current_atlas = i as u32;

                    self.gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as _);
                    self.gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as _);
                    self.gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as _);
                    self.gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as _);
                    self.gl.TexImage2D(
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
            match self.gl.GetError() {
                0 => (),
                err => return Err(format!("Failed to allocate texture on GPU! (OpenGL code {})", err)),
            }

            // upload textures
            for (atl_ref, pixels) in &sprites {
                if self.current_atlas != atl_ref.atlas_id {
                    self.gl.BindTexture(gl::TEXTURE_2D, textures[atl_ref.atlas_id as usize]);
                    self.current_atlas = atl_ref.atlas_id;
                }

                self.gl.TexSubImage2D(
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
            match self.gl.GetError() {
                0 => (),
                err => return Err(format!("Failed to upload textures to GPU! (OpenGL code {})", err)),
            }

            // generate framebuffers
            let mut fbo_ids = Vec::with_capacity(textures.len());
            for tex_id in &textures {
                let mut fbo = 0;
                self.gl.GenFramebuffers(1, &mut fbo);
                self.gl.BindFramebuffer(gl::READ_FRAMEBUFFER, fbo);
                self.gl.FramebufferTexture2D(gl::READ_FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, *tex_id, 0);
                fbo_ids.push(Some(fbo));
            }
            self.gl.BindFramebuffer(gl::READ_FRAMEBUFFER, self.framebuffer_fbo);

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
            self.gl.GetIntegerv(gl::TEXTURE_BINDING_2D, &mut prev_tex2d);

            // upload texture
            self.gl.BindTexture(gl::TEXTURE_2D, self.texture_ids[atlas_ref.atlas_id as usize].unwrap());
            self.gl.TexSubImage2D(
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
            match self.gl.GetError() {
                0 => (),
                err => return Err(format!("Failed to upload texture to GPU! (OpenGL code {})", err)),
            }

            // cleanup
            self.gl.BindTexture(gl::TEXTURE_2D, prev_tex2d as _);
        }
        Ok(atlas_ref)
    }

    fn duplicate_sprite(&mut self, atlas_ref: &AtlasRef) -> Result<AtlasRef, String> {
        let new_sprite = self.create_surface(atlas_ref.w, atlas_ref.h)?;
        unsafe {
            // store previous
            let mut prev_read_fbo = 0;
            self.gl.GetIntegerv(gl::READ_FRAMEBUFFER_BINDING, &mut prev_read_fbo);
            let mut prev_tex2d = 0;
            self.gl.GetIntegerv(gl::TEXTURE_BINDING_2D, &mut prev_tex2d);

            self.gl.BindFramebuffer(gl::READ_FRAMEBUFFER, self.fbo_ids[atlas_ref.atlas_id as usize].unwrap());
            self.gl.BindTexture(gl::TEXTURE_2D, self.texture_ids[new_sprite.atlas_id as usize].unwrap());

            self.gl.CopyTexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA,
                atlas_ref.x,
                atlas_ref.y,
                atlas_ref.w as _,
                atlas_ref.h as _,
                0,
            );

            self.gl.BindFramebuffer(gl::READ_FRAMEBUFFER, prev_read_fbo as _);
            self.gl.BindTexture(gl::TEXTURE_2D, prev_tex2d as _);
        }
        Ok(new_sprite)
    }

    fn delete_sprite(&mut self, atlas_ref: AtlasRef) {
        // this only deletes sprites created with upload_sprite
        self.flush_queue();
        if atlas_ref.atlas_id >= self.stock_atlas_count {
            let tex_id = self.texture_ids[atlas_ref.atlas_id as usize].unwrap();
            unsafe {
                self.gl.DeleteTextures(1, &tex_id);
            }
            self.texture_ids[atlas_ref.atlas_id as usize] = None;
            if let Some(Some(fbo)) = self.fbo_ids.get(atlas_ref.atlas_id as usize) {
                unsafe {
                    self.gl.DeleteFramebuffers(1, fbo);
                }
                self.fbo_ids[atlas_ref.atlas_id as usize] = None;
            }
        }
    }

    fn set_vsync(&self, vsync: bool) {
        unsafe { self.imp.set_swap_interval(if vsync { 1 } else { 0 }) };
    }

    fn get_vsync(&self) -> bool {
        unsafe { self.imp.get_swap_interval() != 0 }
    }

    fn wait_vsync(&self) {
        unsafe { self.imp.wait_vsync() }
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
            self.gl.GetIntegerv(gl::TEXTURE_BINDING_2D, &mut prev_tex2d);
            let mut prev_fbo = 0;
            self.gl.GetIntegerv(gl::READ_FRAMEBUFFER_BINDING, &mut prev_fbo);

            // generate new
            let mut tex_id: GLuint = 0;
            self.gl.GenTextures(1, &mut tex_id);
            self.gl.BindTexture(gl::TEXTURE_2D, tex_id);

            self.gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as _);
            self.gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as _);
            self.gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as _);
            self.gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as _);

            self.gl.TexImage2D(
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
            match self.gl.GetError() {
                0 => (),
                err => return Err(format!("Failed to allocate texture on GPU! (OpenGL code {})", err)),
            }

            // generate fbo
            let mut fbo = 0;
            self.gl.GenFramebuffers(1, &mut fbo);
            self.gl.BindFramebuffer(gl::READ_FRAMEBUFFER, fbo);
            self.gl.FramebufferTexture2D(gl::READ_FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, tex_id, 0);

            // store opengl texture handles
            self.texture_ids[atlas_id as usize] = Some(tex_id);
            self.fbo_ids[atlas_id as usize] = Some(fbo);

            // cleanup
            self.gl.BindTexture(gl::TEXTURE_2D, prev_tex2d as _);
            self.gl.BindFramebuffer(gl::READ_FRAMEBUFFER, prev_fbo as _);
        }
        Ok(AtlasRef { atlas_id, x: 0, y: 0, w: width, h: height, origin_x: 0.0, origin_y: 0.0 })
    }

    fn set_target(&mut self, atlas_ref: &AtlasRef) {
        self.flush_queue();
        if let Some(Some(fbo_id)) = self.fbo_ids.get(atlas_ref.atlas_id as usize) {
            unsafe {
                self.gl.BindFramebuffer(gl::DRAW_FRAMEBUFFER, *fbo_id);
            }
            self.set_view(
                atlas_ref.x,
                atlas_ref.y,
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

    fn reset_target(&mut self) {
        self.flush_queue();
        unsafe {
            self.gl.BindFramebuffer(gl::DRAW_FRAMEBUFFER, self.framebuffer_fbo);

            // reset view
            let (mut fb_width, mut fb_height) = (0, 0);
            self.gl.BindTexture(gl::TEXTURE_2D, self.framebuffer_texture);
            self.gl.GetTexLevelParameteriv(gl::TEXTURE_2D, 0, gl::TEXTURE_WIDTH, &mut fb_width);
            self.gl.GetTexLevelParameteriv(gl::TEXTURE_2D, 0, gl::TEXTURE_HEIGHT, &mut fb_height);
            self.set_view(0, 0, fb_width, fb_height, 0.0, 0, 0, fb_width, fb_height);
        }
    }

    fn resize_framebuffer(&mut self, width: u32, height: u32) {
        self.flush_queue();
        unsafe {
            // get old size
            let old_tex = self.framebuffer_texture;
            self.gl.BindTexture(gl::TEXTURE_2D, old_tex);
            let (mut old_width, mut old_height) = (0, 0);
            self.gl.GetTexLevelParameteriv(gl::TEXTURE_2D, 0, gl::TEXTURE_WIDTH, &mut old_width);
            self.gl.GetTexLevelParameteriv(gl::TEXTURE_2D, 0, gl::TEXTURE_HEIGHT, &mut old_height);
            // set up new texture
            self.gl.GenTextures(1, &mut self.framebuffer_texture);
            self.gl.BindTexture(gl::TEXTURE_2D, self.framebuffer_texture);
            self.gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as _);
            self.gl.TexImage2D(
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
            // set up new fbo
            let old_fbo = self.framebuffer_fbo;
            self.gl.GenFramebuffers(1, &mut self.framebuffer_fbo);
            self.gl.BindFramebuffer(gl::FRAMEBUFFER, self.framebuffer_fbo);
            self.gl.FramebufferTexture2D(
                gl::READ_FRAMEBUFFER,
                gl::COLOR_ATTACHMENT0,
                gl::TEXTURE_2D,
                self.framebuffer_texture,
                0,
            );
            // draw old fb onto new
            self.gl.BindFramebuffer(gl::READ_FRAMEBUFFER, old_fbo);
            let copy_width = width.min(old_width as _);
            let copy_height = height.min(old_height as _);
            self.gl.BlitFramebuffer(
                0,
                0,
                copy_width as _,
                copy_height as _,
                0,
                0,
                copy_width as _,
                copy_height as _,
                gl::COLOR_BUFFER_BIT,
                gl::LINEAR,
            );
            // delete old texture and fbo
            self.gl.DeleteTextures(1, &old_tex);
            self.gl.DeleteFramebuffers(1, &old_fbo);
        }
    }

    fn dump_sprite(&self, atlas_ref: &AtlasRef) -> Box<[u8]> {
        unsafe {
            // store read fbo
            let mut prev_read_fbo: GLint = 0;
            self.gl.GetIntegerv(gl::READ_FRAMEBUFFER_BINDING, &mut prev_read_fbo);

            // bind texture fbo
            self.gl.BindFramebuffer(
                gl::READ_FRAMEBUFFER,
                self.fbo_ids[atlas_ref.atlas_id as usize].expect("Trying to dump nonexistent sprite"),
            );

            // read data
            let len = (atlas_ref.w * atlas_ref.h * 4) as usize;
            let mut data: Vec<u8> = Vec::with_capacity(len);
            data.set_len(len);
            self.gl.ReadPixels(
                atlas_ref.x,
                atlas_ref.y,
                atlas_ref.w,
                atlas_ref.h,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                data.as_mut_ptr().cast(),
            );

            assert_eq!(self.gl.GetError(), 0);

            // cleanup
            self.gl.BindFramebuffer(gl::READ_FRAMEBUFFER, prev_read_fbo as GLuint);

            data.into_boxed_slice()
        }
    }

    fn get_pixels(&self, x: i32, y: i32, w: i32, h: i32) -> Box<[u8]> {
        unsafe {
            let len = (w * h * 4) as usize;
            let mut data: Vec<u8> = Vec::with_capacity(len);
            data.set_len(len);
            self.gl.BindFramebuffer(gl::READ_FRAMEBUFFER, self.framebuffer_fbo);
            self.gl.ReadPixels(x, y, w, h, gl::RGBA, gl::UNSIGNED_BYTE, data.as_mut_ptr().cast());
            data.into_boxed_slice()
        }
    }

    fn draw_raw_frame(&mut self, rgba: Box<[u8]>, w: i32, h: i32, clear_colour: Colour) {
        unsafe {
            // resize framebuffer
            self.resize_framebuffer(w as _, h as _);
            // upload new frame
            self.gl.BindTexture(gl::TEXTURE_2D, self.framebuffer_texture);
            self.gl.TexSubImage2D(gl::TEXTURE_2D, 0, 0, 0, w, h, gl::RGBA, gl::UNSIGNED_BYTE, rgba.as_ptr().cast());

            assert_eq!(self.gl.GetError(), 0);
        }
        self.draw_queue.clear();
        self.present(w as _, h as _);
        self.setup_frame(clear_colour);
    }

    fn dump_dynamic_textures(&self) -> Vec<Option<SavedTexture>> {
        unsafe {
            // store previous
            let mut prev_tex2d = 0;
            self.gl.GetIntegerv(gl::TEXTURE_BINDING_2D, &mut prev_tex2d);

            let mut textures = Vec::with_capacity(self.texture_ids.len() - self.stock_atlas_count as usize);
            for tex_id in self.texture_ids.iter().skip(self.stock_atlas_count as usize).copied() {
                textures.push(match tex_id {
                    Some(tex_id) => {
                        self.gl.BindTexture(gl::TEXTURE_2D, tex_id);
                        let mut width = 0;
                        let mut height = 0;
                        self.gl.GetTexLevelParameteriv(gl::TEXTURE_2D, 0, gl::TEXTURE_WIDTH, &mut width);
                        self.gl.GetTexLevelParameteriv(gl::TEXTURE_2D, 0, gl::TEXTURE_HEIGHT, &mut height);
                        let mut pixels = vec![0; width as usize * height as usize * 4];
                        self.gl.GetTexImage(gl::TEXTURE_2D, 0, gl::RGBA, gl::UNSIGNED_BYTE, pixels.as_mut_ptr().cast());
                        Some(SavedTexture { width, height, pixels: pixels.into_boxed_slice() })
                    },
                    None => None,
                });
            }

            self.gl.BindTexture(gl::TEXTURE_2D, prev_tex2d as _);
            assert_eq!(self.gl.GetError(), 0);

            textures
        }
    }

    fn upload_dynamic_textures(&mut self, textures: &[Option<SavedTexture>]) {
        unsafe {
            for tex_id in self.texture_ids.iter_mut().skip(self.stock_atlas_count as usize) {
                if let Some(tex_id) = tex_id.as_ref() {
                    self.gl.DeleteTextures(1, tex_id);
                }
                *tex_id = None;
            }
            self.texture_ids.resize(self.stock_atlas_count as usize + textures.len(), None);
            for fbo_id in self.fbo_ids.iter_mut().skip(self.stock_atlas_count as usize) {
                if let Some(fbo_id) = fbo_id.as_ref() {
                    self.gl.DeleteFramebuffers(1, fbo_id);
                }
                *fbo_id = None;
            }
            self.fbo_ids.resize(self.stock_atlas_count as usize + textures.len(), None);
            for (i, tex) in textures.iter().enumerate() {
                let i = i + self.stock_atlas_count as usize;
                if let Some(tex) = tex.as_ref() {
                    let mut tex_id = 0;
                    self.gl.GenTextures(1, &mut tex_id);
                    self.gl.BindTexture(gl::TEXTURE_2D, tex_id);

                    self.gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as _);
                    self.gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as _);
                    self.gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as _);
                    self.gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as _);

                    self.gl.TexImage2D(
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
                    self.gl.GenFramebuffers(1, &mut fbo_id);
                    self.gl.BindFramebuffer(gl::READ_FRAMEBUFFER, fbo_id);
                    self.gl.FramebufferTexture2D(
                        gl::READ_FRAMEBUFFER,
                        gl::COLOR_ATTACHMENT0,
                        gl::TEXTURE_2D,
                        tex_id,
                        0,
                    );
                    self.fbo_ids[i] = Some(fbo_id);
                }
            }
            self.gl.BindFramebuffer(gl::READ_FRAMEBUFFER, self.framebuffer_fbo);
            assert_eq!(self.gl.GetError(), 0);
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
            self.gl.GetIntegerv(gl::BLEND_SRC_RGB, &mut src);
            self.gl.GetIntegerv(gl::BLEND_DST_RGB, &mut dst);
        }
        ((src as GLenum).into(), (dst as GLenum).into())
    }

    fn set_blend_mode(&mut self, src: BlendType, dst: BlendType) {
        self.flush_queue();
        unsafe {
            self.gl.BlendFunc(src.into(), dst.into());
        }
    }

    fn get_pixel_interpolation(&self) -> bool {
        self.interpolate_pixels
    }

    fn set_pixel_interpolation(&mut self, lerping: bool) {
        // in DX (and therefore GM) this is set per texture unit, but in GL it's per texture
        // therefore, we need to apply the setting before every draw call
        self.flush_queue();
        self.interpolate_pixels = lerping;
    }

    /// Does anything that's queued to be done.
    fn flush_queue(&mut self) {
        if self.draw_queue.is_empty() {
            return
        }

        unsafe {
            self.gl.BindTexture(gl::TEXTURE_2D, self.texture_ids[self.current_atlas as usize].unwrap());
            let filter_mode = if self.interpolate_pixels { gl::LINEAR } else { gl::NEAREST };
            self.gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, filter_mode as _);
            self.gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, filter_mode as _);

            let mut commands_vbo: GLuint = 0;
            self.gl.GenBuffers(1, &mut commands_vbo);
            self.gl.BindBuffer(gl::ARRAY_BUFFER, commands_vbo);
            self.gl.BufferData(
                gl::ARRAY_BUFFER,
                (size_of::<DrawCommand>() * self.draw_queue.len()) as _,
                self.draw_queue.as_ptr().cast(),
                gl::STATIC_DRAW,
            );

            self.gl.Uniform1i(self.loc_tex, 0 as _);

            // layout (location = 1) in mat4 model_view;
            // layout (location = 6) in vec4 atlas_xywh;
            // layout (location = 7) in vec3 blend;
            // layout (location = 8) in float alpha;
            self.gl.EnableVertexAttribArray(1);
            self.gl.VertexAttribPointer(
                1,
                4,
                gl::FLOAT,
                gl::FALSE,
                size_of::<DrawCommand>() as i32,
                offset_of!(DrawCommand, model_view_matrix) as *const _,
            );
            self.gl.EnableVertexAttribArray(2);
            self.gl.VertexAttribPointer(
                2,
                4,
                gl::FLOAT,
                gl::FALSE,
                size_of::<DrawCommand>() as i32,
                (offset_of!(DrawCommand, model_view_matrix) + (4 * size_of::<f32>())) as *const _,
            );
            self.gl.EnableVertexAttribArray(3);
            self.gl.VertexAttribPointer(
                3,
                4,
                gl::FLOAT,
                gl::FALSE,
                size_of::<DrawCommand>() as i32,
                (offset_of!(DrawCommand, model_view_matrix) + (8 * size_of::<f32>())) as *const _,
            );
            self.gl.EnableVertexAttribArray(4);
            self.gl.VertexAttribPointer(
                4,
                4,
                gl::FLOAT,
                gl::FALSE,
                size_of::<DrawCommand>() as i32,
                (offset_of!(DrawCommand, model_view_matrix) + (12 * size_of::<f32>())) as *const _,
            );
            self.gl.EnableVertexAttribArray(6);
            self.gl.VertexAttribPointer(
                6,
                4,
                gl::INT,
                gl::FALSE,
                size_of::<DrawCommand>() as i32,
                (offset_of!(DrawCommand, atlas_ref) + offset_of!(AtlasRef, x)) as *const _,
            );
            self.gl.EnableVertexAttribArray(7);
            self.gl.VertexAttribPointer(
                7,
                3,
                gl::FLOAT,
                gl::FALSE,
                size_of::<DrawCommand>() as i32,
                offset_of!(DrawCommand, blend) as *const _,
            );
            self.gl.EnableVertexAttribArray(8);
            self.gl.VertexAttribPointer(
                8,
                1,
                gl::FLOAT,
                gl::FALSE,
                size_of::<DrawCommand>() as i32,
                offset_of!(DrawCommand, alpha) as *const _,
            );
            self.gl.VertexAttribDivisor(1, 1);
            self.gl.VertexAttribDivisor(2, 1);
            self.gl.VertexAttribDivisor(3, 1);
            self.gl.VertexAttribDivisor(4, 1);
            self.gl.VertexAttribDivisor(6, 1);
            self.gl.VertexAttribDivisor(7, 1);
            self.gl.VertexAttribDivisor(8, 1);

            self.gl.BindBuffer(gl::ARRAY_BUFFER, self.vbo);

            // layout (location = 5) in vec2 tex_coord;
            self.gl.EnableVertexAttribArray(5);
            self.gl.VertexAttribPointer(5, 2, gl::FLOAT, gl::FALSE, (3 * size_of::<f32>()) as _, 0 as _);

            self.gl.DrawArraysInstanced(gl::TRIANGLE_STRIP, 0, 4, self.draw_queue.len() as i32);

            self.gl.DeleteBuffers(1, &commands_vbo);
        }

        self.draw_queue.clear();
    }

    fn set_view_matrix(&mut self, view: [f32; 16]) {
        self.flush_queue();
        self.view_matrix = view;
        self.update_matrix();
    }

    fn set_viewproj_matrix(&mut self, view: [f32; 16], proj: [f32; 16]) {
        self.flush_queue();
        // flip vertically if drawing to surface because GL textures are flipped vertically vs DX 
        let to_surface = {
            let mut fb_draw = 0;
            unsafe {
                self.gl.GetIntegerv(gl::DRAW_FRAMEBUFFER_BINDING, &mut fb_draw);
            }
            fb_draw != self.framebuffer_fbo as _
        };
        #[rustfmt::skip]
        let proj = if to_surface {
            mat4mult(proj, [
                1.0, 0.0,  0.0, 0.0,
                0.0, -1.0, 0.0, 0.0,
                0.0, 0.0,  1.0, 0.0,
                0.0, 0.0,  0.0, 1.0,
            ])
        } else {
            proj
        };

        self.view_matrix = view;
        self.proj_matrix = proj;

        self.update_matrix();
    }

    fn set_model_matrix(&mut self, model: [f32; 16]) {
        self.flush_queue();
        self.model_matrix = model;
        self.update_matrix();
    }

    fn mult_model_matrix(&mut self, model: [f32; 16]) {
        self.flush_queue();
        self.model_matrix = mat4mult(self.model_matrix, model);
        self.update_matrix();
    }

    fn set_projection_ortho(&mut self, x: f64, y: f64, w: f64, h: f64, angle: f64) {
        // Draw anything that was meant to be drawn with the old view first
        self.flush_queue();

        // Note: sin is negated because it's the same as negating the angle, which is how GM8 does view angles
        let angle = angle.to_radians();
        let sin_angle = -angle.sin() as f32;
        let cos_angle = angle.cos() as f32;

        #[rustfmt::skip]
        let view_matrix: [f32; 16] = {
            // source rectangle's center coordinates aka -(x + w/2) and -(y + h/2)
            let scx = -((x as f32) + (w as f32 / 2.0));
            let scy = -((y as f32) + (h as f32 / 2.0));
            mat4mult(
                // Place camera at (scx, scy, 16000)
                [
                    1.0, 0.0, 0.0,     0.0,
                    0.0, 1.0, 0.0,     0.0,
                    0.0, 0.0, 1.0,     0.0,
                    scx, scy, 16000.0, 1.0,
                ],
                // Rotate to view_angle
                [
                    cos_angle,  sin_angle, 0.0, 0.0,
                    -sin_angle, cos_angle, 0.0, 0.0,
                    0.0,        0.0,       1.0, 0.0,
                    0.0,        0.0,       0.0, 1.0,
                ]
            )
        };

        #[rustfmt::skip]
        let proj_matrix: [f32; 16] = {
            // Squish to screen, flip vertically, and constrain z to range 1 - 32000
            [
                2.0 / w as f32, 0.0,             0.0,            0.0,
                0.0,            -2.0 / h as f32, 0.0,            0.0,
                0.0,            0.0,             1.0 / 31999.0,  0.0,
                0.0,            0.0,             -1.0 / 31999.0, 1.0,
            ]
        };

        self.set_viewproj_matrix(view_matrix, proj_matrix);
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
        self.set_projection_ortho(src_x.into(), src_y.into(), src_w.into(), src_h.into(), src_angle);
         
        // adjust port_y if drawing to screen
        let to_surface = {
            let mut fb_draw = 0;
            unsafe {
                self.gl.GetIntegerv(gl::DRAW_FRAMEBUFFER_BINDING, &mut fb_draw);
            }
            fb_draw != self.framebuffer_fbo as _
        };
        let port_y = if to_surface {
            port_y
        } else {
            let mut fb_height = 0;
            unsafe {
                self.gl.BindTexture(gl::TEXTURE_2D, self.framebuffer_fbo);
                self.gl.GetTexLevelParameteriv(gl::TEXTURE_2D, 0, gl::TEXTURE_HEIGHT, &mut fb_height);
            }
            fb_height - (port_y + port_h)
        };

        // Set viewport (gl::Viewport, gl::Scissor)
        unsafe {
            self.gl.Viewport(port_x, port_y, port_w, port_h);
            self.gl.Scissor(port_x, port_y, port_w, port_h);
        }
    }

    fn clear_view(&mut self, colour: Colour, alpha: f64) {
        self.flush_queue();
        unsafe {
            self.gl.ClearColor(colour.r as f32, colour.g as f32, colour.b as f32, alpha as f32);
            self.gl.Clear(gl::COLOR_BUFFER_BIT);
        }
    }

    fn present(&mut self, window_width: u32, window_height: u32) {
        if window_width == 0 || window_height == 0 {
            // if we continue, intel will dereference a null pointer
            return
        }
        unsafe {
            let mut fb_draw = 0;
            self.gl.GetIntegerv(gl::DRAW_FRAMEBUFFER_BINDING, &mut fb_draw);
            if fb_draw == self.framebuffer_fbo as _ {
                // Finish drawing frame
                self.flush_queue();

                // Get framebuffer size
                let (mut fb_width, mut fb_height) = (0, 0);
                self.gl.BindTexture(gl::TEXTURE_2D, self.framebuffer_texture);
                self.gl.GetTexLevelParameteriv(gl::TEXTURE_2D, 0, gl::TEXTURE_WIDTH, &mut fb_width);
                self.gl.GetTexLevelParameteriv(gl::TEXTURE_2D, 0, gl::TEXTURE_HEIGHT, &mut fb_height);

                // yeah i know but they need to be converted anyway
                let (window_width, window_height) = (window_width as i32, window_height as i32);

                // Scaling
                let (w_x, w_y, w_w, w_h) = match Scaling::Aspect { // TODO
                    Scaling::Fixed(scale) => {
                        // TODO: check if intel access violates when draw region is bigger than window in general
                        let w = (f64::from(fb_width) * scale) as i32;
                        let h = (f64::from(fb_height) * scale) as i32;
                        ((window_width - w) / 2, (window_height - h) / 2, w, h)
                    },
                    Scaling::Aspect => {
                        if fb_width > 0 && fb_height > 0 { // can never be too careful
                            let fixed_width = window_height * fb_width / fb_height;
                            if fixed_width < window_width {
                                // window is too wide
                                ((window_width - fixed_width) / 2, 0, fixed_width, window_height)
                            } else {
                                // window is too tall
                                let fixed_height = window_width * fb_height / fb_width;
                                (0, (window_height - fixed_height) / 2, window_width, fixed_height)
                            }
                        } else {
                            (0, 0, fb_width, fb_height)
                        }
                    },
                    Scaling::Full => (0, 0, window_width, window_height),
                };

                // Temporarily disable scissor test because apparently it disables drawing to the screen if the
                // scissor region is too big on intel
                self.gl.Disable(gl::SCISSOR_TEST);

                // Draw framebuffer to screen
                self.gl.BindFramebuffer(gl::DRAW_FRAMEBUFFER, 0);
                self.clear_view((0.0, 0.0, 0.0).into(), 1.0);
                self.gl.BindFramebuffer(gl::READ_FRAMEBUFFER, self.framebuffer_fbo);
                self.gl.BlitFramebuffer(
                    0,
                    0,
                    fb_width,
                    fb_height,
                    w_x,
                    w_y,
                    w_x+w_w,
                    w_y+w_h,
                    gl::COLOR_BUFFER_BIT,
                    if self.interpolate_pixels { gl::LINEAR } else { gl::NEAREST },
                );
                self.gl.BindFramebuffer(gl::DRAW_FRAMEBUFFER, self.framebuffer_fbo);

                self.gl.Enable(gl::SCISSOR_TEST);

                // Present buffer
                self.imp.swap_buffers();
            }
        }
    }

    fn finish(&mut self, window_width: u32, window_height: u32, clear_colour: Colour) {
        // Present screen
        self.present(window_width, window_height);

        // Start next frame
        self.setup_frame(clear_colour)
    }
}
