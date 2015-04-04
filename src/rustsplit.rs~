extern crate getopts;

use getopts::{optopt, getopts};
use std::old_io::BufferedReader;
use std::process::{Command, Stdio};
use std::old_io::stdin;
use std::{old_io, os};
use std::os::getcwd;
use std::str;

fn main(){
    let cwd = getcwd().unwrap();
        let mut cwd_string = "";
        match cwd.as_str(){
            None => panic!("path can't be converted to a string"),
            Some(s) => cwd_string = s,
        }
        println!("cwd_string is {}", cwd_string);
        let mut split = cwd_string.split_str("/");
        let vec = split.collect::<Vec<&str>>();
        let a = vec[1];
        println!("first is {}", a);
        let slash = "/";
        let mut path = format!("{}{}",a,slash);
        println!("path is {}", path);
        for x in 2..vec.len()-1 {
            path = format!("{}{}", path, vec[x]);
            path = format!("{}{}", path, slash);
            println!("path is {}", path);
        }
        let p = Path::new(path);
        os::change_dir(&p);
        let new_cwd = getcwd().unwrap();
        let mut new_cwd_string = "";
        match new_cwd.as_str(){
            None => panic!("Non"),
            Some(s) => new_cwd_string = s,
        }
        println!("new cwd is {}", new_cwd_string);
        
}
