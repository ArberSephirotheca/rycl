#[test]
fn test_kernels() {
    let t = trybuild::TestCases::new();
    t.pass("tests/macro_tests/valid_kernel_struct_test.rs");
    t.compile_fail("tests/macro_tests/invalid_kernel_struct_test.rs");
    t.compile_fail("tests/macro_tests/invalid_kernel_func_test.rs");
    t.compile_fail("tests/macro_tests/invalid_kernel_func_arg_test.rs");
    t.pass("tests/macro_tests/valid_kernel_func_template_test.rs");
    t.compile_fail("tests/macro_tests/invalid_kernel_func_template_test.rs");
}
