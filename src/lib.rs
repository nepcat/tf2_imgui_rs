#![feature(let_chains)]

pub mod globals;
pub mod imgui;
pub mod menu;
pub mod modules;
pub mod utils;

/* Check if platform is supported
 * Currently supported platforms are:
 * Linux x86_64
 * Windows x86_64 */
#[cfg(not(any(
    all(target_os = "linux", target_arch = "x86_64"),
    all(target_os = "windows", target_arch = "x86_64")
)))]
compile_error!("Unsupported platform");

static mut THIS_LIBRARY: parking_lot::Mutex<Option<crate::utils::this_lib::ThisLib>> =
    parking_lot::Mutex::new(None);

/* Application name */
const APPLICATION_NAME: &str = match option_env!("CARGO_PKG_NAME") {
    Some(application_name) => application_name,
    None => "TF2 ImGui Rust",
};

#[cfg(target_os = "linux")]
#[ctor::ctor]
unsafe fn constructor() {
    eprintln!("{APPLICATION_NAME} constructor called!");
    let Some(os_init) = utils::this_lib::linux::Linux::new(constructor as _) else {
        eprintln!("Failed to get current library path. Manual map unsupported");
        return;
    };

    if let Err(error) = try_init(os_init) {
        eprintln!("Failed to init: {:?}", error);
    }
}

#[cfg(target_os = "windows")]
#[no_mangle]
unsafe extern "system" fn DllMain(
    instance: windows::Win32::Foundation::HMODULE,
    reason: u32,
    _reserved: *const std::os::raw::c_void,
) -> windows::Win32::Foundation::BOOL {
    eprintln!("{APPLICATION_NAME} DllMain called, reason {reason:?}!");
    match reason {
        windows::Win32::System::SystemServices::DLL_PROCESS_ATTACH => {
            let os_init = utils::this_lib::windows::Windows {
                instance,
                ..Default::default()
            };
            if let Err(error) = try_init(os_init) {
                let fmt_error = format!("Failed to init: {:?}", anyhow::anyhow!(error));
                eprintln!("{fmt_error}");
                let _ = crate::utils::windows::message_box::message_box(
                    Default::default(),
                    fmt_error,
                    APPLICATION_NAME,
                    windows::Win32::UI::WindowsAndMessaging::MB_OK
                        | windows::Win32::UI::WindowsAndMessaging::MB_ICONERROR,
                );
                return false.into();
            }
        }
        windows::Win32::System::SystemServices::DLL_PROCESS_DETACH => {
            if let Err(error) = try_destroy() {
                let fmt_error = format!("Failed to destroy: {:?}", anyhow::anyhow!(error));
                eprintln!("{fmt_error}");
                let _ = crate::utils::windows::message_box::message_box(
                    Default::default(),
                    fmt_error,
                    APPLICATION_NAME,
                    windows::Win32::UI::WindowsAndMessaging::MB_OK
                        | windows::Win32::UI::WindowsAndMessaging::MB_ICONERROR,
                );
                return false.into();
            }
        }
        _ => {}
    }
    true.into()
}

unsafe fn try_init(os_init: utils::this_lib::OsInit) -> anyhow::Result<()> {
    eprintln!("{APPLICATION_NAME} loaded!");

    let this_library = utils::this_lib::ThisLib::init(os_init)?;
    *THIS_LIBRARY.lock() = Some(this_library);

    log::info!("{APPLICATION_NAME} is fully initialized!");
    Ok(())
}

unsafe fn try_destroy() -> anyhow::Result<()> {
    eprintln!("{APPLICATION_NAME} unloading...");

    std::mem::drop(THIS_LIBRARY.lock().take());

    eprintln!("{APPLICATION_NAME} is unloaded!");
    Ok(())
}
