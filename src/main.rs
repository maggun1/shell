use std::env;
use std::io::{self, Write};
use std::os::unix::process::CommandExt;
use std::process::Command;
use libc::{kill, SIGTERM, pid_t, fork, waitpid};
use procfs::process::all_processes;

fn main() {
    loop {
        print!("shell ~> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        if input == "\\quit" {
            break;
        }

        let args: Vec<&str> = input.split_whitespace().collect();
        if args.is_empty() {
            continue;
        }

        match args[0] {
            "cd" => {
                match args.len() {
                    1 => {
                        match env::var_os("HOME") {
                            Some(home_dir) => {
                                if let Err(e) = env::set_current_dir(home_dir) {
                                    eprintln!("cd: {}", e);
                                }
                            }
                            None => eprintln!("cd: HOME not set"),
                        }
                    }

                    2 => {
                        let new_dir = args[1];
                        if let Err(e) = env::set_current_dir(new_dir) {
                            eprintln!("cd: {}", e);
                        }
                    }

                    _ => {
                        eprintln!("USAGE: cd <dir>");
                    }
                }
            }

            "pwd" => {
                match env::current_dir() {
                    Ok(path) => println!("{}", path.display()),
                    Err(e) => eprintln!("pwd: {}", e),
                }
            }

            "echo" => {
                match args.len() {
                    1 => println!(),
                    _ => println!("{}", args[1..].join(" ")),
                }
            }

            "kill" => {
                match args.len() {
                    2 => {
                        let pid: pid_t = args[1].parse().expect("Invalid PID");
                        unsafe {
                            if kill(pid, SIGTERM) == -1 {
                                eprintln!("Failed to send signal to process with PID: {}", pid);
                            } else {
                                println!("Sent SIGTERM to process with PID: {}", pid);
                            }
                        }
                    }
                    _ => {
                        eprintln!("USAGE: kill <pid>");
                    }
                }
            }

            "ps" => {
                let processes = all_processes().expect("Failed to get processes");

                println!("{:<10} {:<50} {:<15}", "PID", "COMMAND", "TIME");

                for process in processes {
                    if let Ok(proc) = process {
                        if let Ok(stat) = proc.stat() {
                            let pid = stat.pid;
                            let command = stat.comm;

                            let hours = stat.utime / 3600;
                            let minutes = (stat.utime % 3600) / 60;
                            let seconds = stat.utime % 60;
                            println!(
                                "{:<10} {:<50} {:02}:{:02}:{:02}",
                                pid, command, hours, minutes, seconds
                            );
                        }
                    }
                }
            }

            "exec" => {
                if args.len() < 2 {
                    eprintln!("Usage: exec <program> [args...]");
                }
                else {
                    let program = args[1];
                    let program_args = &args[2..];

                    let error = Command::new(program)
                        .args(program_args)
                        .exec();

                    eprintln!("Failed to execute {}: {}", program, error);
                }
            }

            "fork" => {
                match args.len() {
                    ..2 => {
                        eprintln!("USAGE: fork <program> [args...]");
                    }
                    _ => unsafe {
                        let program = args[1];
                        let program_args = &args[2..];

                        let pid = fork();
                        match pid {
                            0 => {
                                let error = Command::new(program)
                                    .args(program_args)
                                    .exec();

                                eprintln!("Failed to execute {}: {}", program, error);
                            }
                            -1 => {
                                eprintln!("Fork failed");
                            }
                            _ => {
                                waitpid(pid, std::ptr::null_mut(), 0);
                            }
                        }
                    }
                }
            }

            _ => {
                eprintln!("command not found: {}", args[0]);
            }
        }
    }
}
