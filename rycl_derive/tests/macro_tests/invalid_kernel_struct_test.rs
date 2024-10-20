use rycl_derive::kernel_struct;

#[kernel_struct]
pub struct Test {
    pub a: u32,
    pub b: i32,
    pub c: f32,
    pub d: f64,
}

fn main() {
    
}