use std::{
    fs::{read_dir, File},
    io,
};

pub trait FileSystem {
    ///Lists all files and subdirectories in the given path.
    /// In other words, lists the children of path
    fn ls(&self, path: &str) -> Result<Vec<String>, io::Error>;

    fn create_file_stream(&self, path: &str) -> Result<Box<dyn FileStream>, io::Error>;

    ///Opens a file stream using the given path
    fn open_file_stream(&mut self, path: &str) -> Result<Box<dyn FileStream>, io::Error>;

    fn remove_file(&mut self, path: &str) -> Result<(), io::Error>;
}

pub trait FileStream: io::Write + io::Read {}

///Acts as a virtual file system performing actions relative to the root directory
pub struct RealLocalFileSystem {
    pub root: String,
}

impl FileSystem for RealLocalFileSystem {
    fn ls(&self, path: &str) -> Result<Vec<String>, io::Error> {
        //Make the absolute path
        let path = self.root.clone() + &path;

        //Read the directory
        let dir = match read_dir(path) {
            Ok(d) => d,
            Err(e) => return Err(e),
        };

        //Read each file name
        let mut v = Vec::new();
        for d in dir {
            v.push(d.unwrap().file_name().to_string_lossy().to_string());
        }

        Ok(v)
    }

    fn create_file_stream(&self, path: &str) -> Result<Box<dyn FileStream>, io::Error> {
        let path = self.root.clone() + &path;

        match File::create(path) {
            Ok(f) => Ok(Box::new(RealLocalFileStream::from_file(f))),
            Err(e) => Err(e),
        }
    }

    fn open_file_stream(&mut self, path: &str) -> Result<Box<dyn FileStream>, io::Error> {
        match RealLocalFileStream::new(&(self.root.clone() + path)) {
            Ok(fs) => Ok(Box::new(fs)),
            Err(e) => Err(e),
        }
    }

    fn remove_file(&mut self, path: &str) -> Result<(), io::Error> {
        let path = self.root.clone() + &path;

        std::fs::remove_file(path)
    }
}

pub struct RealLocalFileStream {
    file: File,
}

impl RealLocalFileStream {
    pub fn new(absolute_path: &str) -> Result<RealLocalFileStream, io::Error> {
        let file = match File::open(absolute_path) {
            Ok(f) => f,
            Err(e) => return Err(e),
        };

        Ok(RealLocalFileStream { file })
    }
    pub fn from_file(file: File) -> RealLocalFileStream {
        RealLocalFileStream { file }
    }
}

impl io::Read for RealLocalFileStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.file.read(buf)
    }
}

impl io::Write for RealLocalFileStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.file.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.file.flush()
    }
}

impl FileStream for RealLocalFileStream {}
