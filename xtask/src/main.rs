use anyhow::{anyhow, Result};
use std::{
    env,
    process::{Command, Stdio},
};
use xshell::Shell;

#[cfg(not(any(
    all(target_os = "windows", target_arch = "x86_64"),
    all(target_os = "macos", target_arch = "aarch64"),
    all(target_os = "linux", target_arch = "x86_64"),
)))]
compile_error!("Unsupported target platform! Only the following platforms are supported: Windows (x86_64), macOS (aarch64), or Linux (x86_64).");

#[cfg(all(target_os = "windows", target_arch = "x86_64"))]
const NODE_LINK: &str = "https://get.gear.rs/gear-v1.5.0-x86_64-pc-windows-msvc.zip";

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
const NODE_LINK: &str = "https://get.gear.rs/gear-v1.5.0-aarch64-apple-darwin.tar.xz";

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
const NODE_LINK: &str = "https://get.gear.rs/gear-v1.5.0-x86_64-unknown-linux-gnu.tar.xz";

fn main() -> Result<()> {
    let Some(command) = env::args().nth(1) else {
        return Err(anyhow!("command wasn't given"));
    };

    let sh = Shell::new()?;

    sh.change_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/.."));

    let is_file_not_exist = |path| xshell::cmd!(sh, "[ -e {path} ]").quiet().run().is_err();

    let node = || -> Result<_> {
        if is_file_not_exist("target/tmp") {
            xshell::cmd!(sh, "mkdir -p target/tmp").run()?;
        }

        if is_file_not_exist("target/tmp/gear") {
            // Implements a platform-agnostic piping for simultaneous downloading & unpacking the
            // node archive.

            let curl_output = Command::from(xshell::cmd!(sh, "curl -L {NODE_LINK} -o -"))
                .stdout(Stdio::piped())
                .spawn()?
                .stdout
                .ok_or(anyhow!("expected an output from curl"))?;

            if !Command::from(xshell::cmd!(sh, "tar xJ -C target/tmp"))
                .stdin(curl_output)
                .output()?
                .status
                .success()
            {
                anyhow::bail!("failed while unpacking the node archive");
            }
        }

        Ok(())
    };

    match command.as_str() {
        "node" => node()?,
        _ => return Err(anyhow!("unknown command")),
    }

    Ok(())
}
