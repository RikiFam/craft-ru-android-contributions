// Android file locking adapter. Rust's std File locking is unsupported on
// Android; rustix exposes the platform flock through a safe descriptor API.
fn portable_try_lock(file: &File) -> Result<(), TryLockError> {
    #[cfg(target_os = "android")]
    {
        rustix::fs::flock(file, rustix::fs::FlockOperation::NonBlockingLockExclusive).map_err(|e| {
            let e = io::Error::from(e);
            if e.kind() == io::ErrorKind::WouldBlock {
                TryLockError::WouldBlock
            } else {
                TryLockError::Error(e)
            }
        })
    }
    #[cfg(not(target_os = "android"))]
    file.try_lock()
}
fn portable_unlock(file: &File) -> io::Result<()> {
    #[cfg(target_os = "android")]
    { rustix::fs::flock(file, rustix::fs::FlockOperation::Unlock).map_err(io::Error::from) }
    #[cfg(not(target_os = "android"))]
    file.unlock()
}
