use std::fmt::Display;

use num::BigInt;

#[derive(Debug)]
pub enum TokenType {
    Exit,

    Print,
    PrintNum,
    GetNextIn,

    CreateFile { arg: String },
    CreateFileStream { arg: String },
    OpenFileStream { arg: String },
    ReadFileStream,
    WriteFileStream,

    Clear,
    Push { arg: BigInt },
    Pop { arg: u8 },

    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,

    Move { arg: [u8; 2] },
    Copy { arg: [u8; 2] },

    PreComputeJump { arg: String }, //A PreComputedJump stores the label String instead of the label index
    Jump { arg: usize },
    Label { arg: String },

    Equal,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,

    BitAnd,
    BitOr,
    BitXor,
    BitRightShift,
    BitLeftShift,
}

impl Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
