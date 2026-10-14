use std::alloc::{Layout, alloc, dealloc};

#[unsafe(no_mangle)]
pub extern "C" fn ribalta_alloc(len: usize) -> *mut u8 {
    if len == 0 {
        return std::ptr::NonNull::dangling().as_ptr();
    }
    unsafe { alloc(Layout::from_size_align_unchecked(len, 1)) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ribalta_free(ptr: *mut u8, len: usize) {
    if len != 0 {
        unsafe { dealloc(ptr, Layout::from_size_align_unchecked(len, 1)) }
    }
}

unsafe fn read(ptr: *const u8, len: usize) -> String {
    let bytes = unsafe { std::slice::from_raw_parts(ptr, len) };
    String::from_utf8_lossy(bytes).into_owned()
}

fn write(output: String) -> u64 {
    let output = output.into_bytes();
    let out_len = output.len();
    let out_ptr = ribalta_alloc(out_len);
    unsafe { std::ptr::copy_nonoverlapping(output.as_ptr(), out_ptr, out_len) };
    ((out_ptr as u64) << 32) | out_len as u64
}

fn locale(code: u32) -> crate::Locale {
    if code == 1 { crate::Locale::En } else { crate::Locale::Pt }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ribalta_compile(prelude_ptr: *const u8, prelude_len: usize, ptr: *const u8, len: usize, locale_code: u32, options_ptr: *const u8, options_len: usize) -> u64 {
    let prelude = unsafe { read(prelude_ptr, prelude_len) };
    let options = unsafe { read(options_ptr, options_len) };
    write(crate::compile_with_options_to_json(&prelude, &unsafe { read(ptr, len) }, locale(locale_code), &options))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ribalta_check_prelude(ptr: *const u8, len: usize, locale_code: u32) -> u64 {
    write(crate::check_prelude_to_json(&unsafe { read(ptr, len) }, locale(locale_code)))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ribalta_definitions(ptr: *const u8, len: usize) -> u64 {
    write(crate::definitions_to_json(&unsafe { read(ptr, len) }))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ribalta_extract(src_ptr: *const u8, src_len: usize, roots_ptr: *const u8, roots_len: usize) -> u64 {
    let source = unsafe { read(src_ptr, src_len) };
    write(crate::extract_to_json(&source, &unsafe { read(roots_ptr, roots_len) }))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ribalta_conflicts(theme_ptr: *const u8, theme_len: usize, snippet_ptr: *const u8, snippet_len: usize) -> u64 {
    let theme = unsafe { read(theme_ptr, theme_len) };
    write(crate::conflicts_to_json(&theme, &unsafe { read(snippet_ptr, snippet_len) }))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ribalta_merge(theme_ptr: *const u8, theme_len: usize, snippet_ptr: *const u8, snippet_len: usize, res_ptr: *const u8, res_len: usize, label_ptr: *const u8, label_len: usize) -> u64 {
    let theme = unsafe { read(theme_ptr, theme_len) };
    let snippet = unsafe { read(snippet_ptr, snippet_len) };
    let resolutions = unsafe { read(res_ptr, res_len) };
    write(crate::merge_to_json(&theme, &snippet, &resolutions, &unsafe { read(label_ptr, label_len) }))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ribalta_validate_item(kind_ptr: *const u8, kind_len: usize, src_ptr: *const u8, src_len: usize, entry_ptr: *const u8, entry_len: usize, locale_code: u32) -> u64 {
    let kind = unsafe { read(kind_ptr, kind_len) };
    let source = unsafe { read(src_ptr, src_len) };
    write(crate::validate_item_to_json(&kind, &source, &unsafe { read(entry_ptr, entry_len) }, locale(locale_code)))
}
