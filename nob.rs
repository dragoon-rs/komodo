use std::process::{exit, Command, Stdio};

fn _run_cmd(cmd: &str, args: &[&str]) -> i32 {
    let mut cmd = Command::new(cmd)
        .args(args)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap();

    cmd.wait().unwrap().code().unwrap()
}

macro_rules! run_cmd {
    ($cmd:expr $(, $args:expr)*) => {{
        _run_cmd($cmd, &[$($args),*])
    }};
}

fn main() {
    if _run_cmd("cargo", &["test", "linalg"]) != 0 {
        exit(1);
    }
    if run_cmd!("cargo", "test", "linalg") != 0 {
        exit(1);
    }
}
