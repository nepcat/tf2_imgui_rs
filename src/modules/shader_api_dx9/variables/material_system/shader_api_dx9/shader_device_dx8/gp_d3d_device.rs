pub type Type = *mut windows::Win32::Graphics::Direct3D9::IDirect3DDevice9;

/* To find signature to g_pD3DDevice you have to:
 * Find CShaderDeviceDx8::InvokeCreateDevice() in IDA, search for string "Failed to create %s device"
 * Find CShaderDeviceDx8::CreateD3DDevice() through InvokeCreateDevice() references
 *
 * IDirect3DDevice9 *pD3DDevice = InvokeCreateDevice( pHWnd, nAdapter, deviceCreationFlags );
 * if ( !pD3DDevice )
 *    return false;
 * ...
 * g_pD3DDevice = pD3DDevice;
 *
 * Find assignment to g_pD3DDevice in IDA, for example that's how it look on Win64:
 * 48 89 1D     mov cs:qword_1800A30A0, rbx
 * AD 96 07 00
 *
 * 48 89 1D     are op codes, we skip those (offset 3 bytes)
 * AD 96 07 00  those 4 bytes are relative address to g_pD3DDevice */

/* https://github.com/OthmanAba/TeamFortress2/blob/1b81dded673d49adebf4d0958e52236ecc28a956/tf2_src/materialsystem/shaderapidx9/shaderdevicedx8.cpp#L2326 */
#[cfg(all(target_os = "windows", target_arch = "x86_64"))]
static SIGNATURE: &str = "48 89 1D ?? ?? ?? ?? 48 8B CF";
#[cfg(all(target_os = "windows", target_arch = "x86_64"))]
static OFFSET: Option<crate::utils::from_isize::FromIsize> =
    Some(crate::utils::from_isize::FromIsize::Positive(3));

static PATTERN: once_cell::sync::Lazy<patterns::Pattern> =
    once_cell::sync::Lazy::new(|| patterns::Pattern::new(SIGNATURE));

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to find")]
    Find,
}
pub type Result<T> = std::result::Result<T, Error>;

pub unsafe fn find<V1: AsRef<[crate::utils::module::maps::Map]>>(maps: V1) -> Result<Type> {
    for map in maps.as_ref() {
        if let Some(mut address) = map.find_address_by_pattern(&PATTERN) {
            if let Some(offset) = OFFSET {
                address = offset.add(address);
            }

            return Ok(crate::utils::memory::relative_to_absolute_i32(address) as Type);
        }
    }

    Err(Error::Find)
}
