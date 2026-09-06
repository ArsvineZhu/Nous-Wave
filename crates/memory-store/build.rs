fn main() {
    // SQLx embeds migrations at compile time; adding a file must invalidate it.
    println!("cargo:rerun-if-changed=migrations");
}
