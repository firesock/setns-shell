mod env;
mod parse;
mod setns;

use libc::{c_char, c_int, c_void, size_t};
use std::cell::UnsafeCell;
use std::ptr::null;

// static mut seems frowned on, try the new way
// https://github.com/rust-lang/rust/issues/53639
// Inherently unsafe, nevertheless only written to by zsh which we can't
// control, and reset by us, where we don't care about UB
struct CSharedStruct<T>(UnsafeCell<T>);
impl<T> CSharedStruct<T> {
    const fn new(value: T) -> Self {
        Self(UnsafeCell::new(value))
    }

    const fn get(&self) -> *mut T {
        self.0.get()
    }
}
unsafe impl<T> Sync for CSharedStruct<T> {}

// C reprs and any interface with zsh is specified via libc types, including
// reference/pointer types, to be clear they can be nullable, even if Rust can
// specify our values via references.
// Except for function pointers because marking them nullable is more annoying
#[repr(C)]
struct InnerBuiltinTab {
    null: *const c_void,
    name: *const c_char,
    flags: c_int,
}

#[repr(C)]
struct BuiltinTab {
    inner_tab: InnerBuiltinTab,
    func: extern "C" fn(
        *const c_void,
        *const *const c_char,
        *const c_void,
        c_int,
    ) -> c_int,
    min_args: c_int,
    max_args: c_int,
    func_no: c_int,
    options: *const c_void,
    perm_options: *const c_void,
}

// sizes are built from sizeof math in zsh code, so use size_t
#[repr(C)]
struct ModuleFeatures {
    builtin_array: *const BuiltinTab,
    builtin_count: size_t,
    condition_array: *const c_void,
    condition_count: size_t,
    parameter_array: *const c_void,
    parameter_count: size_t,
    math_array: *const c_void,
    math_count: size_t,
    abstract_count: size_t,
}

static BUILTIN_TAB: CSharedStruct<BuiltinTab> =
    CSharedStruct::new(BuiltinTab {
        inner_tab: InnerBuiltinTab {
            null: null(),
            name: b"setns_shell\0".as_ptr() as *const c_char,
            flags: 0,
        },
        func: setns_shell,
        min_args: parse::ARG_COUNT,
        max_args: parse::ARG_COUNT,
        func_no: 0,
        options: null(),
        perm_options: null(),
    });

static MODULE_FEATURES: CSharedStruct<ModuleFeatures> =
    CSharedStruct::new(ModuleFeatures {
        builtin_array: BUILTIN_TAB.get(),
        builtin_count: 1,
        condition_array: null(),
        condition_count: 0,
        parameter_array: null(),
        parameter_count: 0,
        math_array: null(),
        math_count: 0,
        abstract_count: 0,
    });

extern "C" {
    fn featuresarray(
        module: *const c_void,
        features: *const ModuleFeatures,
    ) -> *const c_void;

    fn handlefeatures(
        module: *const c_void,
        features: *const ModuleFeatures,
        enables: *const c_void,
    ) -> c_int;

    fn setfeatureenables(
        module: *const c_void,
        features: *const ModuleFeatures,
        e: *const c_void,
    ) -> c_int;
}

extern "C" fn setns_shell(
    _name: *const c_void,
    args: *const *const c_char,
    _options: *const c_void,
    _func_no: c_int,
) -> c_int {
    // TODO: Better user error reporting?
    let parsed = parse::Args::parse(args);
    if let Ok(parsed_args) = parsed {
        let res = setns::enter_container(parsed_args.pid);
        if let Err(error) = res {
            eprintln!("Unable to enter container: {}", error);
            1
        } else {
            env::NSEnv::discover().write(&parsed_args.zwc_data);
            0
        }
    } else {
        eprintln!("Unable to parse args",);
        1
    }
}

#[no_mangle]
pub extern "C" fn setup_(_module: *const c_void) -> c_int {
    0
}

#[no_mangle]
pub unsafe extern "C" fn features_(
    module: *const c_void,
    features: *mut *const c_void,
) -> c_int {
    *features = featuresarray(module, MODULE_FEATURES.get());
    0
}

#[no_mangle]
pub unsafe extern "C" fn enables_(
    module: *const c_void,
    enables: *const c_void,
) -> c_int {
    handlefeatures(module, MODULE_FEATURES.get(), enables)
}

#[no_mangle]
pub extern "C" fn boot_(_module: *const c_void) -> c_int {
    0
}

#[no_mangle]
pub unsafe extern "C" fn cleanup_(module: *const c_void) -> c_int {
    setfeatureenables(module, MODULE_FEATURES.get(), null())
}

#[no_mangle]
pub unsafe extern "C" fn finish_(_module: *const c_void) -> c_int {
    // zsh doesn't guarantee that finish_ is called paired with setup_
    // success, so rather than using refcounting to track calls, we just
    // clobber everything on finish_ in an unsafe manner. C has our
    // pointers and might still be writing to it for all we know!
    // Also why we used a static chunk of memory rather than the heap,
    // calling heap 'free'/drop wouldn't be fun
    let builtintab_p = BUILTIN_TAB.get();
    let modulefeatures_p = MODULE_FEATURES.get();

    // Duplicate defs to static are because const_fn_fn_ptr_basics is
    // not in Rust stable - can't use fn ptr to setns_shell in shared fn
    // https://github.com/rust-lang/rust/pull/77170 + tracking parent
    *builtintab_p = BuiltinTab {
        inner_tab: InnerBuiltinTab {
            null: null(),
            name: b"setns_shell\0".as_ptr() as *const c_char,
            flags: 0,
        },
        func: setns_shell,
        min_args: parse::ARG_COUNT,
        max_args: parse::ARG_COUNT,
        func_no: 0,
        options: null(),
        perm_options: null(),
    };
    *modulefeatures_p = ModuleFeatures {
        builtin_array: builtintab_p,
        builtin_count: 1,
        condition_array: null(),
        condition_count: 0,
        parameter_array: null(),
        parameter_count: 0,
        math_array: null(),
        math_count: 0,
        abstract_count: 0,
    };
    0
}
