use std::{fs::{self, File}, time::SystemTime, str::FromStr, io::{Write, Read}};

extern crate num;
use num::{bigint::BigInt, ToPrimitive, FromPrimitive};

use crate::token::TokenType;

mod token;

pub struct Stack
{
  dat: Vec<BigInt>,
}

impl Stack
{
  pub fn new() -> Stack
  {
    Stack { dat: Vec::new() }
  }

  pub fn push(&mut self, n: BigInt)
  {
    self.dat.push(n);
  }
  
  pub fn pop(&mut self) -> BigInt
  {
    if self.dat.len() == 0
    {
      return BigInt::from(0);
    }
    self.dat.pop().expect("pop failed")
  }

  pub fn len(&self) -> usize { self.dat.len() }

  pub fn clear(&mut self)
  {
    self.dat.clear();
  }
}

//Takes a character representing one of the stacks and turns it into that stack's index
fn stack_char_to_index(s: &str) -> u8
{
  match s
  {
    "A" => 0u8,
    "B" => 1u8,
    "C" => 2u8,
    _ => 255u8
  }
}

pub fn run_from_file_stream(file_stream: File) {

}

pub fn run_from_file_path(file_path: String) {
  let mut tokens: Vec<TokenType> = Vec::new();

  //Init runtime IO streams and such
  let mut file_stream: File = File::open(file_path).expect("Cannot open file from file_path");

  parse(&mut file_stream, &mut tokens);

  file_stream = File::create("staqdump").expect("Cannot create 'staqdump' temporary file");

  interpret(tokens, file_stream);
}

fn parse(file_stream: &mut File, tokens: &mut Vec<TokenType>)
{
  let start_time: SystemTime = SystemTime::now();

  let mut file: String = "".to_string();//fs::read_to_string(file).expect("Unable to read file");
  file_stream.read_to_string(&mut file).unwrap();

  let lines: Vec<&str> = file.lines().collect();

  //Read each line, and at the end of each add a Clear token
  for line in lines
  {
    //Read the commands in a line
    let commands: Vec<&str> = line.split(' ').collect();
    for command in commands
    {
      //Read the parts of the command
      let parts: Vec<&str> = command.split(':').collect();
      let id: &str = parts[0];

      //Check for newline characters and break
      if id.starts_with("//")
      {
        break;
      }

      //Parse each command based on the identifying first clause
      match id
      {
        "exit" => tokens.push(TokenType::Exit {}),
        "" => (),

        "print" => tokens.push(TokenType::Print {}),
        "printnum" => tokens.push(TokenType::PrintNum {}),
        "createfile" => tokens.push(TokenType::CreateFile { arg: if parts.len() > 1 { parts[1].to_string() } else { "".to_string() } }),
        "createfilestream" => tokens.push(TokenType::CreateFileStream { arg: if parts.len() > 1 { parts[1].to_string() } else { "".to_string() } }),
        "openfilestream" => tokens.push(TokenType::OpenFileStream { arg: if parts.len() > 1 { parts[1].to_string() } else { "".to_string() } }),
        "readfilestream" => tokens.push(TokenType::ReadFileStream {}),
        "writefilestream" => tokens.push(TokenType::WriteFileStream {}),

        "push" => tokens.push(TokenType::Push { arg: BigInt::from_str(parts[1]).expect(format!("Failed parsing Push token argument: {}", command).as_str()) }),
        "pop" => tokens.push(TokenType::Pop { arg: stack_char_to_index(parts[1]) }),

        "+" => tokens.push(TokenType::Add {}),
        "-" => tokens.push(TokenType::Subtract {}),
        "*" => tokens.push(TokenType::Multiply {}),
        "/" => tokens.push(TokenType::Divide {}),
        "%" => tokens.push(TokenType::Modulo {}),

        "move" => tokens.push(TokenType::Move { arg: [stack_char_to_index(parts[1]), stack_char_to_index(parts[2])] }),
        "copy" => tokens.push(TokenType::Copy { arg: [stack_char_to_index(parts[1]), stack_char_to_index(parts[2])] }),

        "jump" => tokens.push(TokenType::PreComputeJump { arg: parts[1].to_string() }),
        "label" => tokens.push(TokenType::Label { arg: parts[1].to_string() }),

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

        _ => println!("Invalid token: {}", command)
      }
    }

    tokens.push(TokenType::Clear);
  }
  //Finish by setting the index of jump tokens
  for i in 0..tokens.len()
  {
    match &tokens[i]
    {
      TokenType::PreComputeJump { arg } =>
      {
        let label: &String = arg;
        let mut index: usize = usize::MAX;
        
        for ib in 0..tokens.len()
        {
          match &tokens[ib] {
            TokenType::Label { arg } => { if arg == label { index = ib; break;}; },
            _ => ()
          }
        }

        //Steps to replacing PreComputeJump with Jump:
        //1. Remove the PreComputeJump object and swap it with the top value of the vector
        //2. Add the new Jump object to the top of the vector
        //3. Swap the new Jump object with the old value at the top of the vector
        let tokens_last_index: usize = tokens.len() - 1;
        tokens.swap_remove(i);
        tokens.push(TokenType::Jump { arg: index });
        tokens.swap(i,  tokens_last_index);
      },
      _ => ()        
    }
  }

  //Debug print out all tokens
  println!();

  let tokens_len: usize = tokens.len();

  for t in tokens
  {
    println!("{}", t);
  }
  println!("\n");

  let total_time = SystemTime::now().duration_since(start_time).expect("Calculating time duration of parsing step failed");
  println!("Compile time taken: {}ms or {}μs", total_time.as_millis(), total_time.as_micros());
  println!("Number of commands: {}\n", tokens_len);
}

fn interpret(tokens: Vec<TokenType>, mut file_stream: File) {
  //Initialization

  //There are three stacks, initialized seperately since they don't implement Copy()
  let mut stacks: [Stack; 3] = [Stack { dat: Vec::new() }, Stack { dat: Vec::new() }, Stack { dat: Vec::new() }];
  let mut token_index: usize = 0;

  //Execution start
  println!("Program execution start\n----");

  let program_start_time: SystemTime = SystemTime::now();

  let program_exit_reason: String;

  loop
  {
    if token_index >= tokens.len()
    {
      program_exit_reason = "successfully reached end of program".to_string();
      break;
    }
    //Execute the correct method for the enum
    match &tokens[token_index]
    {
      TokenType::Exit => { program_exit_reason = format!("exit command called from index {}", token_index); break; },

      TokenType::Print => {
        let stack_len: usize = stacks[2].len();
        let mut s: String = "".to_string();
        for _ in 0..stack_len
        {
          let c: char = stacks[2].pop().to_u8().expect(format!("Invalid char value in print. Index: {}", token_index).as_str()) as char;
          s.push(c);
        }
        print!("{}", s);
      },
      TokenType::PrintNum => {
        let stack_len: usize = stacks[2].len();
        let mut s: String = "".to_string();
        for _ in 0..stack_len
        {
          s += stacks[2].pop().to_string().as_str();
        }
        print!("{}", s);
      },
      TokenType::CreateFile { arg } => {
        let mut path: String;
        if arg.is_empty()
        {
          path = "".to_string();
          for _ in 0..stacks[2].len()
          {
            let c: char = stacks[2].pop().to_u8().expect(format!("Invalid char value in print index {}", token_index).as_str()) as char;
            path.push(c);
          }
        } else
        {
          path = arg.to_string();
        }
        File::create(path).expect(format!("File creation failed. Index {}", token_index).as_str());
      },
      TokenType::CreateFileStream { arg } => {
        let mut path: String;
        if arg.is_empty()
        {
          path = "".to_string();
          for _ in 0..stacks[2].len()
          {
            let c: char = stacks[2].pop().to_u8().expect(format!("Invalid char value in print index {}", token_index).as_str()) as char;
            path.push(c);
          }
        } else
        {
          path = arg.to_string();
        }
        file_stream = File::create(path).expect(format!("File creation failed. Index {}", token_index).as_str());
      },
      TokenType::OpenFileStream { arg } => {
        let mut path: String;
        if arg.is_empty()
        {
          path = "".to_string();
          for _ in 0..stacks[2].len()
          {
            let c: char = stacks[2].pop().to_u8().expect(format!("Invalid char value in print index {}", token_index).as_str()) as char;
            path.push(c);
          }
        } else
        {
          path = arg.to_string();
        }
        let res: Result<File, std::io::Error> = File::open(path);
        if res.is_ok()
        {
          file_stream = res.unwrap();
        } else
        {
          stacks[2].push(BigInt::from_i32(-1).expect("Invalid conversion from -1 to BigInt"));  
        }
      },
      TokenType::ReadFileStream => { let mut arr: [u8; 1] = [0]; file_stream.read(&mut arr); stacks[2].push(BigInt::from_u8(arr[0]).expect("Failed to convert from u8 to BigInt")); },
      TokenType::WriteFileStream => { let mut arr: Vec<u8> = Vec::with_capacity(stacks[2].len()); for _ in 0..stacks[2].len() { arr.push(stacks[2].pop().to_u8().expect(format!("Failure in writefilestream input fro stack C.Index: {}", token_index).as_str())); } file_stream.write(&arr); },

      TokenType::Clear => stacks[2].clear(),
      TokenType::Push { arg } => stacks[2].push(arg.to_owned()),
      TokenType::Pop { arg } => { stacks[(*arg) as usize].pop(); },

      TokenType::Add => { let a: BigInt = stacks[0].pop(); let b: BigInt = stacks[1].pop(); stacks[2].push(a + b) },
      TokenType::Subtract => { let a: BigInt = stacks[0].pop(); let b: BigInt = stacks[1].pop(); stacks[2].push(a - b) },
      TokenType::Multiply => { let a: BigInt = stacks[0].pop(); let b: BigInt = stacks[1].pop(); stacks[2].push(a * b) },
      TokenType::Divide => { let a: BigInt = stacks[0].pop(); let b: BigInt = stacks[1].pop(); stacks[2].push(a / b) },
      TokenType::Modulo => { let a: BigInt = stacks[0].pop(); let b: BigInt = stacks[1].pop(); stacks[2].push(a % b) },

      TokenType::Move { arg } => { let n: BigInt = stacks[arg[0] as usize].pop(); stacks[arg[1] as usize].push(n); },
      TokenType::Copy { arg } => { let n: BigInt = stacks[arg[0] as usize].pop(); stacks[arg[0] as usize].push(n.to_owned()); stacks[arg[1] as usize].push(n); },

      TokenType::Jump { arg } => { let n: BigInt = stacks[2].pop(); if n > BigInt::from(0i32) { token_index =  *arg; } },
      TokenType::Label { arg } => (),
      
      TokenType::Equal => { let a: BigInt = stacks[0].pop(); let b: BigInt = stacks[1].pop(); stacks[2].push(BigInt::from((a == b) as i32)) },
      TokenType::GreaterThan => { let a: BigInt = stacks[0].pop(); let b: BigInt = stacks[1].pop(); stacks[2].push(BigInt::from((a > b) as i32)) },
      TokenType::GreaterThanOrEqual => { let a: BigInt = stacks[0].pop(); let b: BigInt = stacks[1].pop(); stacks[2].push(BigInt::from((a >= b) as i32)) },
      TokenType::LessThan => { let a: BigInt = stacks[0].pop(); let b: BigInt = stacks[1].pop(); stacks[2].push(BigInt::from((a < b) as i32)) },
      TokenType::LessThanOrEqual => { let a: BigInt = stacks[0].pop(); let b: BigInt = stacks[1].pop(); stacks[2].push(BigInt::from((a <= b) as i32)) },

      TokenType::BitAnd => { let a: BigInt = stacks[0].pop(); let b: BigInt = stacks[1].pop(); stacks[2].push(a & b) },
      TokenType::BitOr => { let a: BigInt = stacks[0].pop(); let b: BigInt = stacks[1].pop(); stacks[2].push(a | b) },
      TokenType::BitXor => { let a: BigInt = stacks[0].pop(); let b: BigInt = stacks[1].pop(); stacks[2].push(a ^ b) },
      TokenType::BitRightShift => { let a: BigInt = stacks[0].pop(); let b: BigInt = stacks[1].pop(); stacks[2].push(a >> b.to_i128().expect("shift value too high")) },
      TokenType::BitLeftShift => { let a: BigInt = stacks[0].pop(); let b: BigInt = stacks[1].pop(); stacks[2].push(a << b.to_i128().expect("shift value too high")) },

      _ => println!("Invalid token in execution. Index: {}", token_index)
    }

    token_index += 1;
  }

  let program_time: std::time::Duration = SystemTime::now().duration_since(program_start_time).expect("Time went backwards!");

  std::fs::remove_file("staqdump").expect("Failed to delete 'staqdump' temporary file");

  println!("----\nProgram execution finished: {}\nTime taken: {}ms or {}μs", program_exit_reason, program_time.as_millis(), program_time.as_micros());
}

fn main()
{
  let args: Vec<String> = std::env::args().collect();
  let mut file_path: String = "./in.stq".to_string();
  
  for i in 0..args.len()
  {
    println!("{}", args[i]);
  }

  if args.len() > 1
  {
    file_path = "./".to_string() + &args[1].to_owned();
  }

  run_from_file_path(file_path);
}

