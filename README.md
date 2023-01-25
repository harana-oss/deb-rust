# deb-rust

a pure Rust library for building and reading Deb packages

# Example

```rs
use deb_rust::*
use deb_rust::binary::*

fn main() -> std::io::Result<()> {
    let mut package = DebPackage::new("example")
        .version("0.1.0")
        .description("deb-rust example")
        .with_depend("bash")
        .with_file(DebFile::from_path(
            "target/release/example",
            "/usr/bin/example",
        ));
    package.build()?.write()?;
}

```