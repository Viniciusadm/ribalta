use std::ptr;

use jni_sys::{jbyteArray, jclass, jint, JNIEnv};

unsafe fn read_bytes(env: *mut JNIEnv, array: jbyteArray) -> Vec<u8> {
    let functions = unsafe { &**env };
    let length = unsafe { functions.GetArrayLength.expect("GetArrayLength")(env, array) };
    let mut bytes = vec![0u8; length.max(0) as usize];
    unsafe { functions.GetByteArrayRegion.expect("GetByteArrayRegion")(env, array, 0, length, bytes.as_mut_ptr().cast()) };
    bytes
}

unsafe fn write_bytes(env: *mut JNIEnv, bytes: &[u8]) -> jbyteArray {
    let functions = unsafe { &**env };
    let Ok(length) = jint::try_from(bytes.len()) else { return ptr::null_mut() };
    let array = unsafe { functions.NewByteArray.expect("NewByteArray")(env, length) };
    if !array.is_null() {
        unsafe { functions.SetByteArrayRegion.expect("SetByteArrayRegion")(env, array, 0, length, bytes.as_ptr().cast()) };
    }
    array
}

#[unsafe(no_mangle)]
unsafe extern "system" fn Java_com_archieapps_ribaltareader_compiler_RibaltaCompiler_compileBytes(
    env: *mut JNIEnv,
    _class: jclass,
    prelude: jbyteArray,
    source: jbyteArray,
    english: jint,
    options: jbyteArray,
) -> jbyteArray {
    let prelude = String::from_utf8_lossy(&unsafe { read_bytes(env, prelude) }).into_owned();
    let source = String::from_utf8_lossy(&unsafe { read_bytes(env, source) }).into_owned();
    let options = String::from_utf8_lossy(&unsafe { read_bytes(env, options) }).into_owned();
    let locale = if english == 1 { ribalta::Locale::En } else { ribalta::Locale::Pt };
    let json = ribalta::compile_with_options_to_json(&prelude, &source, locale, &options);
    unsafe { write_bytes(env, json.as_bytes()) }
}
