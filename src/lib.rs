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

const HOOKING_MOV_RAX: u16 = 0xb848; // mov eax,
const HOOKING_PUSH_RAX_RET: u16 = 0xc350; // push eax; ret;

static mut DO_ONESHOT: bool = true;

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
	}
	println!("Hooks activated!");
	println!("Running keypress detection loop");
	// run keypress detection loop
	thread::spawn(|| {
		keypress_detection_loop();
	});
}

fn keypress_detection_loop(){
	loop{
		let mut f5_pressed: i16 = 0;
		unsafe{
			f5_pressed = winapi::um::winuser::GetAsyncKeyState(winapi::um::winuser::VK_F5) & (1 << 15);
		}
		if f5_pressed != 0 {
			unsafe{
				DO_ONESHOT = !DO_ONESHOT;
				println!("DO_ONESHOT = {}", DO_ONESHOT);
			}
		}
		thread::sleep(Duration::from_millis(100));
	}
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
	let mut old_prot: u32 = 0;
	let healthfn_offset = GAME_BASE + HEALTH_DECFN_OFFSET as usize;
	let healthfn_ptr = healthfn_offset as *mut usize;
	let healthfn_mov_rax = healthfn_offset as *mut u16;
	let healthfn_mov_rax_value = (healthfn_offset+2) as *mut usize;
	let healthfn_push_rax_ret = (healthfn_offset+10) as *mut u16;

	println!("[!] Health offset: {:x}, {:x}", healthfn_offset, *healthfn_ptr);
	winapi::um::memoryapi::VirtualProtect(healthfn_ptr as _, 15, winapi::um::winnt::PAGE_EXECUTE_READWRITE, &mut old_prot);
	*healthfn_mov_rax = HOOKING_MOV_RAX;
	*healthfn_mov_rax_value = (health_hook as *const ()) as usize;
	*healthfn_push_rax_ret = HOOKING_PUSH_RAX_RET;
	winapi::um::memoryapi::VirtualProtect(healthfn_ptr as _, 15, old_prot, &mut old_prot);
	println!("[!] Health offset: {:x}, {:x}", healthfn_offset, *healthfn_ptr);
}

unsafe extern "C" fn health_hook(obj: usize, p1: i64, p2:u8, p3:u8){
	println!("Called hook! args({:x}, {:x}, {:x}, {:x})", obj, p1, p2, p3);
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
}
