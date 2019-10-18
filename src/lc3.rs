/// The lc3 module contains the routines for operating the LC3 virtual machine, as well as
/// information and structs that pertain to the LC3 or abstractions that are useful for
/// implementing the VM.
///
/// This module contains the struct representing the machine's state as well as methods
/// for actually running the program.
pub mod consts;

#[macro_use]
mod instruction;

use consts::{MemoryMappedRegister, Op, OpDispatchTable, Register, Trap};
use instruction::getchar;
use itertools::Itertools;
use log::{debug, info};
use num_traits::FromPrimitive;
use std::{
    collections::HashMap,
    fs::File,
    io::{self, Read},
    path::PathBuf,
};

/// The lookup tables for the VM
///
/// We store the lookup tables here so we don't have to keep re-initializing them in the function
pub struct DispatchTables {
    /// The dispatch table for various opcodes
    pub opcodes: OpDispatchTable,
}

impl DispatchTables {
    /// Create a new set of dispatch tables
    ///
    /// This iniitalizes the struct with properly defined dispatch tables
    pub fn new() -> Self {
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
        Self {
            opcodes: op_dispatch_table,
        }
    }
}

/// The data pertaining to the state of the LC3 VM
#[derive(Clone, Debug)]
pub struct LC3 {
    /// A vector representing the memory locations available to the virtual machine.
    ///
    /// The memory addresses are bounded by the limit for the unsigned 16 bit integer, which is
    /// 65536. There are `U16_MAX` addressable locations in memory.
    pub memory: Vec<u16>,

    /// A vector of the available registers in the VM. The registers are defined in the `Register`
    /// enum in `lc3::consts`.
    pub registers: Vec<u16>,

    /// State flag representing whether or not the machine is currently running
    running: bool,

    /// Whether the VM is currently executing a trap code
    trap: bool,
}

impl LC3 {
    /// Create a new LC3 virtual machine state
    ///
    /// This initializes the virtual register and memory vectors. We don't use arrays because Rust
    /// currently has very poor support for them. These should be switched to arrays once const
    /// generics are stabilized. You can play around with them in nightly builds.
    pub fn new() -> Self {
        let mut lc3 = Self {
            memory: vec![0; consts::MEMORY_LIMIT],
            registers: vec![0; consts::Register::COUNT as usize],
            running: false,
            trap: false,
        };
        lc3.registers[Register::PC as usize] = consts::PC_START;
        lc3
    }

    /// Execute the VM
    ///
    /// This will start a run-loop that processes instructions until the stop instruction is
    /// encountered.
    pub fn run_loop(&mut self, tables: &DispatchTables) {
        self.running = true;
        while self.running {
            self.step(tables);
        }
    }

    /// Get the next opcode for execution
    ///
    /// This method will find the register index pointed to by the program counter, which returns a
    /// memory address. This method parses the value at the memory address to figure out the next
    /// operation.
    ///
    /// This will panic if the register is invalid
    pub fn parse_next_op(&self) -> consts::Operation {
        let register_index = self.registers[Register::PC as usize];
        let raw_op = self.memory[register_index as usize];
        let op = FromPrimitive::from_u16(raw_op >> 12).unwrap();

        // If the opcode points to a trapcode, then display the trapcode
        if op == Op::TRAP {
            let trap = FromPrimitive::from_u16(raw_op & 0xFF).unwrap_or(Trap::HALT);
            return consts::Operation::Trap(trap);
        }
        consts::Operation::Op(op)
    }

    /// Read an instruction from the register pointed to by the program counter and execute it
    ///
    /// This is one step of execution in the VM. The VM should continuously run steps in a loop,
    /// though it is split out into a function for easy debugging. This method will read the
    /// instruction, dispatch the appropriate function, and increment the program counter.
    pub fn step(&mut self, tables: &DispatchTables) {
        let op_dispatch_table = &tables.opcodes;
        let instr = self.mem_read(self.registers[Register::PC as usize]);
        self.registers[Register::PC as usize] += 1;
        if let Some(op) = FromPrimitive::from_u16(instr >> 12) {
            info!("read op {:?} ({}) at PC", op, instr);
            let op_fn = op_dispatch_table[&op];
            op_fn(self, instr);
        } else {
            self.running = false;
            println!("PANIC: Unknown opcode encountered");
        }
    }

    /// Read a VM image and load it into memory
    ///
    /// This will read an LC3 image and load it into memory with the specified origin offset.
    pub fn read_image_file(&mut self, filename: &PathBuf) -> io::Result<()> {
        let mut f = File::open(filename)?;

        // The memory limit defines how many 16-bit memory pointers we can have, so we multiply the
        // memory limit by two because we read 8-bit integers.
        let mut buf = Vec::<u8>::with_capacity(consts::MEMORY_LIMIT * 2);
        let read_bytes = f.read_to_end(&mut buf)?;
        debug!("Read {} bytes from the provided image", read_bytes);

        // Rust reads one byte (8 bits) at a time. We will have to account for this and combine two
        // 8-bit integers to one 16-bit integers

        // The "origin" defines the initial offset for where memory should be loaded from the image
        let origin = (u16::from(buf[0]) << 8) | u16::from(buf[1]);
        debug!("Image origin offset: {}", origin);
        let mut mem_idx = origin as usize;

        // Take two bytes at a time and reverse the endian-ness, placing the final 16-bit integer
        // into a memory location
        for mut chunk in &buf.into_iter().skip(2).chunks(2) {
            // Reverse the endian-ness of the incoming 16-bit instruction
            let p: u16 = u16::from(chunk.next().unwrap_or_default())
                | u16::from(chunk.next().unwrap_or_default());
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
