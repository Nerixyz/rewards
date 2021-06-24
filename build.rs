use rustc_version::Channel;
use std::env;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=Cargo.lock");
    println!("cargo:rerun-if-changed=web/package-lock.json");
    println!("cargo:rerun-if-changed=.git/index");

    add_git_info();
    add_rustc_info();
    add_build_info();
}

fn set_env<K: std::fmt::Display, V: std::fmt::Display>(key: K, value: V) {
    println!("cargo:rustc-env={}={}", key, value);
}

fn add_git_info() {
    let info = git_info::get();
    set_env(
        "RW_GIT_INFO",
        format!(
            "{}@{}",
            info.current_branch
                .unwrap_or_else(|| "{no branch}".to_string()),
            info.head
                .last_commit_hash_short
                .unwrap_or_else(|| "{no hash}".to_string())
        ),
    );
}

fn add_rustc_info() {
    let version = rustc_version::version().expect("No rustc");
    let channel = rustc_version::version_meta().expect("No rustc").channel;

    let info = match channel {
        Channel::Dev => format!("dev {}", version),
        Channel::Nightly => format!("nightly {}", version),
        Channel::Beta => format!("beta {}", version),
        Channel::Stable => version.to_string(),
    };

    set_env("RW_RUSTC_INFO", info);
}

fn add_build_info() {
    set_env("RW_BUILD_INFO", env::var("TARGET").expect("No TARGET"));
}
