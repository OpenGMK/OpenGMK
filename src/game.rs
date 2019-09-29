use gm8x::reader::{Settings, WindowsIcon};
use winit::{
    dpi::LogicalSize,
    error::OsError,
    event_loop::EventLoop,
    window::{Fullscreen, Icon, Window, WindowBuilder},
};

pub fn icon_from_win32(raw: &[u8], width: usize) -> Option<Icon> {
    let mut rgba = Vec::with_capacity(raw.len());
    for chunk in raw.rchunks_exact(width * 4) {
        rgba.extend_from_slice(chunk);
        let vlen = rgba.len();
        crate::util::bgra2rgba(rgba.get_mut(vlen - (width * 4)..)?);
    }
    Icon::from_rgba(rgba, width as u32, width as u32).ok()
}

fn get_icon_via_w(icons: &Vec<WindowsIcon>, w: i32) -> Option<Icon> {
    fn closest<'a, I: Iterator<Item = &'a WindowsIcon>>(w: i32, i: I) -> Option<&'a WindowsIcon> {
        i.min_by(|a, b| (a.width as i32 - w).abs().cmp(&(b.width as i32 - w).abs()))
    }

    closest(w, icons.iter().filter(|i| i.original_bpp == 24 || i.original_bpp == 32))
        .or_else(|| closest(w, icons.iter()))
        .and_then(|i| icon_from_win32(&i.bgra_data, i.width as usize))
}

pub fn window(
    title: &str,
    width: u32,
    height: u32,
    icons: &Vec<WindowsIcon>,
    extra: &Settings,
) -> Result<(EventLoop<()>, Window), OsError> {
    let event_loop = EventLoop::new();

    let window_builder = WindowBuilder::new()
        .with_title(title)
        .with_window_icon(get_icon_via_w(icons, 32))
        .with_inner_size(LogicalSize::from((width, height)))
        .with_resizable(extra.allow_resize)
        .with_always_on_top(extra.window_on_top)
        .with_decorations(!extra.dont_draw_border)
        .with_fullscreen(if extra.fullscreen {
            Some(Fullscreen::Borderless(event_loop.primary_monitor()))
        } else {
            None
        });

    let window = window_builder.build(&event_loop)?;

    Ok((event_loop, window))
}
