fn main() {
    // Tell cargo to rerun this script if any of the following files change
    println!("cargo:rerun-if-changed=src/");
    println!("cargo:rerun-if-changed=Cargo.toml");
    
    // Set up platform-specific configurations
    #[cfg(target_os = "macos")]
    {
        // Link against Core Audio framework on macOS
        println!("cargo:rustc-link-lib=framework=CoreAudio");
        println!("cargo:rustc-link-lib=framework=CoreFoundation");
        println!("cargo:rustc-link-lib=framework=AudioToolbox");
    }
    
    #[cfg(target_os = "windows")]
    {
        // Link against Windows audio libraries
        println!("cargo:rustc-link-lib=ole32");
        println!("cargo:rustc-link-lib=oleaut32");
        println!("cargo:rustc-link-lib=uuid");
    }
    
    // Set up version information
    println!("cargo:rustc-env=CARGO_PKG_VERSION={}", env!("CARGO_PKG_VERSION"));
}
