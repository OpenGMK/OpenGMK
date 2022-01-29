fn main() -> Result<(), Box<dyn ::std::error::Error>> {
    println!("cargo:rerun-if-changed=bindings.c");
    let dest = cmake::Config::new("../cimgui").define("IMGUI_STATIC", "yes").build();
    println!("cargo:rustc-link-search=native={}", dest.display());
    println!("cargo:rustc-link-lib=static=cimgui");
    Ok(())
}
