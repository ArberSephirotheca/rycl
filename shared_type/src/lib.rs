/// Marker trait for kernel functions, user should not implement this trait manually
/// This trait is used to check if the customize type is valid in kernel functions
#[allow(dead_code)]
pub trait DeviceStructMarker {}

/// Primitive trait is used to restrict the generic type of device struct
#[allow(dead_code)]
pub trait Primitive {}

impl Primitive for u32 {}
impl Primitive for i32 {}
impl Primitive for f32 {}

#[allow(dead_code)]
pub trait KernelFn {
    fn execute(&self);
}

impl KernelFn for fn() {
    fn execute(&self) {
        (self)(); // Call the function
    }
}
