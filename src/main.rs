use staq_lang_parser::interpreter::run_from_file_path;

extern crate num;

pub mod interpreter;
pub mod optimize;
mod token;
pub mod vfs;

fn main() {
    //println!("{:?}", std::env::args().collect::<Vec<String>>());
    let file_path = std::env::args().nth(1).unwrap();
    run_from_file_path(file_path);
}
