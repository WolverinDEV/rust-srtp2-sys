use std::path::PathBuf;
use std::env;
use std::process::Command;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let build_path = out_dir.join("srtp_build");
    let output_path = out_dir.join("srtp");
    let source_path = env::current_dir().unwrap().join("libsrtp");

    build(&source_path, &build_path, &output_path);
    /*
    if !output_path.join("bindings.rs").exists() {
        let bindings = bindgen::Builder::default()
            .header_contents("wrapper.h", &String::from("#include <srtp2/cipher.h>"))
            .clang_args(vec![format!("-I{}", output_path.join("include").to_string_lossy())])
            .whitelist_function("srtp_.*")
            .generate()
            .expect("failed to generate bindings");

        bindings
            .write_to_file(output_path.join("bindings.rs"))
            .expect("failed to write bindings");
    }
     */
    if cfg!(windows) {
        println!("cargo:rustc-link-search=native={}", output_path.join("lib").to_string_lossy());
    } else {
        let triplet = Command::new("gcc")
            .arg("-dumpmachine")
            .output().expect("failed to get host triplet")
            .stdout;
        let triplet = String::from_utf8(triplet).expect("invalid host triplet format");
        println!("cargo:rustc-link-search=native={}", output_path.join("lib").join(triplet).to_string_lossy());
    }

    println!("cargo:rustc-link-lib=srtp2");
    println!("cargo:rerun-if-env-changed=srtp2_build_type");
}

fn build(source_path: &PathBuf, build_path: &PathBuf, output_path: &PathBuf) {
    if build_path.exists() {
        std::fs::remove_dir_all(build_path).unwrap();
    }
    if output_path.exists() {
        std::fs::remove_dir_all(output_path).unwrap();
    }

    /* setup */
    {
        let mut command = Command::new("meson");

        command.arg("setup");
        command.arg("--prefix");
        command.arg(&output_path);
        command.arg(&build_path);
        command.arg(&source_path);
        let value = env::var("srtp2_build_type").unwrap_or(String::new());
        match value.as_str() {
            "static" => { command.arg("-Ddefault_library=static"); },
            "shared" => { command.arg("-Ddefault_library=shared"); },
            "" => {  },
            _ => panic!("Invalid build type: {:?}", value)
        };

        let result = command.spawn().expect("failed to launch meson command")
            .wait_with_output().expect("failed to await meson output");

        if !result.status.success() {
            panic!("failed to setup srtp build");
        }
    }

    /* compile and install */
    {
        let mut compile = Command::new("meson");
        compile.arg("install"); /* implied compile */
        compile.arg("-C");
        compile.arg(&build_path);

        let success = compile.spawn()
            .expect("failed to spawn meson command")
            .wait().expect("failed to await meson output")
            .success();

        if ! success {
            panic!("Failed to compile/install libsrtp.");
        }
    }
}