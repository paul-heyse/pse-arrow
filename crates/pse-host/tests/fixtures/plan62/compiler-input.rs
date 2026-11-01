// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]
use std::rc::Rc;

pub fn captures() {
    let pair = (String::from("first"), String::from("second"));
    let select = || pair.0.len();
    let consume = move || drop(pair.1);
    let _ = select();
    consume();
}

pub async fn suspended() {
    let first = Rc::new(11u32);
    std::future::pending::<()>().await;
    std::hint::black_box(&first);
    drop(first);
    let second = String::from("second");
    std::future::pending::<()>().await;
    std::hint::black_box(&second);
}

pub fn partial_move(flag: bool) {
    let pair = (String::from("first"), String::from("second"));
    drop(pair.0);
    if flag {
        panic!("selected unwind");
    }
    drop(pair.1);
}

unsafe extern "C-unwind" {
    fn foreign();
}
pub unsafe extern "C" fn cannot_propagate() {
    unsafe {
        foreign();
    }
}
pub unsafe extern "C-unwind" fn may_propagate() {
    unsafe {
        foreign();
    }
}
