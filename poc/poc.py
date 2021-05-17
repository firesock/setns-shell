"""
PoC to setns into a running container after interactive process has loaded
"""


import ctypes
import os
import subprocess


def _get_docker_root_pid() -> int:
    """Return root pid of first container - must be only one AND running alpine"""

    ps_return = subprocess.run(
        ["docker", "ps", "--format", "{{.ID}}-{{.Image}}"],
        capture_output=True,
        timeout=5,
        check=True,
        text=True,
    ).stdout.splitlines()
    assert len(ps_return) == 1

    container_id, image = ps_return[0].split("-", maxsplit=1)
    assert image == "alpine"

    inspect_return = subprocess.run(
        ["docker", "inspect", "--format", "{{.State.Pid}}", container_id],
        capture_output=True,
        timeout=5,
        check=True,
        text=True,
    ).stdout.splitlines()
    assert len(inspect_return) == 1

    return int(inspect_return[0].strip(), base=10)


def nsenter():
    """Enter container namespaces for _get_docker_root_pid()"""

    libc = ctypes.CDLL("libc.so.6", use_errno=True)

    # syscall no's + type sizes taken from amd64
    syscall = libc.syscall
    syscall.argtypes = [ctypes.c_long, ctypes.c_int32, ctypes.c_uint]
    syscall.restype = ctypes.c_int32
    pidfd = syscall(434, _get_docker_root_pid(), 0)
    assert pidfd > -1

    # Lovingly yoinked from sched.h on my system
    CLONE_NEWCGROUP = 0x02000000
    CLONE_NEWIPC = 0x08000000
    CLONE_NEWNET = 0x40000000
    CLONE_NEWNS = 0x00020000
    CLONE_NEWPID = 0x20000000
    CLONE_NEWTIME = 0x00000080
    CLONE_NEWUSER = 0x10000000
    CLONE_NEWUTS = 0x04000000

    setns_flags = (
        CLONE_NEWCGROUP
        | CLONE_NEWIPC
        | CLONE_NEWNET
        | CLONE_NEWNS
        | CLONE_NEWPID
        | CLONE_NEWUSER
        | CLONE_NEWUTS
    )
    setns = libc.setns
    setns.argtypes = [ctypes.c_int32, ctypes.c_int32]
    setns.restype = ctypes.c_int32
    return_ = setns(pidfd, setns_flags)
    if return_ != 0:
        print(os.strerror(ctypes.get_errno()))
    return return_ == 0


if __name__ == "__main__":
    print(nsenter())
