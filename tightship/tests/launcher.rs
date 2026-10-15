#[allow(dead_code)]
mod common;

use std::env;
use std::process::{Command, Output};

use common::{Sandbox, assert_fails_with, assert_succeeds, run, stdout_of};

const BINARIES: [&str; 4] = [
    "macos/tightship",
    "linux-x64/tightship",
    "linux-arm64/tightship",
    "windows-x64/tightship.exe",
];

fn script(sandbox: &Sandbox, relative: &str, text: &str) {
    let file = sandbox.write(relative, text);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o755))
            .expect("an executable");
    }
}

fn plugin() -> Sandbox {
    let sandbox = Sandbox::new();
    script(&sandbox, "bin/tightship", include_str!("../launcher.sh"));
    for binary in BINARIES {
        script(
            &sandbox,
            &format!("bin/{binary}"),
            &format!("#!/bin/sh\necho {binary} \"$@\"\n"),
        );
    }
    sandbox
}

fn launch(sandbox: &Sandbox, system: &str, machine: &str) -> Output {
    script(
        sandbox,
        "tools/uname",
        &format!("#!/bin/sh\ncase \"$1\" in -s) echo '{system}' ;; -m) echo '{machine}' ;; esac\n"),
    );
    let original = env::var_os("PATH").unwrap_or_default();
    let tools = [sandbox.path("tools")]
        .into_iter()
        .chain(env::split_paths(&original));
    let path = env::join_paths(tools).expect("a PATH");
    run(Command::new("sh")
        .arg(sandbox.path("bin/tightship"))
        .args(["vault", "path"])
        .env("PATH", path))
}

#[test]
fn the_launcher_runs_the_binary_its_table_names() {
    let sandbox = plugin();
    let rows = [
        ("Darwin", "arm64", "macos/tightship"),
        ("Darwin", "x86_64", "macos/tightship"),
        ("Linux", "x86_64", "linux-x64/tightship"),
        ("Linux", "aarch64", "linux-arm64/tightship"),
        ("Linux", "arm64", "linux-arm64/tightship"),
        ("MINGW64_NT-10.0", "x86_64", "windows-x64/tightship.exe"),
        ("MINGW64_NT-10.0", "aarch64", "windows-x64/tightship.exe"),
        ("MSYS_NT-10.0", "arm64", "windows-x64/tightship.exe"),
        ("CYGWIN_NT-10.0", "x86_64", "windows-x64/tightship.exe"),
    ];

    for (system, machine, binary) in rows {
        let output = launch(&sandbox, system, machine);

        assert_succeeds(&output);
        assert_eq!(
            stdout_of(&output),
            format!("{binary} vault path\n"),
            "{system} {machine}"
        );
    }
}

#[test]
fn a_machine_without_a_binary_fails() {
    let sandbox = plugin();

    let output = launch(&sandbox, "SunOS", "i86pc");

    assert_fails_with(&output, "error: tightship has no binary for SunOS i86pc");
}

#[cfg(unix)]
#[test]
fn an_install_without_the_executable_bit_runs_and_restores_it() {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::path::Path;

    let sandbox = plugin();
    let launcher = sandbox.path("bin/tightship");
    let binary = sandbox.path("bin/linux-x64/tightship");
    let is_executable =
        |file: &Path| fs::metadata(file).expect("the file").permissions().mode() & 0o100 != 0;
    for file in [&launcher, &binary] {
        fs::set_permissions(file, fs::Permissions::from_mode(0o644)).expect("mode 0644");
    }

    let output = launch(&sandbox, "Linux", "x86_64");

    assert_succeeds(&output);
    assert_eq!(stdout_of(&output), "linux-x64/tightship vault path\n");
    assert!(is_executable(&launcher));
    assert!(is_executable(&binary));
}
