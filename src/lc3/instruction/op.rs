/// Functions that implement opcodes for the LC3 vm
///
/// All of these methods have the same structure: they take a mutable reference to the LC3 VM state
/// and an instruction, which is a 16-bit integer.
use super::{bit_mask, get_arg, sign_extend};
use crate::lc3::{
    consts::Register,
    instruction::trap,
    LC3,
};

pub fn add(vm: &mut LC3, instr: u16) {
    // destination register (DR)
    let r0 = (instr >> 9) & bit_mask(3);

    // first operand (SR1)
    let r1 = (instr >> 6) & bit_mask(3);

    // indicates whether the program is in immediate mode
    let imm_flag = (instr >> 5) & bit_mask(1);

    vm.registers[r0 as usize] = if imm_flag != 0 {
        let imm5 = sign_extend(instr & bit_mask(5), 5);
        vm.registers[r1 as usize].wrapping_add(imm5)
    } else {
        let r2 = instr & bit_mask(3);
        vm.registers[r1 as usize].wrapping_add(vm.registers[r2 as usize])
    };
    vm.update_cond_flag(r0);
}

pub fn ldi(vm: &mut LC3, instr: u16) {
    let r0 = (instr >> 9) & bit_mask(3);
    let pc_offset = sign_extend(instr & 0x1ff, 9);
    let r1 = vm.mem_read(vm.registers[Register::PC as usize].wrapping_add(pc_offset));
    vm.registers[r0 as usize] = vm.mem_read(r1);
    vm.update_cond_flag(r0);
}

pub fn and(vm: &mut LC3, instr: u16) {
    let r0 = (instr >> 9) & bit_mask(3);
    let r1 = (instr >> 6) & bit_mask(3);
    let imm_mode = (instr >> 5) & bit_mask(1) != 0;

    vm.registers[r0 as usize] = if imm_mode {
        let imm5 = sign_extend(instr & bit_mask(5), 5);
        vm.registers[r1 as usize] & imm5
    } else {
        let r2 = instr & bit_mask(3);
        vm.registers[r1 as usize] & vm.registers[r2 as usize]
    };
    vm.update_cond_flag(r0);
}

/// This operation is unused and will abort the VM
pub fn rti(vm: &mut LC3, _: u16) {
    println!("PANIC: illegal instruction (RTI)");
    vm.halt();
}

/// This operation is unused and will abort the VM
pub fn res(vm: &mut LC3, _: u16) {
    println!("PANIC: illegal instruction (RES)");
    vm.halt();
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
    if cond_flag & vm.registers[Register::COND as usize] != 0 {
        vm.registers[Register::PC as usize] = vm.registers[Register::PC as usize].wrapping_add(pc_offset);
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

    vm.registers[Register::R7 as usize] = vm.registers[Register::PC as usize];
    vm.registers[Register::PC as usize] = if long_flag != 0 {
        vm.registers[Register::PC as usize].wrapping_add(long_pc_offset)
    } else {
        vm.registers[r1 as usize]
    };
}

pub fn ld(vm: &mut LC3, instr: u16) {
    let r0 = get_arg(instr, 9, 3);
    let pc_offset = sign_extend(get_arg(instr, 0, 9), 9);
    vm.registers[r0 as usize] = vm.mem_read(vm.registers[Register::PC as usize].wrapping_add(pc_offset));
    vm.update_cond_flag(r0);
}

pub fn ldr(vm: &mut LC3, instr: u16) {
    let r0 = get_arg(instr, 9, 3);
    let base_register = get_arg(instr, 6, 3);
    let offset = sign_extend(get_arg(instr, 0, 6), 6);
    vm.registers[r0 as usize] = vm.mem_read(vm.registers[base_register as usize].wrapping_add(offset));
    vm.update_cond_flag(r0);
}

pub fn lea(vm: &mut LC3, instr: u16) {
    let r0 = get_arg(instr, 9, 3);
    let pc_offset = sign_extend(get_arg(instr, 0, 9), 9);
    vm.registers[r0 as usize] = vm.registers[Register::PC as usize].wrapping_add(pc_offset);
    vm.update_cond_flag(r0);
}

pub fn st(vm: &mut LC3, instr: u16) {
    let r0 = get_arg(instr, 9, 3);
    let pc_offset = sign_extend(get_arg(instr, 0, 9), 9);
    vm.mem_write(
        vm.registers[Register::PC as usize].wrapping_add(pc_offset),
        vm.registers[r0 as usize],
    );
}

pub fn sti(vm: &mut LC3, instr: u16) {
    let r0 = get_arg(instr, 9, 3);
    let pc_offset = sign_extend(get_arg(instr, 0, 9), 9);
    let dst = vm.mem_read(vm.registers[Register::PC as usize].wrapping_add(pc_offset));
    vm.mem_write(dst, vm.registers[r0 as usize]);
}

pub fn str(vm: &mut LC3, instr: u16) {
    let r0 = get_arg(instr, 9, 3);
    let r1 = get_arg(instr, 6, 3);
    let offset = sign_extend(get_arg(instr, 0, 6), 6);
    vm.mem_write(
        vm.registers[r1 as usize].wrapping_add(offset),
        vm.registers[r0 as usize],
    );
}

/// Dispatch table mapping opcode numbers (0-15) to handler functions.
pub const OP_TABLE: [fn(&mut LC3, u16); 16] = [
    br,   // 0  BR
    add,  // 1  ADD
    ld,   // 2  LD
    st,   // 3  ST
    jsr,  // 4  JSR
    and,  // 5  AND
    ldr,  // 6  LDR
    str,  // 7  STR
    rti,  // 8  RTI
    not,  // 9  NOT
    ldi,  // 10 LDI
    sti,  // 11 STI
    jmp,  // 12 JMP
    res,  // 13 RES
    lea,  // 14 LEA
    trap, // 15 TRAP
];

/// This routine is dispatched when a trap code is encountered.
///
/// This method will extract the trap code from the instruction and call the appropriate
/// corresponding function.
pub fn trap(vm: &mut LC3, instr: u16) {
    vm.trap = true;
    let raw_trap_code = (instr & 0xFF) as usize;
    let index = raw_trap_code.wrapping_sub(0x20);
    if index < trap::TRAP_TABLE.len() {
        trap::TRAP_TABLE[index](vm);
        vm.trap = false;
    } else {
        println!("PANIC: Unknown or malformed trap code encountered");
        vm.halt();
    }
}
