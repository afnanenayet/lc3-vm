use crate::lc3::{consts::Register, LC3};
use std::io::{self, Read};

/// Implementations of the trap routines in the LC3 architecture.
///
/// Every method has the same type: `fn(&mut LC3)`, which makes it easy to create function dispatch
/// tables for trap codes.

pub fn puts(vm: &mut LC3) {
    // build up a string by looking for 16 bit integers until we hit the null terminator. The
    // starting location of the string is whatever is at the r0 register
    let start_pos = vm.registers[Register::R0 as usize] as usize;
    let end_pos = vm
        .memory
        .iter()
        .skip(start_pos)
        .position(|x| *x == 0)
        .unwrap();
    print!(
        "{}",
        String::from_utf16_lossy(&vm.memory[start_pos..end_pos])
    );
}

pub fn getc(vm: &mut LC3) {
    // Get the next character from stdin and convert it to a 16 bit integer so we can store it
    // in the R0 register
    let c = io::stdin()
        .bytes()
        .next()
        .and_then(|result| result.ok())
        .map(u16::from)
        .unwrap_or(0);
    vm.registers[Register::R0 as usize] = c;
}

pub fn out(vm: &mut LC3) {
    let r0 = vm.registers[Register::R0 as usize];
    let character = String::from_utf16_lossy(&[r0]);
    print!("{}", character);
}

pub fn r#in(vm: &mut LC3) {
    print!("Enter a character: ");
    let raw_c = io::stdin()
        .bytes()
        .next()
        // We use the `and_then` call because there is no native cast from a u8 to a u16, so we
        // have to promote the type from a `Result` type and do a primitive cast from u8 ->
        // u16.
        .and_then(|result| result.ok())
        .map(u16::from)
        .unwrap_or(0);
    let character = String::from_utf16_lossy(&[raw_c]);
    println!("{}", character);
    vm.registers[Register::R0 as usize] = raw_c;
}

pub fn putsp(vm: &mut LC3) {
    let start_pos = vm.registers[Register::R0 as usize] as usize;
    let end_pos = vm
        .memory
        .iter()
        .skip(start_pos)
        .position(|x| *x == 0)
        .unwrap();

    for &c in &vm.registers[start_pos..end_pos] {
        let char1 = vec![(c & 0xFF) as u8];
        let s = String::from_utf8_lossy(char1.as_slice());
        print!("{}", s);
        let char2 = vec![(c >> 8) as u8];
        let s = String::from_utf8_lossy(char2.as_slice());
        print!("{}", s);
    }
}

pub fn halt(vm: &mut LC3) {
    print!("HALT");
    vm.running = false;
}
