use anyhow::Result;
use tokio::net::TcpStream;

#[derive(Debug, Clone)]
pub struct TcpParams {
    pub nodelay: bool,

    pub recv_buf_size: Option<u32>,

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
    pub fn apply(&self, stream: &TcpStream) -> Result<()> {
        stream.set_nodelay(self.nodelay)?;

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
fn set_sock_opt_u32(
    fd: std::os::unix::io::RawFd,
    level: libc::c_int,
    opt: libc::c_int,
    val: u32,
) -> Result<()> {
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
            level,
            opt,
            val,
            std::io::Error::last_os_error()
        ));
    }
    Ok(())
}
