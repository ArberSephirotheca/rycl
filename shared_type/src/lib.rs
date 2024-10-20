/// Marker trait for kernel functions, user should not implement this trait manually
/// This trait is used to check if the customize type is valid in kernel functions
#[allow(dead_code)]
pub trait DeviceStructMarker {
    fn is_device_struct() -> bool {
        true
    }
}

/// Primitive trait is used to restrict the generic type of device struct
#[allow(dead_code)]
pub trait Primitive {}

impl Primitive for u32 {}
impl Primitive for i32 {}
impl Primitive for f32 {}