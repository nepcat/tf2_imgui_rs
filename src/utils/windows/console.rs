#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Type {
    Attach,
    Alloc,
}

pub fn attach_or_alloc() -> Option<Type> {
    unsafe {
        if windows::Win32::System::Console::AttachConsole(
            windows::Win32::System::Console::ATTACH_PARENT_PROCESS,
        )
        .is_ok()
        {
            Some(Type::Attach)
        } else if windows::Win32::System::Console::AllocConsole().is_ok() {
            Some(Type::Alloc)
        } else {
            None
        }
    }
}
