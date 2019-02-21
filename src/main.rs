use std::env;
use std::io::Write;
use std::process;

use clap::{load_yaml, App};
use env_logger::Builder;
use log::LevelFilter;

mod lumins;
pub use lumins::parse;
use lumins::parse::{Flag, SubCommandType};
pub use lumins::{core, file_ops};

fn main() {
    let yaml = load_yaml!("cli.yml");
    let args = App::from_yaml(yaml).get_matches();

    let (sub_command, flags) = match parse::parse_args(&args) {
        Ok(f) => (f.sub_command, f.flags),
        Err(_) => process::exit(1),
    };

    if flags.contains(&Flag::Verbose) {
        env::set_var("RUST_LOG", "info");
        Builder::new()
            .format(|buf, record| writeln!(buf, "{}", record.args()))
            .filter(None, LevelFilter::Info)
            .init();
    }

    let result = match sub_command.sub_command_type {
        SubCommandType::Copy => core::copy(sub_command.src.unwrap(), sub_command.dest, flags),
        SubCommandType::Delete => core::delete(sub_command.dest, flags),
        SubCommandType::Synchronize => {
            core::synchronize(sub_command.src.unwrap(), sub_command.dest, flags)
        }
    };

    if let Err(e) = result {
        eprintln!("{}", e);
        process::exit(1);
    }
}

#[cfg(test)]
mod test_main {
    use std::fs;
    use std::process::Command;

    #[test]
    fn test_no_args() {
        Command::new("cargo")
            .args(&["build", "--release"])
            .output()
            .unwrap();

        let output = Command::new("target/release/lumins").output().unwrap();

        assert_eq!(output.status.success(), false);
    }

    #[test]
    fn test_no_dest() {
        Command::new("cargo")
            .args(&["build", "--release"])
            .output()
            .unwrap();

        let output = Command::new("target/release/lumins")
            .args(&["sync", "src"])
            .output()
            .unwrap();

        assert_eq!(output.status.success(), false);
    }

    #[test]
    fn test_too_many_args() {
        Command::new("cargo")
            .args(&["build", "--release"])
            .output()
            .unwrap();

        let output = Command::new("target/release/lumins")
            .args(&["sync", "src", "dest", "dest"])
            .output()
            .unwrap();

        assert_eq!(output.status.success(), false);
    }

    #[test]
    fn test_invalid_args() {
        Command::new("cargo")
            .args(&["build", "--release"])
            .output()
            .unwrap();

        let output = Command::new("target/release/lumins")
            .args(&["sync", "a", "dest"])
            .output()
            .unwrap();

        assert_eq!(output.status.success(), false);
    }

    #[cfg(target_family = "unix")]
    #[test]
    fn test_copy() {
        Command::new("cargo")
            .args(&["build", "--release"])
            .output()
            .unwrap();

        const TEST_SOURCE: &str = "target/debug";
        const TEST_DEST: &str = "test_main_test_copy";
        fs::create_dir_all(TEST_DEST).unwrap();

        Command::new("target/release/lumins")
            .args(&["copy", "-v", TEST_SOURCE, TEST_DEST])
            .output()
            .unwrap();

        let diff = Command::new("diff")
            .args(&["-r", TEST_SOURCE, TEST_DEST])
            .output()
            .unwrap();

        assert_eq!(diff.status.success(), true);

        fs::remove_dir_all(TEST_DEST).unwrap();
    }

    #[cfg(target_family = "unix")]
    #[test]
    fn test_secure() {
        Command::new("cargo")
            .args(&["build", "--release"])
            .output()
            .unwrap();

        const TEST_SOURCE: &str = "target/debug";
        const TEST_DEST: &str = "test_main_test_secure";
        fs::create_dir_all(TEST_DEST).unwrap();

        Command::new("target/release/lumins")
            .args(&["sync", "-s", TEST_SOURCE, TEST_DEST])
            .output()
            .unwrap();

        let diff = Command::new("diff")
            .args(&["-r", TEST_SOURCE, TEST_DEST])
            .output()
            .unwrap();

        assert_eq!(diff.status.success(), true);

        fs::remove_dir_all(TEST_DEST).unwrap();
    }

    #[cfg(target_family = "unix")]
    #[test]
    fn test_sequential() {
        Command::new("cargo")
            .args(&["build", "--release"])
            .output()
            .unwrap();

        const TEST_SOURCE: &str = "target/debug";
        const TEST_DEST: &str = "test_main_test_sequential";
        fs::create_dir_all(TEST_DEST).unwrap();

        Command::new("target/release/lumins")
            .args(&["sync", "-S", TEST_SOURCE, TEST_DEST])
            .output()
            .unwrap();

        let diff = Command::new("diff")
            .args(&["-r", TEST_SOURCE, TEST_DEST])
            .output()
            .unwrap();

        assert_eq!(diff.status.success(), true);

        fs::remove_dir_all(TEST_DEST).unwrap();
    }

    #[cfg(target_family = "unix")]
    #[test]
    fn test_sequential_copy() {
        Command::new("cargo")
            .args(&["build", "--release"])
            .output()
            .unwrap();

        const TEST_SOURCE: &str = "target/debug";
        const TEST_DEST: &str = "test_main_test_sequential_copy";
        fs::create_dir_all(TEST_DEST).unwrap();

        Command::new("target/release/lumins")
            .args(&["copy", "-S", TEST_SOURCE, TEST_DEST])
            .output()
            .unwrap();

        let diff = Command::new("diff")
            .args(&["-r", TEST_SOURCE, TEST_DEST])
            .output()
            .unwrap();

        assert_eq!(diff.status.success(), true);

        fs::remove_dir_all(TEST_DEST).unwrap();
    }

    #[cfg(target_family = "unix")]
    #[test]
    fn test_no_delete() {
        Command::new("cargo")
            .args(&["build", "--release"])
            .output()
            .unwrap();

        const TEST_SOURCE1: &str = "test_main_test_no_delete_source1";
        const TEST_SOURCE2: &str = "test_main_test_no_delete_source2";
        const TEST_DEST: &str = "test_main_test_no_delete_out";
        const TEST_EXPECTED: &str = "test_main_test_no_delete_expected";
        const TEST_FILE1: &str = "Cargo.toml";
        const TEST_FILE2: &str = "Cargo.lock";

        fs::create_dir_all(TEST_SOURCE1).unwrap();
        fs::create_dir_all(TEST_SOURCE2).unwrap();
        fs::create_dir_all(TEST_DEST).unwrap();
        fs::create_dir_all(TEST_EXPECTED).unwrap();

        fs::copy(TEST_FILE1, [TEST_SOURCE1, TEST_FILE1].join("/")).unwrap();
        fs::copy(TEST_FILE2, [TEST_SOURCE2, TEST_FILE2].join("/")).unwrap();
        fs::copy(TEST_FILE1, [TEST_EXPECTED, TEST_FILE1].join("/")).unwrap();
        fs::copy(TEST_FILE2, [TEST_EXPECTED, TEST_FILE2].join("/")).unwrap();

        Command::new("target/release/lumins")
            .args(&["copy", TEST_SOURCE1, TEST_DEST])
            .output()
            .unwrap();

        Command::new("target/release/lumins")
            .args(&["sync", "-n", TEST_SOURCE2, TEST_DEST])
            .output()
            .unwrap();

        let diff = Command::new("diff")
            .args(&["-r", TEST_DEST, TEST_EXPECTED])
            .output()
            .unwrap();

        assert_eq!(diff.status.success(), true);

        fs::remove_dir_all(TEST_SOURCE1).unwrap();
        fs::remove_dir_all(TEST_SOURCE2).unwrap();
        fs::remove_dir_all(TEST_DEST).unwrap();
        fs::remove_dir_all(TEST_EXPECTED).unwrap();
    }
}
