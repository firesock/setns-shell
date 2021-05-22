use libc::c_char;

#[derive(Debug)]
struct NullPtr;

impl std::fmt::Display for NullPtr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Null pointer received")
    }
}

impl std::error::Error for NullPtr {}

pub fn pid_from_args(
    args: *const *const c_char,
) -> Result<libc::pid_t, Box<dyn std::error::Error>> {
    unsafe {
        if args.is_null() {
            return Err(Box::new(NullPtr));
        };
        if (*args).is_null() {
            return Err(Box::new(NullPtr));
        };
        Ok(std::ffi::CStr::from_ptr(*args).to_str()?.parse()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pid_from_args_parses_nums() {
        let num = b"577\0".as_ptr() as *const c_char;
        assert_eq!(pid_from_args(&num).unwrap(), 577);
    }

    #[test]
    fn pid_from_args_fails_letters() {
        let val = b"five\0".as_ptr() as *const c_char;
        assert!(pid_from_args(&val).is_err());
    }

    #[test]
    fn pid_from_args_fails_non_ascii() {
        let val = b"\xFE\0".as_ptr() as *const c_char;
        assert!(pid_from_args(&val).is_err());
    }

    #[test]
    fn pid_from_args_fails_empty() {
        let val = b"\0".as_ptr() as *const c_char;
        assert!(pid_from_args(&val).is_err());
    }

    #[test]
    fn pid_from_args_fails_null_array() {
        assert!(pid_from_args(std::ptr::null()).is_err());
    }

    #[test]
    fn pid_from_args_fails_null_first_string() {
        let val = std::ptr::null();
        assert!(pid_from_args(&val).is_err());
    }
}
