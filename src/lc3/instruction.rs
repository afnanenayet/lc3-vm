/// This module contains helper methods to implement various instructions
use crate::lc3::consts::{ConditionFlag, Op};
use std::{
    collections::HashMap,
    io::{stdin, Read},
};

/// Extend an immediate mode value to be 16 bits
///
/// 1's are filled in for negative values and 0's are filled in for positive values.
pub fn sign_extend(x: u16, bit_count: usize) -> u16 {
    let mut x = x;

    if (x >> (bit_count - 1)) & 1 != 0 {
        x |= 0xFFFF << bit_count
    }
    x
}

/// Get the condition flag given the value of the `COND` register
///
/// This method expects the value at the `COND` register, which it will turn into a `ConditionFlag`
/// enum that the program can understand.
pub fn get_cond_flag(value: u16) -> ConditionFlag {
    if value == 0 {
        ConditionFlag::ZRO
    } else if value >> 15 != 0 {
        ConditionFlag::NEG
    } else {
        ConditionFlag::POS
    }
}

/// Create a bit mask for the first n bits
///
/// This method creates a bit mask for a given number of bits, assuming that you want to mask for
/// the first `n` bits of some number. For example, masking 3 bits means you want the bit mask to
/// be 7 (111 in binary).
pub fn bit_mask(num_bits: u16) -> u16 {
    (0..num_bits).map(|x: u16| (2 as u16).pow(x.into())).sum()
}

/// Retrieve an argument in an instruction
///
/// This method gets a slice of bits given the starting position of an instruction and the length
/// of the bits. This is meant to be used to extract the arguments from an instruction.
pub fn get_arg(instruction: u16, start_pos: u16, length: u16) -> u16 {
    (instruction >> start_pos) & bit_mask(length)
}

/// Convert a number from big-endian to little-endian
///
/// This method converts a 16 bit integer to have a different endian character. This is only
/// necessary of platforms that are little endian.
fn to_le(x: u16) -> u16 {
    (x << 8) | (x >> 8)
}

/// Replicates the behavior of C's `getchar()`
///
/// This method will read one byte from STDIN
pub fn getchar() -> u8 {
    let mut buf = [0];
    stdin().read(&mut buf);
    buf[0]
}

/// Generate a function dispatch table for opcodes
///
/// This method generates a hashmap where the keys are opcode enums and the values are pointers to
/// functions that can modify the VM. This method will generate an entry for every opcode -
/// function pair.
macro_rules! op_dispatch_table {
    ( $( ($op:expr, $fn:expr) ),* ) => {
        {
            let mut table: HashMap<consts::Op, fn(&mut LC3, u16)> = HashMap::new();
            $( table.insert($op, $fn); )*
            table
        }
    };
}

/// Functions that implement opcodes for the LC3 vm
///
/// All of these methods have the same structure: they take a mutable reference to the LC3 VM state
/// and an instruction, which is a 16-bit integer.
pub mod op {
    use super::{bit_mask, get_arg, sign_extend};
    use crate::lc3::{consts::Register, LC3};

    pub fn add(vm: &mut LC3, instr: u16) {
        // destination register (DR)
        let r0 = (instr >> 9) & bit_mask(3);

        // first operand (SR1)
        let r1 = (instr >> 6) & bit_mask(3);

        // indicates whether the program is in immediate mode
        let imm_flag = (instr >> 5) & bit_mask(1);

        vm.registers[r0 as usize] = if imm_flag != 0 {
            let imm5 = sign_extend(instr & 0x5, 5);
            vm.registers[r1 as usize] + imm5
        } else {
            let r2 = instr & bit_mask(3);
            vm.registers[r1 as usize] + vm.registers[r2 as usize]
        };
        vm.update_cond_flag(r0);
    }

    pub fn ldi(vm: &mut LC3, instr: u16) {
        let r0 = (instr >> 9) & bit_mask(3);
        let pc_offset = sign_extend(instr & 0x1ff, 9);
        let r1 = vm.mem_read(vm.registers[Register::PC as usize] + pc_offset);
        vm.registers[r0 as usize] = vm.mem_read(r1);
        vm.update_cond_flag(r0);
    }

    pub fn and(vm: &mut LC3, instr: u16) {
        let r0 = (instr >> 9) & bit_mask(3);
        let r1 = (instr >> 6) & bit_mask(3);
        let imm_mode = (instr >> 5) & bit_mask(1) != 0;

        vm.registers[r0 as usize] = if imm_mode {
            let imm5 = sign_extend(instr & bit_mask(5), 5);
            vm.registers[r1 as usize] + imm5
        } else {
            let r2 = instr & bit_mask(3);
            vm.registers[r1 as usize] + vm.registers[r2 as usize]
        };
        vm.update_cond_flag(r0);
    }

    /// This operation is unused and will abort the VM
    pub fn rti(vm: &mut LC3, instr: u16) {
        // TODO: abort
    }

    /// This operation is unused and will abort the VM
    pub fn res(vm: &mut LC3, instr: u16) {
        // TODO: abort
    }

    pub fn not(vm: &mut LC3, instr: u16) {
        let r0 = get_arg(instr, 9, 3);
        let r1 = get_arg(instr, 6, 3);
        vm.registers[r0 as usize] = !vm.registers[r1 as usize];
        vm.update_cond_flag(r0);
    }

    pub fn br(vm: &mut LC3, instr: u16) {
        let pc_offset = sign_extend(get_arg(instr, 0, 9), 9);
        let cond_flag = get_arg(instr, 9, 3);
        if cond_flag & vm.registers[Register::COND as usize] != 1 {
            vm.registers[Register::PC as usize] += pc_offset;
        }
    }

    pub fn jmp(vm: &mut LC3, instr: u16) {
        let base_register = get_arg(instr, 6, 3);
        vm.registers[Register::PC as usize] = vm.registers[base_register as usize];
    }

    pub fn jsr(vm: &mut LC3, instr: u16) {
        let r1 = get_arg(instr, 6, 3);
        let long_flag = get_arg(instr, 11, 1);
        let long_pc_offset = sign_extend(get_arg(instr, 0, 11), 11);

        vm.registers[Register::PC as usize] = if long_flag != 0 {
            long_pc_offset
        } else {
            vm.registers[r1 as usize]
        };
    }

    pub fn ld(vm: &mut LC3, instr: u16) {
        let r0 = get_arg(instr, 9, 3);
        let pc_offset = sign_extend(get_arg(instr, 0, 9), 9);
        vm.registers[r0 as usize] = vm.mem_read(vm.registers[Register::PC as usize] + pc_offset);
        vm.update_cond_flag(r0);
    }

    pub fn ldr(vm: &mut LC3, instr: u16) {
        let r0 = get_arg(instr, 9, 3);
        let base_register = get_arg(instr, 6, 3);
        let offset = sign_extend(get_arg(instr, 0, 6), 6);
        vm.registers[r0 as usize] = vm.mem_read(vm.registers[base_register as usize] + offset);
        vm.update_cond_flag(r0);
    }

    pub fn lea(vm: &mut LC3, instr: u16) {
        let r0 = get_arg(instr, 9, 3);
        let pc_offset = get_arg(instr, 0, 9);
        vm.mem_write(
            vm.registers[Register::PC as usize] + pc_offset,
            vm.registers[r0 as usize],
        );
    }

    pub fn st(vm: &mut LC3, instr: u16) {
        let r0 = get_arg(instr, 9, 3);
        let pc_offset = sign_extend(get_arg(instr, 0, 9), 9);
        vm.mem_write(
            vm.registers[Register::PC as usize] + pc_offset,
            vm.registers[r0 as usize],
        );
    }

    pub fn sti(vm: &mut LC3, instr: u16) {
        let r0 = get_arg(instr, 9, 3);
        let pc_offset = sign_extend(get_arg(instr, 0, 9), 9);
        let dst = vm.mem_read(vm.registers[Register::PC as usize] + pc_offset);
        vm.mem_write(dst, vm.registers[r0 as usize]);
    }

    pub fn str(vm: &mut LC3, instr: u16) {
        let r0 = get_arg(instr, 9, 3);
        let r1 = get_arg(instr, 6, 3);
        let offset = sign_extend(get_arg(instr, 0, 6), 6);
        vm.mem_write(
            vm.registers[r1 as usize] + offset,
            vm.registers[r0 as usize],
        );
    }
}
