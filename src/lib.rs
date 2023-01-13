use std::{ops::{Deref, DerefMut}, borrow::BorrowMut};

use windows::Win32::{
    Foundation::HINSTANCE,
    System::{
        Console::{AllocConsole, FreeConsole, SetConsoleTitleA},
        LibraryLoader::{FreeLibraryAndExitThread, GetModuleHandleA},
    },
    UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_F1, VK_F2},
};
#[repr(C)]
struct Player {
    _a : u32,
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

impl DerefMut for PlayerPtr{
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe{
            let mut ptr = &mut (*(self.0));
            ptr.borrow_mut()
        }
        }
}


fn get_player(modbase: usize) -> PlayerPtr{
    let ply_ptr = unsafe { *((modbase + 0x0017E0A8) as *const usize) + 0xe8 } as *mut Player;

    PlayerPtr(ply_ptr)
}

fn hack(hmodule: HINSTANCE) {
    unsafe {
        AllocConsole();
        SetConsoleTitleA(pcstr("CoolHack"));
    }
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

    unsafe {
        FreeConsole();
        FreeLibraryAndExitThread(hmodule, 0);
    }
}

entrypoint!(hack);

pub fn pcstr(text: &str) -> windows::core::PCSTR {
    windows::core::PCSTR(format!("{text}\0").as_ptr())
}

// Need to get hmodule
#[macro_export]
macro_rules! entrypoint {
    ($func:ident) => {
        #[no_mangle]
        pub extern "system" fn DllMain(hmodule: HINSTANCE, reason: usize, _: *const ()) -> usize {
            if reason == 1 {
                std::thread::spawn(move || $func(hmodule));
            }
            1
        }
    };
}
