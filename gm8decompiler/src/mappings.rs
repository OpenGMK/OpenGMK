use std::collections::{HashMap, HashSet};

pub mod constants {
    pub const ANSI_CHARSET: f64 = 0.0f64;
    pub const ARABIC_CHARSET: f64 = 178.0;
    pub const BALTIC_CHARSET: f64 = 186.0;
    pub const CHINESEBIG5_CHARSET: f64 = 136.0;
    pub const DEFAULT_CHARSET: f64 = 1.0;
    pub const EASTEUROPE_CHARSET: f64 = 238.0;
    pub const GB2312_CHARSET: f64 = 134.0;
    pub const GREEK_CHARSET: f64 = 161.0;
    pub const HANGEUL_CHARSET: f64 = 129.0;
    pub const HEBREW_CHARSET: f64 = 177.0;
    pub const JOHAB_CHARSET: f64 = 130.0;
    pub const MAC_CHARSET: f64 = 77.0;
    pub const OEM_CHARSET: f64 = 255.0;
    pub const RUSSIAN_CHARSET: f64 = 204.0;
    pub const SHIFTJIS_CHARSET: f64 = 128.0;
    pub const SYMBOL_CHARSET: f64 = 2.0;
    pub const THAI_CHARSET: f64 = 222.0;
    pub const TURKISH_CHARSET: f64 = 162.0;
    pub const VIETNAMESE_CHARSET: f64 = 163.0;
    pub const ALL: f64 = -3.0;
    pub const BM_ADD: f64 = 1.0;
    pub const BM_DEST_ALPHA: f64 = 7.0;
    pub const BM_DEST_COLOR: f64 = 9.0;
    pub const BM_INV_DEST_ALPHA: f64 = 8.0;
    pub const BM_INV_DEST_COLOR: f64 = 10.0;
    pub const BM_INV_SRC_ALPHA: f64 = 6.0;
    pub const BM_INV_SRC_COLOR: f64 = 4.0;
    pub const BM_MAX: f64 = 2.0;
    pub const BM_NORMAL: f64 = 0.0;
    pub const BM_ONE: f64 = 2.0;
    pub const BM_SRC_ALPHA: f64 = 5.0;
    pub const BM_SRC_ALPHA_SAT: f64 = 11.0;
    pub const BM_SRC_COLOR: f64 = 3.0;
    pub const BM_SUBTRACT: f64 = 3.0;
    pub const BM_ZERO: f64 = 1.0;
    pub const BUTTON_TYPE: f64 = 1.0;
    pub const C_AQUA: f64 = 16776960.0;
    pub const C_BLACK: f64 = 0.0;
    pub const C_BLUE: f64 = 16711680.0;
    pub const C_DKGRAY: f64 = 4210752.0;
    pub const C_FUCHSIA: f64 = 16711935.0;
    pub const C_GRAY: f64 = 8421504.0;
    pub const C_GREEN: f64 = 32768.0;
    pub const C_LIME: f64 = 65280.0;
    pub const C_LTGRAY: f64 = 12632256.0;
    pub const C_MAROON: f64 = 128.0;
    pub const C_NAVY: f64 = 8388608.0;
    pub const C_OLIVE: f64 = 32896.0;
    pub const C_ORANGE: f64 = 4235519.0;
    pub const C_PURPLE: f64 = 8388736.0;
    pub const C_RED: f64 = 255.0;
    pub const C_SILVER: f64 = 12632256.0;
    pub const C_TEAL: f64 = 8421376.0;
    pub const C_WHITE: f64 = 16777215.0;
    pub const C_YELLOW: f64 = 65535.0;
    pub const CR_APPSTART: f64 = -19.0;
    pub const CR_ARROW: f64 = -2.0;
    pub const CR_BEAM: f64 = -4.0;
    pub const CR_CROSS: f64 = -3.0;
    pub const CR_DEFAULT: f64 = 0.0;
    pub const CR_DRAG: f64 = -12.0;
    pub const CR_HANDPOINT: f64 = -21.0;
    pub const CR_HELP: f64 = -20.0;
    pub const CR_HOURGLASS: f64 = -11.0;
    pub const CR_HSPLIT: f64 = -14.0;
    pub const CR_MULTIDRAG: f64 = -16.0;
    pub const CR_NO: f64 = -18.0;
    pub const CR_NODROP: f64 = -13.0;
    pub const CR_NONE: f64 = -1.0;
    pub const CR_SIZE_ALL: f64 = -22.0;
    pub const CR_SIZE_NESW: f64 = -6.0;
    pub const CR_SIZE_NS: f64 = -7.0;
    pub const CR_SIZE_NWSE: f64 = -8.0;
    pub const CR_SIZE_WE: f64 = -9.0;
    pub const CR_SQLWAIT: f64 = -17.0;
    pub const CR_UPARROW: f64 = -10.0;
    pub const CR_VSPLIT: f64 = -15.0;
    pub const DEVICE_IOS_IPAD: f64 = 2.0;
    pub const DEVICE_IOS_IPHONE: f64 = 0.0;
    pub const DEVICE_IOS_IPHONE_RETINA: f64 = 1.0;
    pub const DLL_CDECL: f64 = 0.0;
    pub const DLL_STDCALL: f64 = 1.0;
    pub const EF_CLOUD: f64 = 9.0;
    pub const EF_ELLIPSE: f64 = 2.0;
    pub const EF_EXPLOSION: f64 = 0.0;
    pub const EF_FIREWORK: f64 = 3.0;
    pub const EF_FLARE: f64 = 8.0;
    pub const EF_RAIN: f64 = 10.0;
    pub const EF_RING: f64 = 1.0;
    pub const EF_SMOKE: f64 = 4.0;
    pub const EF_SMOKEUP: f64 = 5.0;
    pub const EF_SNOW: f64 = 11.0;
    pub const EF_SPARK: f64 = 7.0;
    pub const EF_STAR: f64 = 6.0;
    pub const EV_ALARM: f64 = 2.0;
    pub const EV_ANIMATION_END: f64 = 7.0;
    pub const EV_BOUNDARY: f64 = 1.0;
    pub const EV_CLOSE_BUTTON: f64 = 30.0;
    pub const EV_COLLISION: f64 = 4.0;
    pub const EV_CREATE: f64 = 0.0;
    pub const EV_DESTROY: f64 = 1.0;
    pub const EV_DRAW: f64 = 8.0;
    pub const EV_END_OF_PATH: f64 = 8.0;
    pub const EV_GAME_END: f64 = 3.0;
    pub const EV_GAME_START: f64 = 2.0;
    pub const EV_GLOBAL_LEFT_BUTTON: f64 = 50.0;
    pub const EV_GLOBAL_LEFT_PRESS: f64 = 53.0;
    pub const EV_GLOBAL_LEFT_RELEASE: f64 = 56.0;
    pub const EV_GLOBAL_MIDDLE_BUTTON: f64 = 52.0;
    pub const EV_GLOBAL_MIDDLE_PRESS: f64 = 55.0;
    pub const EV_GLOBAL_MIDDLE_RELEASE: f64 = 58.0;
    pub const EV_GLOBAL_PRESS: f64 = 12.0;
    pub const EV_GLOBAL_RELEASE: f64 = 13.0;
    pub const EV_GLOBAL_RIGHT_BUTTON: f64 = 51.0;
    pub const EV_GLOBAL_RIGHT_PRESS: f64 = 54.0;
    pub const EV_GLOBAL_RIGHT_RELEASE: f64 = 57.0;
    pub const EV_JOYSTICK1_BUTTON1: f64 = 21.0;
    pub const EV_JOYSTICK1_BUTTON2: f64 = 22.0;
    pub const EV_JOYSTICK1_BUTTON3: f64 = 23.0;
    pub const EV_JOYSTICK1_BUTTON4: f64 = 24.0;
    pub const EV_JOYSTICK1_BUTTON5: f64 = 25.0;
    pub const EV_JOYSTICK1_BUTTON6: f64 = 26.0;
    pub const EV_JOYSTICK1_BUTTON7: f64 = 27.0;
    pub const EV_JOYSTICK1_BUTTON8: f64 = 28.0;
    pub const EV_JOYSTICK1_DOWN: f64 = 19.0;
    pub const EV_JOYSTICK1_LEFT: f64 = 16.0;
    pub const EV_JOYSTICK1_RIGHT: f64 = 17.0;
    pub const EV_JOYSTICK1_UP: f64 = 18.0;
    pub const EV_JOYSTICK2_BUTTON1: f64 = 36.0;
    pub const EV_JOYSTICK2_BUTTON2: f64 = 37.0;
    pub const EV_JOYSTICK2_BUTTON3: f64 = 38.0;
    pub const EV_JOYSTICK2_BUTTON4: f64 = 39.0;
    pub const EV_JOYSTICK2_BUTTON5: f64 = 40.0;
    pub const EV_JOYSTICK2_BUTTON6: f64 = 41.0;
    pub const EV_JOYSTICK2_BUTTON7: f64 = 42.0;
    pub const EV_JOYSTICK2_BUTTON8: f64 = 43.0;
    pub const EV_JOYSTICK2_DOWN: f64 = 34.0;
    pub const EV_JOYSTICK2_LEFT: f64 = 31.0;
    pub const EV_JOYSTICK2_RIGHT: f64 = 32.0;
    pub const EV_JOYSTICK2_UP: f64 = 33.0;
    pub const EV_KEYBOARD: f64 = 5.0;
    pub const EV_KEYPRESS: f64 = 9.0;
    pub const EV_KEYRELEASE: f64 = 10.0;
    pub const EV_LEFT_BUTTON: f64 = 0.0;
    pub const EV_LEFT_PRESS: f64 = 4.0;
    pub const EV_LEFT_RELEASE: f64 = 7.0;
    pub const EV_MIDDLE_BUTTON: f64 = 2.0;
    pub const EV_MIDDLE_PRESS: f64 = 6.0;
    pub const EV_MIDDLE_RELEASE: f64 = 9.0;
    pub const EV_MOUSE: f64 = 6.0;
    pub const EV_MOUSE_ENTER: f64 = 10.0;
    pub const EV_MOUSE_LEAVE: f64 = 11.0;
    pub const EV_MOUSE_WHEEL_DOWN: f64 = 61.0;
    pub const EV_MOUSE_WHEEL_UP: f64 = 60.0;
    pub const EV_NO_BUTTON: f64 = 3.0;
    pub const EV_NO_MORE_HEALTH: f64 = 9.0;
    pub const EV_NO_MORE_LIVES: f64 = 6.0;
    pub const EV_OTHER: f64 = 7.0;
    pub const EV_OUTSIDE: f64 = 0.0;
    pub const EV_RIGHT_BUTTON: f64 = 1.0;
    pub const EV_RIGHT_PRESS: f64 = 5.0;
    pub const EV_RIGHT_RELEASE: f64 = 8.0;
    pub const EV_ROOM_END: f64 = 5.0;
    pub const EV_ROOM_START: f64 = 4.0;
    pub const EV_STEP: f64 = 3.0;
    pub const EV_STEP_BEGIN: f64 = 1.0;
    pub const EV_STEP_END: f64 = 2.0;
    pub const EV_STEP_NORMAL: f64 = 0.0;
    pub const EV_TRIGGER: f64 = 11.0;
    pub const EV_USER0: f64 = 10.0;
    pub const EV_USER1: f64 = 11.0;
    pub const EV_USER10: f64 = 20.0;
    pub const EV_USER11: f64 = 21.0;
    pub const EV_USER12: f64 = 22.0;
    pub const EV_USER13: f64 = 23.0;
    pub const EV_USER14: f64 = 24.0;
    pub const EV_USER15: f64 = 25.0;
    pub const EV_USER2: f64 = 12.0;
    pub const EV_USER3: f64 = 13.0;
    pub const EV_USER4: f64 = 14.0;
    pub const EV_USER5: f64 = 15.0;
    pub const EV_USER6: f64 = 16.0;
    pub const EV_USER7: f64 = 17.0;
    pub const EV_USER8: f64 = 18.0;
    pub const EV_USER9: f64 = 19.0;
    pub const FA_ARCHIVE: f64 = 32.0;
    pub const FA_BOTTOM: f64 = 2.0;
    pub const FA_CENTER: f64 = 1.0;
    pub const FA_DIRECTORY: f64 = 16.0;
    pub const FA_HIDDEN: f64 = 2.0;
    pub const FA_LEFT: f64 = 0.0;
    pub const FA_MIDDLE: f64 = 1.0;
    pub const FA_READONLY: f64 = 1.0;
    pub const FA_RIGHT: f64 = 2.0;
    pub const FA_SYSFILE: f64 = 4.0;
    pub const FA_TOP: f64 = 0.0;
    pub const FA_VOLUMEID: f64 = 8.0;
    pub const FALSE: f64 = 0.0;
    pub const GLOBAL: f64 = -5.0;
    pub const LOCAL: f64 = -7.0;
    pub const MB_ANY: f64 = -1.0;
    pub const MB_LEFT: f64 = 1.0;
    pub const MB_MIDDLE: f64 = 3.0;
    pub const MB_NONE: f64 = 0.0;
    pub const MB_RIGHT: f64 = 2.0;
    pub const NOONE: f64 = -4.0;
    pub const OS_ANDROID: f64 = 5.0;
    pub const OS_IOS: f64 = 4.0;
    pub const OS_MACOSX: f64 = 2.0;
    pub const OS_PSP: f64 = 3.0;
    pub const OS_WIN32: f64 = 0.0;
    pub const OTHER: f64 = -2.0;
    pub const PI: f64 = unsafe {
        // 3.141592653589793, but we use the exact bytes from GM8 to avoid any compiler-mangling
        std::mem::transmute(u64::from_le_bytes([0x18, 0x2D, 0x44, 0x54, 0xFB, 0x21, 0x09, 0x40]))
    };
    pub const PR_LINELIST: f64 = 2.0;
    pub const PR_LINESTRIP: f64 = 3.0;
    pub const PR_POINTLIST: f64 = 1.0;
    pub const PR_TRIANGLEFAN: f64 = 6.0;
    pub const PR_TRIANGLELIST: f64 = 4.0;
    pub const PR_TRIANGLESTRIP: f64 = 5.0;
    pub const PS_CHANGE_ALL: f64 = 0.0;
    pub const PS_CHANGE_MOTION: f64 = 2.0;
    pub const PS_CHANGE_SHAPE: f64 = 1.0;
    pub const PS_DEFLECT_HORIZONTAL: f64 = 1.0;
    pub const PS_DEFLECT_VERTICAL: f64 = 0.0;
    pub const PS_DISTR_GAUSSIAN: f64 = 1.0;
    pub const PS_DISTR_INVGAUSSIAN: f64 = 2.0;
    pub const PS_DISTR_LINEAR: f64 = 0.0;
    pub const PS_FORCE_CONSTANT: f64 = 0.0;
    pub const PS_FORCE_LINEAR: f64 = 1.0;
    pub const PS_FORCE_QUADRATIC: f64 = 2.0;
    pub const PS_SHAPE_DIAMOND: f64 = 2.0;
    pub const PS_SHAPE_ELLIPSE: f64 = 1.0;
    pub const PS_SHAPE_LINE: f64 = 3.0;
    pub const PS_SHAPE_RECTANGLE: f64 = 0.0;
    pub const PT_SHAPE_CIRCLE: f64 = 5.0;
    pub const PT_SHAPE_CLOUD: f64 = 11.0;
    pub const PT_SHAPE_DISK: f64 = 1.0;
    pub const PT_SHAPE_EXPLOSION: f64 = 10.0;
    pub const PT_SHAPE_FLARE: f64 = 8.0;
    pub const PT_SHAPE_LINE: f64 = 3.0;
    pub const PT_SHAPE_PIXEL: f64 = 0.0;
    pub const PT_SHAPE_RING: f64 = 6.0;
    pub const PT_SHAPE_SMOKE: f64 = 12.0;
    pub const PT_SHAPE_SNOW: f64 = 13.0;
    pub const PT_SHAPE_SPARK: f64 = 9.0;
    pub const PT_SHAPE_SPHERE: f64 = 7.0;
    pub const PT_SHAPE_SQUARE: f64 = 2.0;
    pub const PT_SHAPE_STAR: f64 = 4.0;
    pub const SE_CHORUS: f64 = 1.0;
    pub const SE_COMPRESSOR: f64 = 32.0;
    pub const SE_ECHO: f64 = 2.0;
    pub const SE_EQUALIZER: f64 = 64.0;
    pub const SE_FLANGER: f64 = 4.0;
    pub const SE_GARGLE: f64 = 8.0;
    pub const SE_NONE: f64 = 0.0;
    pub const SE_REVERB: f64 = 16.0;
    pub const SELF: f64 = -1.0;
    pub const TEXT_TYPE: f64 = 0.0;
    pub const TRUE: f64 = 1.0;
    pub const TY_REAL: f64 = 0.0;
    pub const TY_STRING: f64 = 1.0;
    pub const VK_ADD: f64 = 107.0;
    pub const VK_ALT: f64 = 18.0;
    pub const VK_ANYKEY: f64 = 1.0;
    pub const VK_BACKSPACE: f64 = 8.0;
    pub const VK_CONTROL: f64 = 17.0;
    pub const VK_DECIMAL: f64 = 110.0;
    pub const VK_DELETE: f64 = 46.0;
    pub const VK_DIVIDE: f64 = 111.0;
    pub const VK_DOWN: f64 = 40.0;
    pub const VK_END: f64 = 35.0;
    pub const VK_ENTER: f64 = 13.0;
    pub const VK_ESCAPE: f64 = 27.0;
    pub const VK_F1: f64 = 112.0;
    pub const VK_F10: f64 = 121.0;
    pub const VK_F11: f64 = 122.0;
    pub const VK_F12: f64 = 123.0;
    pub const VK_F2: f64 = 113.0;
    pub const VK_F3: f64 = 114.0;
    pub const VK_F4: f64 = 115.0;
    pub const VK_F5: f64 = 116.0;
    pub const VK_F6: f64 = 117.0;
    pub const VK_F7: f64 = 118.0;
    pub const VK_F8: f64 = 119.0;
    pub const VK_F9: f64 = 120.0;
    pub const VK_HOME: f64 = 36.0;
    pub const VK_INSERT: f64 = 45.0;
    pub const VK_LALT: f64 = 164.0;
    pub const VK_LCONTROL: f64 = 162.0;
    pub const VK_LEFT: f64 = 37.0;
    pub const VK_LSHIFT: f64 = 160.0;
    pub const VK_MULTIPLY: f64 = 106.0;
    pub const VK_NOKEY: f64 = 0.0;
    pub const VK_NUMPAD0: f64 = 96.0;
    pub const VK_NUMPAD1: f64 = 97.0;
    pub const VK_NUMPAD2: f64 = 98.0;
    pub const VK_NUMPAD3: f64 = 99.0;
    pub const VK_NUMPAD4: f64 = 100.0;
    pub const VK_NUMPAD5: f64 = 101.0;
    pub const VK_NUMPAD6: f64 = 102.0;
    pub const VK_NUMPAD7: f64 = 103.0;
    pub const VK_NUMPAD8: f64 = 104.0;
    pub const VK_NUMPAD9: f64 = 105.0;
    pub const VK_PAGEDOWN: f64 = 34.0;
    pub const VK_PAGEUP: f64 = 33.0;
    pub const VK_PAUSE: f64 = 19.0;
    pub const VK_PRINTSCREEN: f64 = 44.0;
    pub const VK_RALT: f64 = 165.0;
    pub const VK_RCONTROL: f64 = 163.0;
    pub const VK_RETURN: f64 = 13.0;
    pub const VK_RIGHT: f64 = 39.0;
    pub const VK_RSHIFT: f64 = 161.0;
    pub const VK_SHIFT: f64 = 16.0;
    pub const VK_SPACE: f64 = 32.0;
    pub const VK_SUBTRACT: f64 = 109.0;
    pub const VK_TAB: f64 = 9.0;
    pub const VK_UP: f64 = 38.0;
}

const CONSTANTS: [(&str, f64); 317] = [
    ("ANSI_CHARSET", constants::ANSI_CHARSET),
    ("ARABIC_CHARSET", constants::ARABIC_CHARSET),
    ("BALTIC_CHARSET", constants::BALTIC_CHARSET),
    ("CHINESEBIG5_CHARSET", constants::CHINESEBIG5_CHARSET),
    ("DEFAULT_CHARSET", constants::DEFAULT_CHARSET),
    ("EASTEUROPE_CHARSET", constants::EASTEUROPE_CHARSET),
    ("GB2312_CHARSET", constants::GB2312_CHARSET),
    ("GREEK_CHARSET", constants::GREEK_CHARSET),
    ("HANGEUL_CHARSET", constants::HANGEUL_CHARSET),
    ("HEBREW_CHARSET", constants::HEBREW_CHARSET),
    ("JOHAB_CHARSET", constants::JOHAB_CHARSET),
    ("MAC_CHARSET", constants::MAC_CHARSET),
    ("OEM_CHARSET", constants::OEM_CHARSET),
    ("RUSSIAN_CHARSET", constants::RUSSIAN_CHARSET),
    ("SHIFTJIS_CHARSET", constants::SHIFTJIS_CHARSET),
    ("SYMBOL_CHARSET", constants::SYMBOL_CHARSET),
    ("THAI_CHARSET", constants::THAI_CHARSET),
    ("TURKISH_CHARSET", constants::TURKISH_CHARSET),
    ("VIETNAMESE_CHARSET", constants::VIETNAMESE_CHARSET),
    ("all", constants::ALL),
    ("bm_add", constants::BM_ADD),
    ("bm_dest_alpha", constants::BM_DEST_ALPHA),
    ("bm_dest_color", constants::BM_DEST_COLOR),
    ("bm_inv_dest_alpha", constants::BM_INV_DEST_ALPHA),
    ("bm_inv_dest_color", constants::BM_INV_DEST_COLOR),
    ("bm_inv_src_alpha", constants::BM_INV_SRC_ALPHA),
    ("bm_inv_src_color", constants::BM_INV_SRC_COLOR),
    ("bm_max", constants::BM_MAX),
    ("bm_normal", constants::BM_NORMAL),
    ("bm_one", constants::BM_ONE),
    ("bm_src_alpha", constants::BM_SRC_ALPHA),
    ("bm_src_alpha_sat", constants::BM_SRC_ALPHA_SAT),
    ("bm_src_color", constants::BM_SRC_COLOR),
    ("bm_subtract", constants::BM_SUBTRACT),
    ("bm_zero", constants::BM_ZERO),
    ("button_type", constants::BUTTON_TYPE),
    ("c_aqua", constants::C_AQUA),
    ("c_black", constants::C_BLACK),
    ("c_blue", constants::C_BLUE),
    ("c_dkgray", constants::C_DKGRAY),
    ("c_fuchsia", constants::C_FUCHSIA),
    ("c_gray", constants::C_GRAY),
    ("c_green", constants::C_GREEN),
    ("c_lime", constants::C_LIME),
    ("c_ltgray", constants::C_LTGRAY),
    ("c_maroon", constants::C_MAROON),
    ("c_navy", constants::C_NAVY),
    ("c_olive", constants::C_OLIVE),
    ("c_orange", constants::C_ORANGE),
    ("c_purple", constants::C_PURPLE),
    ("c_red", constants::C_RED),
    ("c_silver", constants::C_SILVER),
    ("c_teal", constants::C_TEAL),
    ("c_white", constants::C_WHITE),
    ("c_yellow", constants::C_YELLOW),
    ("cr_appstart", constants::CR_APPSTART),
    ("cr_arrow", constants::CR_ARROW),
    ("cr_beam", constants::CR_BEAM),
    ("cr_cross", constants::CR_CROSS),
    ("cr_default", constants::CR_DEFAULT),
    ("cr_drag", constants::CR_DRAG),
    ("cr_handpoint", constants::CR_HANDPOINT),
    ("cr_help", constants::CR_HELP),
    ("cr_hourglass", constants::CR_HOURGLASS),
    ("cr_hsplit", constants::CR_HSPLIT),
    ("cr_multidrag", constants::CR_MULTIDRAG),
    ("cr_no", constants::CR_NO),
    ("cr_nodrop", constants::CR_NODROP),
    ("cr_none", constants::CR_NONE),
    ("cr_size_all", constants::CR_SIZE_ALL),
    ("cr_size_nesw", constants::CR_SIZE_NESW),
    ("cr_size_ns", constants::CR_SIZE_NS),
    ("cr_size_nwse", constants::CR_SIZE_NWSE),
    ("cr_size_we", constants::CR_SIZE_WE),
    ("cr_sqlwait", constants::CR_SQLWAIT),
    ("cr_uparrow", constants::CR_UPARROW),
    ("cr_vsplit", constants::CR_VSPLIT),
    ("device_ios_ipad", constants::DEVICE_IOS_IPAD),
    ("device_ios_iphone", constants::DEVICE_IOS_IPHONE),
    ("device_ios_iphone_retina", constants::DEVICE_IOS_IPHONE_RETINA),
    ("dll_cdecl", constants::DLL_CDECL),
    ("dll_stdcall", constants::DLL_STDCALL),
    ("ef_cloud", constants::EF_CLOUD),
    ("ef_ellipse", constants::EF_ELLIPSE),
    ("ef_explosion", constants::EF_EXPLOSION),
    ("ef_firework", constants::EF_FIREWORK),
    ("ef_flare", constants::EF_FLARE),
    ("ef_rain", constants::EF_RAIN),
    ("ef_ring", constants::EF_RING),
    ("ef_smoke", constants::EF_SMOKE),
    ("ef_smokeup", constants::EF_SMOKEUP),
    ("ef_snow", constants::EF_SNOW),
    ("ef_spark", constants::EF_SPARK),
    ("ef_star", constants::EF_STAR),
    ("ev_alarm", constants::EV_ALARM),
    ("ev_animation_end", constants::EV_ANIMATION_END),
    ("ev_boundary", constants::EV_BOUNDARY),
    ("ev_close_button", constants::EV_CLOSE_BUTTON),
    ("ev_collision", constants::EV_COLLISION),
    ("ev_create", constants::EV_CREATE),
    ("ev_destroy", constants::EV_DESTROY),
    ("ev_draw", constants::EV_DRAW),
    ("ev_end_of_path", constants::EV_END_OF_PATH),
    ("ev_game_end", constants::EV_GAME_END),
    ("ev_game_start", constants::EV_GAME_START),
    ("ev_global_left_button", constants::EV_GLOBAL_LEFT_BUTTON),
    ("ev_global_left_press", constants::EV_GLOBAL_LEFT_PRESS),
    ("ev_global_left_release", constants::EV_GLOBAL_LEFT_RELEASE),
    ("ev_global_middle_button", constants::EV_GLOBAL_MIDDLE_BUTTON),
    ("ev_global_middle_press", constants::EV_GLOBAL_MIDDLE_PRESS),
    ("ev_global_middle_release", constants::EV_GLOBAL_MIDDLE_RELEASE),
    ("ev_global_press", constants::EV_GLOBAL_PRESS),
    ("ev_global_release", constants::EV_GLOBAL_RELEASE),
    ("ev_global_right_button", constants::EV_GLOBAL_RIGHT_BUTTON),
    ("ev_global_right_press", constants::EV_GLOBAL_RIGHT_PRESS),
    ("ev_global_right_release", constants::EV_GLOBAL_RIGHT_RELEASE),
    ("ev_joystick1_button1", constants::EV_JOYSTICK1_BUTTON1),
    ("ev_joystick1_button2", constants::EV_JOYSTICK1_BUTTON2),
    ("ev_joystick1_button3", constants::EV_JOYSTICK1_BUTTON3),
    ("ev_joystick1_button4", constants::EV_JOYSTICK1_BUTTON4),
    ("ev_joystick1_button5", constants::EV_JOYSTICK1_BUTTON5),
    ("ev_joystick1_button6", constants::EV_JOYSTICK1_BUTTON6),
    ("ev_joystick1_button7", constants::EV_JOYSTICK1_BUTTON7),
    ("ev_joystick1_button8", constants::EV_JOYSTICK1_BUTTON8),
    ("ev_joystick1_down", constants::EV_JOYSTICK1_DOWN),
    ("ev_joystick1_left", constants::EV_JOYSTICK1_LEFT),
    ("ev_joystick1_right", constants::EV_JOYSTICK1_RIGHT),
    ("ev_joystick1_up", constants::EV_JOYSTICK1_UP),
    ("ev_joystick2_button1", constants::EV_JOYSTICK2_BUTTON1),
    ("ev_joystick2_button2", constants::EV_JOYSTICK2_BUTTON2),
    ("ev_joystick2_button3", constants::EV_JOYSTICK2_BUTTON3),
    ("ev_joystick2_button4", constants::EV_JOYSTICK2_BUTTON4),
    ("ev_joystick2_button5", constants::EV_JOYSTICK2_BUTTON5),
    ("ev_joystick2_button6", constants::EV_JOYSTICK2_BUTTON6),
    ("ev_joystick2_button7", constants::EV_JOYSTICK2_BUTTON7),
    ("ev_joystick2_button8", constants::EV_JOYSTICK2_BUTTON8),
    ("ev_joystick2_down", constants::EV_JOYSTICK2_DOWN),
    ("ev_joystick2_left", constants::EV_JOYSTICK2_LEFT),
    ("ev_joystick2_right", constants::EV_JOYSTICK2_RIGHT),
    ("ev_joystick2_up", constants::EV_JOYSTICK2_UP),
    ("ev_keyboard", constants::EV_KEYBOARD),
    ("ev_keypress", constants::EV_KEYPRESS),
    ("ev_keyrelease", constants::EV_KEYRELEASE),
    ("ev_left_button", constants::EV_LEFT_BUTTON),
    ("ev_left_press", constants::EV_LEFT_PRESS),
    ("ev_left_release", constants::EV_LEFT_RELEASE),
    ("ev_middle_button", constants::EV_MIDDLE_BUTTON),
    ("ev_middle_press", constants::EV_MIDDLE_PRESS),
    ("ev_middle_release", constants::EV_MIDDLE_RELEASE),
    ("ev_mouse", constants::EV_MOUSE),
    ("ev_mouse_enter", constants::EV_MOUSE_ENTER),
    ("ev_mouse_leave", constants::EV_MOUSE_LEAVE),
    ("ev_mouse_wheel_down", constants::EV_MOUSE_WHEEL_DOWN),
    ("ev_mouse_wheel_up", constants::EV_MOUSE_WHEEL_UP),
    ("ev_no_button", constants::EV_NO_BUTTON),
    ("ev_no_more_health", constants::EV_NO_MORE_HEALTH),
    ("ev_no_more_lives", constants::EV_NO_MORE_LIVES),
    ("ev_other", constants::EV_OTHER),
    ("ev_outside", constants::EV_OUTSIDE),
    ("ev_right_button", constants::EV_RIGHT_BUTTON),
    ("ev_right_press", constants::EV_RIGHT_PRESS),
    ("ev_right_release", constants::EV_RIGHT_RELEASE),
    ("ev_room_end", constants::EV_ROOM_END),
    ("ev_room_start", constants::EV_ROOM_START),
    ("ev_step", constants::EV_STEP),
    ("ev_step_begin", constants::EV_STEP_BEGIN),
    ("ev_step_end", constants::EV_STEP_END),
    ("ev_step_normal", constants::EV_STEP_NORMAL),
    ("ev_trigger", constants::EV_TRIGGER),
    ("ev_user0", constants::EV_USER0),
    ("ev_user1", constants::EV_USER1),
    ("ev_user10", constants::EV_USER10),
    ("ev_user11", constants::EV_USER11),
    ("ev_user12", constants::EV_USER12),
    ("ev_user13", constants::EV_USER13),
    ("ev_user14", constants::EV_USER14),
    ("ev_user15", constants::EV_USER15),
    ("ev_user2", constants::EV_USER2),
    ("ev_user3", constants::EV_USER3),
    ("ev_user4", constants::EV_USER4),
    ("ev_user5", constants::EV_USER5),
    ("ev_user6", constants::EV_USER6),
    ("ev_user7", constants::EV_USER7),
    ("ev_user8", constants::EV_USER8),
    ("ev_user9", constants::EV_USER9),
    ("fa_archive", constants::FA_ARCHIVE),
    ("fa_bottom", constants::FA_BOTTOM),
    ("fa_center", constants::FA_CENTER),
    ("fa_directory", constants::FA_DIRECTORY),
    ("fa_hidden", constants::FA_HIDDEN),
    ("fa_left", constants::FA_LEFT),
    ("fa_middle", constants::FA_MIDDLE),
    ("fa_readonly", constants::FA_READONLY),
    ("fa_right", constants::FA_RIGHT),
    ("fa_sysfile", constants::FA_SYSFILE),
    ("fa_top", constants::FA_TOP),
    ("fa_volumeid", constants::FA_VOLUMEID),
    ("false", constants::FALSE),
    ("global", constants::GLOBAL),
    ("local", constants::LOCAL),
    ("mb_any", constants::MB_ANY),
    ("mb_left", constants::MB_LEFT),
    ("mb_middle", constants::MB_MIDDLE),
    ("mb_none", constants::MB_NONE),
    ("mb_right", constants::MB_RIGHT),
    ("noone", constants::NOONE),
    ("os_android", constants::OS_ANDROID),
    ("os_ios", constants::OS_IOS),
    ("os_macosx", constants::OS_MACOSX),
    ("os_psp", constants::OS_PSP),
    ("os_win32", constants::OS_WIN32),
    ("other", constants::OTHER),
    ("pi", constants::PI),
    ("pr_linelist", constants::PR_LINELIST),
    ("pr_linestrip", constants::PR_LINESTRIP),
    ("pr_pointlist", constants::PR_POINTLIST),
    ("pr_trianglefan", constants::PR_TRIANGLEFAN),
    ("pr_trianglelist", constants::PR_TRIANGLELIST),
    ("pr_trianglestrip", constants::PR_TRIANGLESTRIP),
    ("ps_change_all", constants::PS_CHANGE_ALL),
    ("ps_change_motion", constants::PS_CHANGE_MOTION),
    ("ps_change_shape", constants::PS_CHANGE_SHAPE),
    ("ps_deflect_horizontal", constants::PS_DEFLECT_HORIZONTAL),
    ("ps_deflect_vertical", constants::PS_DEFLECT_VERTICAL),
    ("ps_distr_gaussian", constants::PS_DISTR_GAUSSIAN),
    ("ps_distr_invgaussian", constants::PS_DISTR_INVGAUSSIAN),
    ("ps_distr_linear", constants::PS_DISTR_LINEAR),
    ("ps_force_constant", constants::PS_FORCE_CONSTANT),
    ("ps_force_linear", constants::PS_FORCE_LINEAR),
    ("ps_force_quadratic", constants::PS_FORCE_QUADRATIC),
    ("ps_shape_diamond", constants::PS_SHAPE_DIAMOND),
    ("ps_shape_ellipse", constants::PS_SHAPE_ELLIPSE),
    ("ps_shape_line", constants::PS_SHAPE_LINE),
    ("ps_shape_rectangle", constants::PS_SHAPE_RECTANGLE),
    ("pt_shape_circle", constants::PT_SHAPE_CIRCLE),
    ("pt_shape_cloud", constants::PT_SHAPE_CLOUD),
    ("pt_shape_disk", constants::PT_SHAPE_DISK),
    ("pt_shape_explosion", constants::PT_SHAPE_EXPLOSION),
    ("pt_shape_flare", constants::PT_SHAPE_FLARE),
    ("pt_shape_line", constants::PT_SHAPE_LINE),
    ("pt_shape_pixel", constants::PT_SHAPE_PIXEL),
    ("pt_shape_ring", constants::PT_SHAPE_RING),
    ("pt_shape_smoke", constants::PT_SHAPE_SMOKE),
    ("pt_shape_snow", constants::PT_SHAPE_SNOW),
    ("pt_shape_spark", constants::PT_SHAPE_SPARK),
    ("pt_shape_sphere", constants::PT_SHAPE_SPHERE),
    ("pt_shape_square", constants::PT_SHAPE_SQUARE),
    ("pt_shape_star", constants::PT_SHAPE_STAR),
    ("se_chorus", constants::SE_CHORUS),
    ("se_compressor", constants::SE_COMPRESSOR),
    ("se_echo", constants::SE_ECHO),
    ("se_equalizer", constants::SE_EQUALIZER),
    ("se_flanger", constants::SE_FLANGER),
    ("se_gargle", constants::SE_GARGLE),
    ("se_none", constants::SE_NONE),
    ("se_reverb", constants::SE_REVERB),
    ("self", constants::SELF),
    ("text_type", constants::TEXT_TYPE),
    ("true", constants::TRUE),
    ("ty_real", constants::TY_REAL),
    ("ty_string", constants::TY_STRING),
    ("vk_add", constants::VK_ADD),
    ("vk_alt", constants::VK_ALT),
    ("vk_anykey", constants::VK_ANYKEY),
    ("vk_backspace", constants::VK_BACKSPACE),
    ("vk_control", constants::VK_CONTROL),
    ("vk_decimal", constants::VK_DECIMAL),
    ("vk_delete", constants::VK_DELETE),
    ("vk_divide", constants::VK_DIVIDE),
    ("vk_down", constants::VK_DOWN),
    ("vk_end", constants::VK_END),
    ("vk_enter", constants::VK_ENTER),
    ("vk_escape", constants::VK_ESCAPE),
    ("vk_f1", constants::VK_F1),
    ("vk_f10", constants::VK_F10),
    ("vk_f11", constants::VK_F11),
    ("vk_f12", constants::VK_F12),
    ("vk_f2", constants::VK_F2),
    ("vk_f3", constants::VK_F3),
    ("vk_f4", constants::VK_F4),
    ("vk_f5", constants::VK_F5),
    ("vk_f6", constants::VK_F6),
    ("vk_f7", constants::VK_F7),
    ("vk_f8", constants::VK_F8),
    ("vk_f9", constants::VK_F9),
    ("vk_home", constants::VK_HOME),
    ("vk_insert", constants::VK_INSERT),
    ("vk_lalt", constants::VK_LALT),
    ("vk_lcontrol", constants::VK_LCONTROL),
    ("vk_left", constants::VK_LEFT),
    ("vk_lshift", constants::VK_LSHIFT),
    ("vk_multiply", constants::VK_MULTIPLY),
    ("vk_nokey", constants::VK_NOKEY),
    ("vk_numpad0", constants::VK_NUMPAD0),
    ("vk_numpad1", constants::VK_NUMPAD1),
    ("vk_numpad2", constants::VK_NUMPAD2),
    ("vk_numpad3", constants::VK_NUMPAD3),
    ("vk_numpad4", constants::VK_NUMPAD4),
    ("vk_numpad5", constants::VK_NUMPAD5),
    ("vk_numpad6", constants::VK_NUMPAD6),
    ("vk_numpad7", constants::VK_NUMPAD7),
    ("vk_numpad8", constants::VK_NUMPAD8),
    ("vk_numpad9", constants::VK_NUMPAD9),
    ("vk_pagedown", constants::VK_PAGEDOWN),
    ("vk_pageup", constants::VK_PAGEUP),
    ("vk_pause", constants::VK_PAUSE),
    ("vk_printscreen", constants::VK_PRINTSCREEN),
    ("vk_ralt", constants::VK_RALT),
    ("vk_rcontrol", constants::VK_RCONTROL),
    ("vk_return", constants::VK_RETURN),
    ("vk_right", constants::VK_RIGHT),
    ("vk_rshift", constants::VK_RSHIFT),
    ("vk_shift", constants::VK_SHIFT),
    ("vk_space", constants::VK_SPACE),
    ("vk_subtract", constants::VK_SUBTRACT),
    ("vk_tab", constants::VK_TAB),
    ("vk_up", constants::VK_UP),
];

const KERNEL_VARS: &[&str] = &[
    "x",
    "y",
    "xprevious",
    "yprevious",
    "xstart",
    "ystart",
    "hspeed",
    "vspeed",
    "direction",
    "speed",
    "friction",
    "gravity",
    "gravity_direction",
    "object_index",
    "id",
    "alarm",
    "solid",
    "visible",
    "persistent",
    "depth",
    "bbox_left",
    "bbox_right",
    "bbox_top",
    "bbox_bottom",
    "sprite_index",
    "image_index",
    "image_single",
    "image_number",
    "sprite_width",
    "sprite_height",
    "sprite_xoffset",
    "sprite_yoffset",
    "image_xscale",
    "image_yscale",
    "image_angle",
    "image_alpha",
    "image_blend",
    "image_speed",
    "mask_index",
    "path_index",
    "path_position",
    "path_positionprevious",
    "path_speed",
    "path_scale",
    "path_orientation",
    "path_endaction",
    "timeline_index",
    "timeline_position",
    "timeline_speed",
    "timeline_running",
    "timeline_loop",
    "argument_relative",
    "argument0",
    "argument1",
    "argument2",
    "argument3",
    "argument4",
    "argument5",
    "argument6",
    "argument7",
    "argument8",
    "argument9",
    "argument10",
    "argument11",
    "argument12",
    "argument13",
    "argument14",
    "argument15",
    "argument",
    "argument_count",
    "room",
    "room_first",
    "room_last",
    "transition_kind",
    "transition_steps",
    "score",
    "lives",
    "health",
    "game_id",
    "working_directory",
    "temp_directory",
    "program_directory",
    "instance_count",
    "instance_id",
    "room_width",
    "room_height",
    "room_caption",
    "room_speed",
    "room_persistent",
    "background_color",
    "background_showcolor",
    "background_visible",
    "background_foreground",
    "background_index",
    "background_x",
    "background_y",
    "background_width",
    "background_height",
    "background_htiled",
    "background_vtiled",
    "background_xscale",
    "background_yscale",
    "background_hspeed",
    "background_vspeed",
    "background_blend",
    "background_alpha",
    "view_enabled",
    "view_current",
    "view_visible",
    "view_xview",
    "view_yview",
    "view_wview",
    "view_hview",
    "view_xport",
    "view_yport",
    "view_wport",
    "view_hport",
    "view_angle",
    "view_hborder",
    "view_vborder",
    "view_hspeed",
    "view_vspeed",
    "view_object",
    "mouse_x",
    "mouse_y",
    "mouse_button",
    "mouse_lastbutton",
    "keyboard_key",
    "keyboard_lastkey",
    "keyboard_lastchar",
    "keyboard_string",
    "cursor_sprite",
    "show_score",
    "show_lives",
    "show_health",
    "caption_score",
    "caption_lives",
    "caption_health",
    "fps",
    "current_time",
    "current_year",
    "current_month",
    "current_day",
    "current_weekday",
    "current_hour",
    "current_minute",
    "current_second",
    "event_type",
    "event_number",
    "event_object",
    "event_action",
    "secure_mode",
    "debug_mode",
    "error_occurred",
    "error_last",
    "gamemaker_registered",
    "gamemaker_pro",
    "gamemaker_version",
    "os_type",
    "os_device",
    "os_browser",
    "os_version",
    "browser_width",
    "browser_height",
    "display_aa",
    "async_load",
];

pub fn make_constants_map() -> HashMap<&'static [u8], f64> {
    CONSTANTS.iter().map(|(s, v)| (s.as_bytes(), *v)).collect()
}

pub fn make_kernel_vars_lut() -> HashSet<&'static [u8]> {
    KERNEL_VARS.iter().copied().map(|x| (x.as_bytes())).collect()
}
