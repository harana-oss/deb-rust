# deb-rust

a pure Rust library for building and reading Deb packages

deb-rust provides an easy to use, programmatic interface for reading and
writing Deb packages.

# Examples

### Writing

```rs
use std::fs::File;
use deb_rust::*;
use deb_rust::binary::*;

fn main() -> std::io::Result<()> {
    let mut package = DebPackage::new("example");
    
    package = package
        .set_version("0.1.0")
        .set_description("deb-rust example")
        .set_architecture(DebArchitecture::Amd64)
        .with_depend("bash")
        .with_file(DebFile::from_path(
            "target/release/example",
            "/usr/bin/example",
        )?);
        
    package.build()?.write(File::create("example.deb")?)?;
    Ok(())
}
```

### Reading

```rs
use std::fs;
use std::fs::File;
use deb_rust::*;
use deb_rust::binary::*;

fn main() -> std::io::Result<()> {
    let package = DebPackage::from(File::open("example.deb")?)?;
    
    let name = package.name();
    let version = package.version();
    
    for file in package.files() {
        fs::write(file.path(), file.contents())?;
    }
    Ok(())
}
```

### Reading and Writing

```rs
use std::fs::File;
use deb_rust::*;
use deb_rust::binary::*;

fn main() -> std::io::Result<()> {
    let mut package = DebPackage::from(File::open("example.deb")?)?;

    package = package
        .set_name("rename")
        .set_description("some example idfk");
        
    package.build()?.write(File::create("new.deb")?)?;
    Ok(())
}
```
