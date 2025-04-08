//! Process management syscalls

use crate::config::PAGE_SIZE;
use crate::mm::{translate_ptr, translate_ptr_mut, PageTable, VirtAddr, VirtPageNum};
use crate::task::{if_vpn_valid, insert_framed_area_in_cur_task, shrink_area_in_cur_task};
use crate::{
    task::{
        current_user_token, exit_current_and_run_next, suspend_current_and_run_next, TASK_MANAGER,
    },
    timer::get_time_us,
};
#[repr(C)]
#[derive(Debug)]
/// time struct
pub struct TimeVal {
    /// sec
    pub sec: usize,
    /// usec
    pub usec: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(_exit_code: i32) -> ! {
    trace!("kernel: sys_exit");
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(_ts: *mut TimeVal, _tz: usize) -> isize {
    let us = get_time_us();
    let time: TimeVal = TimeVal {
        sec: us / 1_000_000,
        usec: us % 1_000_000,
    };
    let phy_add = translate_ptr_mut(current_user_token(), _ts).unwrap();
    *phy_add = time;
    0
}

/// TODO: Finish sys_trace to pass testcases
/// HINT: You might reimplement it with virtual memory management.
pub fn sys_trace(_trace_request: usize, _id: usize, _data: usize) -> isize {
    match _trace_request {
        0 => {
            let id = _id as *const u8;
            if !if_vpn_valid(_id) {
                return -1;
            }
            let res = translate_ptr(current_user_token(), id);
            match res {
                Ok(i) => *i as isize,
                Err(_) => -1,
            }
        }
        1 => {
            if !if_vpn_valid(_id) {
                return -1;
            }
            let id = _id as *mut u8;
            let data = _data as u8;
            let res = translate_ptr_mut(current_user_token(), id);
            match res {
                Ok(i) => {
                    *i = data;
                    0
                }
                Err(_) => -1,
            }
        }
        2 => TASK_MANAGER.get_cur_task_syscall(_id),
        _ => -1,
    }
}

/// map
pub fn sys_mmap(start: usize, len: usize, prot: usize) -> isize {
    if (prot & !0x7) != 0 {
        println!("invalid prot: {}", prot);
        return -1;
    }
    if (prot & 0x7) == 0 {
        println!("invalid prot: {}", prot);
        return -1;
    }
    let start_va = VirtAddr::from(start);
    if start_va.0 % PAGE_SIZE != 0 {
        println!("start not aligned! start: {}", start_va.0);
        return -1;
    }
    let end_va = VirtAddr::from(start + len);
    for v in start..start + len {
        if if_vpn_valid(v) {
            println!("already mmaped, start_va: {}, start: {}", start_va.0, start);
            return -1;
        }
    }
    println!(
        "map success: start_va: {}, end_va: {}",
        start_va.0, end_va.0
    );
    insert_framed_area_in_cur_task(start_va, end_va, prot as u8);
    0
}

pub fn sys_mumap(start: usize, len: usize) -> isize {
    let start_va = VirtAddr::from(start);
    let start_vpn = start_va.floor();
    if start_va.0 % PAGE_SIZE != 0 {
        println!("start not aligned! start: {}", start_va.0);
        return -1;
    }
    let end_vpn = VirtPageNum::from(start + len);
    let token = current_user_token();
    let mut page_table = PageTable::from_token(token);
    for v in start..start + len {
        if !if_vpn_valid(v) {
            println!("haven't mmaped, start_va: {}, start: {}", start_va.0, start);
            return -1;
        }
    }
    shrink_area_in_cur_task(start_vpn, end_vpn, &mut page_table);
    0
}
