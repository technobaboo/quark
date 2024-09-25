use crate::prelude::*;

use std::ffi::{c_char, CStr};

pub unsafe fn str_slice_from_const_arr<'a>(
    ptr: *const *const c_char,
    len: usize,
) -> &'a [*const c_char] {
    unsafe { std::slice::from_raw_parts(ptr, len) }
}

pub unsafe fn enumerate<I: Clone>(
    input_count: u32,
    output_count: &mut Option<u32>,
    items_ptr: *mut I,
    items: &[I],
) -> XrResult {
    // if output_count.is_none() {
    // 	return Err(XrErr::ERROR_VALIDATION_FAILURE);
    // }
    *output_count = Some(items.len() as u32);
    if input_count == 0 || items_ptr.is_null() {
        return Ok(());
    }
    if input_count < items.len() as u32 {
        return Err(XrErr::ERROR_SIZE_INSUFFICIENT);
    }
    if items_ptr.is_null() {
        return Ok(());
    }
    std::ptr::copy_nonoverlapping(items.as_ptr(), items_ptr, items.len());

    Ok(())
}

pub unsafe fn str_from_const_char<'a>(ptr: *const c_char) -> Result<&'a str, XrErr> {
    if ptr.is_null() {
        return Err(XrErr::ERROR_VALIDATION_FAILURE);
    }

    CStr::from_ptr(ptr)
        .to_str()
        .map_err(|_| XrErr::ERROR_VALIDATION_FAILURE)
}

pub fn copy_str_to_buffer<const N: usize>(string: &str, buf: &mut [c_char; N]) {
    buf.fill(0);
    unsafe {
        std::ptr::copy_nonoverlapping(
            string.as_ptr() as *const i8,
            buf.as_mut_ptr(),
            string.len().min(N),
        )
    }
}
