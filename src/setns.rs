pub fn enter_container(container_pid: libc::pid_t) {
    // TODO: Error handling
    // TODO: Close pidfd
    unsafe {
        let pidfd = libc::syscall(libc::SYS_pidfd_open, container_pid, 0);
        use std::convert::TryInto;
        let res = libc::setns(
            pidfd.try_into().unwrap(),
            libc::CLONE_NEWCGROUP
                | libc::CLONE_NEWIPC
                | libc::CLONE_NEWNET
                | libc::CLONE_NEWNS
                | libc::CLONE_NEWPID
                | libc::CLONE_NEWUSER
                | libc::CLONE_NEWUTS,
        );
        assert!(res == 0)
    }
}
