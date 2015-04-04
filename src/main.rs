// from pa2-latest
// gash.rs
//
// Starting code for PA2
// Running on Rust 1.0.0 - build 02-21
//
// Brandeis University - cs146a - Spring 2015

extern crate getopts;

use getopts::{optopt, getopts};
use std::old_io::BufferedReader;
use std::process::{Command, Stdio};
use std::old_io::stdin;
use std::{old_io, os};
use std::str;
use std::env;
use std::old_io::fs::PathExtensions; //this is for checking if file exists in cd
use std::collections::LinkedList;
use std::thread; //for spawning multiple threads of piped commands
use std::sync::mpsc::{channel, Sender, Receiver};
use std::os::getcwd;
const MSG_SIZE: usize = 128; //how big the buffer will be (bytes)
//the message struct is what we'll be passing from sender to receiver
struct message{
    info: [u8;MSG_SIZE],
    length: usize,
    eof: bool
}
struct Shell<'a> {
    cmd_prompt: &'a str,
}

impl <'a>Shell<'a> {
    fn new(prompt_str: &'a str) -> Shell<'a> {
        Shell { cmd_prompt: prompt_str }
    }

    fn run(&self) {
        let mut stdin = BufferedReader::new(stdin());
        let mut a = LinkedList::<String>::new();
        loop {
            old_io::stdio::print(self.cmd_prompt.as_slice());
            old_io::stdio::flush();
            let x = stdin.read_line();
            let line = x.unwrap();
            let cmd_line = line.trim();
            a.push_back(cmd_line.to_string());
            //let program = cmd_line.splitn(1, ' ').nth(0).expect("no program");
            let command_split = cmd_line.split_str("|");
            let command_vec = command_split.collect::<Vec<&str>>();
            for i in 0..command_vec.len()
            {
                let full_cmd = command_vec[i].trim();
                let copy = &full_cmd; //has to do with lifetime...
                let mut program = full_cmd.splitn(1, ' ').nth(0).expect("no program");
                //println!("program is {}", program);
                //start a thread for the command
                
                match program {
                    ""      =>  { continue; }
                    "exit"  =>  { return; }
                    "cd"    =>  {self.cd(cmd_line);}
                    "history" => {self.history(&a);}
                     _       => {self.run_cmdline(cmd_line);}
                }   
            }
        }
    }
    fn history(&self, a: &LinkedList<String>)
    {
        for e in a.iter()
        {
            println!("{}", e);
        }
    }
    //Mark-- this function is what I've added, cd works!
    fn cd(&self, cmd_line: &str)
    {
        let cmd_split = cmd_line.split_str(" ");
        let cmd_vec = cmd_split.collect::<Vec<&str>>();
        if cmd_vec.len() < 2 //means just cd typed in, nothing following
        {
            //I think this should go to root... I'm pretty sure that's /home
            let root_path = "/home";
            let p = Path::new(root_path);
            assert!(env::set_current_dir(&p).is_ok());
            println!("{}", p.display()); 
        }
        else
        {
            let following = cmd_line.splitn(2, ' ').nth(1).expect("no program");
        
            let cwd = getcwd().unwrap();
            let mut cwd_string = "";
            match cwd.as_str()
            {
                None => panic!("path can't be converted to a string"),
                Some(s) => cwd_string = s,
            }
            //if user said cd . then stay in current working directory
            if following == "."
            {
                println!("stay in current directory.");
            }
            else if following == ".."
            {
                let mut split = cwd_string.split_str("/");
                let vec = split.collect::<Vec<&str>>();
                let a = vec[1];
                let slash = "/";
                let mut path = format!("{}{}",slash,a);
                //want path NOT to have whatever was after the last /
                for x in 2..vec.len()-1 
                {
                    path = format!("{}{}", path, slash);
                    path = format!("{}{}", path, vec[x]);
                }
                //set path to be one up in the directory structure
                let p = Path::new(path);
                assert!(env::set_current_dir(&p).is_ok());
                println!("{}", p.display()); 
            }
            else{
                let slash = "/";
                let mut path = format!("{}{}", cwd_string, slash);
                path = format!("{}{}",path, following);
                let p = Path::new(path);
                //if a valid file was sent with cd command, change current working directory to that
                //otherwise say that file doesn't exist and prompt again
                if p.exists(){
                  //println!("file exists!");
                  assert!(env::set_current_dir(&p).is_ok());
                  println!("{}", p.display());
                }
                else{
                    println!("gash: cd: {}: No such file or directory", following);
                }
            }
        }
    }
    fn run_cmdline(&self, cmd_line: &str) {
        let argv: Vec<&str> = cmd_line.split(' ').filter_map(|x| {
            if x == "" {
                None
            } else {
                Some(x)
            }
        }).collect();

        match argv.first() {
            Some(&program) => self.run_cmd(cmd_line),
            None => (),
        };
    }

    fn run_cmd(&self, cmd_line: &str) {
        let cm_line = cmd_line.clone();  //it says it cannot infer an appropriate lifetime for parameter 'a due to conflicting requirements...?
        //We want to split up the cmd_line by pipes
        let pipes: Vec<&str> = cm_line.split('|').filter_map(|x| {
            if x == "" {
                None
            } else {
                Some(x)
            }
        }).collect();
        let mut args: Vec<Vec<&str>> = Vec::new();
        let mut programs: Vec<&str> = Vec::new();
        let mut argvs: Vec<&[&str]> = Vec::new();
        //in a for loop, make a channel for every pair of pipes
        for i in 0..pipes.len(){
        //would declare channel here
        let (tx, rx) = channel::<message>();
        args[i] = pipes[i].split(' ').filter_map(|x| {
                    if x == "" {
                        None
                    } else {
                        Some(x)
                    }
                }).collect();
                programs[i] = args[i].first().unwrap();
                argvs[i] = args[i].tail();
        if self.cmd_exists(programs[i]) {
            thread::spawn(move|| {
            let output = Command::new(programs[i]).args(argvs[i]).output().unwrap_or_else(|e| {panic!("failed to execute process: {}", e)});
            let stderr=String::from_utf8_lossy(&output.stderr);
            let stdout=String::from_utf8_lossy(&output.stdout);
            if !"".eq(stdout.as_slice()) {
                print!("{}", stdout);
            }
            if !"".eq(stderr.as_slice()) {
                print!("{}", stderr);
            }
            //having some trouble getting stdout into buf (stdout is of type Cow<'_,str>?
            let buf:[u8;MSG_SIZE] = stdout;
            let msg = message{info:buf, length: stdout.len(), eof: false };
            loop{tx.send(msg).unwrap();};}); //end of thread
        } else {
            println!("{}: command not found", programs[i]);
        }
        loop{rx.recv();}
        }//end of for loop
    }

    fn cmd_exists(&self, cmd_path: &str) -> bool {
        Command::new("which").arg(cmd_path).stdout(Stdio::capture()).status().unwrap().success()
    }
}

fn get_cmdline_from_args() -> Option<String> {
    /* Begin processing program arguments and initiate the parameters. */
    let args = os::args();

    let opts = &[
        getopts::optopt("c", "", "", "")
    ];

    getopts::getopts(args.tail(), opts).unwrap().opt_str("c")
}

fn main() {
    let opt_cmd_line = get_cmdline_from_args();

    match opt_cmd_line {
        Some(cmd_line) => Shell::new("").run_cmdline(cmd_line.as_slice()),
        None           => Shell::new("gash > ").run(),
    }
}
