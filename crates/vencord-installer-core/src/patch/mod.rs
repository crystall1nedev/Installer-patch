use crate::Error;

pub mod patch_mod;
#[cfg(feature = "openasar")]
pub mod patch_openasar;

pub async fn rename(old: &std::path::Path, new: &std::path::Path) -> Result<(), Error> {
    match tokio::fs::rename(old, new).await {
        Ok(_) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
            #[cfg(target_os = "linux")]
            {
                use std::process::Command;
                let status = Command::new("pkexec")
                    .arg("mv")
                    .arg(old)
                    .arg(new)
                    .status()?;
                if status.success() {
                    Ok(())
                } else {
                    Err(Error::ErrPermissionDenied)
                }
            }
            #[cfg(not(target_os = "linux"))]
            {
                Err(Error::ErrPermissionDenied)
            }
        }
        Err(e) => Err(Error::from(e)),
    }
}

pub async fn copy(old: &std::path::Path, new: &std::path::Path) -> Result<(), Error> {
    match tokio::fs::copy(old, new).await {
        Ok(_) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
            #[cfg(target_os = "linux")]
            {
                use std::process::Command;
                let status = Command::new("pkexec")
                    .arg("cp")
                    .arg(old)
                    .arg(new)
                    .status()?;
                if status.success() {
                    Ok(())
                } else {
                    Err(Error::ErrPermissionDenied)
                }
            }
            #[cfg(not(target_os = "linux"))]
            {
                Err(Error::ErrPermissionDenied)
            }
        }
        Err(e) => Err(Error::from(e)),
    }
}

pub async fn remove_file(path: &std::path::Path) -> Result<(), Error> {
    match tokio::fs::remove_file(path).await {
        Ok(_) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
            #[cfg(target_os = "linux")]
            {
                use std::process::Command;
                let status = Command::new("pkexec").arg("rm").arg(path).status()?;
                if status.success() {
                    Ok(())
                } else {
                    Err(Error::ErrPermissionDenied)
                }
            }
            #[cfg(not(target_os = "linux"))]
            {
                Err(Error::ErrPermissionDenied)
            }
        }
        Err(e) => Err(Error::from(e)),
    }
}
