#![allow(clippy::mut_from_ref)]

#[derive(Debug, Clone)]
pub struct Map {
    pub start: usize,
    pub size: usize,
}

impl Map {
    pub fn end(&self) -> usize {
        self.start + self.size
    }

    pub unsafe fn get_memory_slice(&self) -> &[u8] {
        std::slice::from_raw_parts(self.start as *mut u8, self.size)
    }

    pub unsafe fn get_mut_memory_slice(&self) -> &mut [u8] {
        std::slice::from_raw_parts_mut(self.start as *mut u8, self.size)
    }

    /*pub unsafe fn find_address_by_pattern(&self, pattern: &patterns::Pattern) -> Option<usize> {
        let slice = self.get_memory_slice();
        let mut iterator = pattern.matches(slice);

        Some(self.start + iterator.next()?)
    }

    pub unsafe fn find_addresses_by_pattern(&self, pattern: &patterns::Pattern) -> Vec<usize> {
        let mut result = Vec::new();

        let slice = self.get_memory_slice();
        let iterator = pattern.matches(slice);

        for address in iterator {
            result.push(self.start + address);
        }

        result
    }*/

    #[cfg(target_os = "windows")]
    pub fn get_from_module(module: &super::Module) -> windows::core::Result<Self> {
        let mut module_info = Default::default();
        unsafe {
            windows::Win32::System::ProcessStatus::GetModuleInformation(
                windows::Win32::System::Threading::GetCurrentProcess(),
                *module.get_handle(),
                &mut module_info,
                std::mem::size_of::<windows::Win32::System::ProcessStatus::MODULEINFO>() as u32,
            )
        }?;

        Ok(Self {
            start: module_info.lpBaseOfDll as usize,
            size: module_info.SizeOfImage as usize,
        })
    }

    #[cfg(target_os = "linux")]
    pub fn from_line(s: &str) -> anyhow::Result<(Self, String)> {
        use anyhow::Context;
        let mut current_pos = 0;
        let base_pos = s
            .chars()
            .skip(current_pos)
            .position(|x| x == '-')
            .context("Failed to find base '-'")?;

        let base = s
            .chars()
            .skip(current_pos)
            .take(base_pos)
            .collect::<String>();
        current_pos += base_pos + 1;

        let ceiling_pos = s
            .chars()
            .skip(current_pos)
            .position(|x| x == ' ')
            .context("Failed to find ceiling ' '")?;
        let ceiling = s
            .chars()
            .skip(current_pos)
            .take(ceiling_pos)
            .collect::<String>();
        current_pos += ceiling_pos + 1;

        let permissions_pos = s
            .chars()
            .skip(current_pos)
            .position(|x| x == ' ')
            .context("Failed to find permissions ' '")?;
        let _permissions = s
            .chars()
            .skip(current_pos)
            .take(permissions_pos)
            .collect::<String>();
        current_pos += permissions_pos + 1;

        let offset_pos = s
            .chars()
            .skip(current_pos)
            .position(|x| x == ' ')
            .context("Failed to find offset ' '")?;
        let _offset = s
            .chars()
            .skip(current_pos)
            .take(offset_pos)
            .collect::<String>();
        current_pos += offset_pos + 1;

        let dev_major_pos = s
            .chars()
            .skip(current_pos)
            .position(|x| x == ':')
            .context("Failed to find dev major ':'")?;
        let _dev_major = s
            .chars()
            .skip(current_pos)
            .take(dev_major_pos)
            .collect::<String>();
        current_pos += dev_major_pos + 1;

        let dev_minor_pos = s
            .chars()
            .skip(current_pos)
            .position(|x| x == ' ')
            .context("Failed to find dev minor ' '")?;
        let _dev_minor = s
            .chars()
            .skip(current_pos)
            .take(dev_minor_pos)
            .collect::<String>();
        current_pos += dev_minor_pos + 1;

        let inode_pos = s
            .chars()
            .skip(current_pos)
            .position(|x| x == ' ')
            .context("Failed to find inode ' '")?;
        let _inode = s
            .chars()
            .skip(current_pos)
            .take(inode_pos)
            .collect::<String>();
        current_pos += inode_pos + 1;

        let name = if let Some(name_pos) = s.chars().skip(current_pos).position(|x| x != ' ') {
            current_pos += name_pos;
            let name = s
                .chars()
                .skip(current_pos)
                .take(s.chars().count() - current_pos)
                .collect::<String>();
            Some(name)
        } else {
            None
        };

        let name = name.context("No name provided")?;

        let start = usize::from_str_radix(&base, 16).context("Failed to parse base from string")?;
        let ceiling =
            usize::from_str_radix(&(ceiling), 16).context("Failed to parse ceiling from string")?;
        let size = ceiling - start;

        Ok((Self { start, size }, name))
    }
}

pub type Maps = std::collections::HashMap<String, Vec<Map>>;

#[cfg(target_os = "linux")]
pub fn parse_proc_maps<S1: AsRef<str>>(buffer: S1) -> Maps {
    let mut result = Maps::new();

    let mut lines: Vec<&str> = buffer.as_ref().split('\n').collect();
    /* Delete last element because its always empty! */
    lines.pop();

    for line in lines {
        // log::debug!("Parsing map line: {line}");
        match Map::from_line(line) {
            Ok((map, name)) => match result.get_mut(&name) {
                Some(vec) => vec.push(map),
                None => {
                    result.insert(name, vec![map]);
                }
            },
            Err(_error) =>
                /*log::error!("Failed to parse map line {line}, reason: {error:?}")*/
                {}
        }
    }

    result
}

// lol
pub fn find_closest_map<S1: AsRef<str>>(maps: &mut Maps, str: S1) -> Option<(String, Vec<Map>)> {
    let str = str.as_ref();
    let (name_wtf, _) = maps.iter().find(|(key, _)| key.contains(str))?;
    let name = name_wtf.clone();
    let _ = name_wtf;
    let map = unsafe { maps.remove(&name).unwrap_unchecked() };
    Some((name, map))
}
