/// Constant values that pertain to the LC3 virtual machine

/// An enum representing the different types of registers
///
/// The different opcodes are commented if they have special functionality. The R[N] registers are
/// normal registers that can store memory.
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub enum Register {
    R0 = 0,
    R1,
    R2,
    R3,
    R4,
    R5,
    R6,
    R7,

    /// Program counter
    PC,

    /// The condition register
    ///
    /// The condition register contains the `ConditionFlag` from the last operation.
    COND,

    /// The total number of registers as defined by the enum
    COUNT,
}

/// The available opcodes for the LC3 VM
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub enum Op {
    /// Branch
    BR = 0,

    /// Add
    ADD,

    /// Load
    LD,

    /// Store
    ST,

    /// Jump register
    JSR,

    /// Bitwise and
    AND,

    /// Load register
    LDR,

    /// Store register
    STR,

    /// This opcode is unused
    RTI,

    /// Bitwise not
    NOT,

    /// Load indirect
    LDI,

    /// Store indirect
    STI,

    /// Jump
    JMP,

    /// Reserve (this opcode is unused)
    RES,

    /// Load effective address
    LEA,

    /// Execute trap
    TRAP,
}

/// The trap routines available for LC3
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub enum Trap {
    /// Get a character from the keyboard (not echoed onto the terminal)
    GETC = 0x20,

    /// Output a character
    OUT = 0x21,

    /// Output a word string
    PUTS = 0x22,

    /// Get a character from the keyboard and echo it to the terminal
    IN = 0x23,

    /// Output a byte string
    PUTSP = 0x24,

    /// Halt the program
    HALT = 0x25,
}

/// These are the different condition flags that can be used for an operation. These flags allow a
/// program to check logical conditions.
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub enum ConditionFlag {
    /// Positive: P
    POS = 1 << 0,

    /// Zero: Z
    ZRO = 1 << 1,

    /// Negative: N
    NEG = 1 << 2,
}

/// The addresses of the available memory mapped registers
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub enum MemoryMappedRegister {
    /// Keyboard status
    KBSR = 0xFE00,

    /// Keyboard data
    KBDR = 0xFE02,
}

/// The number of pointers that can be addressed. The LC3 virtual machine has 16-bit pointers, so
/// the maximum addressable value is the max value of an unsigned 16 bit integer. 2 ^ 16 = 65536.
pub const MEMORY_LIMIT: usize = std::u16::MAX as usize;

/// The default start position for the program counter
pub const PC_START: u16 = 0x3000;
