use anyhow::{Result, anyhow};
use std::fs::{File, OpenOptions};
use std::io::Write as _;
use std::path::{Path, PathBuf};

#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt as _;

/// `InstanceLock` ensures that only one instance of the daemon is running at a time.
pub struct InstanceLock {
    lock_file_path: PathBuf,
    #[cfg(unix)]
    _lock_file: File,
    #[cfg(windows)]
    _lock_file: File,
}

impl InstanceLock {
    /// The path where the lock file will be stored.
    #[cfg(unix)]
    const LOCK_FILE_PATH: &'static str = "/tmp/dball-daemon.lock";

    #[cfg(windows)]
    const LOCK_FILE_PATH: &'static str = r"C:\temp\dball-daemon.lock";

    /// Acquires an instance lock, ensuring that only one instance of the daemon is running at a time.
    pub async fn acquire() -> Result<Self> {
        let lock_file_path = PathBuf::from(Self::LOCK_FILE_PATH);

        // Check if an existing lock file exists
        if lock_file_path.exists() {
            Self::check_existing_lock(&lock_file_path).await?;
        }

        // Create the lock file
        let lock_file = Self::create_lock_file(&lock_file_path)?;

        log::info!("Acquired instance lock at {}", lock_file_path.display());

        Ok(Self {
            lock_file_path,
            _lock_file: lock_file,
        })
    }

    /// Check if an existing lock file exists
    /// and if the process it refers to is still running.
    async fn check_existing_lock(lock_file_path: &Path) -> Result<()> {
        // Read the PID from the lock file
        let contents = std::fs::read_to_string(lock_file_path)?;

        if let Ok(pid) = contents.trim().parse::<u32>() {
            // check if the process is still running
            if Self::is_process_running(pid) {
                return Err(anyhow!(
                    "Another daemon instance is already running with PID: {}",
                    pid
                ));
            } else {
                log::warn!("Found stale lock file with PID: {pid}, removing...");
                std::fs::remove_file(lock_file_path)?;
            }
        } else {
            log::warn!("Found invalid lock file, removing...");
            std::fs::remove_file(lock_file_path)?;
        }

        Ok(())
    }

    /// Create the lock file and write the current process ID to it.
    fn create_lock_file(lock_file_path: &Path) -> Result<File> {
        // Ensure the parent directory exists
        if let Some(parent) = lock_file_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut options = OpenOptions::new();
        options.create(true).write(true).truncate(true);

        #[cfg(unix)]
        options.mode(0o600); // Set permissions to read/write for owner only

        let mut file = options.open(lock_file_path)?;

        // Write the current process ID to the lock file
        let pid = std::process::id();
        write!(file, "{pid}")?;
        file.flush()?;

        Ok(file)
    }

    /// Check if a process with the given PID is running.
    fn is_process_running(pid: u32) -> bool {
        #[cfg(unix)]
        {
            // kill -0 used to check if a process exists
            // This does not actually send a signal, just checks existence
            use std::process::Command;

            // 使用kill -0命令检查进程是否存在
            match Command::new("kill").args(["-0", &pid.to_string()]).output() {
                Ok(output) => output.status.success(),
                Err(_) => false,
            }
        }

        #[cfg(windows)]
        {
            // 在Windows上，尝试打开进程句柄
            // TODO: 实现Windows进程检查
            let _ = pid; // 避免未使用警告
            log::warn!("Process existence check not implemented for Windows");
            false
        }

        #[cfg(not(any(unix, windows)))]
        {
            // 其他平台暂时假设进程不存在
            let _ = pid;
            log::warn!("Process existence check not implemented for this platform");
            false
        }
    }
    /// Get the path of the lock file.
    pub fn lock_file_path(&self) -> &Path {
        &self.lock_file_path
    }
}

impl Drop for InstanceLock {
    fn drop(&mut self) {
        // Remove the lock file when the instance lock is dropped
        if self.lock_file_path.exists() {
            if let Err(e) = std::fs::remove_file(&self.lock_file_path) {
                log::error!("Failed to remove lock file: {e}");
            } else {
                log::info!("Released instance lock");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_instance_lock() {
        // 创建一个临时锁文件路径用于测试
        let test_lock_path = "/tmp/dball-daemon-test.lock";

        // 确保测试开始前没有锁文件
        let _ = std::fs::remove_file(test_lock_path);

        // 这个测试需要修改InstanceLock来支持自定义路径
        // 目前先跳过实际的锁测试

        // 测试进程检查功能
        let current_pid = std::process::id();
        assert!(InstanceLock::is_process_running(current_pid));

        // 测试一个不存在的PID
        assert!(!InstanceLock::is_process_running(99999));
    }
}
