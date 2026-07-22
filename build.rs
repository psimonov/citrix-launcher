fn main() {
    println!("cargo:rerun-if-changed=assets/icons/citrix-vdi-launcher.ico");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut resource = winresource::WindowsResource::new();
        resource.set_icon("assets/icons/citrix-vdi-launcher.ico");
        resource.compile().expect("compile Windows icon resource");
    }
}
