/// The lc3 module contains the routines for operating the LC3 virtual machine, as well as
/// information and structs that pertain to the LC3 or abstractions that are useful for
/// implementing the VM.
///
/// This module contains the struct representing the machine's state as well as methods
/// for actually running the program.
pub mod consts;

mod instruction;

use consts::{MemoryMappedRegister, Op, Register, Trap};
use instruction::getchar;
use instruction::op::OP_TABLE;
use itertools::Itertools;
use log::{debug, info};
use num_traits::FromPrimitive;
use std::{
    fs::File,
    io::{self, Read},
    path::PathBuf,
};

/// The data pertaining to the state of the LC3 VM
pub struct LC3 {
    /// The memory locations available to the virtual machine.
    ///
    /// The memory addresses are bounded by the limit for the unsigned 16 bit integer, which is
    /// 65536. There are `U16_MAX` addressable locations in memory.
    pub memory: Box<[u16; consts::MEMORY_LIMIT]>,

    /// The available registers in the VM. The registers are defined in the `Register`
    /// enum in `lc3::consts`.
    pub registers: [u16; consts::REGISTER_COUNT],

    /// State flag representing whether or not the machine is currently running
    running: bool,

    /// Whether the VM is currently executing a trap code
    trap: bool,
}

impl LC3 {
    /// Create a new LC3 virtual machine state
    pub fn new() -> Self {
        // Allocate memory on the heap via Vec to avoid stack overflow,
        // then convert to a boxed array.
        let memory: Box<[u16; consts::MEMORY_LIMIT]> = {
            let v = vec![0u16; consts::MEMORY_LIMIT];
            let boxed_slice = v.into_boxed_slice();
            // SAFETY: boxed_slice has exactly MEMORY_LIMIT elements, matching the array size.
            unsafe { Box::from_raw(Box::into_raw(boxed_slice) as *mut [u16; consts::MEMORY_LIMIT]) }
        };
        let mut lc3 = Self {
            memory,
            registers: [0u16; consts::REGISTER_COUNT],
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
    pub fn run_loop(&mut self) {
        self.running = true;
        while self.is_running() {
            self.step();
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
    pub fn step(&mut self) {
        let instr = self.mem_read(self.registers[Register::PC as usize]);
        self.registers[Register::PC as usize] = self.registers[Register::PC as usize].wrapping_add(1);
        let op_index = (instr >> 12) as usize;
        info!("read op {} ({}) at PC", op_index, instr);
        OP_TABLE[op_index](self, instr);
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
            let p: u16 = u16::from(chunk.next().unwrap_or_default()) << 8
                | u16::from(chunk.next().unwrap_or_default());
            self.memory[mem_idx] = p;
            mem_idx += 1;
        }
        Ok(())
    }

    /// Halt the VM
    pub fn halt(&mut self) {
        self.running = false;
    }

    /// Check if the VM is currently running
    pub fn is_running(&self) -> bool {
        self.running
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
