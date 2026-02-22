use tokio::net::TcpStream;
use anyhow::Result;

/// Tunable TCP socket parameters applied to accepted/connected TCP streams.
///
/// These map directly to kernel socket options. All fields have defaults that
/// preserve the previous hard-coded behaviour, so existing callers are fully
/// backward-compatible.
#[derive(Debug, Clone)]
pub struct TcpParams {
    /// Disable Nagle's algorithm (TCP_NODELAY). Reduces latency for small writes
    /// at the cost of slightly higher packet count. Default: true.
    pub nodelay: bool,

    /// Socket-level receive buffer size in bytes (SO_RCVBUF).
    /// `None` means leave the kernel default unchanged.
    pub recv_buf_size: Option<u32>,

    /// Socket-level send buffer size in bytes (SO_SNDBUF).
    /// `None` means leave the kernel default unchanged.
    pub send_buf_size: Option<u32>,
}

impl Default for TcpParams {
    fn default() -> Self {
        Self {
            nodelay: true,
            recv_buf_size: None,
            send_buf_size: None,
        }
    }
}

impl TcpParams {
    /// Apply these parameters to a connected or accepted `TcpStream`.
    pub fn apply(&self, stream: &TcpStream) -> Result<()> {
        stream.set_nodelay(self.nodelay)?;

        // SO_RCVBUF / SO_SNDBUF are set via the underlying std socket.
        use std::os::unix::io::AsRawFd;
        let fd = stream.as_raw_fd();

        if let Some(size) = self.recv_buf_size {
            set_sock_opt_u32(fd, libc::SOL_SOCKET, libc::SO_RCVBUF, size)?;
        }
        if let Some(size) = self.send_buf_size {
            set_sock_opt_u32(fd, libc::SOL_SOCKET, libc::SO_SNDBUF, size)?;
        }

        Ok(())
    }
}

#[cfg(unix)]
fn set_sock_opt_u32(fd: std::os::unix::io::RawFd, level: libc::c_int, opt: libc::c_int, val: u32) -> Result<()> {
    let val = val as libc::c_int;
    let ret = unsafe {
        libc::setsockopt(
            fd,
            level,
            opt,
            &val as *const _ as *const libc::c_void,
            std::mem::size_of::<libc::c_int>() as libc::socklen_t,
        )
    };
    if ret != 0 {
        return Err(anyhow::anyhow!(
            "setsockopt(level={}, opt={}, val={}) failed: {}",
            level, opt, val, std::io::Error::last_os_error()
        ));
    }
    Ok(())
}
