#![cfg(windows)]

use std::{ffi::OsStr, mem, os::windows::ffi::OsStrExt, ptr};

#[allow(non_snake_case)]
#[repr(C)]
struct DEVMODEW {
    dmDeviceName: [u16; 32],
    dmSpecVersion: u16,
    dmDriverVersion: u16,
    dmSize: u16,
    dmDriverExtra: u16,
    dmFields: u32,
    useless: [u8; 16],
    dmColor: i16,
    dmDuplex: i16,
    dmYResolution: i16,
    dmTTOption: i16,
    dmCollate: i16,
    dmFormName: [u16; 32],
    dmLogPixels: u16,
    dmBitsPerPel: u32,
    dmPelsWidth: u32,
    dmPelsHeight: u32,
    useless2: u32,
    dmDisplayFrequency: u32,
    dmICMMethod: u32,
    dmICMIntent: u32,
    dmMediaType: u32,
    dmDitherType: u32,
    dmReserved1: u32,
    dmReserved2: u32,
    dmPanningWidth: u32,
    dmPanningHeight: u32,
}

const ENUM_CURRENT_SETTINGS: u32 = u32::MAX;

#[link(name = "user32")]
extern "system" {
    fn EnumDisplaySettingsW(lpszDeviceName: *const u16, iModeNum: u32, lpDevMode: *mut DEVMODEW) -> i32;
    fn GetDiskFreeSpaceExW(
        lpDirectoryName: *const u16,
        lpFreeBytesAvailableToCaller: *mut u64,
        lpTotalNumberOfBytes: *mut u64,
        lpTotalNumberOfFreeBytes: *mut u64,
    ) -> i32;
}

fn get_display_settings() -> Option<DEVMODEW> {
    unsafe {
        let mut device = DEVMODEW { dmSize: mem::size_of::<DEVMODEW>() as _, ..mem::zeroed() };
        let response = EnumDisplaySettingsW(ptr::null(), ENUM_CURRENT_SETTINGS, &mut device);
        (response != 0).then(|| device)
    }
}

pub fn display_width() -> Option<u32> {
    get_display_settings().map(|dm| dm.dmPelsWidth)
}

pub fn display_height() -> Option<u32> {
    get_display_settings().map(|dm| dm.dmPelsHeight)
}

pub fn display_frequency() -> Option<u32> {
    get_display_settings().map(|dm| dm.dmDisplayFrequency)
}

pub fn display_colour_depth() -> Option<u32> {
    get_display_settings().map(|dm| dm.dmBitsPerPel)
}

fn trim_drive(drive: Option<char>) -> Vec<u16> {
    match drive {
        Some(letter) => {
            let mut out: Vec<u16> = vec![0, 0, b':'.into(), b'\\'.into(), 0];
            letter.encode_utf16(&mut out[..]);
            out
        },
        None => {
            let dir = std::env::current_dir().unwrap();
            let osstr: &OsStr = dir.as_ref();
            osstr.encode_wide().chain(Some(0)).collect()
        },
    }
}

pub fn disk_free(drive: Option<char>) -> Option<u64> {
    let mut free = 0;
    let path = trim_drive(drive);
    let response = unsafe { GetDiskFreeSpaceExW(path.as_ptr(), &mut free, ptr::null_mut(), ptr::null_mut()) };
    (response != 0).then(|| free)
}

pub fn disk_size(drive: Option<char>) -> Option<u64> {
    let mut size = 0;
    let path = trim_drive(drive);
    let response = unsafe { GetDiskFreeSpaceExW(path.as_ptr(), ptr::null_mut(), &mut size, ptr::null_mut()) };
    (response != 0).then(|| size)
}
