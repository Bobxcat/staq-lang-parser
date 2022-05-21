extern crate num;

pub mod interpreter;
mod token;

use interpreter::*;

#[cfg(test)]
mod tests {
    use crate::interpreter::run_from_file_path;

    #[test]
    fn from_file() {
        let args: Vec<String> = std::env::args().collect();
        run_from_file_path(args.last().expect("No args given").clone());
    }
    #[test]
    fn hello_world() {
        run_from_file_path("hello_world.stq".to_string());
    }
}
