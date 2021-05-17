#[repr(C)]
struct ModuleFeatures {
    builtin_array: *const libc::c_void,
    builtin_count: libc::size_t,
    condition_array: *const libc::c_void,
    condition_count: libc::size_t,
    parameter_array: *const libc::c_void,
    parameter_count: libc::size_t,
    math_array: *const libc::c_void,
    math_count: libc::size_t,
    abstract_count: libc::size_t,
}

const MODULE_FEATURES: ModuleFeatures = ModuleFeatures {
    builtin_array: std::ptr::null(),
    builtin_count: 0,
    condition_array: std::ptr::null(),
    condition_count: 0,
    parameter_array: std::ptr::null(),
    parameter_count: 0,
    math_array: std::ptr::null(),
    math_count: 0,
    abstract_count: 0,
};

extern "C" {
    fn featuresarray(
        module: *const libc::c_void,
        features: *const ModuleFeatures,
    ) -> *const libc::c_void;

    fn handlefeatures(
        module: *const libc::c_void,
        features: *const ModuleFeatures,
        enables: *const libc::c_void,
    ) -> libc::c_int;

    fn setfeatureenables(
        module: *const libc::c_void,
        features: *const ModuleFeatures,
        e: *const libc::c_void,
    ) -> libc::c_int;
}

#[no_mangle]
pub extern "C" fn setup_(_module: *const libc::c_void) -> libc::c_int {
    println!("Zsh dynamic module in Rust!");

    use std::io::Write;
    std::io::stdout().flush().unwrap();

    return 0;
}

#[no_mangle]
pub unsafe extern "C" fn features_(
    module: *const libc::c_void,
    features: *mut *const libc::c_void,
) -> libc::c_int {
    *features = featuresarray(module, &MODULE_FEATURES as *const ModuleFeatures);
    return 0;
}

#[no_mangle]
pub unsafe extern "C" fn enables_(
    module: *const libc::c_void,
    enables: *const libc::c_void,
) -> libc::c_int {
    return handlefeatures(module, &MODULE_FEATURES as *const ModuleFeatures, enables);
}

#[no_mangle]
pub extern "C" fn boot_(_module: *const libc::c_void) -> libc::c_int {
    return 0;
}

#[no_mangle]
pub unsafe extern "C" fn cleanup_(module: *const libc::c_void) -> libc::c_int {
    return setfeatureenables(
        module,
        &MODULE_FEATURES as *const ModuleFeatures,
        std::ptr::null(),
    );
}

#[no_mangle]
pub extern "C" fn finish_(_module: *const libc::c_void) -> libc::c_int {
    return 0;
}
