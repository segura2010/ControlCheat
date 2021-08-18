
mod ProcessInjection;

fn main() {
	let current_dir = std::env::current_exe().unwrap();
	let dll_path = current_dir.as_path().parent().unwrap().join("control_cheat_dll.dll");
	if !dll_path.exists(){
		println!("ERROR: {} does not exists!!", dll_path.to_str().unwrap());
		return;
	}
	let pid = ProcessInjection::find_process("Control_DX11.exe");
	if pid == 0xFFFFFFFF{
		println!("ERROR: Process not found!");
		return;
	}
	println!("PID: {} ({:x})", pid, pid);
	println!("Trying to inject: {}", dll_path.to_str().unwrap());
	unsafe{
		ProcessInjection::inject_dll(pid, dll_path.to_str().unwrap());
	}
}