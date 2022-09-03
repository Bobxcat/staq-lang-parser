use std::{
    env,
    fs::{self, File},
    io::{Read, Write},
    path::PathBuf,
    str::FromStr,
    time::SystemTime,
};

extern crate num;
use num::{bigint::BigInt, FromPrimitive, ToPrimitive};

use crate::{
    optimize::optimize,
    token::TokenType,
    vfs::{FileStream, FileSystem, RealLocalFileSystem},
};

pub struct Stack {
    dat: Vec<BigInt>,
}

impl Stack {
    pub fn new() -> Stack {
        Stack { dat: Vec::new() }
    }

    pub fn push(&mut self, n: BigInt) {
        self.dat.push(n);
    }

    pub fn pop(&mut self) -> BigInt {
        if self.dat.len() == 0 {
            return BigInt::from(0);
        }
        self.dat.pop().expect("pop failed")
    }

    pub fn len(&self) -> usize {
        self.dat.len()
    }

    pub fn clear(&mut self) {
        self.dat.clear();
    }
}

//Takes a character representing one of the stacks and turns it into that stack's index
fn stack_char_to_index(s: &str) -> u8 {
    match s {
        "A" => 0u8,
        "B" => 1u8,
        "C" => 2u8,
        _ => 255u8,
    }
}

pub fn run_from_string(string: String, mut file_system: Box<dyn FileSystem>) {
    let mut tokens: Vec<TokenType> = Vec::new();

    parse(string, &mut tokens);

    interpret(tokens, file_system);
}

pub fn run_from_file_path(file_path: String) {
    let mut s = String::new();

    File::open(file_path.clone())
        .expect("Cannot open file from file_path")
        .read_to_string(&mut s)
        .unwrap();

    //Init runtime IO system
    //Make the file system local to the StaqLang program's path
    //This means the root is the program's parent directory
    let root = PathBuf::from_str(&file_path)
        .unwrap()
        .parent()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string()
        + "/";
    let file_system: Box<dyn FileSystem> = Box::new(RealLocalFileSystem { root });

    run_from_string(s, file_system);
}

fn parse(file: String, tokens: &mut Vec<TokenType>) {
    let start_time: SystemTime = SystemTime::now();

    let lines: Vec<&str> = file.lines().collect();

    //Read each line, and at the end of each add a Clear token
    for line in lines {
        //Read the commands in a line
        let commands: Vec<&str> = line.split(' ').collect();
        for command in commands {
            //Read the parts of the command
            let parts: Vec<&str> = command.split(':').collect();
            let id: &str = parts[0];

            //Check for newline characters and break
            if id.starts_with("//") {
                break;
            }

            //Parse each command based on the identifying first clause
            match id {
                "exit" => tokens.push(TokenType::Exit {}),
                "" => (),

                "print" => tokens.push(TokenType::Print {}),
                "printnum" => tokens.push(TokenType::PrintNum {}),
                "getnextin" => tokens.push(TokenType::GetNextIn {}),

                "createfile" => tokens.push(TokenType::CreateFile {
                    arg: if parts.len() > 1 {
                        parts[1].to_string()
                    } else {
                        "".to_string()
                    },
                }),
                "createfilestream" => tokens.push(TokenType::CreateFileStream {
                    arg: if parts.len() > 1 {
                        parts[1].to_string()
                    } else {
                        "".to_string()
                    },
                }),
                "openfilestream" => tokens.push(TokenType::OpenFileStream {
                    arg: if parts.len() > 1 {
                        parts[1].to_string()
                    } else {
                        "".to_string()
                    },
                }),
                "readfilestream" => tokens.push(TokenType::ReadFileStream {}),
                "writefilestream" => tokens.push(TokenType::WriteFileStream {}),

                "push" => tokens.push(TokenType::Push {
                    arg: BigInt::from_str(parts[1]).expect(
                        format!("Failed parsing Push token argument: {}", command).as_str(),
                    ),
                }),
                "pop" => tokens.push(TokenType::Pop {
                    arg: stack_char_to_index(parts[1]),
                }),

                "+" => tokens.push(TokenType::Add {}),
                "-" => tokens.push(TokenType::Subtract {}),
                "*" => tokens.push(TokenType::Multiply {}),
                "/" => tokens.push(TokenType::Divide {}),
                "%" => tokens.push(TokenType::Modulo {}),

                "move" => tokens.push(TokenType::Move {
                    arg: [stack_char_to_index(parts[1]), stack_char_to_index(parts[2])],
                }),
                "copy" => tokens.push(TokenType::Copy {
                    arg: [stack_char_to_index(parts[1]), stack_char_to_index(parts[2])],
                }),

                "jump" => tokens.push(TokenType::PreComputeJump {
                    arg: parts[1].to_string(),
                }),
                "label" => tokens.push(TokenType::Label {
                    arg: parts[1].to_string(),
                }),

                "==" => tokens.push(TokenType::Equal {}),
                ">" => tokens.push(TokenType::GreaterThan {}),
                ">=" => tokens.push(TokenType::GreaterThanOrEqual {}),
                "<" => tokens.push(TokenType::LessThan {}),
                "<=" => tokens.push(TokenType::LessThanOrEqual {}),

                "&" => tokens.push(TokenType::BitAnd {}),
                "|" => tokens.push(TokenType::BitOr {}),
                "^" => tokens.push(TokenType::BitXor {}),
                ">>" => tokens.push(TokenType::BitRightShift {}),
                "<<" => tokens.push(TokenType::BitLeftShift {}),

                _ => println!("Invalid token: {}", command),
            }
        }

        tokens.push(TokenType::Clear);
    }

    optimize(tokens);

    //Debug print out all tokens
    println!();

    let tokens_len: usize = tokens.len();

    for i in 0..tokens_len {
        println!("{}. {}", i, tokens[i]);
    }
    println!("\n");

    let total_time = SystemTime::now()
        .duration_since(start_time)
        .expect("Calculating time duration of parsing step failed");
    println!(
        "Compile time taken: {}ms or {}μs",
        total_time.as_millis(),
        total_time.as_micros()
    );
    println!("Number of commands: {}\n", tokens_len);
}

fn interpret(tokens: Vec<TokenType>, mut file_system: Box<dyn FileSystem>) {
    //Initialization

    let mut file_stream_write: Box<dyn FileStream> = file_system
        .create_file_stream("staqdump")
        .expect("Could not create staqdump (write)");
    let mut file_stream_read: Box<dyn FileStream> = file_system
        .open_file_stream("staqdump")
        .expect("Could not open staqdump (read)");

    //There are three stacks, initialized seperately since they don't implement Copy()
    let mut stacks: [Stack; 3] = [
        Stack { dat: Vec::new() },
        Stack { dat: Vec::new() },
        Stack { dat: Vec::new() },
    ];
    let mut token_index: usize = 0;

    //Execution start
    println!("Program execution start\n----");

    let program_start_time: SystemTime = SystemTime::now();

    let program_exit_reason: String;

    loop {
        if token_index >= tokens.len() {
            program_exit_reason = "successfully reached end of program".to_string();
            break;
        }
        //Execute the correct method for the enum
        match &tokens[token_index] {
            TokenType::Exit => {
                program_exit_reason = format!("exit command called from index {}", token_index);
                break;
            }

            TokenType::Print => {
                let stack_len: usize = stacks[2].len();
                let mut s: String = "".to_string();
                for _ in 0..stack_len {
                    let c: char = stacks[2].pop().to_u8().expect(
                        format!("Invalid char value in print. Index: {}", token_index).as_str(),
                    ) as char;
                    s.push(c);
                }
                print!("{}", s);
            }
            TokenType::PrintNum => {
                let stack_len: usize = stacks[2].len();
                let mut s: String = "".to_string();
                for _ in 0..stack_len {
                    s += stacks[2].pop().to_string().as_str();
                }
                print!("{}", s);
            }
            TokenType::GetNextIn => {
                let mut arr = [0];
                if std::io::stdin().read_exact(&mut arr).is_ok() {
                    stacks[2].push(BigInt::from_i16(arr[0] as i16).unwrap());
                }
            }

            TokenType::CreateFile { arg } => {
                let mut path: String;
                if arg.is_empty() {
                    path = "".to_string();
                    for _ in 0..stacks[2].len() {
                        let c: char = stacks[2].pop().to_u8().expect(
                            format!("Invalid char value in print index {}", token_index).as_str(),
                        ) as char;
                        path.push(c);
                    }
                } else {
                    path = arg.to_string();
                }

                //Discard the file stream since the only importance is whether or not the file was successfully created
                if let Ok(_) = file_system.create_file_stream(&path) {
                    //Signal success
                    stacks[2]
                        .push(BigInt::from_i32(1).expect("Invalid conversion from 1 to BigInt"));
                } else {
                    //Signal failure
                    stacks[2]
                        .push(BigInt::from_i32(-1).expect("Invalid conversion from -1 to BigInt"));
                }
            }
            TokenType::CreateFileStream { arg } => {
                let mut path: String;
                if arg.is_empty() {
                    path = "".to_string();
                    for _ in 0..stacks[2].len() {
                        let c: char = stacks[2].pop().to_u8().expect(
                            format!("Invalid char value in print index {}", token_index).as_str(),
                        ) as char;
                        path.push(c);
                    }
                } else {
                    path = arg.to_string();
                }

                //If the file doesn't open properly, push -1 to the c stack. Otherwise, push 1
                if let Ok(f) = file_system.create_file_stream(&path) {
                    file_stream_write = f;
                    //Signal success
                    stacks[2]
                        .push(BigInt::from_i32(1).expect("Invalid conversion from 1 to BigInt"));
                } else {
                    //Signal failure
                    stacks[2]
                        .push(BigInt::from_i32(-1).expect("Invalid conversion from -1 to BigInt"));
                }
            }
            TokenType::OpenFileStream { arg } => {
                let mut path: String;
                if arg.is_empty() {
                    path = "".to_string();
                    for _ in 0..stacks[2].len() {
                        let c: char = stacks[2].pop().to_u8().expect(
                            format!("Invalid char value in print index {}", token_index).as_str(),
                        ) as char;
                        path.push(c);
                    }
                } else {
                    path = arg.to_string();
                }

                //If the file doesn't open properly, push -1 to the c stack. Otherwise, push 1
                if let Ok(f) = file_system.open_file_stream(&path) {
                    file_stream_read = f;
                    //Signal success
                    stacks[2]
                        .push(BigInt::from_i32(1).expect("Invalid conversion from 1 to BigInt"));
                } else {
                    //Signal failure
                    stacks[2]
                        .push(BigInt::from_i32(-1).expect("Invalid conversion from -1 to BigInt"));
                }
            }
            TokenType::ReadFileStream => {
                let mut arr: [u8; 1] = [1];
                match file_stream_read.read(&mut arr) {
                    Ok(bytes_read) => {
                        let push_value: BigInt = if bytes_read == 0 {
                            BigInt::from_i32(-1).expect("Failed to convert -1 to BigInt")
                        } else {
                            BigInt::from_u8(arr[0]).expect("Failed to convert from u8 to BigInt")
                        };

                        stacks[2].push(push_value);

                        //Signal success
                        stacks[2].push(
                            BigInt::from_i32(1).expect("Invalid conversion from 1 to BigInt"),
                        );
                    }
                    Err(_) => {
                        //Signal failure
                        stacks[2]
                            .push(BigInt::from_i32(-1).expect("Failed to convert -1 to BigInt"));
                    }
                }
            }
            TokenType::WriteFileStream => {
                let mut arr: Vec<u8> = Vec::with_capacity(stacks[2].len());
                for _ in 0..stacks[2].len() {
                    arr.push(
                        stacks[2].pop().to_u8().expect(
                            format!(
                                "Failure in writefilestream input fro stack C.Index: {}",
                                token_index
                            )
                            .as_str(),
                        ),
                    );
                }
                match file_stream_write.write(&arr) {
                    Ok(_) => {
                        //Signal success
                        stacks[2].push(
                            BigInt::from_i32(1).expect("Invalid conversion from 1 to BigInt"),
                        );
                    }
                    Err(_) => {
                        //Signal failure
                        stacks[2]
                            .push(BigInt::from_i32(-1).expect("Failed to convert -1 to BigInt"));
                    }
                }
            }

            TokenType::Clear => stacks[2].clear(),
            TokenType::Push { arg } => stacks[2].push(arg.to_owned()),
            TokenType::Pop { arg } => {
                stacks[(*arg) as usize].pop();
            }

            TokenType::Add => {
                let a: BigInt = stacks[0].pop();
                let b: BigInt = stacks[1].pop();
                stacks[2].push(a + b)
            }
            TokenType::Subtract => {
                let a: BigInt = stacks[0].pop();
                let b: BigInt = stacks[1].pop();
                stacks[2].push(a - b)
            }
            TokenType::Multiply => {
                let a: BigInt = stacks[0].pop();
                let b: BigInt = stacks[1].pop();
                stacks[2].push(a * b)
            }
            TokenType::Divide => {
                let a: BigInt = stacks[0].pop();
                let b: BigInt = stacks[1].pop();
                stacks[2].push(a / b)
            }
            TokenType::Modulo => {
                let a: BigInt = stacks[0].pop();
                let b: BigInt = stacks[1].pop();
                stacks[2].push(a % b)
            }

            TokenType::Move { arg } => {
                let n: BigInt = stacks[arg[0] as usize].pop();
                stacks[arg[1] as usize].push(n);
            }
            TokenType::Copy { arg } => {
                let n: BigInt = stacks[arg[0] as usize].pop();
                stacks[arg[0] as usize].push(n.to_owned());
                stacks[arg[1] as usize].push(n);
            }

            TokenType::Jump { arg } => {
                let n: BigInt = stacks[2].pop();
                if n > BigInt::from(0i32) {
                    token_index = *arg;
                }
            }
            TokenType::Label { arg } => (),

            TokenType::Equal => {
                let a: BigInt = stacks[0].pop();
                let b: BigInt = stacks[1].pop();
                stacks[2].push(BigInt::from((a == b) as i32))
            }
            TokenType::GreaterThan => {
                let a: BigInt = stacks[0].pop();
                let b: BigInt = stacks[1].pop();
                stacks[2].push(BigInt::from((a > b) as i32))
            }
            TokenType::GreaterThanOrEqual => {
                let a: BigInt = stacks[0].pop();
                let b: BigInt = stacks[1].pop();
                stacks[2].push(BigInt::from((a >= b) as i32))
            }
            TokenType::LessThan => {
                let a: BigInt = stacks[0].pop();
                let b: BigInt = stacks[1].pop();
                stacks[2].push(BigInt::from((a < b) as i32))
            }
            TokenType::LessThanOrEqual => {
                let a: BigInt = stacks[0].pop();
                let b: BigInt = stacks[1].pop();
                stacks[2].push(BigInt::from((a <= b) as i32))
            }

            TokenType::BitAnd => {
                let a: BigInt = stacks[0].pop();
                let b: BigInt = stacks[1].pop();
                stacks[2].push(a & b)
            }
            TokenType::BitOr => {
                let a: BigInt = stacks[0].pop();
                let b: BigInt = stacks[1].pop();
                stacks[2].push(a | b)
            }
            TokenType::BitXor => {
                let a: BigInt = stacks[0].pop();
                let b: BigInt = stacks[1].pop();
                stacks[2].push(a ^ b)
            }
            TokenType::BitRightShift => {
                let a: BigInt = stacks[0].pop();
                let b: BigInt = stacks[1].pop();
                stacks[2].push(a >> b.to_i128().expect("shift value too high"))
            }
            TokenType::BitLeftShift => {
                let a: BigInt = stacks[0].pop();
                let b: BigInt = stacks[1].pop();
                stacks[2].push(a << b.to_i128().expect("shift value too high"))
            }

            _ => println!("Invalid token in execution. Index: {}", token_index),
        }

        token_index += 1;
    }

    let program_time: std::time::Duration = SystemTime::now()
        .duration_since(program_start_time)
        .expect("Time went backwards!");

    file_system
        .remove_file("staqdump")
        .expect("Failed to remove 'staqdump' temporary file");

    println!(
        "\n----\nProgram execution finished: {}\nTime taken: {}ms or {}μs",
        program_exit_reason,
        program_time.as_millis(),
        program_time.as_micros()
    );
}
