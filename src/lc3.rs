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
use instruction::getchar;
use itertools::Itertools;
use std::{
    collections::HashMap,
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
        let op_dispatch_table = op_dispatch_table![
            (Op::BR, instruction::op::br),
            (Op::LD, instruction::op::ld),
            (Op::ADD, instruction::op::add),
            (Op::LD, instruction::op::ld),
            (Op::ST, instruction::op::st),
            (Op::JSR, instruction::op::jsr),
            (Op::AND, instruction::op::and),
            (Op::LDR, instruction::op::ldr),
            (Op::STR, instruction::op::str),
            (Op::RTI, instruction::op::rti),
            (Op::NOT, instruction::op::not),
            (Op::LDI, instruction::op::ldi),
            (Op::STI, instruction::op::sti),
            (Op::JMP, instruction::op::jmp),
            (Op::RES, instruction::op::res),
            (Op::LEA, instruction::op::lea),
            (Op::TRAP, instruction::op::trap)
        ];
        while self.running {
            // Find the method that corresponds to the operation in the dispatch table. If the
            // opcode is not in the dispatch table then the VM will panic and quit. If the opcode
            // is fine, then we can invoke the function and modify the VM's state as necessary.
            // TODO: better/more constructive error handling
            let op_fn = op_dispatch_table
                .get(&op)
                .or_else(|| panic!("Aborting: unsupported or malformed OPCODE was supplied"))
                .unwrap();
            op_fn(self, instr);
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
        let origin = ((buf[0] as u16) << 8) | (buf[1] as u16);
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
    pub fn mem_write(&mut self, addr: u16, val: u16) {
        self.memory[addr as usize] = val;
    }

    /// Returns the value at a particular memory address
    ///
    /// This also has support for memory mapped registers, such as for the keyboard.
    pub fn mem_read(&mut self, addr: u16) -> u16 {
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
}
