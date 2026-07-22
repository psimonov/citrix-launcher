#[cfg(windows)]
use anyhow::bail;
use anyhow::{Context, Result};
#[cfg(windows)]
use windows_sys::Win32::{
    Foundation::LocalFree,
    Security::Cryptography::{CRYPT_INTEGER_BLOB, CryptProtectData, CryptUnprotectData},
};

#[cfg(windows)]
pub fn protect(data: &[u8]) -> Result<Vec<u8>> {
    crypt(data, true)
}
#[cfg(windows)]
pub fn unprotect(data: &[u8]) -> Result<Vec<u8>> {
    crypt(data, false)
}

#[cfg(windows)]
fn crypt(data: &[u8], encrypt: bool) -> Result<Vec<u8>> {
    if data.is_empty() {
        return Ok(Vec::new());
    }
    let input = CRYPT_INTEGER_BLOB {
        cbData: data.len().try_into().context("Секрет слишком длинный")?,
        pbData: data.as_ptr() as *mut u8,
    };
    let mut output = CRYPT_INTEGER_BLOB {
        cbData: 0,
        pbData: std::ptr::null_mut(),
    };
    let ok = unsafe {
        if encrypt {
            CryptProtectData(
                &input,
                std::ptr::null(),
                std::ptr::null(),
                std::ptr::null(),
                std::ptr::null(),
                0,
                &mut output,
            )
        } else {
            CryptUnprotectData(
                &input,
                std::ptr::null_mut(),
                std::ptr::null(),
                std::ptr::null(),
                std::ptr::null(),
                0,
                &mut output,
            )
        }
    };
    if ok == 0 {
        bail!("Windows DPAPI: {}", std::io::Error::last_os_error());
    }
    let result =
        unsafe { std::slice::from_raw_parts(output.pbData, output.cbData as usize) }.to_vec();
    unsafe { LocalFree(output.pbData as *mut _) };
    Ok(result)
}

#[cfg(not(windows))]
const SERVICE: &str = "ru.local.CitrixVdiLauncher";
#[cfg(not(windows))]
pub fn store(name: &str, value: &str) -> Result<()> {
    let entry =
        keyring::Entry::new(SERVICE, name).context("Открытие системного хранилища секретов")?;
    if value.is_empty() {
        let _ = entry.delete_credential();
        Ok(())
    } else {
        entry
            .set_password(value)
            .context("Запись в системное хранилище секретов")
    }
}
#[cfg(not(windows))]
pub fn load(name: &str) -> Result<String> {
    let entry =
        keyring::Entry::new(SERVICE, name).context("Открытие системного хранилища секретов")?;
    match entry.get_password() {
        Ok(v) => Ok(v),
        Err(keyring::Error::NoEntry) => Ok(String::new()),
        Err(e) => Err(e).context("Чтение системного хранилища секретов"),
    }
}

#[cfg(all(test, windows))]
mod tests {
    #[test]
    fn dpapi_round_trip() {
        let source = b"citrix-vdi-launcher-test-secret";
        let encrypted = super::protect(source).unwrap();
        assert_ne!(encrypted, source);
        assert_eq!(super::unprotect(&encrypted).unwrap(), source);
    }
}
