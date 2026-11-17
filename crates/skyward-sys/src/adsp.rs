//! The ADSP, the processor the phone's battery is read behind: the kernel
//! starts it as the phone boots, loading its firmware from the modem
//! partition, and in a recovery it gives up before that partition is
//! mounted and never asks again. The fuel gauge then stays down, and the
//! health HAL that fastbootd waits for finds no battery. Once the firmware
//! is where the kernel's firmware loader looks, the processor can be
//! started from here, and the gauge comes up by itself.

use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};

/// Where the kernel shows the processors it starts.
const REMOTEPROCS: &str = "/sys/class/remoteproc";

/// Start the ADSP and say it runs. Its firmware has to be where the
/// kernel's loader looks by now: the write waits in the kernel until the
/// firmware is loaded and the processor started, and fails with the reason
/// where it could not be.
pub fn start() -> Result<()> {
    let state = adsp()?.join("state");
    if said(&state)? == "running" {
        return Ok(());
    }
    std::fs::write(&state, "start").with_context(|| format!("write start to {}", state.display()))?;
    let now = said(&state)?;
    if now != "running" {
        bail!("the ADSP is {now:?} after being started, not \"running\"");
    }
    Ok(())
}

/// What the state file says, without the newline the kernel ends it with.
fn said(state: &Path) -> Result<String> {
    let said = std::fs::read_to_string(state).with_context(|| format!("read {}", state.display()))?;
    Ok(said.trim().to_string())
}

/// The folder the kernel shows the ADSP in.
fn adsp() -> Result<PathBuf> {
    let entries = std::fs::read_dir(REMOTEPROCS).with_context(|| format!("read {REMOTEPROCS}"))?;
    for entry in entries {
        let path = entry.with_context(|| format!("read {REMOTEPROCS}"))?.path();
        let name = std::fs::read_to_string(path.join("name")).unwrap_or_default();
        if is_adsp(&name) {
            return Ok(path);
        }
    }
    bail!("no ADSP among the processors the kernel shows at {REMOTEPROCS}")
}

/// Whether a name the kernel gives a processor is the ADSP's, as its
/// recovery's own names the modem's, the computing one's, the sensors' and
/// the secure one's apart from it.
fn is_adsp(name: &str) -> bool {
    name.trim_end().ends_with("adsp")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_adsp_is_told_from_the_phones_other_processors() {
        assert!(is_adsp("3000000.remoteproc-adsp\n"));
        assert!(!is_adsp("4080000.remoteproc-mss\n"));
        assert!(!is_adsp("32300000.remoteproc-cdsp\n"));
        assert!(!is_adsp("a3380000.remoteproc-soccp\n"));
        assert!(!is_adsp("188101c.remoteproc-spss\n"));
        assert!(!is_adsp(""));
    }
}