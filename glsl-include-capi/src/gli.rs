use std;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use glsl_include;

pub struct Context<'a> {
    raw: glsl_include::Context<'a>,
    source_map: Option<glsl_include::SourceMap<'a>>,
    err: Option<glsl_include::Error>,
}

impl<'a> Context<'a> {
    pub fn new(raw: glsl_include::Context<'a>) -> Context<'a> {
        Context {
            raw: raw,
            source_map: None,
            err: None,
        }
    }
}

#[no_mangle]
pub extern "C" fn gli_ctx_new<'a>() -> *const Context<'a> {
    Box::into_raw(Box::new(Context::new(glsl_include::Context::new())))
}

#[no_mangle]
pub extern "C" fn gli_ctx_free<'a>(ctx: *const Context<'a>) {
    if ctx.is_null() {
        return;
    }
    unsafe {
        Box::from_raw(ctx as *mut Context);
    }
}

#[no_mangle]
pub extern "C" fn gli_str_free(p: *mut c_char) {
    if p.is_null() {
        return;
    }
    unsafe {
        let _ = CString::from_raw(p);
    }
}

#[no_mangle]
pub extern "C" fn gli_include<'a>(
    ctx: *mut Context<'a>,
    file: *const c_char,
    content: *const c_char,
) {
    let file_c_str: &CStr = unsafe { CStr::from_ptr(file) };
    let file_str: &str = file_c_str.to_str().unwrap();
    let content_c_str: &CStr = unsafe { CStr::from_ptr(content) };
    let content_str: &str = content_c_str.to_str().unwrap();
    let ctx = unsafe { &mut *ctx };
    ctx.raw.include(file_str, content_str);
}

#[no_mangle]
pub extern "C" fn gli_expand_to_str<'a>(
    ctx: *mut Context<'a>,
    src: *const c_char,
) -> *const c_char {
    let c_str = unsafe { CStr::from_ptr(src) };
    let src = c_str.to_str().unwrap();
    let ctx = unsafe { &mut *ctx };
    match ctx.raw.expand_to_string(src) {
        Ok((expanded_src, source_map)) => {
            ctx.source_map = Some(source_map);
            ctx.err = None;
            let ret = CString::new(expanded_src).unwrap();
            ret.into_raw()
        }
        Err(e) => {
            ctx.source_map = None;
            ctx.err = Some(e);
            std::ptr::null()
        }
    }
}

#[no_mangle]
pub extern "C" fn gli_get_source_mapping<'a>(
    ctx: *const Context<'a>,
    expanded_line_num: c_int,
    src_file: *mut *const c_char,
    src_line_num: *mut c_int,
) {
    let ctx = unsafe { &*ctx };
    if let Some(ref source_map) = ctx.source_map {
        let fl = source_map.get(expanded_line_num as usize);
        if let Some(fl) = fl {
            if let Some(ref file) = fl.file {
                let file = CString::new(*file).expect("Error constructing CString");
                unsafe {
                    *src_file = file.into_raw();
                }
            } else {
                unsafe {
                    *src_file = std::ptr::null_mut();
                }
            }
            unsafe {
                *src_line_num = fl.line as i32;
            }
        } else {
            unsafe {
                *src_file = std::ptr::null_mut();
                *src_line_num = -1;
            }
        }
    } else {
        unsafe {
            *src_file = std::ptr::null_mut();
            *src_line_num = -1;
        }
    }
}

#[no_mangle]
pub extern "C" fn gli_get_error_str<'a>(ctx: *const Context<'a>) -> *const c_char {
    let ctx = unsafe { &*ctx };
    if let Some(ref err) = ctx.err {
        let fmt_err = format!("{}", err);
        let ret = CString::new(fmt_err).unwrap();
        ret.into_raw()
    } else {
        std::ptr::null()
    }
}
