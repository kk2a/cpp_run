use std::process::{Command, Stdio};
use std::fs::File;
use std::io::{Write, Read};
use std::env;
use std::time::{Duration, Instant};
use wait_timeout::ChildExt;
use std::path::Path;
use std::thread;

const IN_PATH: &str = "in.txt";
const OUT_PATH: &str = "out.txt";
const TIME_LIMIT_SEC: u64 = 10;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <source_path>", args[0]);
        return;
    }

    let source_path = &args[1];
    let source_dir = Path::new(source_path).parent().unwrap();
    let source_cpp = Path::new(source_path).file_name().unwrap().to_str().unwrap();
    env::set_current_dir(&source_dir).expect("Failed to change directory");

    let out = source_cpp.split('.').collect::<Vec<&str>>()[0];
    let compile_args = vec!["-std=c++23", "-O2", "-Wall", "-Wextra", "-DKK2", "-fexec-charset=CP932", source_cpp, "-o", out, "-I/Users/include/"];
    let mut time_limited = false;

    if args.contains(&String::from("-t")) {
        time_limited = true;
    }

    let in_file = File::open(IN_PATH).expect("Failed to open input file");
    let mut out_file = File::create(OUT_PATH).expect("Failed to create output file");

    let compile_output = Command::new("g++")
        .args(&compile_args)
        .output()
        .expect("Failed to compile C++ program");

    if !compile_output.stderr.is_empty() {
        out_file.write_all(b"===== Start of compile error message =====\n").expect("Failed to write compile error message to file");
        out_file.write_all(&compile_output.stderr).expect("Failed to write compile errors to file");
        out_file.write_all(b"\n===== End of compile error message =====\n").expect("Failed to write compile error message to file");
    }

    if !compile_output.status.success() {
        return;
    }

    let mut child = Command::new("./".to_owned() + out)
        .stdin(Stdio::from(in_file))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to execute command");

    let stdout = child.stdout.take().expect("Failed to get stdout");
    let stderr = child.stderr.take().expect("Failed to get stderr");

    let stdout_thread = thread::spawn(move || {
        let mut buffer = Vec::new();
        let mut reader = stdout;
        reader.read_to_end(&mut buffer).expect("Failed to read stdout");
        buffer
    });

    let stderr_thread = thread::spawn(move || {
        let mut buffer = Vec::new();
        let mut reader = stderr;
        reader.read_to_end(&mut buffer).expect("Failed to read stderr");
        buffer
    });

    let start = Instant::now();

    if time_limited {
        let timeout = Duration::from_secs(TIME_LIMIT_SEC);
        if child.wait_timeout(timeout).unwrap().is_none() {
            child.kill().unwrap();
            out_file.write_all(b"===== Time limit exceeded =====\n").expect("Failed to write time limit exceeded message to file");
        }
    } else {
        child.wait().expect("Failed to wait on child");
    }

    let duration = start.elapsed();

    // exit status
    if let Some(exit_status) = child.wait().ok().and_then(|status| status.code()) {
        out_file.write_all(format!("exit status: {}\n", exit_status).as_bytes()).expect("Failed to write exit status to file");
    }

    out_file.write_all(format!("execution time: {:.2?}\n", duration).as_bytes()).expect("Failed to write execution time to file");

    let stdout_buffer = stdout_thread.join().expect("Failed to join stdout thread");
    out_file.write_all(b"===== Start of output message =====\n").expect("Failed to write output message to file");
    out_file.write_all(&stdout_buffer).expect("Failed to write to output file");
    out_file.write_all(b"\n===== End of output message =====\n").expect("Failed to write output message to file");

    let stderr_buffer = stderr_thread.join().expect("Failed to join stderr thread");
    out_file.write_all(b"===== Start of error message =====\n").expect("Failed to write error message to file");
    out_file.write_all(&stderr_buffer).expect("Failed to write to error file");
    out_file.write_all(b"\n===== End of error message =====\n").expect("Failed to write error message to file");
}
