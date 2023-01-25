# deb-rust

a pure Rust library for building and reading Deb packages

deb-rust provides an easy to use, programmatic interface for reading and
writing Deb packages.

# Example

```rs
use std::fs::File;
use deb_rust::*;
use deb_rust::binary::*;

fn main() -> std::io::Result<()> {
    let mut package = DebPackage::new("example")
        .set_version("0.1.0")
        .set_description("deb-rust example")
        .with_depend("bash")
        .with_file(DebFile::from_path(
            "target/release/example",
            "/usr/bin/example",
        )?);
    package.build()?.write(File::create("example.deb")?)?;
    Ok(())
}
```
