pub(crate) trait DeviceCtx {
    fn device_type(&self) -> i32;
    fn device_id(&self) -> i32;
    fn entry_point(&self) -> &str;
    // Additional methods specific to the backend behavior
}
