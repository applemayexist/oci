#![feature(c_size_t, decl_macro, strict_overflow_ops, try_with_capacity)]

// there's only 12 `unsafe`s, i'm sure they're all fine!

mod config;
#[allow(dead_code, non_camel_case_types, non_upper_case_globals)]
mod sys;

use std::ffi::{CStr, c_char, c_int, c_uint};
use std::process::ExitCode;
use std::ptr::null;
use std::slice;

static mut ARGC: c_uint = 0;
static mut ARGV: *const *const c_char = null();

// this is stupid. in Rust, we need to use .init_array to get argv instead of a copy of it.
#[unsafe(link_section = ".init_array")]
#[used]
static CAPTURE_ARGV: extern "C" fn(c_int, *const *const c_char, *const *const c_char) = {
    extern "C" fn capture_argv(
        argc: c_int,
        argv: *const *const c_char,
        _env: *const *const c_char,
    ) {
        unsafe {
            ARGC = argc.try_into().unwrap_or_else(|_| {
                eprintln!("argc was negative.");
                sys::_exit(1)
            });
            ARGV = argv;
        }
    }
    capture_argv
};

unsafe fn streq(s1: *const c_char, s2: *const c_char) -> bool {
    unsafe { sys::strcmp(s1, s2) == 0 }
}

macro static_assert {
    ($cond:expr $(,)?) => { const _: () = assert!($cond); },
    ($cond:expr, $($arg:tt)+) => { const _: () = assert!($cond, $($arg)+); },
}

fn main() -> ExitCode {
    let argc = unsafe { ARGC };
    let argv = unsafe { ARGV };

    if argc == 0 {
        eprintln!("Missing argv[0].");
        return ExitCode::FAILURE;
    }

 /*   let offset = if unsafe {
        streq(
            sys::program_invocation_short_name.cast_const(),
            c"sd-spawn".as_ptr(),
        )
    } {
        if argc == 1 {
            eprintln!("Usage: sd-spawn COMMAND [ARGUMENTS…]");
            return ExitCode::FAILURE;
        }

        1
    } else {
        0
    };
*/
    const BASE_ARGS: &[&CStr] = &[c"systemd-run", c"--user", c"-dtqG", c"--"];

    static_assert!(usize::BITS >= u32::BITS, "usize must be at least 32-bits");
    let Ok(mut child_args) = Vec::try_with_capacity(BASE_ARGS.len().strict_add(argc as usize))
    else {
        eprintln!("Out of memory.");
        return ExitCode::FAILURE;
    };

    child_args.extend(BASE_ARGS.iter().map(|cstr| cstr.as_ptr()));
    if !unsafe {
        streq(
            sys::program_invocation_short_name.cast_const(),
            c"sd-spawn".as_ptr(),
        )
    } {
        child_args.push(unsafe{sys::program_invocation_short_name});
    }
    child_args.extend_from_slice(unsafe {
        slice::from_raw_parts(argv.add(1), argc as usize)
    });

    unsafe {
        sys::execv(
            config::SYSTEMD_RUN_PATH.as_ptr(),
            child_args.as_ptr() as *const *mut c_char,
        )
    };
    eprintln!("Failed to execute systemd-run.");
    ExitCode::FAILURE
}
