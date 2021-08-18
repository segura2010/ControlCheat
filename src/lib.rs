extern crate user32;
extern crate winapi;

use std::thread;
use std::time::Duration;

use std::ffi::CString;

const GAME_EXE:&str = "Control_DX11.exe";
static mut GAME_BASE: usize = 0;
const AMMO_DECFN_OFFSET: usize = 0x3B7500;
const AMMO_MINSS_OFFSET: usize = 0x3B7570;

const HEALTH_DECFN_OFFSET: usize = 0x3246D0;
static mut healthfn_saved: unsafe extern "C" fn (usize, f32, f32, usize, usize, usize) = health_hook;

const ENERGY_DECFN_OFFSET: usize = 0xF3740;
static mut dec_energyfn_saved: unsafe extern "C" fn (usize, f32) -> usize = decrement_energy_hook;

const HOOKING_POP_RAX: u8 = 0x58; // push eax
const HOOKING_PUSH_RAX: u8 = 0x50; // pop eax
const HOOKING_MOV_RAX: u16 = 0xb848; // mov eax,
const HOOKING_MOV_R15: u16 = 0xbf49; // mov r15,
const HOOKING_PUSH_RAX_RET: u16 = 0xc350; // push rax; ret;
const HOOKING_PUSH_R15_RET: u32 = 0x90c35741; // push r15; ret; nop;

static mut DO_ONESHOT: bool = true;
static mut INF_HEALTH_ACTIVE: bool = true;
static mut INF_ENERGY_ACTIVE: bool = true;

#[no_mangle]
extern "system" fn DllMain(hinstDLL: *const u8, fdwReason: u32, lpReserved: *const u8) -> u32 {

	match fdwReason{
		winapi::um::winnt::DLL_PROCESS_ATTACH => {
			initialize();
		},
		winapi::um::winnt::DLL_PROCESS_DETACH => {
			// TODO: disable hooks?
		},
		_ => (),
	}

	return 1;
}

fn initialize(){
	unsafe {
		// activate console for debugging
		winapi::um::consoleapi::AllocConsole();
		println!("=================");
		println!("= CONTROL CHEAT =");
		println!("=================");

		// get game base address
		let game_main_module = CString::new(GAME_EXE).unwrap();
		let main_handle = winapi::um::libloaderapi::GetModuleHandleA(game_main_module.as_ptr()) as usize;
		GAME_BASE = main_handle;
		println!("Main module handle/base addr: {:x}", main_handle);
	}

	println!("Activating hooks..");
	// set up hooks
	unsafe{
		enable_infinite_ammo();
		enable_infinite_health();
		enable_infinite_energy();
	}
	println!("Hooks activated!");
	println!("Running keypress detection loop");
	print_help();
	// run keypress detection loop
	thread::spawn(|| {
		keypress_detection_loop();
	});
}

fn print_help(){
	println!("F5 -> Enable/disable one shot kills");
	println!("F6 -> Enable/disable inifinite energy");
	println!("F7 -> Enable/disable infinite health and one shot kills");
	println!("F1 -> Help");
}

fn keypress_detection_loop(){
	loop{
		let mut f1_pressed: i16 = 0;
		let mut f5_pressed: i16 = 0;
		let mut f6_pressed: i16 = 0;
		let mut f7_pressed: i16 = 0;
		unsafe{
			f1_pressed = winapi::um::winuser::GetAsyncKeyState(winapi::um::winuser::VK_F1) & (1 << 15);
			f5_pressed = winapi::um::winuser::GetAsyncKeyState(winapi::um::winuser::VK_F5) & (1 << 15);
			f6_pressed = winapi::um::winuser::GetAsyncKeyState(winapi::um::winuser::VK_F6) & (1 << 15);
			f7_pressed = winapi::um::winuser::GetAsyncKeyState(winapi::um::winuser::VK_F7) & (1 << 15);
		}
		if f1_pressed != 0 {
			print_help();
		}
		if f5_pressed != 0 {
			unsafe{
				DO_ONESHOT = !DO_ONESHOT;
				println!("DO_ONESHOT = {}", DO_ONESHOT);
			}
		}
		if f6_pressed != 0 {
			unsafe{
				INF_ENERGY_ACTIVE = !INF_ENERGY_ACTIVE;
				println!("INF_ENERGY_ACTIVE = {}", INF_ENERGY_ACTIVE);
			}
		}
		if f7_pressed != 0 {
			unsafe{
				INF_HEALTH_ACTIVE = !INF_HEALTH_ACTIVE;
				println!("INF_HEALTH_ACTIVE = {}", INF_HEALTH_ACTIVE);
			}
		}
		thread::sleep(Duration::from_millis(50));
	}
}

unsafe fn set_hook(original_func: usize, hook_func: usize, num_bytes_to_save: usize) -> usize{
	let mut old_prot: u32 = 0;
	let originalfn_ptr = original_func as *mut usize;
	let originalfn_mov_rax = original_func as *mut u16;
	let originalfn_mov_rax_value = (original_func+2) as *mut usize;
	let originalfn_push_rax_ret = (original_func+10) as *mut u16;

	// save initial bytes of function to be able to call it later
	let saved_bytes = winapi::um::memoryapi::VirtualAlloc(
		0 as _, num_bytes_to_save + 15,
		winapi::um::winnt::MEM_COMMIT | winapi::um::winnt::MEM_RESERVE,
		winapi::um::winnt::PAGE_EXECUTE_READWRITE);
	// copy bytes
	std::ptr::copy_nonoverlapping(originalfn_ptr as *const u8, saved_bytes as *mut u8, num_bytes_to_save);

	println!("[!] Original code at {:x}: {:x}", original_func, *originalfn_ptr);
	winapi::um::memoryapi::VirtualProtect(originalfn_ptr as _, 15, winapi::um::winnt::PAGE_EXECUTE_READWRITE, &mut old_prot);
	*originalfn_mov_rax = HOOKING_MOV_RAX;
	*originalfn_mov_rax_value = (hook_func as *const ()) as usize;
	*originalfn_push_rax_ret = HOOKING_PUSH_RAX_RET;
	winapi::um::memoryapi::VirtualProtect(originalfn_ptr as _, 15, old_prot, &mut old_prot);
	println!("[!] Replaced code at {:x}: {:x}", original_func, *originalfn_ptr);

	let end_of_saved_bytes = (saved_bytes as usize) + num_bytes_to_save;
	let end_of_saved_bytes_mov_rax = end_of_saved_bytes as *mut u16;
	let end_of_saved_bytes_mov_rax_value = (end_of_saved_bytes+2) as *mut usize;
	let end_of_saved_bytes_push_rax_ret = (end_of_saved_bytes+10) as *mut u32;
	*end_of_saved_bytes_mov_rax = HOOKING_MOV_R15;
	*end_of_saved_bytes_mov_rax_value = ((original_func+num_bytes_to_save) as *const ()) as usize;
	*end_of_saved_bytes_push_rax_ret = HOOKING_PUSH_R15_RET;
	println!("[!] Saved bytes of original function at {:x}: {:x}", saved_bytes as usize, *(saved_bytes as *const usize));

	saved_bytes as usize
}

unsafe fn enable_infinite_ammo(){
	let mut old_prot: u32 = 0;
	let ammo_offset = GAME_BASE + AMMO_MINSS_OFFSET as usize;
	let ammo_qty = (ammo_offset+2) as *mut u8;

	println!("[!] Ammo offset: {:x}, {:x}", ammo_offset, *ammo_qty);
	winapi::um::memoryapi::VirtualProtect(ammo_qty as _, 1, winapi::um::winnt::PAGE_EXECUTE_READWRITE, &mut old_prot);
	*ammo_qty = 0x10;
	winapi::um::memoryapi::VirtualProtect(ammo_qty as _, 1, old_prot, &mut old_prot);
	println!("[!] Ammo offset: {:x}, {:x}", ammo_offset, *ammo_qty);
}

unsafe extern "fastcall" fn enable_infinite_health(){
	let healthfn_address = GAME_BASE + HEALTH_DECFN_OFFSET as usize;
	let new_original_entry_point = set_hook(healthfn_address, std::mem::transmute(health_hook as *const ()), 21);
	healthfn_saved = std::mem::transmute(new_original_entry_point);
}

unsafe extern "C" fn health_hook(obj: usize, f1:f32, f2: f32, p1: usize, p2:usize, p3:usize){
	if INF_HEALTH_ACTIVE{
		println!("Called hook! args({:x}, {},{}, {:x}, {:x}, {:x})", obj, f1,f2,p1, p2, p3);
		let is_player = (obj +  0xA8) as *const u8;
		if *is_player == 1{
			println!("is a player..");
		} else {
			let obj_health = (obj + 0x64) as *mut f32;
			println!("is a monster (health = {})!", *obj_health);
			if DO_ONESHOT || (*obj_health < 0.2f32){
				*obj_health = 0f32;
			} else {
				*obj_health = 0.1f32;
			}
		}
	} else {
		healthfn_saved(obj, f1, f2, p1, p2, p3);
	}
}

unsafe fn enable_infinite_energy(){
	let energyfn_address = GAME_BASE + ENERGY_DECFN_OFFSET as usize;
	let new_original_entry_point = set_hook(energyfn_address, (decrement_energy_hook as *const ()) as usize, 13);
	dec_energyfn_saved = std::mem::transmute(new_original_entry_point);
}

unsafe extern "C" fn decrement_energy_hook(obj: usize, mut p1: f32) -> usize{
	if INF_ENERGY_ACTIVE {
		p1 = 0f32;
	}

	let ret = dec_energyfn_saved(obj, p1);
	//println!("[!] Energy Hook -> args({:x}, {}) = {:x}", obj, p1, ret);
	ret
}
