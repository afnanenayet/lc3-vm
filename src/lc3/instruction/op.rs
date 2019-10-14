/// Functions that implement opcodes for the LC3 vm
///
/// All of these methods have the same structure: they take a mutable reference to the LC3 VM state
/// and an instruction, which is a 16-bit integer.
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
