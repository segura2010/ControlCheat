
// Hooking constants with opcodes for used instructions to set up the hooks
const HOOKING_POP_RAX: u8 = 0x58; // push eax
const HOOKING_PUSH_RAX: u8 = 0x50; // pop eax
const HOOKING_MOV_RAX: u16 = 0xb848; // mov eax,
const HOOKING_MOV_R15: u16 = 0xbf49; // mov r15,
const HOOKING_PUSH_RAX_RET: u16 = 0xc350; // push rax; ret;
const HOOKING_PUSH_R15_RET: u32 = 0x90c35741; // push r15; ret; nop;

pub unsafe fn set_hook(original_func: usize, hook_func: usize, num_bytes_to_save: usize) -> usize{
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

    //println!("[!] Original code at {:x}: {:x}", original_func, *originalfn_ptr);
    winapi::um::memoryapi::VirtualProtect(originalfn_ptr as _, 15, winapi::um::winnt::PAGE_EXECUTE_READWRITE, &mut old_prot);
    *originalfn_mov_rax = HOOKING_MOV_RAX;
    *originalfn_mov_rax_value = (hook_func as *const ()) as usize;
    *originalfn_push_rax_ret = HOOKING_PUSH_RAX_RET;
    winapi::um::memoryapi::VirtualProtect(originalfn_ptr as _, 15, old_prot, &mut old_prot);
    //println!("[!] Replaced code at {:x}: {:x}", original_func, *originalfn_ptr);

    // write final code:
    //  mov r15, saved_bytes+num_bytes_to_save
    //  push r15
    //  ret
    // this code allows us to jump to rest of the original code and finally execute
    // the original function
    let end_of_saved_bytes = (saved_bytes as usize) + num_bytes_to_save;
    let end_of_saved_bytes_mov_rax = end_of_saved_bytes as *mut u16;
    let end_of_saved_bytes_mov_rax_value = (end_of_saved_bytes+2) as *mut usize;
    let end_of_saved_bytes_push_rax_ret = (end_of_saved_bytes+10) as *mut u32;
    *end_of_saved_bytes_mov_rax = HOOKING_MOV_R15;
    *end_of_saved_bytes_mov_rax_value = ((original_func+num_bytes_to_save) as *const ()) as usize;
    *end_of_saved_bytes_push_rax_ret = HOOKING_PUSH_R15_RET;
    //println!("[!] Saved bytes of original function at {:x}: {:x}", saved_bytes as usize, *(saved_bytes as *const usize));

    saved_bytes as usize
}

