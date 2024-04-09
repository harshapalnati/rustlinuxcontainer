extern crate nix;

              
extern crate libc;

            

            

              
use nix::mount::{mount,  MsFlags};

              
use nix::sched::{clone, unshare, CloneFlags};

              
use nix::sys::wait::{waitpid, WaitPidFlag};

              
use nix::unistd::{execvp, chroot};

              
use nix::sys::signal::{self, Signal};

use std::ffi::CString;

              
use std::os::unix::prelude::AsRawFd;

              
//use std::os::unix::io::FromRawFd;

              
use std::fs;

              
//use std::os::unix::fs::symlink;

              
use std::process;

fn network_namespace() {

              
    // Create a new network namespace

              
    match unsafe { unshare(CloneFlags::CLONE_NEWNET) } {

              
        Ok(_) => {

              
            // Perform network-related configuration within the new network namespace

            

            

              
            println!("We are in the new network namespace!");

              
        }

        Err(err) => eprintln!("Failed to create new network namespace: {:?}", err),

              
    }

              
}

fn pid_namespace() {

              
    // Create a new PID namespace

              
    match unsafe { unshare(CloneFlags::CLONE_NEWPID) } {

              
        Ok(_) => {

              
            // Perform PID-related configuration within the new PID namespace

            

            

              
            println!("We are in the new PID namespace!");

              
        }

              
        Err(err) => eprintln!("Failed to create new PID namespace: {:?}", err),

              
    }

              
}

fn mount_namespace() {

              
    // Create a new mount namespace

              
    match unsafe { unshare(CloneFlags::CLONE_NEWNS) } {

              
        Ok(_) => {

              
            // Create a new directory to be used as the new root

              
            fs::create_dir_all("/tmp/newroot").expect("Failed to create /tmp/newroot directory");

            

            

              
            // Make the mounts in the new mount namespace private

              
            mount(

              
                None::<&str>,

              
                "/",

              
                None::<&str>,

              
                MsFlags::MS_PRIVATE | MsFlags::MS_REC,

              
                None::<&str>,

              
            )

              
            .expect("Failed to make mounts private");

            

            

              
            // Mount the /proc file system as private within the new mount namespace

              
            mount::<str, str, str, str>(

              
                Some("proc"),

              
                "/proc",

              
                Some("proc"),

              
                MsFlags::MS_PRIVATE,

              
                None::<&str>,

              
            )

              
            .expect("Failed to mount /proc");

            

            

              
            // Set the new root as the current root

              
            chroot("/").expect("Failed to change root");

            

            

              
            println!("We are in the new mount namespace!");

              
        }

              
        Err(err) => eprintln!("Failed to create new mount namespace: {:?}", err),

              
    }

              
}

            

            

              
fn child_function() -> isize {

              
    network_namespace();

              
    pid_namespace();

              
    mount_namespace();

            

            

              
    // Execute an interactive shell within the namespace

              
    let program = CString::new("/bin/sh").unwrap();

              
    let args = [

              
        CString::new("/bin/sh").unwrap(),

              
    ];

            

            

              
    execvp(&program, &args).expect("Failed to execute program");

            

            

              
    // The execvp call replaces the current process, so this line should not be reached

              
    println!("Execvp failed!");

            

            

              
    // Exit the child process

              
    process::exit(1);

              
}

            

            

              
fn main() {

              
    let stack_size = 1024 * 1024; // 1MB stack size for the child process

              
    let mut child_stack = vec![0; stack_size];

            

            

              
    let flags = CloneFlags::CLONE_NEWNET | CloneFlags::CLONE_NEWPID | CloneFlags::CLONE_NEWNS;

            

            

              
    match unsafe {

              
        let child_stack_ptr = child_stack.as_mut_ptr();

              
        let child_stack_slice = std::slice::from_raw_parts_mut(child_stack_ptr, stack_size);

            

            

              
        clone(

              
            Box::new(child_function),

              
            child_stack_slice,

              
            flags,

              
            None,

              
        )

              
    } {

              
        Ok(child_pid) => {

              
            if child_pid == nix::unistd::Pid::from_raw(0) {

              
                // Parent process

              
                waitpid(child_pid, Some(WaitPidFlag::empty())).expect("Failed to wait for child process");

              
                println!("Child process terminated");

              
            } else {

              
                // Child process

              
                // Set up signal handling for SIGCHLD

              
                unsafe {

              
                    signal::signal(Signal::SIGCHLD, signal::SigHandler::SigIgn)

              
                        .expect("Failed to set SIGCHLD handler");

              
                }

              
            }

              
        }

              
        Err(err) => eprintln!("Failed to create new process: {:?}", err),

              
    }

              
}
