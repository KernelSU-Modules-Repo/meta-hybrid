use std::{
    env,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};

#[derive(Debug, Clone, Copy, ValueEnum)]
enum Arch {
    #[value(name = "arm64")]
    Arm64,
    #[value(name = "arm")]
    Arm,
    #[value(name = "x86_64")]
    X86_64,
    #[value(name = "riscv64")]
    Riscv64,
}

impl Arch {
    fn target(&self) -> &'static str {
        match self {
            Arch::Arm64 => "aarch64-linux-android",
            Arch::Arm => "armv7-linux-androideabi",
            Arch::X86_64 => "x86_64-linux-android",
            Arch::Riscv64 => "riscv64-linux-android",
        }
    }

    fn android_abi(&self) -> &'static str {
        match self {
            Arch::Arm64 => "arm64-v8a",
            Arch::Arm => "armeabi-v7a",
            Arch::X86_64 => "x86_64",
            Arch::Riscv64 => "riscv64",
        }
    }
    fn api_level(&self) -> &'static str {
        match self {
            Arch::Riscv64 => "35",
            _ => "29",
        }
    }
}

#[derive(Parser)]
#[command(name = "xtask")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand)]
enum Commands {
    Build {
        #[arg(long, default_value = "false")]
        release: bool,
        #[arg(long, default_value = "arm64")]
        arch: Arch,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Build { release, arch } => {
            build(release, arch)?;
        }
    }
    Ok(())
}

fn build(release: bool, arch: Arch) -> Result<()> {
    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    if !matches!(arch, Arch::Riscv64) {
        let status = Command::new("rustup")
            .args(["target", "add", arch.target()])
            .status()
            .context("Failed to add rust target")?;

        if !status.success() {
            eprintln!("Warning: Failed to auto-install target {}", arch.target());
        }
    }
    let mut cmd = Command::new(&cargo);
    
    cmd.arg("ndk")
       .arg("-t").arg(arch.target())
       .arg("-p").arg(arch.api_level());

    cmd.arg("build");
    if matches!(arch, Arch::Riscv64) {
        cmd.arg("-Z").arg("build-std");
        cmd.arg("--target").arg(arch.target());
    }

    if release {
        cmd.arg("--release");
    }

    let status = cmd.status().context("Failed to run cargo ndk build")?;
    if !status.success() {
        anyhow::bail!("Build failed");
    }

    Ok(())
}
