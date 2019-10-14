/// Constant values that pertain to the LC3 virtual machine

/// An enum representing the different types of registers
#[derive(Copy, Clone, Debug)]
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
    COND,

    /// The total number of registers as defined by the enum
    COUNT,
}

/// The available opcodes for the LC3 VM
#[derive(Copy, Clone, Debug)]
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

/// These are the different condition flags that can be used for an operation. These flags allow a
/// program to check logical conditions.
pub enum ConditionFlag {
    /// Positive: P
    POS = 1 << 0,

    /// Zero: Z
    ZRO = 1 << 1,

    /// Negative: N
    NEG = 1 << 2,
}

/// The number of pointers that can be addressed. The LC3 virtual machine has 16-bit pointers, so
/// the maximum addressable value is the max value of an unsigned 16 bit integer. 2 ^ 16 = 65536.
pub const MEMORY_LIMIT: usize = std::u16::MAX as usize;

/// The default start position for the program counter
pub const PC_START: u16 = 0x3000;
