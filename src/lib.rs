extern crate user32;
extern crate winapi;

use std::ffi::CString;

fn initialize(){
	let lp_text = CString::new("Control Cheat").unwrap();
	let lp_caption = CString::new("Cheat injected!").unwrap();

	unsafe {
		user32::MessageBoxA(
			std::ptr::null_mut(),
			lp_text.as_ptr(),
			lp_caption.as_ptr(),
			winapi::um::winuser::MB_OK | winapi::um::winuser::MB_ICONINFORMATION
		);
	
		// activate console for debugging
		winapi::um::consoleapi::AllocConsole();

		let game_main_module = CString::new("notepad.exe").unwrap();
		let main_handle = winapi::um::libloaderapi::GetModuleHandleA(game_main_module.as_ptr());
		println!("Main module handle/base addr: {:?}", main_handle);
	}
}

#[no_mangle]
extern "system" fn DllMain(hinstDLL: *const u8, fdwReason: u32, lpReserved: *const u8) -> u32 {

	match fdwReason{
		winapi::um::winnt::DLL_PROCESS_ATTACH => {
			initialize();
		},
		winapi::um::winnt::DLL_PROCESS_DETACH => {
			
		}
		_ => println!("Nothing to do :)"),
	}

	return 1;
}