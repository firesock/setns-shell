// TODO: Something smarter with errors (+ errno translation)
// See thiserror + nix crates

struct PidFd(libc::c_int);

impl PidFd {
    fn open(pid: libc::pid_t) -> Result<Self, std::io::Error> {
        let sys_ret = unsafe { libc::syscall(libc::SYS_pidfd_open, pid, 0) };

        if sys_ret < 0 {
            Err(std::io::Error::last_os_error())
        } else {
            use std::convert::TryInto;
            // pidfd_open shouldn't return things that don't fit into c_int
            Ok(Self(
                sys_ret
                    .try_into()
                    .expect("pidfd_open didn't give a valid fd on success"),
            ))
        }
    }
}

impl Drop for PidFd {
    fn drop(&mut self) {
        unsafe {
            // See man-page, ignoring close return should be fine
            libc::close(self.0);
        }
    }
}

pub fn enter_container(
    container_pid: libc::pid_t,
) -> Result<(), std::io::Error> {
    let pidfd = PidFd::open(container_pid)?;
    let res = unsafe {
        libc::setns(
            pidfd.0,
            libc::CLONE_NEWCGROUP
                | libc::CLONE_NEWIPC
                | libc::CLONE_NEWNET
                | libc::CLONE_NEWNS
                | libc::CLONE_NEWPID
                | libc::CLONE_NEWUSER
                | libc::CLONE_NEWUTS,
        )
    };

    if res != 0 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(())
    }
}
