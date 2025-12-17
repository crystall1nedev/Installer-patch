pub mod patch_mod;
#[cfg(feature = "openasar")]
pub mod patch_openasar;

use crate::Error;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum FileOperation {
    Move { from: PathBuf, to: PathBuf },
    Copy { from: PathBuf, to: PathBuf },
    Remove { path: PathBuf },
    Cmd { string: String },
}

impl FileOperation {
    #[cfg(target_os = "linux")]
    fn to_shell_command(&self) -> String {
        match self {
            FileOperation::Move { from, to } => {
                format!("mv '{}' '{}'", from.display(), to.display())
            }
            FileOperation::Copy { from, to } => {
                format!("cp '{}' '{}'", from.display(), to.display())
            }
            FileOperation::Remove { path } => {
                format!("rm '{}'", path.display())
            }
            FileOperation::Cmd { string } => string.clone(),
        }
    }
}

pub async fn execute_file_operations(operations: &[FileOperation]) -> Result<(), Error> {
    let mut needs_elevated = false;

    log::debug!("Running operations: {:#?}", operations);

    for operation in operations {
        let result = match operation {
            FileOperation::Move { from, to } => tokio::fs::rename(from, to).await,
            FileOperation::Copy { from, to } => tokio::fs::copy(from, to).await.map(|_| ()),
            FileOperation::Remove { path } => tokio::fs::remove_file(path).await,
            FileOperation::Cmd { .. } => Ok(()),
        };

        if let Err(e) = result {
            if e.kind() == std::io::ErrorKind::PermissionDenied {
                needs_elevated = true;
                break;
            } else {
                return Err(Error::from(e));
            }
        }
    }

    // If we need elevated permissions, execute all operations with pkexec
    // in a single command, so it only prompts once
    if needs_elevated {
        #[cfg(target_os = "linux")]
        {
            log::warn!("Permission was denied, attempting to use pkexec instead...");

            use std::process::Command;

            let commands: Vec<String> = operations.iter().map(|op| op.to_shell_command()).collect();

            let combined_command = commands.join(" && ");

            let status = Command::new("pkexec")
                .arg("sh")
                .arg("-c")
                .arg(&combined_command)
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
    } else {
        Ok(())
    }
}
