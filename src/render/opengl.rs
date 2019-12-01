//! OpenGL bindings & functions
//!
//! The raw bindings are generated at build time, see build.rs

/// Auto-generated OpenGL bindings from gl_generator
#[allow(clippy::all)]
pub mod gl {
    include!(concat!(env!("OUT_DIR"), "/gl_bindings.rs"));
}

use super::Renderer;
use glutin::{
    event_loop::EventLoop,
    window::{Fullscreen, Icon, Window, WindowBuilder},
    ContextWrapper, PossiblyCurrent, {Api, ContextBuilder, GlProfile, GlRequest},
};

pub struct OpenGLRenderer {
    ctx: ContextWrapper<PossiblyCurrent, ()>,
    el: EventLoop<()>,
    window: Window,
}

impl OpenGLRenderer {
    pub fn new(
        title: &str,
        size: (u32, u32),
        icon: Option<(Vec<u8>, u32, u32)>,
        resizable: bool,
        on_top: bool,
        decorations: bool,
        fullscreen: bool,
        vsync: bool,
    ) -> Result<Self, String> {
        let el = EventLoop::new();
        let wb = WindowBuilder::new()
            .with_title(title)
            .with_window_icon(icon.and_then(|(data, w, h)| Icon::from_rgba(data, w, h).ok()))
            .with_inner_size(size.into())
            .with_resizable(resizable)
            .with_always_on_top(on_top)
            .with_decorations(decorations)
            .with_visible(false)
            .with_fullscreen(if fullscreen {
                // TODO: Allow overriding primary monitor
                Some(Fullscreen::Borderless(el.primary_monitor()))
            } else {
                None
            });

        let ctx = ContextBuilder::new()
            .with_gl(GlRequest::Specific(Api::OpenGl, (3, 3)))
            .with_gl_profile(GlProfile::Core)
            .with_hardware_acceleration(Some(true))
            .with_vsync(vsync)
            .build_windowed(wb, &el)
            .map_err(|err| err.to_string())?;

        let (ctx, window) = unsafe { ctx.make_current().map_err(|(_self, err)| err.to_string())?.split() };

        gl::load_with(|s| ctx.get_proc_address(s) as *const _);

        Ok(Self { ctx, el, window })
    }
}

impl Renderer for OpenGLRenderer {}
