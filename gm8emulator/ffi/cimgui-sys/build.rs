fn main() -> Result<(), Box<dyn ::std::error::Error>> {
    println!("cargo:rerun-if-changed=bindings.c");
    let dest = cmake::Config::new("../cimgui").define("IMGUI_STATIC", "yes").build();
    println!("cargo:rustc-link-search=native={}", dest.display());
    {
        // horrible workaround for windows gnu builds
        let out_dir = std::env::var("OUT_DIR").unwrap();
        let a_path = out_dir.clone() + "/cimgui.a";
        if std::path::Path::new(&a_path).is_file() {
            std::fs::copy(a_path, out_dir.clone() + "/libcimgui.a").unwrap();
        }
        println!("cargo:rustc-link-search=native={}", out_dir);
    }
    println!("cargo:rustc-link-lib=static=cimgui");
    Ok(())
}
