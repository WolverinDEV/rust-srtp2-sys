use std::path::PathBuf;
use std::env;
use std::process::Command;
use build_utils::build::LibraryType;

fn main() {
    if let Ok(_library) = pkg_config::Config::new()
        .atleast_version("2.4.0")
        .probe("libsrtp2") {
        println!("Found libsrtp2 via pkg config");
        return;
    }

    let source = build_utils::source::BuildSourceGit::builder("https://github.com/cisco/libsrtp.git".to_owned())
        .revision(Some("7d351de".to_string()))
        .build();

    let meson = build_utils::build::MesonBuild::builder()
        .build();

    let mut build_builder = build_utils::Build::builder()
        .name("libsrtp2")
        .source(Box::new(source))
        .add_step(Box::new(meson));

    #[cfg(windows)]
    {
        /* libsrtp2 has trouble building shared libraries on windows */
        build_builder = build_builder.library_type(LibraryType::Static);
    }

    match build_builder.build().expect("failed to generate build").execute() {
        Ok(result) => {
            result.emit_cargo();

            /* TODO: Generate bindings */
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
        },
        Err(error) => {
            println!(error.pretty_format());
            panic!("failed to execute usrsctp build");
        }
    }
}
