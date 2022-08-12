use std::{path::PathBuf, process::Command};

fn main() {
    // println!("cargo:rerun-if-changed=src/bin/shook/main.rs");
    std::fs::write(
        PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("version.rs"),
        indoc::formatdoc!(
            r#"
            pub const GIT_BRANCH: &str = "{git_branch}";
            pub const GIT_REVISION: &str = "{git_revision}";
            pub const BUILD_TIME: &str = "{build_time}";
        "#,
            git_branch = get_branch().expect("should be in git"),
            git_revision = get_revision().expect("should be in git"),
            build_time = current_time(),
        ),
    )
    .unwrap()
}

fn current_time() -> String {
    time::OffsetDateTime::now_local()
        .unwrap()
        .format(&time::format_description::well_known::Rfc2822)
        .unwrap()
}

fn get_branch() -> Option<String> {
    get_git(Some("--abbrev-ref"))
}

fn get_revision() -> Option<String> {
    get_git(None)
}

fn get_git(s: Option<&str>) -> Option<String> {
    let mut cmd = Command::new("git");
    cmd.arg("rev-parse");
    if let Some(s) = s {
        cmd.arg(s);
    };
    let out = cmd.arg("@").output().ok()?.stdout;

    std::str::from_utf8(&out)
        .ok()
        .map(<str>::trim)
        .map(|s| s.to_string())
}
