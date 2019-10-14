/// The lc3 module contains the routines for operating the LC3 virtual machine, as well as
/// information and structs that pertain to the LC3 or abstractions that are useful for
/// implementing the VM.
///
/// This module contains the struct representing the machine's state as well as methods
/// for actually running the program.
mod consts;
mod instruction;

use consts::Register;
use instruction::{bit_mask, get_arg, get_cond_flag, sign_extend};

/// The data pertaining to the state of the LC3 VM
#[derive(Clone, Debug)]
pub struct LC3 {
    /// A vector representing the memory locations available to the virtual machine.
    ///
    /// The memory addresses are bounded by the limit for the unsigned 16 bit integer, which is
    /// 65536. There are `U16_MAX` addressable locations in memory.
    memory: Vec<u16>,

    /// A vector of the available registers in the VM. The registers are defined in the `Register`
    /// enum in `lc3::consts`.
    registers: Vec<u16>,
}

impl LC3 {
    /// Create a new LC3 virtual machine state
    ///
    /// This initializes the virtual register and memory vectors. We don't use arrays because Rust
    /// currently has very poor support for them. These should be switched to arrays once const
    /// generics are stabilized. You can play around with them in nightly builds.
    pub fn new() -> Self {
        Self {
            memory: Vec::with_capacity(consts::MEMORY_LIMIT),
            registers: Vec::with_capacity(consts::Register::COUNT as usize),
        }
    }

    /// Execute the VM
    ///
    /// This will start a run-loop that processes instructions until the stop instruction is
    /// encountered.
    pub fn run_loop(&mut self) -> u32 {
        // Our circuit for determining whether execution should be terminated
        let mut running = true;

        let mut op = consts::Op::LD;
        // let mut instr = mem_read(reg[Register::PC]++);
        // let mut op = instr >> 12;

        // fetch instruction here TODO: implement mem_read
        while running {
            //match op {
            //// TODO match each op here
            //};
        }
        unimplemented!();
    }

    /// Update the condition flag
    ///
    /// This method must be used any time a value is written to a register. It will find the
    /// condition used at the updated register and update the condition register to reflect that
    /// value.
    fn update_cond_flag(&mut self, r: u16) {
        // The raw bit value of the condition register
        let raw_cond = self.registers[r as usize];
        // The converted condition value
        let cond_flag = instruction::get_cond_flag(raw_cond);
        self.registers[Register::COND as usize] = cond_flag as u16;
    }

    fn op_add(&mut self, instr: u16) {
        // destination register (DR)
        let r0 = (instr >> 9) & bit_mask(3);

        // first operand (SR1)
        let r1 = (instr >> 6) & bit_mask(3);

        // indicates whether the program is in immediate mode
        let imm_flag = (instr >> 5) & bit_mask(1);

        self.registers[r0 as usize] = if imm_flag != 0 {
            let imm5 = sign_extend(instr & 0x5, 5);
            self.registers[r1 as usize] + imm5
        } else {
            let r2 = instr & bit_mask(3);
            self.registers[r1 as usize] + self.registers[r2 as usize]
        };
        self.update_cond_flag(r0);
    }

    fn op_ldi(&mut self, instr: u16) {
        let r0 = (instr >> 9) & bit_mask(3);
        let pc_offset = sign_extend(instr & 0x1ff, 9);
        //self.registers[r0 as usize] =
        //mem_read(mem_read(self.registers[Register::PC as usize] + pc_offset));
        self.update_cond_flag(r0);
    }

    fn op_and(&mut self, instr: u16) {
        let r0 = (instr >> 9) & bit_mask(3);
        let r1 = (instr >> 6) & bit_mask(3);
        let imm_mode = (instr >> 5) & bit_mask(1) != 0;

        self.registers[r0 as usize] = if imm_mode {
            let imm5 = sign_extend(instr & bit_mask(5), 5);
            self.registers[r1 as usize] + imm5
        } else {
            let r2 = instr & bit_mask(3);
            self.registers[r1 as usize] + self.registers[r2 as usize]
        };
        self.update_cond_flag(r0);
    }

    /// This operation is unused
    fn op_rti(&mut self, instr: u16) {
        // TODO: abort
    }

    /// This operation is unused
    fn op_res(&mut self, instr: u16) {
        // TODO: abort
    }

    fn op_not(&mut self, instr: u16) {
        let r0 = get_arg(instr, 9, 3);
        let r1 = get_arg(instr, 6, 3);
        self.registers[r0 as usize] = !self.registers[r1 as usize];
        self.update_cond_flag(r0);
    }

    fn op_br(&mut self, instr: u16) {
        let pc_offset = sign_extend(get_arg(instr, 0, 9), 9);
        let cond_flag = get_arg(instr, 9, 3);
        if cond_flag & self.registers[Register::COND as usize] != 1 {
            self.registers[Register::PC as usize] += pc_offset;
        }
    }

    fn op_jmp(&mut self, instr: u16) {
        let base_register = get_arg(instr, 6, 3);
        self.registers[Register::PC as usize] = self.registers[base_register as usize];
    }

    fn op_jsr(&mut self, instr: u16) {
        let r1 = get_arg(instr, 6, 3);
        let long_flag = get_arg(instr, 11, 1);
        let long_pc_offset = sign_extend(get_arg(instr, 0, 11), 11);

        self.registers[Register::PC as usize] = if long_flag != 0 {
            long_pc_offset
        } else {
            self.registers[r1 as usize]
        };
    }

    fn op_ld(&mut self, instr: u16) {
        let r0 = get_arg(instr, 9, 3);
        let pc_offset = sign_extend(get_arg(instr, 0, 9), 9);
        self.registers[r0 as usize] = mem_read(self.registers[Register::PC] + pc_offset);
        self.update_cond_flag(r0);
    }

    fn op_ldr(&mut self, instr: u16) {
        let r0 = get_arg(instr, 9, 3);
        let base_register = get_arg(instr, 6, 3);
        let offset = sign_extend(get_arg(instr, 0, 6), 6);
        self.registers[r0 as usize] = mem_read(self.registers[r1 as usize] + offset);
        self.update_cond_flag(r0);
    }

    fn op_lea(&mut self, instr: u16) {
        let r0 = get_arg(instr, 9, 3);
        let pc_offset = get_arg(instr, 0, 9);
        self.mem_write(
            self.registers[Register::PC as usize] + pc_offset,
            self.registers[r0 as usize],
        );
    }

    fn op_st(&mut self, instr: u16) {
        let r0 = get_arg(instr, 9, 3);
        let pc_offset = sign_extend(get_arg(instr, 0, 9), 9);
        self.mem_write(
            self.registers[Register::PC as usize] + pc_offset,
            self.registers[r0 as usize],
        );
    }

    fn op_sti(&mut self, instr: u16) {
        let r0 = get_arg(instr, 9, 3);
        let pc_offset = sign_extend(get_arg(instr, 0, 9), 9);
        self.mem_write(
            mem_read(self.registers[Register::PC as usize] + pc_offset),
            self.registers[r0 as usize],
        );
    }

    fn op_str(&mut self, instr: u16) {
        let r0 = get_arg(instr, 9, 3);
        let r1 = get_arg(instr, 6, 3);
        let offset = sign_extend(get_arg(instr, 0, 6), 6);
        self.mem_write(
            self.registers[r1 as usize] + offset,
            self.registers[r0 as usize],
        );
    }
}
