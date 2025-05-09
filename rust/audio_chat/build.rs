use std::io::Result;

#[allow(unexpected_cfgs)]
#[allow(dead_code)]
fn add_lib(name: impl AsRef<str>, _static: bool) {
    #[cfg(not(feature = "test"))]
    println!(
        "cargo:rustc-link-lib={}{}",
        if _static { "static=" } else { "" },
        name.as_ref()
    );
}

fn main() -> Result<()> {
    println!("cargo::rustc-check-cfg=cfg(frb_expand)");

    let target = std::env::var("TARGET").expect("ERR: Could not check the target for the build.");

    if target.contains("android") {
        add_lib("c++_shared", false);
    }

    Ok(())
}
