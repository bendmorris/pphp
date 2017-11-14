#![crate_type = "staticlib"]

#[macro_use]
pub mod ast;
pub mod context;
pub mod php;
pub mod rules;

#[macro_use]
extern crate lazy_static;

use std::os::raw::c_char;
use std::ffi::CStr;
use ast::ZendAst;

#[no_mangle]
pub extern "C" fn rust_pphp_optimize_ast(zast: ZendAst) {
    rules::apply_all(zast);
}

#[no_mangle]
pub extern "C" fn rust_pphp_add_rule(replace: *const c_char, with: *const c_char) -> php::zend_bool {
    let replace = unsafe {CStr::from_ptr(replace)}.to_str().unwrap().to_string();
    let with = unsafe {CStr::from_ptr(with)}.to_str().unwrap().to_string();
    match rules::custom::CustomSubstitution::try_create(replace, with) {
        Some(rule) => {
            rules::add_rule(Box::new(rule));
            1
        }
        None => {
            0
        }
    }
}

#[no_mangle]
pub extern "C" fn rust_pphp_set_debug_trace(enabled: php::zend_bool) {
    ast::set_debug_trace(enabled != 0);
}
