
mod Hooking;

use std::thread;
use std::time::Duration;

use std::ffi::CString;

// Constants with game's exe's name, function offsets, saved entry points for original functions, etc.
const GAME_EXE:&str = "Control_DX11.exe";
static mut GAME_BASE: usize = 0;

const AMMO_DECFN_OFFSET: usize = 0x3B7500;
const AMMO_MINSS_OFFSET: usize = 0x3B7570;

const HEALTH_DECFN_OFFSET: usize = 0x3246D0;
static mut healthfn_saved: unsafe extern "C" fn (usize, f32, f32, usize, usize, usize) = health_hook;

const ENERGY_DECFN_OFFSET: usize = 0xF3740;
static mut dec_energyfn_saved: unsafe extern "C" fn (usize, f32) -> usize = decrement_energy_hook;

// Contants to manage enable/disable cheats
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

unsafe fn enable_infinite_ammo(){
	let mut old_prot: u32 = 0;
	let ammo_offset = GAME_BASE + AMMO_MINSS_OFFSET as usize;
	let ammo_qty = (ammo_offset+2) as *mut u8;

	println!("[!] Ammo offset: {:x}, {:x}", ammo_offset, *ammo_qty);
	winapi::um::memoryapi::VirtualProtect(ammo_qty as _, 1, winapi::um::winnt::PAGE_EXECUTE_READWRITE, &mut old_prot);
	*ammo_qty = 0x10; // here we just change minss with movss :)
	winapi::um::memoryapi::VirtualProtect(ammo_qty as _, 1, old_prot, &mut old_prot);
	println!("[!] Ammo offset: {:x}, {:x}", ammo_offset, *ammo_qty);
}

unsafe extern "fastcall" fn enable_infinite_health(){
	let healthfn_address = GAME_BASE + HEALTH_DECFN_OFFSET as usize;
	let new_original_entry_point = Hooking::set_hook(healthfn_address, std::mem::transmute(health_hook as *const ()), 21);
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
	let new_original_entry_point = Hooking::set_hook(energyfn_address, (decrement_energy_hook as *const ()) as usize, 13);
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
