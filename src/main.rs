//mod lib;
//use crate::lib::*;

extern crate user32;
extern crate winapi;

use std::path::PathBuf;
use std::env;
use std::mem::*;
use std::ffi::CString;
use sysinfo::{ProcessExt, System, SystemExt};

fn main() {
	let current_dir = env::current_exe().unwrap();
	let dll_path = current_dir.as_path().parent().unwrap().join("control_cheat.dll");
	if !dll_path.exists(){
		println!("ERROR: {} not exists!!", dll_path.to_str().unwrap());
		return;
	}
	println!("Trying to inject: {}", dll_path.to_str().unwrap());
	let pid = find_process("Control_DX11.exe");
	println!("PID: {}", pid);
	if pid == 0{
		return;
	}
	unsafe{
		inject_dll(pid, dll_path.to_str().unwrap());
	}
}

fn find_process(proc_str: &str) -> u32{
	let s = sysinfo::System::new_all();
	for (pid, process) in s.processes(){
		if process.name().eq(proc_str) {
			return *pid as u32;
		}
	}
	0
}

type RemoteThreadProc = unsafe extern "system" fn(winapi::shared::minwindef::LPVOID) -> u32;
unsafe fn inject_dll(pid: u32, dll_path: &str){
	let hProcess = winapi::um::processthreadsapi::OpenProcess(winapi::um::winnt::PROCESS_ALL_ACCESS, 1, pid) as u32;
	println!("hProcess={}", hProcess);

	let dll_path_addr = winapi::um::memoryapi::VirtualAllocEx(
		hProcess as _, 0 as _, dll_path.len()+1,
		winapi::um::winnt::MEM_COMMIT | winapi::um::winnt::MEM_RESERVE, winapi::um::winnt::PAGE_EXECUTE_READWRITE);
	println!("base addr = {:x}", dll_path_addr as usize);

	winapi::um::memoryapi::WriteProcessMemory(hProcess as _, dll_path_addr as _, dll_path.as_ptr() as _, dll_path.len(), 0 as _);

	let kernel32_handle = winapi::um::libloaderapi::LoadLibraryA(CString::new("kernel32.dll").unwrap().as_ptr() as _);
	let loadlibrarya_addr = winapi::um::libloaderapi::GetProcAddress(kernel32_handle, CString::new("LoadLibraryA").unwrap().as_ptr() as _) as *const ();
	println!("kernel32 handle={:x}, LoadLibraryA = {:x}", kernel32_handle as usize, loadlibrarya_addr as usize);

	let thread_handle = winapi::um::processthreadsapi::CreateRemoteThread(hProcess as _, 0 as _, 0 as _, Some(std::mem::transmute(loadlibrarya_addr)), dll_path_addr, 0, 0 as _);
	println!("Thread={:x}", thread_handle as usize);

	winapi::um::handleapi::CloseHandle(hProcess as _);
}