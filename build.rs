use std::process::Command;

// Example custom build script.
fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=templates/**/*.html");

    Command::new("node_modules/.bin/tailwindcss")
        .arg("-c")
        .arg("tailwind/tailwind.config.js")
        .arg("-i")
        .arg("tailwind/base.css")
        .arg("-o")
        .arg("static/css/styles.css")
        .output()
        .expect("failed to execute process");
}
