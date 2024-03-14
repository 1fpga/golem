fn main() -> Result<(), Box<dyn std::error::Error>> {
    vergen::EmitBuilder::builder()
        .all_build()
        .all_git()
        .emit()?;

    println!("cargo:rustc-link-search=lib/sysroot/lib");
    // println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN");

    Ok(())
}
