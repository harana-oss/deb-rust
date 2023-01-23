/*
    deb-rust - Rust library for building and reading Deb packages
    Copyright (C) 2022  NotSludgeBomb

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

use std::fs;
use std::io::{Error, ErrorKind};
use std::path::PathBuf;

use crate::binary::*;
use crate::*;

#[test]
fn build_simple_package() -> std::io::Result<()> {
    DebPackage::new("test")
        .set_version("0.1.0")
        .set_architecture(DebArchitecture::Amd64)
        .with_depend("bash")
        .set_maintainer("NotSludgeBomb <notsludgebomb@protonmail.com>")
        .set_description("test package for deb-rust")
        .with_file(DebFile::from_buf(
            "#!/usr/bin/bash\necho hello world!"
                .to_string()
                .as_bytes()
                .to_vec(),
            33188,
            PathBuf::from("/usr/bin/hello"),
        ))
        .build()?
        .write(fs::File::create("test.deb")?)?;
    Ok(())
}

#[test]
fn read_simple_package() -> std::io::Result<()> {
    let reader = DebPackage::from(fs::File::open("tester.deb")?)?;

    let checks = [
        reader.name() == "test",
        reader.version() == "0.1.0",
        reader.priority() == &DebPriority::Optional,
        reader.architecture() == &DebArchitecture::Amd64,
        reader.essential() == false,
        reader.depends() == &vec!["bash".to_string()],
        reader.maintainer() == "NotSludgeBomb <notsludgebomb@protonmail.com>",
        reader.description() == "test package for deb-rust",
    ];

    for i in checks {
        if !i {
            return Err(Error::new(
                ErrorKind::Other,
                "value of read field is incorrect",
            ));
        }
    }

    Ok(())
}
