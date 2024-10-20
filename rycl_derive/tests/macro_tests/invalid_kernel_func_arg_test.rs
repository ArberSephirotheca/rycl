use rycl_derive::{kernel_struct, kernel_fn};

struct Test {
    a: f64,
}
#[kernel_fn]
fn test_kernel_func(a: u32, b: i32, t: Test, num_thread_blocks: u32, thread_block_size: u32) {
    println!("Hello from kernel function");
}

fn main() {
}