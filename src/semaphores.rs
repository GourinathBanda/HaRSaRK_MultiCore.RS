use crate::task_manager::{get_RT, release};
use cortex_m::interrupt::free as execute_critical;

#[derive(Clone, Copy)]
pub struct SCB {
    pub flags: u32,
    pub tasks: u32,
}

static mut SCB_TABLE: [SCB; 32] = [SCB { flags: 0, tasks: 0 }; 32];

pub fn signal_and_release(semaphore: usize, tasks_mask: &u32) {
    execute_critical(|_| {
        let scb_table = unsafe { &mut SCB_TABLE };
        scb_table[semaphore].flags |= *tasks_mask;
        release(&scb_table[semaphore].tasks);
    })
}

pub fn semaphore_set_tasks(semaphore: usize, tasks_mask: &u32) {
    execute_critical(|_| {
        let scb_table = unsafe { &mut SCB_TABLE };
        scb_table[semaphore].tasks |= *tasks_mask;
    });
}

pub fn test_and_reset(semaphore: usize) -> bool {
    execute_critical(|_| {
        let scb_table = unsafe { &mut SCB_TABLE };
        let rt = get_RT() as u32;
        if scb_table[semaphore].flags & (1 << rt) == 1 {
            scb_table[semaphore].flags &= !(1 << rt);
            return true;
        } else {
            return false;
        }
    })
}