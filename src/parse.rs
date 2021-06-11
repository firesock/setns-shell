use libc::c_char;
use std::ffi::CStr;

#[derive(Debug)]
struct NullPtr;

impl std::fmt::Display for NullPtr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Null pointer received")
    }
}

impl std::error::Error for NullPtr {}

#[derive(Debug, PartialEq)]
pub struct Args {
    pub pid: libc::pid_t,
    pub zwc_data: Vec<u8>,
}

impl Args {
    pub fn parse(
        args: *const *const c_char,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        unsafe {
            if args.is_null() {
                return Err(Box::new(NullPtr));
            };

            // We pray that the array is the right size
            const ARRAY_SIZE: usize = 2;
            let cstr_args = std::slice::from_raw_parts(args, ARRAY_SIZE)
                .iter()
                .map(|&p| {
                    (!p.is_null())
                        .then(|| p)
                        .and_then(|p| Some(CStr::from_ptr(p)))
                })
                .try_fold(Vec::new(), |mut acc, maybe_cstr| {
                    maybe_cstr.and_then(|cstr| {
                        acc.push(cstr);
                        Some(acc)
                    })
                })
                .ok_or(NullPtr)?;

            use std::convert::TryInto;
            let [pid_cstr, zwc_cstr]: [&CStr; ARRAY_SIZE] =
                cstr_args.try_into().unwrap();

            use std::io::Read;
            let mut zwc_data = Vec::new();
            std::fs::File::open(zwc_cstr.to_str()?)?
                .read_to_end(&mut zwc_data)?;

            Ok(Self {
                pid: pid_cstr.to_str()?.parse()?,
                zwc_data: zwc_data,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Args::parse TEST ONLY, assumes UTF-8
    macro_rules! cstr {
        ( $l:literal ) => {
            concat!($l, "\0").as_ptr() as *const c_char
        };
    }

    // Args::parse TEST ONLY, assumes size passed is correct
    macro_rules! carr {
        ( $( $e:expr ),* ) => {
            (&([ $($e,)* ])).as_ptr()
        };
    }

    #[test]
    fn cstr_converts_string_literal() {
        assert_eq!(
            unsafe { CStr::from_ptr(cstr!("577")).to_bytes_with_nul() },
            b"577\0"
        );
    }

    #[test]
    fn carr_converts_array_literal() {
        assert_eq!(
            unsafe {
                std::slice::from_raw_parts(
                    carr![cstr!("566"), cstr!("5678")],
                    2,
                )
                .iter()
                .map(|&s| CStr::from_ptr(s).to_bytes_with_nul())
                .collect::<Vec<&[u8]>>()
            },
            [&b"566\0"[..], &b"5678\0"[..]],
        );
    }

    macro_rules! tempfile_cpath {
        ( $f:ident, $s:ident ) => {
            #[allow(unused_mut)]
            let mut $f = tempfile::NamedTempFile::new().unwrap();
            let path =
                std::ffi::CString::new(($f).path().to_str().unwrap()).unwrap();
            let $s = (&path).as_ptr();
        };
    }

    #[test]
    fn args_parse_fails_null_array() {
        assert!(Args::parse(std::ptr::null()).is_err());
    }

    // Rely on caller to ensure args array size, no tests written for len != 2

    #[test]
    fn args_parse_fails_null_arguments() {
        tempfile_cpath!(zwc_file, zwc_cpath);
        assert!(Args::parse(carr![cstr!("577"), std::ptr::null()]).is_err());
        assert!(Args::parse(carr![std::ptr::null(), zwc_cpath]).is_err());
    }

    #[test]
    fn args_parse_builds_return_correctly() {
        tempfile_cpath!(zwc_file, zwc_cpath);
        assert_eq!(
            Args::parse(carr![cstr!("577"), zwc_cpath]).unwrap(),
            Args {
                pid: 577,
                zwc_data: vec![]
            }
        );
    }

    #[test]
    fn args_parse_parses_pid_correctly() {
        tempfile_cpath!(zwc_file, zwc_cpath);
        assert_eq!(
            Args::parse(carr![cstr!("5778"), zwc_cpath]).unwrap().pid,
            5778
        );
    }

    #[test]
    fn args_parse_reads_text_correctly() {
        use std::io::Write;
        tempfile_cpath!(zwc_file, zwc_cpath);
        let write_file = zwc_file.as_file_mut();
        write_file.write_all(b"test_data").unwrap();
        write_file.flush().unwrap();
        write_file.sync_all().unwrap();

        assert_eq!(
            Args::parse(carr![cstr!("577"), zwc_cpath])
                .unwrap()
                .zwc_data,
            b"test_data"
        );
    }

    // TODO: Test the error return types

    #[test]
    fn args_parse_fails_invalid_pids() {
        tempfile_cpath!(zwc_file, zwc_cpath);
        assert!(Args::parse(carr![cstr!("five"), zwc_cpath]).is_err());
        assert!(Args::parse(carr![cstr!(""), zwc_cpath]).is_err());
        assert!(Args::parse(carr![
            b"\xFE\0".as_ptr() as *const c_char,
            zwc_cpath
        ])
        .is_err());
    }

    #[test]
    fn args_parse_fails_invalid_paths() {
        assert!(Args::parse(carr![cstr!("577"), cstr!("")]).is_err());
        assert!(Args::parse(carr![
            cstr!("577"),
            b"\xFE\0".as_ptr() as *const c_char
        ])
        .is_err());
    }

    #[test]
    fn args_parse_fails_missing_files() {
        tempfile_cpath!(zwc_file, zwc_cpath);
        std::mem::drop(zwc_file);

        assert!(Args::parse(carr![cstr!("577"), zwc_cpath]).is_err());
    }
}
