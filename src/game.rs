use gm8x::deps::minio::ReadPrimitives;
use gm8x::reader::Settings;
use winit::{
    dpi::LogicalSize,
    error::OsError,
    event_loop::EventLoop,
    window::{Fullscreen, Icon, Window, WindowBuilder},
};

pub fn icon_from_win32<I: AsRef<[u8]>>(ico: I) -> Option<Icon> {
    let raw = ico.as_ref();
    let mut cur = std::io::Cursor::new(raw);
    let start = cur.read_u32_le().ok()? as usize;
    let width = cur.read_u32_le().ok()? as usize;
    let px_len = width.pow(2) * 4;
    cur.into_inner()
        .get(start..(start + px_len))
        .and_then(|data| {
            let mut rgba = Vec::with_capacity(px_len);
            for chunk in data.rchunks_exact(width * 4) {
                rgba.extend_from_slice(chunk);
                let vlen = rgba.len();
                crate::util::bgra2rgba(rgba.get_mut(vlen - (width * 4)..)?);
            }
            Some((rgba, width as u32))
        })
        .and_then(|(rgba, w)| Icon::from_rgba(rgba, w, w).ok())
}

pub fn window(
    title: &str,
    width: u32,
    height: u32,
    icon: Option<Icon>,
    extra: &Settings,
) -> Result<(EventLoop<()>, Window), OsError> {
    let event_loop = EventLoop::new();

    let window_builder = WindowBuilder::new()
        .with_title(title)
        .with_window_icon(icon)
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
