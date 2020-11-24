#![allow(dead_code)]
use std::sync::{mpsc, Arc, Mutex};
use std::time::Duration;

pub struct ThreadPool {
    sender: Option<mpsc::Sender<Box<dyn Fn() + Send>>>,
    thread_nums: u32,
    ch_done: mpsc::Receiver<()>,
}

impl ThreadPool {
    pub fn new(n: u32) -> Self {
        let (tx, rx) = mpsc::channel();
        // multi-receiver
        let mr = Arc::new(Mutex::new(rx));

        let (tx_done, rx_done) = mpsc::channel();

        for _ in 0..n {
            let mr_clone = mr.clone();
            let tx_done_clone = tx_done.clone();
            std::thread::spawn(move || loop {
                let m = mr_clone.lock().unwrap();
                let f: Box<dyn Fn() + Send> = match m.recv() {
                    Ok(f) => f,
                    Err(_) => {
                        tx_done_clone.send(()).ok();
                        return;
                    }
                };
                drop(m);
                f();
            });
        }

        Self {
            sender: Some(tx),
            thread_nums: n,
            ch_done: rx_done,
        }
    }

    pub fn run<F: Fn() + Send + 'static>(&self, f: F) {
        if let Some(ref c) = self.sender {
            c.send(Box::new(f)).unwrap()
        }
    }

    pub fn wait(mut self) {
        self.sender.take();

        for _ in 0..self.thread_nums {
            self.ch_done.recv().unwrap();
        }
    }
}

fn main() {
    let tp = ThreadPool::new(10);

    for i in 0..100 {
        tp.run(move || {
            std::thread::sleep(Duration::from_millis(1200));
            println!("run = {}", i);
        })
    }

    tp.wait();
}
