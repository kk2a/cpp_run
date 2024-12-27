use std::process::{Command, Stdio};
use std::fs::File;
use std::io::{Write, Read};
use std::env;
use std::time::Duration;
use wait_timeout::ChildExt;
use std::path::Path;

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
    // println!("source_path: {:?}", source_path);
    // println!("source_dir: {:?}", source_dir);
    // println!("source_cpp: {:?}", source_cpp);
    // let current_dir = env::current_dir().unwrap();
    // println!("current_dir: {:?}", current_dir);


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
        .spawn() // これで非同期処理
        .expect("Failed to execute command");

    if time_limited {
        let timeout = Duration::from_secs(TIME_LIMIT_SEC);
        if child.wait_timeout(timeout).unwrap().is_none() {
            child.kill().unwrap();
            let _ = child.wait().unwrap(); // プロセスが終了するのを待つ
            out_file.write_all(b"===== Time limit exceeded =====\n").expect("Failed to write time limit exceeded message to file");
        }
    } else {
        child.wait().expect("Failed to wait on child");
    }

    if let Some(mut stdout) = child.stdout.take() {
        let mut buffer = Vec::new();
        stdout.read_to_end(&mut buffer).expect("Failed to read stdout");
        out_file.write_all(b"===== Start of output message =====\n").expect("Failed to write output message to file");
        out_file.write_all(&buffer).expect("Failed to write to output file");
        out_file.write_all(b"\n===== End of output message =====\n").expect("Failed to write output message to file");
    }

    if let Some(mut stderr) = child.stderr.take() {
        let mut buffer = Vec::new();
        stderr.read_to_end(&mut buffer).expect("Failed to read stderr");
        out_file.write_all(b"===== Start of error message =====\n").expect("Failed to write error message to file");
        out_file.write_all(&buffer).expect("Failed to write to error file");
        out_file.write_all(b"\n===== End of error message =====\n").expect("Failed to write error message to file");
    }
}
