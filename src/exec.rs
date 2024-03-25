use log::debug;

use std::process::{Command, Stdio};

use std::io::Read;
use std::{thread, time};

pub struct RemoteHostCall {
    pub private_key: Option<String>,
}

#[allow(dead_code)]
pub fn exec_command(
    program: &str,
    command: Vec<&str>,
    build_arg: &Vec<String>,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut exec_command = Command::new(program);
    exec_command.args(command);
    for build_arg_item in build_arg {
        let parts: Vec<&str> = build_arg_item.splitn(2, '=').collect();
        if parts.len() < 2 {
            panic!("Invalid build-arg without `=`: `{}`", build_arg_item)
        }
        exec_command.env(parts[0], parts[1]);
    }
    debug!(
        "Start command: {:?} envs:{:?} args:{:?}",
        program,
        &exec_command.get_envs(),
        &exec_command.get_args(),
    );
    let output = exec_command.output().expect("failed to execute process");

    let status = output.status;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    debug!(
        "Executed command: {:?} envs:{:?} args:{:?} {:?} {:?} {:?}",
        program,
        &exec_command.get_envs(),
        &exec_command.get_args(),
        status,
        stdout,
        stderr
    );
    if !status.success() {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!(
                "Command failed: {:?} envs:{:?} args:{:?} status:{:?} stdout:{:?} stderr:{:?}",
                program,
                &exec_command.get_envs(),
                &exec_command.get_args(),
                status,
                stdout,
                stderr
            ),
        )));
    }
    Ok(stdout)
}

pub fn exec_command_with_threaded_read(
    program: &str,
    command: Vec<&str>,
    build_arg: &Vec<String>,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut exec_command = Command::new(program);
    exec_command.args(command.clone());
    for build_arg_item in build_arg {
        let parts: Vec<&str> = build_arg_item.splitn(2, '=').collect();
        if parts.len() < 2 {
            panic!("Invalid build-arg without `=`: `{}`", build_arg_item)
        }
        exec_command.env(parts[0], parts[1]);
    }
    debug!(
        "Start command: {:?} {:?} envs:{:?} args:{:?}",
        program,
        &command,
        &exec_command.get_envs(),
        &exec_command.get_args(),
    );

    let mut process = exec_command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start echo process");

    let stdout = process.stdout.take().unwrap();
    let stderr = process.stderr.take().unwrap();
    std::thread::spawn(move || {
        let mut stdout = stdout;
        let mut stderr = stderr;
        let mut t = term::stdout().unwrap();
        debug!("out");
        let mut closing = false;
        loop {
            let mut buffer = [0; 1024];
            let n = stdout.read(&mut buffer).unwrap();
            if n == 0 {
                closing = true;
            }
            if !closing {
                let s = String::from_utf8_lossy(&buffer[..n]);

                t.cursor_up().expect("Failed to move cursor up.");
                t.delete_line().expect("Failed to delete cursor up.");

                debug!("out: {}", s.replace('\n', "\\n"));
            }

            let mut buffer = [0; 1024];
            let n = stderr.read(&mut buffer).unwrap();
            if closing && n == 0 {
                debug!("finish-finish");
                break;
            }
            let s = String::from_utf8_lossy(&buffer[..n]);

            t.cursor_up().expect("Failed to move cursor up.");
            t.delete_line().expect("Failed to delete cursor up.");

            debug!("err: {}", s.replace('\n', "\\n"));

            let ten_millis = time::Duration::from_millis(100);
            thread::sleep(ten_millis);
        }
    });

    loop {
        let st = process.try_wait()?;
        match st {
            None => debug!("still running"),
            Some(status) => debug!("exited with: {}", status),
        }
        if st.is_some() {
            break;
        }
        let ten_millis = time::Duration::from_millis(500);
        thread::sleep(ten_millis);
    }

    let output = exec_command.output().expect("failed to execute process");

    let status = output.status;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    debug!(
        "Executed command:{:?} args:{:?} status:{:?}",
        program, &command, status
    );
    if !status.success() {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!(
                "Command failed: {:?} envs:{:?} args:{:?} status:{:?} stdout:{:?} stderr:{:?}",
                program,
                &exec_command.get_envs(),
                &exec_command.get_args(),
                status,
                stdout,
                stderr
            ),
        )));
    }
    Ok("".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    fn test_exec_command_with_threaded_read() {
        let h = exec_command_with_threaded_read("echo", vec!["hello"], &vec![]).unwrap();
        assert!(true)
    }
}
