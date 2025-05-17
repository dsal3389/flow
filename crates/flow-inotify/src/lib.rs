use std::io;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::os::fd::{RawFd, AsRawFd};
use std::task::Poll;
use std::ffi::CString;

use futures::Stream;
use inotify_sys as ffi;


pub struct InotifyEvent;

/// the inotify builder type is as the name suggest, is to
/// build inotify types that will monitor given paths
#[derive(Debug, Default)]
pub struct InotifyBuilder {
    entries: Vec<PathBuf>
}

impl InotifyBuilder {
    /// add path to the inotify for monitoring
    pub fn add_entry<P>(mut self, path: P) -> InotifyBuilder
    where
        P: AsRef<Path>
    {
        self.entries.push(path.as_ref().to_path_buf());
        self
    }

    /// build the inotify type and initialize it with the given entries
    pub fn build<const N: usize>(self) -> Result<Inotify<N>, Box<dyn std::error::Error>> {
        let fd = unsafe { ffi::inotify_init1(ffi::IN_NONBLOCK) };

        // create the inotify here is if we have error later, the inotify.drop
        // will be triggered
        let inotify = Inotify { fd, buffer: [0_u8; N] };

        // convert each entry path to a cstring and add
        // it to the event watch for the inotify
        for entry in self.entries {
            let pathname = CString::new(entry.as_os_str().as_bytes())?;
            unsafe { ffi::inotify_add_watch(fd, pathname.as_ptr(), ffi::IN_CLOSE_WRITE); }
        }

        Ok(inotify)
    }
}

#[derive(Debug)]
pub struct Inotify<const N: usize> {
    fd: RawFd,
    buffer: [u8; N]
}

impl<const N: usize> Stream for Inotify<N> {
    type Item = io::Result<InotifyEvent>;
    fn poll_next(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Option<Self::Item>> {
        todo!()
    }
}

impl<const N: usize> AsRawFd for Inotify<N> {
    fn as_raw_fd(&self) -> RawFd {
        self.fd
    }
}

/// when dropping, free the inotify resources
impl<const N: usize> Drop for Inotify<N> {
    fn drop(&mut self) {
        unsafe { ffi::close(self.fd) };
    }
}

#[cfg(test)]
mod tests {

}
