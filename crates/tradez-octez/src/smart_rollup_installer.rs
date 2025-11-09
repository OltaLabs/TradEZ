use std::{path::Path, process::Command};

use crate::logging::run_command;

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
    run_command(
        &mut command,
        "smart-rollup-installer",
        config.verbose,
        config.print_commands,
        "Failed to spawn smart-rollup-installer command",
        "Failed to wait for smart-rollup-installer command",
    );
}
