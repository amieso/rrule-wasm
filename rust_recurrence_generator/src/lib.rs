mod recurrence_generator;

pub mod c_api {

    use chrono::DateTime;

    use crate::recurrence_generator::RecurrenceGenerator;
    use rrule::Tz;
    use std::{
        ffi::{c_char, CStr, CString},
        ptr,
    };

    #[repr(C)]
    pub struct StringArray {
        // Pointer to array of C strings
        strings: *mut *mut i8,

        // Number of strings
        len: usize,
    }

    #[no_mangle]
    pub extern "C" fn recurrence_generator_generate(
        rule: *const c_char,
        after: *const c_char,
        before: *const c_char,
    ) -> *mut StringArray {
        let c_str_rule = unsafe {
            assert!(!rule.is_null());
            CStr::from_ptr(rule)
        };
        let c_str_after = unsafe {
            assert!(!after.is_null());
            CStr::from_ptr(after)
        };
        let c_str_before = unsafe {
            assert!(!before.is_null());
            CStr::from_ptr(before)
        };

        let rule_str = c_str_rule.to_str().unwrap().to_owned();
        let after_str = c_str_after.to_str().unwrap().to_owned();
        let before_str = c_str_before.to_str().unwrap().to_owned();

        let result =
            RecurrenceGenerator::recurrence_dates_between(&rule_str, &after_str, &before_str);

        match result {
            Ok(dates) => dates_to_string_array(dates),
            Err(_e) => {
                let string_array = StringArray {
                    strings: ptr::null_mut(),
                    len: 0,
                };
                return Box::into_raw(Box::new(string_array));
            }
        }
    }

    #[no_mangle]
    pub extern "C" fn free_string_array(array: *mut StringArray) {
        if !array.is_null() {
            unsafe {
                let array = Box::from_raw(array);
                let slice = std::slice::from_raw_parts_mut(array.strings, array.len);
                for &mut ptr in slice {
                    if !ptr.is_null() {
                        // This will drop the CString and free the memory
                        let _ = CString::from_raw(ptr);
                    }
                }
                // Now the array's memory is managed by Rust again and will be freed when the Box is dropped
            }
        }
    }

    // helper to convert dates into pointers that we can pass to C/Swift
    fn dates_to_string_array(dates: Vec<DateTime<Tz>>) -> *mut StringArray {
        let mut c_strings: Vec<*mut i8> = Vec::with_capacity(dates.len());

        for date in dates {
            match CString::new(date.to_rfc3339().to_string()) {
                Ok(s) => {
                    let c_str = CString::new(s).unwrap();
                    c_strings.push(c_str.into_raw());
                }
                Err(_e) => {}
            }
        }

        let string_array = StringArray {
            strings: c_strings.as_mut_ptr(),
            len: c_strings.len(),
        };

        // Prevent Rust from freeing the Vec
        std::mem::forget(c_strings);

        return Box::into_raw(Box::new(string_array));
    }
}
