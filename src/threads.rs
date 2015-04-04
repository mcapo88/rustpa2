//this is an example of multithreading that works, I don't really 
//know how to apply it to our project but maybe it will help somehow

use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread;

static NTHREADS: usize = 3;

fn main() {
    let (tx, rx) = channel();
    
    for id in 0..NTHREADS {
        let thread_tx = tx.clone();
        thread::spawn(move || {
            thread_tx.send(id).unwrap();
            
            println!("thread {} finished", id);
        });
    }
    
    let mut ids = Vec::with_capacity(NTHREADS);
    for _ in 0..NTHREADS{
        ids.push(rx.recv());
    }
    
    println!("{:?}", ids);
}
