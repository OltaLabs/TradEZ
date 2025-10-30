use std::{path::Path, process::Command};

pub struct SmartRollupInstallerConfig {
    pub print_commands: bool,
    pub verbose: bool,
}

pub fn create_installer(
    kernel_path: &Path,
    preimages_folder: &Path,
    output_path: &Path,
    config: SmartRollupInstallerConfig,
) {
    let mut command = Command::new("smart-rollup-installer");
    command
        .arg("get-reveal-installer")
        .arg("--upgrade-to")
        .arg(kernel_path)
        .arg("--output")
        .arg(output_path)
        .arg("--preimages-dir")
        .arg(preimages_folder)
        .arg("--display-root-hash");
    if config.verbose {
        command.stdout(std::process::Stdio::inherit());
        command.stderr(std::process::Stdio::inherit());
    } else {
        command.stdout(std::process::Stdio::piped());
        command.stderr(std::process::Stdio::piped());
    }
    if config.print_commands {
        println!("> {:?}", command);
    }
    command
        .spawn()
        .expect("Failed to spawn cargo build command")
        .wait()
        .expect("Failed to wait for cargo build command");
}
