//! Implementation of syscalls
//!
//! The single entry point to all system calls, [`syscall()`], is called
//! whenever userspace wishes to perform a system call using the `ecall`
//! instruction. In this case, the processor raises an 'Environment call from
//! U-mode' exception, which is handled as one of the cases in
//! [`crate::trap::trap_handler`].
//!
//! For clarity, each single syscall is implemented as its own function, named
//! `sys_` then the name of the syscall. You can find functions like this in
//! submodules, and you should also implement syscalls this way.
const SYSCALL_WRITE: usize = 64;
/// exit syscall
const SYSCALL_EXIT: usize = 93;
/// yield syscall
const SYSCALL_YIELD: usize = 124;
/// gettime syscall
const SYSCALL_GET_TIME: usize = 169;
/// sbrk syscall
const SYSCALL_SBRK: usize = 214;
/// munmap syscall
const SYSCALL_MUNMAP: usize = 215;
/// mmap syscall
const SYSCALL_MMAP: usize = 222;
/// trace syscall
const SYSCALL_TRACE: usize = 410;

mod fs;
mod process;

use fs::*;
pub use process::TimeVal;
use process::*;
/// store syscall_count
#[derive(Clone, Copy)]
pub struct SyscallCount {
    write_count: isize,
    exit_count: isize,
    yield_count: isize,
    get_count: isize,
    trace_count: isize,
}

impl SyscallCount {
    /// init, all 0
    pub fn new() -> Self {
        Self {
            write_count: 0,
            exit_count: 0,
            yield_count: 0,
            get_count: 0,
            trace_count: 0,
        }
    }
    /// get syscall count
    pub fn get_count(&self, id: usize) -> isize {
        match id {
            SYSCALL_WRITE => self.write_count,
            SYSCALL_EXIT => self.exit_count,
            SYSCALL_GET_TIME => self.get_count,
            SYSCALL_YIELD => self.yield_count,
            SYSCALL_TRACE => self.trace_count,
            _ => -1,
        }
    }

    /// count once for current task (impl for SyscallCount)
    pub fn count_once_syscall(&mut self, id: usize) {
        match id {
            SYSCALL_WRITE => self.write_count += 1,
            SYSCALL_EXIT => self.exit_count += 1,
            SYSCALL_GET_TIME => self.get_count += 1,
            SYSCALL_YIELD => self.yield_count += 1,
            SYSCALL_TRACE => self.trace_count += 1,
            _ => (),
        }
    }
}

use crate::task::TASK_MANAGER;
/// handle syscall exception with `syscall_id` and other arguments
pub fn syscall(syscall_id: usize, args: [usize; 3]) -> isize {
    match syscall_id {
        SYSCALL_WRITE => {
            TASK_MANAGER.cur_task_syscall_count(SYSCALL_WRITE);
            sys_write(args[0], args[1] as *const u8, args[2])
        }
        SYSCALL_EXIT => {
            TASK_MANAGER.cur_task_syscall_count(SYSCALL_EXIT);
            sys_exit(args[0] as i32)
        }
        SYSCALL_YIELD => {
            TASK_MANAGER.cur_task_syscall_count(SYSCALL_YIELD);
            sys_yield()
        }
        SYSCALL_GET_TIME => {
            TASK_MANAGER.cur_task_syscall_count(SYSCALL_GET_TIME);
            sys_get_time(args[0] as *mut TimeVal, args[1])
        }
        SYSCALL_TRACE => {
            TASK_MANAGER.cur_task_syscall_count(SYSCALL_TRACE);
            sys_trace(args[0], args[1], args[2])
        }
        SYSCALL_SBRK => 1,
        SYSCALL_MUNMAP => sys_mumap(args[0], args[1]),
        SYSCALL_MMAP => sys_mmap(args[0], args[1], args[2]),
        _ => panic!("Unsupported syscall_id: {}", syscall_id),
    }
}
