#[macro_use]
extern crate lazy_static;


pub mod example;
pub mod ast;
pub mod utils;
pub mod templete;

//use test::Bencher;

pub mod engine;
pub mod core;
pub mod decode;

use utils::time_util;
use chrono::Local;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, BufReader};
use xml::EventReader;
use xml::reader::XmlEvent;
use std::fs;
use serde_json::json;
use rbatis_macro_derive::RbatisMacro;
use rbatis_macro::RbatisMacro;
use std::thread::{sleep, spawn};
use std::time::Duration;
use async_std::task;

use std::sync::Mutex;
use crate::utils::bencher::Bencher;
use crate::example::activity::Activity;

lazy_static! {
    static ref ARRAY: Mutex<Vec<u8>> = Mutex::new(vec![]);
}



fn main() {
//    ARRAY.lock().unwrap().push(1);
//    println!("{:?}",ARRAY.lock().unwrap().get(0).unwrap());

    let task = task::spawn(async {
        let id = task::current().id();
        println!("{:?}", id);
        task::sleep(Duration::from_millis(1000)).await;
    });

    task::block_on(async {
        println!("waiting for the task");
        let res = task.await;
        println!("task ended with result {:?}", res);
    });
}




//cargo test --release --package rbatis --bin rbatis bench_main --all-features -- --nocapture --exact
#[test]
fn bench_main() {
    let mut b =Bencher::new(1000000);
    b.iter( || {
         //println!("asdf");
         let js:Activity=serde_json::from_str(r#"{"id":"","name":"","version":0}"#).unwrap();
    });
}