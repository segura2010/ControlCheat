//mod lib;
//use crate::lib::*;

extern crate user32;
extern crate winapi;

use std::ffi::CString;


fn main() {
	println!("Hello, world!");

	let lp_text = CString::new("Hello, world!").unwrap();
	let lp_caption = CString::new("MessageBox Example").unwrap();

	unsafe {
		user32::MessageBoxA(
			std::ptr::null_mut(),
			lp_text.as_ptr(),
			lp_caption.as_ptr(),
			winapi::um::winuser::MB_OK | winapi::um::winuser::MB_ICONINFORMATION
		);
	}
}