/// This module contains helper methods to implement various instructions
use crate::lc3::consts::{ConditionFlag, Op};
use std::{
    collections::HashMap,
    io::{stdin, Read},
};

pub mod op;

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

