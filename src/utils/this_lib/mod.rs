#![allow(clippy::missing_safety_doc)]
#![allow(unused_mut)]

#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "linux")]
pub type OsInit = linux::Linux;
#[cfg(target_os = "windows")]
pub type OsInit = windows::Windows;

#[derive(Debug)]
pub struct ThisLib {
    _os: OsInit,
}

impl ThisLib {
    pub unsafe fn init(mut os_init: OsInit) -> anyhow::Result<Self> {
        use anyhow::Context;

        /* DisableThreadLibraryCalls on windows without static crt */
        #[cfg(all(target_os = "windows", not(target_feature = "crt-static")))]
        ::windows::Win32::System::LibraryLoader::DisableThreadLibraryCalls(os_init.instance)
            .context("Failed to disable thread library calls")?;

        /* Initialize console on windows developer build */
        #[cfg(all(target_os = "windows", any(debug_assertions, feature = "developer")))]
        {
            os_init.we_initialized_console = crate::utils::windows::console::attach_or_alloc();
        }

        /* Initialize logger */
        cfg_if::cfg_if! {
            if #[cfg(any(debug_assertions, feature = "developer"))] {
                let log_filter: &str = "debug";
            } else {
                let log_filter: &str = "info";
            }
        }
        env_logger::try_init_from_env(env_logger::Env::new().default_filter_or(log_filter))
            .context("Failed to intialize logger")?;
        log::debug!("Logger initialized succesfully!");

        /* Get modules */
        let modules = crate::modules::Modules::get().context("Failed to get modules")?;
        log::debug!("Got modules succesfully!");
        /* Make modules global so they can be accessed on our hooked functions
         * We have to do it here to prevent race conditions between hooked function and hooks_init() function */
        let _ = crate::globals::MODULES.set(modules);
        /* Don't check, we just set them one line ago */
        let modules = crate::globals::MODULES.get_unchecked();

        /* Initialize hooks */
        modules.hooks_init().context("Failed to initialize hooks")?;
        log::debug!("Hooks are initialized succesfully!");

        /* Print some debug information */
        log::debug!("SteamClient {:#?}", modules.sdl2.hooks);

        Ok(Self { _os: os_init })
    }
}

impl Drop for ThisLib {
    fn drop(&mut self) {
        unsafe {
            if let Some(modules) = crate::globals::MODULES.get() {
                if let Err(error) = modules.hooks_restore() {
                    log::error!("Failed to restore hooks, {:?}", anyhow::anyhow!(error));
                }
            }
        }
    }
}
