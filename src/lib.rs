extern crate num;

pub mod interpreter;
pub mod optimize;
mod token;
pub mod vfs;

use interpreter::*;
use vfs::FileStream;

fn assert_file_contents_equal<F>(mut file: F, contents: &str)
where
    F: FileStream,
{
    let mut s = String::new();
    file.read_to_string(&mut s).unwrap();
    assert_eq!(s, contents);
}

#[cfg(test)]
mod tests {
    use crate::{
        interpreter::run_from_file_path,
        vfs::{FileSystem, RealLocalFileSystem, VirtualFileSystem},
    };

    #[test]
    fn vfs() {
        //RealLocalFileSystem
        {
            println!("\n\nRealLocalFileSystem:\n");
            let mut fs = RealLocalFileSystem {
                root: std::env::current_dir()
                    .unwrap()
                    .as_path()
                    .to_string_lossy()
                    .to_string()
                    + "/",
            };
            println!("{:#?}\n\n{:#?}", fs.ls("."), fs.ls("src"));

            println!("\nWrite:\n");

            let mut f0_w = fs.create_file_stream("my_file.txt").unwrap();
            f0_w.write_all("This is my file. Isn't that crazy?\n\n".as_bytes())
                .unwrap();
            f0_w.write_all("!!!Breaking News!!!\n\tThis is still my file.".as_bytes())
                .unwrap();

            println!("\nRead:\n");

            let mut f0_r = fs.open_file_stream("my_file.txt").unwrap();
            let mut buf = String::new();
            f0_r.read_to_string(&mut buf).unwrap();
            println!("{}", buf);

            //Clean up
            fs.remove_file("my_file.txt")
                .expect("Couldn't remove my_file.txt");
        }
        //VirtualFileSystem
        {
            println!("\n\nVirtualFileSystem:\n");
            let mut fs = VirtualFileSystem::new();
            println!("{:#?}\n\n{:#?}", fs.ls(""), fs.ls("src"));

            println!("\nWrite:\n");

            let mut f0_w = fs.create_file_stream("my_file.txt").unwrap();
            f0_w.write_all("This is my file. Isn't that crazy?\n\n".as_bytes())
                .unwrap();
            f0_w.write_all("!!!Breaking News!!!\n\tThis is still my file.".as_bytes())
                .unwrap();

            println!("\nRead:\n");

            let mut f0_r = fs.open_file_stream("my_file.txt").unwrap();
            let mut buf = String::new();
            f0_r.read_to_string(&mut buf).unwrap();
            println!("{}", buf);

            println!("{:#?}\n\n{:#?}", fs.ls(""), fs.ls("src"));

            //Clean up
            fs.remove_file("my_file.txt")
                .expect("Couldn't remove my_file.txt");

            println!("{:#?}\n\n{:#?}", fs.ls(""), fs.ls("src"));
        }
    }
}
