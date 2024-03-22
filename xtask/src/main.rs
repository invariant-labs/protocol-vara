use anyhow::{anyhow, Result};
use std::{
    env,
    process::{Command, Stdio},
};
use xshell::Shell;

const NODE_LINK: &str = "https://get.gear.rs/gear-v1.1.0-x86_64-unknown-linux-gnu.tar.xz";

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
