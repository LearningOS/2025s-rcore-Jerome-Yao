//!Implementation of [`TaskManager`]
use super::TaskControlBlock;
use crate::mm::{MapPermission, PageTable, VirtAddr, VirtPageNum};
use crate::sync::UPSafeCell;
#[allow(unused)]
use crate::task::{current_task, TaskStatus};
use alloc::collections::VecDeque;
use alloc::sync::Arc;
use lazy_static::*;

#[allow(unused)]
pub const BIGSTRIDE: isize = 1000;
///A array of `TaskControlBlock` that is thread-safe
pub struct TaskManager {
    ready_queue: VecDeque<Arc<TaskControlBlock>>,
}

/// A simple FIFO scheduler.
impl TaskManager {
    ///Creat an empty TaskManager
    pub fn new() -> Self {
        Self {
            ready_queue: VecDeque::new(),
        }
    }
    /// Add process back to ready queue
    pub fn add(&mut self, task: Arc<TaskControlBlock>) {
        self.ready_queue.push_back(task);
    }
    /// Take a process out of the ready queue
    pub fn fetch(&mut self) -> Option<Arc<TaskControlBlock>> {
        let mut min_stride = isize::MAX;
        let mut index: usize = 0;
        for tcb in &self.ready_queue {
            let tcb_inner = tcb.inner_exclusive_access();
            if min_stride > tcb_inner.stride && tcb_inner.task_status == TaskStatus::Ready {
                min_stride = tcb_inner.stride;
                index += 1;
            }
            drop(tcb_inner);
        }
        let target_task = self.ready_queue.remove(index - 1).unwrap();
        let mut target_inner = target_task.inner_exclusive_access();
        // let mut target_stride = &target_task.stride;
        target_inner.stride += BIGSTRIDE / target_inner.priority;
        drop(target_inner);
        Some(target_task)
    }

    /// determine whether a vpn valid in current task memory_set
    pub fn if_vpn_valid_in_cur_task(&self, vpn: VirtPageNum) -> bool {
        // let inner = self.inner.exclusive_access();
        // inner.tasks[inner.current_task].memory_set.if_vpn_valid(vpn)
        let cur_task = current_task().unwrap();
        let flag = cur_task
            .inner_exclusive_access()
            .memory_set
            .if_vpn_valid(vpn);
        drop(cur_task);
        flag
    }

    /// insert_framed_area in current_task memory_set
    pub fn insert_framed_area(
        &self,
        start_va: VirtAddr,
        end_va: VirtAddr,
        permission: MapPermission,
    ) {
        let cur_task = current_task().unwrap();
        cur_task
            .inner_exclusive_access()
            .memory_set
            .insert_framed_area(start_va, end_va, permission);
    }

    /// unmap a range of vpn
    pub fn shrink_area(
        &self,
        start_vpn: VirtPageNum,
        end_vpn: VirtPageNum,
        page_table: &mut PageTable,
    ) {
        let cur_task = current_task().unwrap();
        cur_task
            .inner_exclusive_access()
            .memory_set
            .shrink_area(start_vpn, end_vpn, page_table);
    }
}

lazy_static! {
    /// TASK_MANAGER instance through lazy_static!
    pub static ref TASK_MANAGER: UPSafeCell<TaskManager> =
        unsafe { UPSafeCell::new(TaskManager::new()) };
}

/// Add process to ready queue
pub fn add_task(task: Arc<TaskControlBlock>) {
    //trace!("kernel: TaskManager::add_task");
    TASK_MANAGER.exclusive_access().add(task);
}

/// Take a process out of the ready queue
pub fn fetch_task() -> Option<Arc<TaskControlBlock>> {
    //trace!("kernel: TaskManager::fetch_task");
    TASK_MANAGER.exclusive_access().fetch()
}
