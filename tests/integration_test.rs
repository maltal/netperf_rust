use std::net::TcpStream;
use std::process::{Child, Command};
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

static TEST_MUTEX: Mutex<()> = Mutex::new(());

struct NetserverRust {
    child: Child,
}

impl NetserverRust {
    const BIND_PORT: u16 = 12865;
    const MAX_ATTEMPTS: usize = 40;
    const POLL_DELAY_MS: u64 = 250;
    const READY_GRACE_MS: u64 = 200;

    fn wait_for_bind(child: &mut Child) {
        for _ in 0..Self::MAX_ATTEMPTS {
            if let Ok(Some(status)) = child.try_wait() {
                panic!("Netserver_rust exited early with status: {}", status);
            }

            if Self::is_listening() {
                thread::sleep(Duration::from_millis(Self::READY_GRACE_MS));
                return;
            }

            thread::sleep(Duration::from_millis(Self::POLL_DELAY_MS));
        }

        let _ = child.kill();
        panic!("Netserver_rust failed to start after 10 seconds");
    }

    fn is_listening() -> bool {
        TcpStream::connect(format!("localhost:{}", Self::BIND_PORT)).is_ok()
    }

    fn start() -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_netserver_rust"))
            .spawn()
            .expect("Failed to start netserver_rust");

        Self::wait_for_bind(&mut child);
        NetserverRust { child }
    }
}

impl Drop for NetserverRust {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        // Give some time for the OS to release ports
        thread::sleep(Duration::from_millis(100));
    }
}

fn run_netperf(args: &[&str]) {
    let output = Command::new(env!("CARGO_BIN_EXE_netperf_rust"))
        .args(args)
        .output()
        .expect("Failed to run netperf");

    assert!(
        output.status.success(),
        "netperf failed with args {:?}: {}",
        args,
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_tcp_stream_v4() {
    let _guard = match TEST_MUTEX.lock() {
        Ok(g) => g,
        Err(e) => e.into_inner(),
    };
    let _server = NetserverRust::start();
    run_netperf(&["-H", "localhost", "-l", "1", "-t", "TCP_STREAM"]);
}

#[test]
fn test_tcp_stream_v6() {
    let _guard = match TEST_MUTEX.lock() {
        Ok(g) => g,
        Err(e) => e.into_inner(),
    };
    let _server = NetserverRust::start();
    run_netperf(&["-H", "localhost", "-6", "-l", "1", "-t", "TCP_STREAM"]);
}

#[test]
fn test_tcp_rr_options() {
    let _guard = match TEST_MUTEX.lock() {
        Ok(g) => g,
        Err(e) => e.into_inner(),
    };
    let _server = NetserverRust::start();
    run_netperf(&[
        "-H",
        "localhost",
        "-l",
        "1",
        "-t",
        "TCP_RR",
        "-s",
        "65536",
        "-S",
        "65536",
        "-D",
    ]);
}

#[test]
fn test_udp_stream() {
    let _guard = match TEST_MUTEX.lock() {
        Ok(g) => g,
        Err(e) => e.into_inner(),
    };
    let _server = NetserverRust::start();
    run_netperf(&["-H", "localhost", "-l", "1", "-t", "UDP_STREAM"]);
}

#[test]
fn test_tcp_cc() {
    let _guard = match TEST_MUTEX.lock() {
        Ok(g) => g,
        Err(e) => e.into_inner(),
    };
    let _server = NetserverRust::start();
    run_netperf(&["-H", "localhost", "-l", "1", "-t", "TCP_CC"]);
}
