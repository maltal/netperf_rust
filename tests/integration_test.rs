use std::process::{Command, Child};
use std::thread;
use std::time::Duration;
use std::net::TcpStream;
use std::sync::Mutex;

static TEST_MUTEX: Mutex<()> = Mutex::new(());

struct Netserver {
    child: Child,
}

impl Netserver {
    const BIND_PORT: u16 = 12865;
    const MAX_ATTEMPTS: usize = 40;
    const POLL_DELAY_MS: u64 = 250;
    const READY_GRACE_MS: u64 = 200;
    const LOOPBACK_ADDRS: [&'static str; 2] = ["127.0.0.1", "[::1]"];

    fn wait_for_bind(child: &mut Child) {
        for _ in 0..Self::MAX_ATTEMPTS {
            if let Ok(Some(status)) = child.try_wait() {
                panic!("Netserver exited early with status: {}", status);
            }

            if Self::is_listening() {
                thread::sleep(Duration::from_millis(Self::READY_GRACE_MS));
                return;
            }

            thread::sleep(Duration::from_millis(Self::POLL_DELAY_MS));
        }

        let _ = child.kill();
        panic!("Netserver failed to start after 10 seconds");
    }

    fn is_listening() -> bool {
        Self::LOOPBACK_ADDRS.iter().any(|addr| {
            TcpStream::connect(format!("{}:{}", addr, Self::BIND_PORT)).is_ok()
        })
    }

    fn start() -> Self {
        let mut child = Command::new("./target/debug/netserver_rust")
            .spawn()
            .expect("Failed to start netserver");

        Self::wait_for_bind(&mut child);
        Netserver { child }
    }
}

impl Drop for Netserver {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        // Give some time for the OS to release ports
        thread::sleep(Duration::from_millis(500));
    }
}

fn run_netperf(args: &[&str]) {
    let output = Command::new("./target/debug/netperf_rust")
        .args(args)
        .output()
        .expect("Failed to run netperf");
    
    assert!(output.status.success(), "netperf failed with args {:?}: {}", args, String::from_utf8_lossy(&output.stderr));
}

#[test]
fn test_tcp_stream_v4() {
    let _guard = match TEST_MUTEX.lock() {
        Ok(g) => g,
        Err(e) => e.into_inner(),
    };
    let _server = Netserver::start();
    run_netperf(&["-H", "127.0.0.1", "-l", "1", "-t", "TCP_STREAM"]);
}

#[test]
fn test_tcp_stream_v6() {
    let _guard = match TEST_MUTEX.lock() {
        Ok(g) => g,
        Err(e) => e.into_inner(),
    };
    let _server = Netserver::start();
    run_netperf(&["-H", "::1", "-6", "-l", "1", "-t", "TCP_STREAM"]);
}

#[test]
fn test_tcp_rr_options() {
    let _guard = match TEST_MUTEX.lock() {
        Ok(g) => g,
        Err(e) => e.into_inner(),
    };
    let _server = Netserver::start();
    run_netperf(&["-H", "127.0.0.1", "-l", "1", "-t", "TCP_RR", "-s", "65536", "-S", "65536", "-D"]);
}

#[test]
fn test_udp_stream() {
    let _guard = match TEST_MUTEX.lock() {
        Ok(g) => g,
        Err(e) => e.into_inner(),
    };
    let _server = Netserver::start();
    run_netperf(&["-H", "127.0.0.1", "-l", "1", "-t", "UDP_STREAM"]);
}

#[test]
fn test_tcp_cc() {
    let _guard = match TEST_MUTEX.lock() {
        Ok(g) => g,
        Err(e) => e.into_inner(),
    };
    let _server = Netserver::start();
    run_netperf(&["-H", "127.0.0.1", "-l", "1", "-t", "TCP_CC"]);
}
