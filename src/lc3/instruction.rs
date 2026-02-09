/// This module contains helper methods to implement various instructions
use crate::lc3::consts::ConditionFlag;
use std::io::{stdin, Read};

pub mod op;
pub mod trap;

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
    if num_bits >= 16 {
        0xFFFF
    } else {
        (1u16 << num_bits) - 1
    }
}

/// Retrieve an argument in an instruction
///
/// This method gets a slice of bits given the starting position of an instruction and the length
/// of the bits. This is meant to be used to extract the arguments from an instruction.
pub fn get_arg(instruction: u16, start_pos: u16, length: u16) -> u16 {
    (instruction >> start_pos) & bit_mask(length)
}

/// Replicates the behavior of C's `getchar()`
///
/// This method will read one byte from STDIN
pub fn getchar() -> u8 {
    let mut buf = [0];
    stdin().read(&mut buf).unwrap_or_default();
    buf[0]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sign_extend_positive() {
        // 5-bit value 0b00101 = 5, top bit is 0 → stays 5
        assert_eq!(sign_extend(0b00101, 5), 5);
    }

    #[test]
    fn sign_extend_negative() {
        // 5-bit value 0b10000 = 16, top bit is 1 → sign-extend to 0xFFF0
        assert_eq!(sign_extend(0b10000, 5), 0xFFF0);
    }

    #[test]
    fn sign_extend_zero() {
        assert_eq!(sign_extend(0, 5), 0);
    }

    #[test]
    fn sign_extend_all_ones_5bit() {
        // 5-bit all ones = 0x1F → sign-extend = 0xFFFF
        assert_eq!(sign_extend(0x1F, 5), 0xFFFF);
    }

    #[test]
    fn sign_extend_9bit_positive() {
        // 9-bit value 0xFF (255), top bit (bit 8) is 0 → stays 0xFF
        assert_eq!(sign_extend(0xFF, 9), 0xFF);
    }

    #[test]
    fn sign_extend_9bit_negative() {
        // 9-bit value 0x1FF (all ones), top bit is 1 → 0xFFFF
        assert_eq!(sign_extend(0x1FF, 9), 0xFFFF);
    }

    #[test]
    fn bit_mask_1() {
        assert_eq!(bit_mask(1), 0b1);
    }

    #[test]
    fn bit_mask_3() {
        assert_eq!(bit_mask(3), 0b111);
    }

    #[test]
    fn bit_mask_5() {
        assert_eq!(bit_mask(5), 0b11111);
    }

    #[test]
    fn bit_mask_16() {
        assert_eq!(bit_mask(16), 0xFFFF);
    }

    #[test]
    fn get_arg_dr_field() {
        // instruction: DR=5 at bits [11:9]
        // 5 << 9 = 0x0A00
        let instr: u16 = 0b101 << 9;
        assert_eq!(get_arg(instr, 9, 3), 5);
    }

    #[test]
    fn get_arg_sr1_field() {
        // SR1=3 at bits [8:6]
        let instr: u16 = 0b011 << 6;
        assert_eq!(get_arg(instr, 6, 3), 3);
    }

    #[test]
    fn get_arg_imm_flag() {
        // imm flag at bit 5
        let instr: u16 = 1 << 5;
        assert_eq!(get_arg(instr, 5, 1), 1);
        assert_eq!(get_arg(0u16, 5, 1), 0);
    }

    #[test]
    fn get_cond_flag_zero() {
        assert_eq!(get_cond_flag(0), ConditionFlag::ZRO);
    }

    #[test]
    fn get_cond_flag_positive() {
        assert_eq!(get_cond_flag(1), ConditionFlag::POS);
        assert_eq!(get_cond_flag(0x7FFF), ConditionFlag::POS);
    }

    #[test]
    fn get_cond_flag_negative() {
        assert_eq!(get_cond_flag(0x8000), ConditionFlag::NEG);
        assert_eq!(get_cond_flag(0xFFFF), ConditionFlag::NEG);
    }
}
