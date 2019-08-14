//use core::alloc::
use crate::semaphores::SCB;
use crate::task_manager::{get_RT, release};
use cortex_m::interrupt::free as execute_critical;
use cortex_m_semihosting::hprintln;

const MAX_BUFFER_SIZE: usize = 32;

pub type Buffer = &'static [u32];

#[derive(Clone, Copy)]
struct TCB {
    dest_buffer: [u32; MAX_BUFFER_SIZE],
    msg_size: usize,
}

static mut TCB_TABLE: [TCB; 32] = [TCB { dest_buffer: [0; 32], msg_size: 0 }; 32];

#[derive(Clone, Copy)]
struct MCB {
    receivers: u32,
    src_buffer: Buffer,
}

static mut MCB_TABLE: [MCB; 32] = [MCB {
    receivers: 0,
    src_buffer: &[],
}; 32];

static mut MsgSCB_TABLE: [SCB; 32] = [SCB { flags: 0, tasks: 0 }; 32];

pub fn broadcast(var: usize) {
    execute_critical(|_| {
        let mcb = unsafe { MCB_TABLE[var] };

        copy(&mcb.receivers, mcb.src_buffer);
        msg_signal_release(var, &mcb.receivers);
    })
}

fn copy (tasks_mask: &u32, src_msg: Buffer) {
    let tcb_table = unsafe { &mut TCB_TABLE };
    for tid in 1..32 {
        let tid_mask = (1<<tid);
        if tasks_mask & tid_mask == tid_mask {
            for i in 0..src_msg.len() {
                tcb_table[tid].dest_buffer[i] = src_msg[i];
            }
            tcb_table[tid].msg_size = src_msg.len();
        }
    }
}

pub fn receive<'a >(var: usize) -> Result<&'a [u32], ()> {
    execute_critical(|_| {
        let tcb_table = unsafe { &mut TCB_TABLE };
        let mcb_table = unsafe { &mut MCB_TABLE };
        let rt = get_RT();

        if (msg_test_reset(var)) {
            return Ok(&tcb_table[rt].dest_buffer[0..tcb_table[rt].msg_size]);
        }
        Err(())
    })
}

fn msg_signal_release(semaphore: usize, tasks_mask: &u32) {
    let scb_table = unsafe { &mut MsgSCB_TABLE };
    scb_table[semaphore].flags |= *tasks_mask;
    release(&scb_table[semaphore].tasks);
}

fn msg_test_reset(semaphore: usize) -> bool {
    let scb_table = unsafe { &mut MsgSCB_TABLE };
    let rt = get_RT() as u32;
    let rt_mask = (1 << rt);
    if scb_table[semaphore].flags & rt_mask == rt_mask {
        scb_table[semaphore].flags &= !rt_mask;
        return true;
    } else {
        return false;
    }
}

pub fn configure_msg(var: usize, tasks: &u32, receivers: &u32, src_msg: Buffer) {
    execute_critical(|_| {
        let mcb_table = unsafe { &mut MCB_TABLE };
        let scb_table = unsafe { &mut MsgSCB_TABLE };

        mcb_table[var].src_buffer = src_msg;
        scb_table[var].tasks |= *tasks;
        mcb_table[var].receivers |= *receivers;
    })
}