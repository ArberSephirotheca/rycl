use rycl_derive::{kernel_struct, kernel_fn};

#[kernel_fn]
fn test_kernel_func(a: u32, b: i32, thread_block_num: u32) {
    println!("Hello from kernel function");
}

fn main() {
}