fn main() {
    let mut res = winres::WindowsResource::new();

    // 这些信息会出现在 exe 属性里
    res.set("FileDescription", "Rust工具");
    res.set("ProductName", "gxTools");
    res.set("CompanyName", "玻璃球");
    res.set("LegalCopyright", "玻璃球");
    res.set("FileVersion", "1.0.0.0");
    res.set("ProductVersion", "1.0.0.0");

    res.compile().unwrap();
}
