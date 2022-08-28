use std::{
    cell::RefCell,
    fmt::Debug,
    fs::{read_dir, File},
    io,
    rc::Rc,
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

//Real Local File System

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
        println!("create: {}", path);
        match File::create(path) {
            Ok(f) => Ok(Box::new(RealLocalFileStream::from_file(f))),
            Err(e) => Err(e),
        }
    }

    fn open_file_stream(&mut self, path: &str) -> Result<Box<dyn FileStream>, io::Error> {
        println!("open: {}", &(self.root.clone() + path));
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

//Virtual File System
//Note: only accepts '/' as a path delimiter

///Takes an input path str and splits it into its component parts by using '/' as a delimiter
fn split_path_str(path: &str) -> Vec<String> {
    let mut v = Vec::new();

    //Only accepts non-empty components
    for s in path.split('/') {
        if !s.is_empty() {
            v.push(s.to_string());
        }
    }

    v
}

///Gets the parent of the given VirtualPath. A convenience function
fn get_parent(vp: &Rc<RefCell<VirtualPath>>) -> Option<Rc<RefCell<VirtualPath>>> {
    match &*vp.borrow() {
        VirtualPath::File { parent, .. } => Some(parent.clone()),
        VirtualPath::Dir { parent, .. } => parent.clone(),
    }
}

///Gets the name of the given VirtualPath. A convenience function
fn get_name(vp: &Rc<RefCell<VirtualPath>>) -> String {
    match &*vp.borrow() {
        VirtualPath::File { name, .. } | VirtualPath::Dir { name, .. } => name.clone(),
    }
}

pub struct VirtualFileSystem {
    root: Rc<RefCell<VirtualPath>>,
}

impl VirtualFileSystem {
    pub fn new() -> VirtualFileSystem {
        VirtualFileSystem {
            root: Rc::new(RefCell::new(VirtualPath::Dir {
                name: "root".to_string(),
                children: Vec::new(),
                parent: None,
            })),
        }
    }

    ///Gets either the directory or file at the given path if it is successfully found
    fn get(&self, path: &str) -> Option<Rc<RefCell<VirtualPath>>> {
        Self::get_from_head(&self.root, path)
    }

    fn get_from_head(
        search_head: &Rc<RefCell<VirtualPath>>,
        path: &str,
    ) -> Option<Rc<RefCell<VirtualPath>>> {
        let path = split_path_str(path);
        Self::get_from_head_path_segments(search_head, path)
    }

    ///Gets either the directory or file at the given path relative to the search head if it is successfully found
    fn get_from_head_path_segments(
        search_head: &Rc<RefCell<VirtualPath>>,
        path: Vec<String>,
    ) -> Option<Rc<RefCell<VirtualPath>>> {
        let mut search_head = search_head.clone();

        //Iterate through each path segment and find the next matching child until reaching the end of all path segments
        for path_segment in path {
            match &*(*search_head.clone()).borrow() {
                //Current search head

                //If the head reached a file and there is remaining path to search, it failed
                VirtualPath::File { .. } => return None,
                VirtualPath::Dir { name, children, .. } => {
                    //Find the child with the matching name and set it as the search head
                    let mut matching_child_found: bool = false;
                    for c in children {
                        //Next search head

                        match &*(**c).borrow() {
                            VirtualPath::File { name, .. } | VirtualPath::Dir { name, .. } => {
                                //If the child's name matches the path segment, advance the search head and move on to the next path segment
                                if *name == path_segment {
                                    search_head = c.clone();
                                    matching_child_found = true;
                                    break;
                                }
                            }
                        }
                    }

                    //If no matching child was found, return None. The search failed
                    if !matching_child_found {
                        return None;
                    }
                }
            }
        }

        //If the path_segment search succeeded every single time, then the path was valid. The search succeeded
        Some(search_head)
    }

    ///Creates a file with the given path and returns a reference to its VirtualPath.
    /// Will override any existing file at the given path, emptying its data
    fn create_file(&self, path: &str) -> Result<Rc<RefCell<VirtualPath>>, io::Error> {
        let path_segments = split_path_str(path);

        if path_segments.len() == 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput {},
                format!(
                    "Cannot create file at \"{}\". The given path has no segments",
                    path
                ),
            ));
        }

        let mut search_head = self.root.clone();

        //Here are the iterative steps:
        //1- Advance the search head by 1 level
        //2- If the search head fails to advance, create a subdirectory under search_head to enable its advance and move search_head to that directory
        //Finally:
        //1- If the file already exists, clear its data

        //NOTE: First, all but the final path segment will be search/created as directories
        //Then, the final path segment will be created as a file

        for i in 0..(path_segments.len() - 1) {
            let seg = &path_segments[i];

            //Search from the current search_head only advancing by 1 path_segment
            match Self::get_from_head_path_segments(&search_head, vec![seg.clone()]) {
                Some(new_head) => search_head = new_head.clone(),
                //Since there wasn't a matching directory under this one, attempt to add a new subdirectory
                None => {
                    let new_dir = Rc::new(RefCell::new(VirtualPath::Dir {
                        name: seg.clone(),
                        children: Vec::with_capacity(1),
                        parent: Some(search_head.clone()),
                    }));
                    match &mut *search_head.borrow_mut() {
                        //Cannot add a subdirectory to a file. Return an error
                        VirtualPath::File { .. } => {
                            return Err(io::Error::new(
                                io::ErrorKind::InvalidInput {},
                                "One of the directories in the path was actually a file",
                            ))
                        }
                        //Can add a subdirectory to a directory. Proceed as normal
                        VirtualPath::Dir { children, .. } => children.push(new_dir.clone()),
                    }
                    //Set the search head to the new subdirectory
                    search_head = new_dir;
                }
            }
        }

        //Creating the file from the final segment
        {
            let search_head_ref = &mut *search_head.borrow_mut();
            match search_head_ref {
                VirtualPath::File { .. } => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput {},
                        "One of the directories in the path was actually a file",
                    ))
                }
                VirtualPath::Dir { children, .. } => {
                    //Create the file
                    let file = VirtualPath::File {
                        name: path_segments.last().unwrap().clone(),
                        data: Rc::new(RefCell::new(Vec::new())),
                        parent: search_head.clone(),
                    };
                    let file = Rc::new(RefCell::new(file));

                    //Make the file one of the children and return it
                    children.push(file.clone());
                    return Ok(file);
                }
            }
        }
    }
}

impl FileSystem for VirtualFileSystem {
    fn ls(&self, path: &str) -> Result<Vec<String>, io::Error> {
        match self.get(path) {
            Some(vp) => match &*(*vp.clone()).borrow() {
                VirtualPath::File { name, data, .. } => Err(io::Error::new(
                    io::ErrorKind::InvalidInput {},
                    "Expected a directory path, found a file path",
                )),
                VirtualPath::Dir { name, children, .. } => {
                    let mut v = Vec::new();

                    for c in children {
                        match &*(**c).borrow() {
                            VirtualPath::File { name, .. } | VirtualPath::Dir { name, .. } => {
                                v.push(name.clone());
                            }
                        }
                    }

                    Ok(v)
                }
            },
            None => Err(io::Error::new(
                io::ErrorKind::NotFound {},
                "Directory not found",
            )),
        }
    }

    fn create_file_stream(&self, path: &str) -> Result<Box<dyn FileStream>, io::Error> {
        match self.create_file(path) {
            Ok(vp) => Ok(Box::new(VirtualFileStream {
                file: vp.clone(),
                mode: FileStreamMode::WriteOnly,
                pointer_pos: 0,
            })),
            Err(e) => Err(e),
        }
    }

    fn open_file_stream(&mut self, path: &str) -> Result<Box<dyn FileStream>, io::Error> {
        match self.get(path) {
            Some(vp) => match &*(*vp).borrow() {
                VirtualPath::File { name, data, .. } => Ok(Box::new(VirtualFileStream {
                    file: vp.clone(),
                    mode: FileStreamMode::ReadOnly,
                    pointer_pos: 0,
                })),
                VirtualPath::Dir { name, children, .. } => Err(io::Error::new(
                    io::ErrorKind::InvalidInput {},
                    "Expected a directory path, found a directory path",
                )),
            },
            None => Err(io::Error::new(io::ErrorKind::NotFound {}, "File not found")),
        }
    }

    ///Removes a file from the VirtualFileSystem.
    /// This should never be called if there are active file streams on this file, but it won't stop you
    fn remove_file(&mut self, path: &str) -> Result<(), io::Error> {
        let path_segments = split_path_str(path);

        match self.get(path) {
            Some(file) => {
                let parent = get_parent(&file).unwrap();
                let file_name = get_name(&file);

                let parent_ref = &mut *parent.borrow_mut();
                //Find the index of the file in the parent's list
                match parent_ref {
                    VirtualPath::File { .. } => Err(io::Error::new(
                        io::ErrorKind::Other {},
                        "Parent of path was a file",
                    )),
                    VirtualPath::Dir { children, .. } => {
                        for i in 0..children.len() {
                            //There should never be multiple files of the same name in a directory, so stop looking for more after removing one.
                            if get_name(&children[i]) == file_name {
                                children.remove(i);
                                return Ok(());
                            }
                        }

                        Err(io::Error::from(io::ErrorKind::NotFound {}))
                    }
                }
            }
            None => Err(io::Error::from(io::ErrorKind::NotFound {})),
        }
    }
}

//A path can either be a directory or a file
#[derive(Clone)]
enum VirtualPath {
    File {
        name: String,
        data: Rc<RefCell<Vec<u8>>>, //EOF is considered reached once the end of 'data' has been reached
        parent: Rc<RefCell<VirtualPath>>, //A file always has a parent (the file system can't just be a single file)
    },
    Dir {
        name: String,
        children: Vec<Rc<RefCell<VirtualPath>>>,
        parent: Option<Rc<RefCell<VirtualPath>>>, //A directory doesn't always have a parent
    },
}

//Default implementation but without serializing the parent
impl Debug for VirtualPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::File { name, data, parent } => f
                .debug_struct("File")
                .field("name", name)
                .field("data", data)
                .finish(),
            Self::Dir {
                name,
                children,
                parent,
            } => f
                .debug_struct("Dir")
                .field("name", name)
                .field("children", children)
                .finish(),
        }
    }
}

#[derive(PartialEq)]
enum FileStreamMode {
    WriteOnly,
    ReadOnly,
}

pub struct VirtualFileStream {
    file: Rc<RefCell<VirtualPath>>,
    mode: FileStreamMode,
    pointer_pos: usize,
}

impl VirtualFileStream {
    fn get_data(&self) -> Rc<RefCell<Vec<u8>>> {
        match &*(*self.file).borrow() {
            VirtualPath::File { data, .. } => data.clone(),
            VirtualPath::Dir {
                name,
                ..
            } => panic!("VirtualFileStream created which references directory, not file. Directory name: {}", name),
        }
    }
}

impl io::Read for VirtualFileStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        //If the stream can't read, return error
        if self.mode != FileStreamMode::ReadOnly {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "Permission denied. Tried reading from write-only file stream",
            ));
        }

        //Get a reference to the underlying file data
        let data = self.get_data();
        let data = &*(*data).borrow();

        for i in 0..buf.len() {
            //If the pointer has reached EOF, return number of bytes written
            if self.pointer_pos >= data.len() {
                return Ok(i);
            }

            buf[i] = data[self.pointer_pos];

            self.pointer_pos += 1;
        }

        //If there was no premature return, then all bytes were written to the buffer
        Ok(buf.len())
    }
}

impl io::Write for VirtualFileStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        //TMP
        println!("{:#?}", self.file);

        //If the stream can't read, return error
        if self.mode != FileStreamMode::WriteOnly {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "Permission denied. Tried writing from read-only file stream",
            ));
        }

        //Get a mutable reference to the underlying file data
        let data = self.get_data();
        let data = &mut *(*data).borrow_mut();

        for i in 0..buf.len() {
            data.push(buf[i]);
        }

        //Move pointer to the current end of file, the pointer is where the next write will take place
        self.pointer_pos = data.len();

        Ok(buf.len())
    }

    //Currently, does nothing
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl FileStream for VirtualFileStream {}
