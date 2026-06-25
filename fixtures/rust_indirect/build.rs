fn main() {
    #[cfg(unix)]
    println!("cargo:rustc-link-arg=-rdynamic");
}
