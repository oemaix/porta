use std::path::PathBuf;
use std::process::Command;

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn second_access_point_is_tried_after_the_first_throws() {
    let root = manifest_dir();
    let out_dir = root.join("target/ap-walk-test");
    std::fs::create_dir_all(&out_dir).unwrap();
    let bin = out_dir.join("ap_walk");
    let compile = Command::new("g++")
        .args([
            "-std=c++20",
            "-Wall",
            "-Wextra",
            "-Werror",
            "-I",
        ])
        .arg(root.join("third_party/cspot/cspot/include"))
        .arg(root.join("tests/ap_walk.cpp"))
        .arg("-o")
        .arg(&bin)
        .status()
        .expect("g++ is in the Porta shell");
    assert!(compile.success(), "ap walk test failed to compile");
    let run = Command::new(&bin).status().expect("ap walk test runs");
    assert!(run.success());
}

#[test]
fn nanopb_plugin_script_stays_relative() {
    let root = manifest_dir();
    let cmake = std::fs::read_to_string(root.join("components/porta_connect/CMakeLists.txt"))
        .unwrap();
    assert!(
        !cmake.contains("file(WRITE \"${CSPOT_ROOT}/bell/external/nanopb"),
        "the build must not rewrite the submodule plugin script"
    );
    assert!(cmake.contains("porta-protoc-gen-nanopb"));
    let plugin = std::fs::read_to_string(
        root.join("third_party/cspot/cspot/bell/external/nanopb/generator/protoc-gen-nanopb"),
    )
    .unwrap();
    assert!(plugin.contains("exec \"$MYPATH/nanopb_generator.py\" --protoc-plugin"));
    assert!(!plugin.contains(".embuild"));
    assert!(!plugin.contains("exec /"));
}
