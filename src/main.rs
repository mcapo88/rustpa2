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
use std::io::{Read, Write};
use std::{old_io, os};
use std::str;
use std::env;
use std::old_io::fs::PathExtensions; //this is for checking if file exists in cd
use std::collections::LinkedList;
use std::thread; //for spawning multiple threads of piped commands
use std::sync::mpsc::{channel, Sender, Receiver};
use std::os::getcwd;
const MSG_SIZE: usize = 128; //how big the buffer will be (bytes)
//the message struct is what we'll be passing from sender to receiver;
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
           
                let mut program = cmd_line.splitn(1, ' ').nth(0).expect("no program");
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
    //we probably don't need this
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
    
        let pipes: Vec<&str> = cmd_line.split('|').filter_map(|x| { //split command on pipe
            if x == "" {
                None
            } else {
                Some(x)
            }
        }).collect();
        let last = pipes[pipes.len()-1];
        //println!("last pipe is {}", last);
        let last_split = last.split(' ');
        let last_parts: Vec<&str> = last_split.collect();
        let last2 = last_parts[last_parts.len()-1];
        //println!("last symbol is {}", last2);
        let (tx, rx) = channel::<message>(); //initial channel
        tx.send(message{info:[0;MSG_SIZE], length:0, eof: true}); //send end of file 
        let mut old_rx: Receiver<message> = rx; //initialize previous receiver

        //in a for loop, make a channel for every pair of pipes
        for i in 0..pipes.len(){
            let (tx, rx) = channel::<message>();//declare new channel here
            let my_rx = old_rx;  //set my_rx to be the previous receiver
            old_rx = rx; //now set prev rcvr to current rcvr
            let pipe = pipes[i].to_string(); //spawn a thread for next command
            thread::spawn(move|| {
            let arg: Vec<&str> = pipe.split(' ').filter_map(|x| {
                    if x == "" {
                        None
                    } else {
                        Some(x)
                    }
            }).collect();
            let program = arg.first().unwrap();
            let argv = arg.tail();
            if Shell::cmd_exists(program) {
                let process = match Command::new(program).args(argv).stdin(Stdio::capture()).stdout(Stdio::capture()).stderr(Stdio::capture()).spawn() {
                    Err(why) => panic!("couldn't spawn {}: {}", program, why.description()),
                    Ok(process) => process,
                };
                {
                let mut stdin = process.stdin.unwrap();
                //receive stdout from the previous command
                loop{
                    let message = match my_rx.recv(){
                        Err(why) => {println!("error reading from recv {}", why);
                            break;},
                        Ok(num) => {num},
                    };
                    //if the message does not fill the whole buffer, only read in 
                    //the message, not the 0's.
                    match stdin.write(&message.info[0..message.length]) {
                        Err(why) => {break;},
                        Ok(hm) => {},
                    };

                    if message.eof{ break;}
                }
                }
                let mut stdout = process.stdout.unwrap();
                //send the output to the next command in the chain
                loop{ //reads byte sized blocks of given command thread 
                    let mut result: [u8; MSG_SIZE] = [0;MSG_SIZE];
                    let buffer_s = match stdout.read(&mut result) {
                        Err(why) => {panic!("couldn't read stdout {}", why.description()); },
                        Ok(num) => {if num == 0 { break;} num},
                    };
                    //println!("buff size is: {}", buffer_s);
                    let message = message{info: result, length: buffer_s, eof: buffer_s<MSG_SIZE};
                    tx.send(message);
                }
            } else {
                println!("{}: command not found", program);
            }});//end of thread
        }//end of for loop
        //get the output from the final command in the chain, then print it
        if last2 == "&" //If true run command in the background
        {
            thread::spawn(move|| {
                loop{
                    let message = match old_rx.recv(){
                        Err(why) => {println!("error reading from recv {}", why); break;},
                        Ok(num) => {num}
                    };
                    if message.eof{ break;}
                }
            });
            return;
        } else {
            loop{
                let message = match old_rx.recv(){
                    Err(why) => {println!("error reading from recv {}", why); break;},
                    Ok(num) => {num}
                };
                print!("{}", String::from_utf8_lossy(&message.info));
                if message.eof{ break;}
            }}
    }

    fn cmd_exists(cmd_path: &str) -> bool {
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
