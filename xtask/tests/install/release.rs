//! A released installer: the zip it downloads and the sha256 it carries.

use super::*;

#[test]
fn an_installer_outside_a_release_downloads_nothing() {
    for installer in INSTALLERS {
        let env = MockEnvironment::new();
        env.add_mock_script("agy", AGY_THAT_INSTALLS);

        let output = env.run(installer, &["--harness", "antigravity-cli", "--yes"]);
        assert_installer_fails_with(
            installer,
            &output,
            &format!(
                "error: this {file} belongs to no release; run https://github.com/ATNexusLab/tightship/releases/latest/download/{file}, or pass {} <folder>",
                installer.flag("from-dir"),
                file = installer.file()
            ),
        );
        assert!(!env.home_dir.join(".gemini/config/plugins").exists());
    }
}

#[test]
fn a_released_installer_installs_the_zip_whose_sha256_it_carries() {
    let (zip, sha256) = release_zip("antigravity-cli");
    for installer in INSTALLERS {
        let env = MockEnvironment::new();
        env.add_mock_script("agy", AGY_THAT_INSTALLS);
        fs::write(
            env.downloads_dir.join("tightship-antigravity-cli.zip"),
            &zip,
        )
        .expect("the release download");

        let released = env.released(installer, &[&sha256]);
        let output = env.run_script(
            installer,
            &released,
            &["--harness", "antigravity-cli", "--yes"],
        );

        assert_exit(installer, &output, 0);
        assert!(
            env.home_dir
                .join(".gemini/config/plugins/tightship/plugin.json")
                .is_file(),
            "{installer:?}"
        );
    }
}

#[test]
fn a_released_installer_refuses_a_zip_whose_sha256_differs() {
    let (zip, sha256) = release_zip("antigravity-cli");
    let published = "0".repeat(64);
    for installer in INSTALLERS {
        let env = MockEnvironment::new();
        env.add_mock_script("agy", AGY_THAT_INSTALLS);
        fs::write(
            env.downloads_dir.join("tightship-antigravity-cli.zip"),
            &zip,
        )
        .expect("the release download");

        let released = env.released(installer, &[&published]);
        let output = env.run_script(
            installer,
            &released,
            &["--harness", "antigravity-cli", "--yes"],
        );

        assert_installer_fails_with(
            installer,
            &output,
            &format!(
                "error: tightship-antigravity-cli.zip has sha256 {sha256}; {RELEASE_TAG} published {published}"
            ),
        );
        assert!(
            !env.home_dir
                .join(".gemini/config/plugins/tightship")
                .exists(),
            "{installer:?}"
        );
    }
}

#[test]
fn piped_installer_never_installs_a_dist_folder_from_the_current_directory() {
    for installer in INSTALLERS {
        let env = MockEnvironment::new();
        let agy_log = env.home_dir.join("agy.log");
        env.add_mock_script(
            "agy",
            &format!(
                r#"#!/bin/sh
if [ "$1" = "--version" ]; then
    echo "agy 1.4.0"
    exit 0
fi
echo "$@" >> "{}"
exit 0
"#,
                agy_log.display()
            ),
        );

        let untrusted_clone = tempfile::tempdir().expect("a temporary directory");
        let planted = untrusted_clone.path().join("dist/antigravity-cli");
        fs::create_dir_all(planted.join("bin")).expect("planted dist");
        fs::write(planted.join("plugin.json"), "{}").expect("planted plugin.json");

        let released = env.released(installer, &[&"0".repeat(64)]);
        let output = env.run_piped(
            installer,
            &released,
            untrusted_clone.path(),
            &["--harness", "antigravity-cli", "--yes"],
        );

        assert_installer_fails_with(
            installer,
            &output,
            "error: failed to download tightship-antigravity-cli.zip",
        );
        let agy_calls = fs::read_to_string(&agy_log).unwrap_or_default();
        assert!(
            !agy_calls.contains("dist/antigravity-cli"),
            "{installer:?}: the planted dist folder was installed: {agy_calls}"
        );
    }
}
