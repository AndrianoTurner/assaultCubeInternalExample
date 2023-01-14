use std::{
    borrow::BorrowMut,
    ops::{Deref, DerefMut}, ffi::c_void,
};
use once_cell::sync::Lazy;
use std::sync::Mutex;
use windows::Win32::{
    Foundation::HINSTANCE,
    System::{
        Console::{AllocConsole, FreeConsole, SetConsoleTitleA},
        LibraryLoader::{FreeLibraryAndExitThread, GetModuleHandleA, GetProcAddress},
    },
    UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_F1, VK_F2},
};

use trampoline::TrampolineHook;

static SwapBuffersHook: Lazy<Mutex<Option<TrampolineHook>>> = Lazy::new(|| {
    Mutex::new(None)
});

#[repr(C)]
struct Player {
    _padding: u32,
    health: u32,
    armor: u32,
}
#[repr(C)]
struct PlayerPtr(*mut Player);

impl Deref for PlayerPtr {
    type Target = Player;
    fn deref(&self) -> &Self::Target {
        unsafe {
            let ptr = &(*(self.0));
            ptr
        }
    }
}

impl DerefMut for PlayerPtr {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            let ptr = &mut (*(self.0));
            ptr.borrow_mut()
        }
    }
}



fn get_player(modbase: usize) -> PlayerPtr {
    let ply_ptr = unsafe { *((modbase + 0x0017E0A8) as *const usize) + 0xe8 } as *mut Player;

    PlayerPtr(ply_ptr)
}

fn hack(hmodule: HINSTANCE) {
    let modbase = unsafe { GetModuleHandleA(None).unwrap().0 as usize };

    let mut ply = get_player(modbase);

    loop {
        unsafe {
            if GetAsyncKeyState(VK_F1.0 as _) & 0x1 == 1 {
                break;
            }
            if GetAsyncKeyState(113) & 0x1 == 1 {
                ply.armor = 100;
                println!("{}", (ply).armor)
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    unload(hmodule)
}

fn unload(hmodule: HINSTANCE){
    unsafe {
        SwapBuffersHook.lock().unwrap().as_mut().unwrap().unhook();
        FreeConsole();
        FreeLibraryAndExitThread(hmodule, 0);
    }
}


fn pcstr(text: &str) -> windows::core::PCSTR {
    windows::core::PCSTR(format!("{text}\0").as_ptr())
}

extern "stdcall" fn wgl_swap_buffers(hdc : HINSTANCE) -> usize{
    let gateway = SwapBuffersHook.lock().unwrap().as_ref().unwrap().gateway();
    let gateway_call : extern "stdcall" fn (hdc : HINSTANCE) -> usize;
    gateway_call = unsafe {
        std::mem::transmute(gateway)
    };

    gateway_call(hdc);
    1
}

#[no_mangle]
pub extern "system" fn DllMain(hmodule: HINSTANCE, reason: usize, _: *const ()) -> usize {
    if reason == 1 {
        unsafe{
            AllocConsole();
            SetConsoleTitleA(pcstr("CoolHack"));

            let opengl32 = GetModuleHandleA(pcstr("opengl32.dll")).unwrap();
            let swapbufs = GetProcAddress(opengl32, pcstr("wglSwapBuffers")).unwrap();
            let hook = TrampolineHook::hook(swapbufs as *mut c_void, wgl_swap_buffers as *mut c_void, 21).unwrap();
            *SwapBuffersHook.lock().unwrap() = Some(hook);
        } 
       
        std::thread::spawn(move || hack(hmodule));
    }
    1
}
