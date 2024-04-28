use super::{file::File, info::OpenFlags};

pub const STDIN: usize = 0;
pub const STDOUT: usize = 1;
pub const STDERR: usize = 2;

pub struct Stdin;
pub struct Stdout;
pub struct Stderr;


impl File for Stdout {
    fn read(&self, buf: &mut [u8], flags: OpenFlags) -> Option<usize>{
        None
    }

    fn write(&self, buf: &[u8], flags: OpenFlags) -> Option<usize>{
        if let Ok(data) = core::str::from_utf8(buf) {
            print!("{}", data);
            Some(buf.len())
        } else {
            None
        }
    }
}

impl File for Stdin {
    fn read(&self, buf: &mut [u8], flags: OpenFlags) -> Option<usize> {
        todo!()
    }

    fn write(&self, buf: &[u8], flags: OpenFlags) -> Option<usize> {
        None
    }
}

impl File for Stderr {
    fn read(&self, buf: &mut [u8], flags: OpenFlags) -> Option<usize> {
        todo!()
    }

    fn write(&self, buf: &[u8], flags: OpenFlags) -> Option<usize> {
        todo!()
    }
}