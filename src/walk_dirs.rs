use std::fs::DirEntry;
use std::path::Path;
use std::{collections::VecDeque, fs::ReadDir};
use std::{fs, io};

pub struct RecursiveDirIter {
    stack: VecDeque<ReadDir>,
}

impl RecursiveDirIter {
    pub fn new(root: &Path) -> io::Result<Self> {
        let mut stack = VecDeque::new();
        if root.is_dir() {
            stack.push_front(fs::read_dir(root)?);
        }
        Ok(Self { stack })
    }
}

impl Iterator for RecursiveDirIter {
    type Item = io::Result<DirEntry>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(top) = self.stack.front_mut() {
            match top.next() {
                Some(Ok(dir_entry)) => {
                    let path = dir_entry.path();
                    if path.is_dir() {
                        match fs::read_dir(&path) {
                            Ok(sub_dir) => self.stack.push_front(sub_dir),
                            Err(err) => return Some(Err(err)),
                        }
                    }
                    return Some(Ok(dir_entry));
                }
                Some(Err(err)) => return Some(Err(err)),
                None => {
                    self.stack.pop_front();
                }
            }
        }

        None
    }
}

pub fn walk_dirs(path: &Path) -> io::Result<RecursiveDirIter> {
    RecursiveDirIter::new(path)
}
