#![allow (dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals, unused_assignments, unused_mut)]
#![register_tool (c2rust)]
#![feature (register_tool)]
#[no_mangle]
pub extern "C" fn insertion_sort (n : i32, p : * mut i32) {
    let mut i : i32 = 1;
    unsafe {
        while i < n {
            let tmp : i32 = * p.offset (i as isize);
            let mut j : i32 = i;
            while j > 0 && * p.offset ((j - 1i32) as isize) > tmp {
                * p.offset (j as isize) = * p.offset ((j - 1i32) as isize);
                j -= 1
            };
            * p.offset (j as isize) = tmp;
            i += 1
        };
    }
}
