/// The lc3 module contains the routines for operating the LC3 virtual machine, as well as
/// information and structs that pertain to the LC3 or abstractions that are useful for
/// implementing the VM.
///
/// This module contains the struct representing the machine's state as well as methods
/// for actually running the program.
mod consts;

#[macro_use]
mod instruction;

use consts::{MemoryMappedRegister, Op, Register};
use instruction::{bit_mask, get_arg, getchar, sign_extend};
use itertools::Itertools;
use std::{
    fs::File,
    io::{self, Read},
};

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

    /// State flag representing whether or not the machine is currently running
    running: bool,
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
            running: false,
        }
    }

    /// Execute the VM
    ///
    /// This will start a run-loop that processes instructions until the stop instruction is
    /// encountered.
    pub fn run_loop(&mut self) -> u32 {
        // Our circuit for determining whether execution should be terminated
        self.running = true;

        let mut op = consts::Op::LD;
        // let mut instr = mem_read(reg[Register::PC]++);
        // let mut op = instr >> 12;

        // fetch instruction here TODO: implement mem_read
        let instr = 0;
        while self.running {
            match op {
                Op::BR => self.op_br(instr),
                Op::LD => self.op_ld(instr),
                Op::ADD => self.op_add(instr),
                Op::LD => self.op_ld(instr),
                Op::ST => self.op_st(instr),
                Op::JSR => self.op_jsr(instr),
                Op::AND => self.op_and(instr),
                Op::LDR => self.op_ldr(instr),
                Op::STR => self.op_str(instr),
                Op::RTI => self.op_rti(instr),
                Op::NOT => self.op_not(instr),
                Op::LDI => self.op_ldi(instr),
                Op::STI => self.op_sti(instr),
                Op::JMP => self.op_jmp(instr),
                Op::RES => self.op_res(instr),
                Op::LEA => self.op_lea(instr),
                _ => panic!("Unsupported opcode encountered. Aborting."),
                //Op::TRAP => self.op_trap(instr), // TODO
            }
        }
        unimplemented!();
    }

    /// Read a VM image and load it into memory
    ///
    /// This will read an LC3 image and load it into memory with the specified origin offset.
    fn read_image_file(&mut self, filename: &str) -> io::Result<()> {
        let mut f = File::open(filename)?;
        let mut buf = Vec::<u8>::with_capacity(consts::MEMORY_LIMIT);
        let origin_bytes = f.read_to_end(&mut buf);

        // Rust reads one byte (8 bits) at a time. We will have to account for this and combine two
        // 8-bit integers to one 16-bit integers

        // The "origin" defines the initial offset for where memory should be loaded from the image
        let origin = (buf[0] << 8) | buf[1];
        let mut mem_idx = origin as usize;

        // Take two bytes at a time and reverse the endian-ness, placing the final 16-bit integer
        // into a memory location
        for mut chunk in &buf.into_iter().skip(2).chunks(2) {
            let p: u16 =
                (chunk.next().unwrap_or(0) as u16) << 8 | (chunk.next().unwrap_or(0) as u16);
            self.memory[mem_idx] = p;
            mem_idx += 1;
        }
        Ok(())
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

    /// Write a value to some memory location
    ///
    /// This will write a value to the VM's memory bank given the value and the pointer address.
    fn mem_write(&mut self, addr: u16, val: u16) {
        self.memory[addr as usize] = val;
    }

    /// Returns the value at a particular memory address
    ///
    /// This also has support for memory mapped registers, such as for the keyboard.
    fn mem_read(&mut self, addr: u16) -> u16 {
        if addr == MemoryMappedRegister::KBSR as u16 {
            if false {
                // TODO implement `check_key`
                self.memory[MemoryMappedRegister::KBSR as usize] = 1 << 15;
                self.memory[MemoryMappedRegister::KBDR as usize] = getchar().into();
            } else {
                self.memory[MemoryMappedRegister::KBSR as usize] = 0;
            }
        }
        self.memory[addr as usize]
    }

    /****** opcode implementations ******/

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
        self.registers[r0 as usize] =
            self.mem_read(self.registers[Register::PC as usize] + pc_offset);
        self.update_cond_flag(r0);
    }

    fn op_ldr(&mut self, instr: u16) {
        let r0 = get_arg(instr, 9, 3);
        let base_register = get_arg(instr, 6, 3);
        let offset = sign_extend(get_arg(instr, 0, 6), 6);
        self.registers[r0 as usize] =
            self.mem_read(self.registers[base_register as usize] + offset);
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
        let dst = self.mem_read(self.registers[Register::PC as usize] + pc_offset);
        self.mem_write(dst, self.registers[r0 as usize]);
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

    /****** trap code implementations ******/

    fn trap_puts(&mut self, instr: u16) {
        // build up a string by looking for 16 bit integers until we hit the null terminator. The
        // starting location of the string is whatever is at the r0 register
        let start_pos = self.registers[Register::R0 as usize] as usize;
        let mut end_pos = self
            .memory
            .iter()
            .skip(start_pos)
            .position(|x| *x == 0)
            .unwrap();
        print!(
            "{}",
            String::from_utf16_lossy(&self.memory[start_pos..end_pos])
        );
    }

    fn trap_getc(&mut self) {
        // Get the next character from stdin and convert it to a 16 bit integer so we can store it
        // in the R0 register
        let c = io::stdin()
            .bytes()
            .next()
            .and_then(|result| result.ok())
            .map(|byte| byte as u16)
            .unwrap_or(0);
        self.registers[Register::R0 as usize] = c;
    }

    fn trap_out(&mut self) {
        let r0 = self.registers[Register::R0 as usize];
        let character = String::from_utf16_lossy(&[r0]);
        print!("{}", character);
    }

    fn trap_in(&mut self) {
        print!("Enter a character: ");
        let raw_c = io::stdin()
            .bytes()
            .next()
            .and_then(|result| result.ok())
            .map(|byte| byte as u16)
            .unwrap_or(0);
        let character = String::from_utf16_lossy(&[raw_c]);
        println!("{}", character);
        self.registers[Register::R0 as usize] = raw_c;
    }

    fn trap_putsp(&mut self) {
        let start_pos = self.registers[Register::R0 as usize] as usize;
        let mut end_pos = self
            .memory
            .iter()
            .skip(start_pos)
            .position(|x| *x == 0)
            .unwrap();

        for &c in &self.registers[start_pos..end_pos] {
            let char1 = vec![(c & 0xFF) as u8];
            let s = String::from_utf8_lossy(char1.as_slice());
            print!("{}", s);
            let char2 = vec![(c >> 8) as u8];
            let s = String::from_utf8_lossy(char2.as_slice());
            print!("{}", s);
        }
    }

    fn trap_halt(&mut self) {
        print!("HALT");
        self.running = false;
    }
}
