use std::io::{Lines, Result};
use std::io::{BufRead, BufReader};
use std::fs::File;

pub trait Source: Iterator<Item = Result<String>> {

    fn size(&self) -> u64;
}

pub struct FileSource {
    len: u64,
    source: Lines<BufReader<File>>,
}

impl FileSource {
    pub fn new(path: &str) -> Result<Self> {
        let file = File::open(path)?;
        let len = file.metadata().unwrap().len();
        let reader = BufReader::new(file);
        Ok(FileSource {
            source: reader.lines(),
            len: len,
        })
    }
}

impl Source for FileSource {

    fn size(&self) -> u64 {
        self.len
    }
}

impl Iterator for FileSource {
    type Item = Result<String>;

    fn next(&mut self) -> Option<Self::Item> {
        self.source.next()
    }
}
