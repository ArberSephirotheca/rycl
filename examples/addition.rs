use rycl_derive::kernel_fn;

#[kernel_fn]
fn add(a: i32, b: i32, num_thread_blocks: u32, thread_block_size: u32) -> i32 {
    a + b
}

fn main() {
    let result = add(1, 1, 1, 1);
    println!("Addition result from kernel: {}", result);
}
