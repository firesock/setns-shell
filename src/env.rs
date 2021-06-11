// TODO: Errors
pub struct NSEnv {
    username: String,
    home: String,
    hostname: String,
    path: String,
}

impl NSEnv {
    pub fn discover() -> Self {
        // login system is responsible for HOME envvar from kernel
        // Cribbed the list of envvars to set from an absolutely cursory
        // look at what OpenSSH does
        let (username, home, hostname) = unsafe {
            // TODO: nulls
            let passwd = libc::getpwuid(libc::geteuid());
            // TODO: Think harder about string conversion
            let username = std::ffi::CStr::from_ptr((*passwd).pw_name)
                .to_str()
                .unwrap()
                .to_owned();
            let home = std::ffi::CStr::from_ptr((*passwd).pw_dir)
                .to_str()
                .unwrap()
                .to_owned();

            // man page said POSIX was 255, even if linux is smaller
            let mut hostname_str = vec![0; 256];
            libc::gethostname(
                hostname_str.as_mut_ptr() as *mut i8,
                hostname_str.len(),
            );

            let hostname = String::from_utf8(
                hostname_str.drain(..).take_while(|&b| b != 0).collect(),
            )
            .unwrap();
            (username, home, hostname)
        };

        let sh_output = std::process::Command::new("/bin/sh")
            .arg("-l")
            .arg("-c")
            .arg("printenv")
            .env_clear()
            .output()
            .unwrap()
            .stdout;

        // TODO: Something smarter about multiline envvars
        // TODO: Only being used for PATH now
        let mut environ: std::collections::HashMap<String, String> =
            String::from_utf8(sh_output)
                .unwrap()
                .lines()
                .filter(|s| s.contains("="))
                .map(|s| s.split("="))
                .map(|mut s| (s.next().unwrap(), s.next().unwrap()))
                .map(|(k, v)| (k.to_owned(), v.to_owned()))
                .collect();

        Self {
            username: username,
            home: home,
            hostname: hostname,
            path: environ.remove("PATH").unwrap(),
        }
    }

    pub fn write(self, zwc_data: &[u8]) {
        use std::io::Write;
        // Ensure there are no attempts to reference old-env
        std::env::remove_var("TMPDIR");
        let zwc_path = std::env::temp_dir().join("full.zwc");
        let mut zwc_file = std::fs::File::create(&zwc_path).unwrap();
        zwc_file.write_all(zwc_data).unwrap();
        zwc_file.sync_data().unwrap();
        std::mem::drop(zwc_file);

        // set_var confuses the shell
        println!("unset HISTFILE;");
        println!("export {}={};", "HOME", &self.home);
        println!("export {}={};", "USER", &self.username);
        println!("export {}={};", "HOSTNAME", &self.hostname);
        println!("export {}={};", "PATH", &self.path);

        println!("export {}={};", "FPATH", zwc_path.display());
        println!("cd {};", &self.home);
    }
}
