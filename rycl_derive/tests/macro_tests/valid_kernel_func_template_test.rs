use rycl_derive::{kernel_fn, kernel_struct};
use shared_type::DeviceStructMarker;

#[kernel_struct]
struct Test {
    a: f32,
}
#[kernel_fn]
fn test_kernel_func<T>(a: u32, b: i32, t: T, num_thread_blocks: u32, thread_block_size: u32) {
    println!("Hello from kernel function");
}

fn main() {
    test_kernel_func::<Test>(1, 2, Test { a: 3.0 }, 4, 5);
}