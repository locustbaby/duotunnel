use anyhow::Result;
use tokio::net::TcpStream;

#[derive(Debug, Clone)]
pub struct TcpParams {
    pub nodelay: bool,

    pub recv_buf_size: Option<u32>,

    pub send_buf_size: Option<u32>,

    /// SO_KEEPALIVE: emit keepalive probes on idle connections.
    /// Enables detection of silently-dead peers (e.g. NAT timeout, host crash).
    pub keepalive: bool,

    /// TCP_USER_TIMEOUT (Linux only): milliseconds to wait for unacknowledged
    /// data before the kernel closes the connection.  Prevents connections
    /// from hanging for ~15 min after a network partition.
    /// 0 = disabled (use OS default).
    pub user_timeout_ms: u32,
}

impl Default for TcpParams {
    fn default() -> Self {
        Self {
            nodelay: true,
            // 4 MB socket buffers — matches a 10GbE link at ~40ms RTT (BDP).
            // The kernel doubles the value internally (Linux doubles SO_RCVBUF),
            // so the effective window is ~8 MB, enough for high-throughput
            // long-distance links without manual tuning.
            recv_buf_size: Some(4 * 1024 * 1024),
            send_buf_size: Some(4 * 1024 * 1024),
            keepalive: true,
            // 30 s — aggressive enough to catch dead upstreams quickly.
            user_timeout_ms: 30_000,
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

        if self.keepalive {
            set_sock_opt_u32(fd, libc::SOL_SOCKET, libc::SO_KEEPALIVE, 1)?;
        }

        #[cfg(target_os = "linux")]
        if self.user_timeout_ms > 0 {
            set_sock_opt_u32(
                fd,
                libc::IPPROTO_TCP,
                libc::TCP_USER_TIMEOUT,
                self.user_timeout_ms,
            )?;
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
