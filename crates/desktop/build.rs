fn main() {
    println!("cargo:rerun-if-changed=../../assets/AppIcon.ico");

    #[cfg(windows)]
    {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("../../assets/AppIcon.ico");
        res.set("ProductName", "CottonMD");
        res.set(
            "FileDescription",
            "CottonMD — Lightweight Local-first Markdown Knowledge Workspace",
        );
        res.set("LegalCopyright", "MIT OR Apache-2.0");
        if let Err(err) = res.compile() {
            panic!("failed to compile Windows icon resource: {err}");
        }
    }
}
