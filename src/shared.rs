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
use std::path::{Path, PathBuf};
#[cfg(unix)]
use std::os::unix::fs::MetadataExt;

// Represents the various architectures Deb supports, according to
// https://wiki.debian.org/SupportedArchitectures
#[derive(Debug, PartialEq, Eq)]
pub enum DebArchitecture {
    All,
    Alpha,
    Arm,
    Armel,
    Armhf,
    Arm64,
    Hppa,
    I386,
    Amd64,
    Ia64,
    M68k,
    Mips,
    Mipsel,
    Mips64el,
    PowerPC,
    PowerSPE,
    Ppc64,
    Ppc64el,
    Riscv64,
    S390,
    S390x,
    Sh4,
    Sparc4,
    X32,
    HurdI386,
    NetbsdI386,
    NetbsdAlpha,
    KFreebsdI386,
    KFreebsdAmd64,
}

impl DebArchitecture {
    // Converts DebArchitecture to &str
    pub fn as_str(&self) -> &str {
        match self {
            DebArchitecture::All => "all",
            DebArchitecture::Alpha => "Alpha",
            DebArchitecture::Arm => "Arm",
            DebArchitecture::Armel => "Armel",
            DebArchitecture::Armhf => "armhf",
            DebArchitecture::Arm64 => "arm64",
            DebArchitecture::Hppa => "hppa",
            DebArchitecture::I386 => "i386",
            DebArchitecture::Amd64 => "amd64",
            DebArchitecture::Ia64 => "ia64",
            DebArchitecture::M68k => "m68k",
            DebArchitecture::Mips => "mips",
            DebArchitecture::Mipsel => "mipsel",
            DebArchitecture::Mips64el => "mips64el",
            DebArchitecture::PowerPC => "PowerPC",
            DebArchitecture::PowerSPE => "PowerSPE",
            DebArchitecture::Ppc64 => "PPC64",
            DebArchitecture::Ppc64el => "ppc64el",
            DebArchitecture::Riscv64 => "riscv64",
            DebArchitecture::S390 => "s390",
            DebArchitecture::S390x => "s390x",
            DebArchitecture::Sh4 => "SH4",
            DebArchitecture::Sparc4 => "sparc4",
            DebArchitecture::X32 => "x32",
            DebArchitecture::HurdI386 => "hurd-i386",
            DebArchitecture::NetbsdI386 => "netbsd-i386",
            DebArchitecture::NetbsdAlpha => "netbsd-alpha",
            DebArchitecture::KFreebsdI386 => "kfreebsd-i386",
            DebArchitecture::KFreebsdAmd64 => "kfreebsd-amd64",
        }
    }

    // Converts &str to DebArchitecture
    pub fn from(input: &str) -> std::io::Result<Self> {
        match input {
            "all" => Ok(DebArchitecture::All),
            "Alpha" => Ok(DebArchitecture::Alpha),
            "Arm" => Ok(DebArchitecture::Arm),
            "Armel" => Ok(DebArchitecture::Armel),
            "armhf" => Ok(DebArchitecture::Armhf),
            "arm64" => Ok(DebArchitecture::Arm64),
            "hppa" => Ok(DebArchitecture::Hppa),
            "i386" => Ok(DebArchitecture::I386),
            "amd64" => Ok(DebArchitecture::Amd64),
            "ia64" => Ok(DebArchitecture::Ia64),
            "m68k" => Ok(DebArchitecture::M68k),
            "mips" => Ok(DebArchitecture::Mips),
            "mipsel" => Ok(DebArchitecture::Mipsel),
            "mips64el" => Ok(DebArchitecture::Mips64el),
            "PowerPC" => Ok(DebArchitecture::PowerPC),
            "PowerSPE" => Ok(DebArchitecture::PowerSPE),
            "PPC64" => Ok(DebArchitecture::Ppc64),
            "ppc64el" => Ok(DebArchitecture::Ppc64el),
            "riscv64" => Ok(DebArchitecture::Riscv64),
            "s390" => Ok(DebArchitecture::S390),
            "s390x" => Ok(DebArchitecture::S390x),
            "SH4" => Ok(DebArchitecture::Sh4),
            "sparc4" => Ok(DebArchitecture::Sparc4),
            "x32" => Ok(DebArchitecture::X32),
            "hurd-i386" => Ok(DebArchitecture::HurdI386),
            "netbsd-i386" => Ok(DebArchitecture::NetbsdI386),
            "netbsd-alpha" => Ok(DebArchitecture::NetbsdAlpha),
            "kfreebsd-i386" => Ok(DebArchitecture::KFreebsdI386),
            "kfreebsd-amd64" => Ok(DebArchitecture::KFreebsdAmd64),
            &_ => Err(Error::new(ErrorKind::Other, "invalid architecture name")),
        }
    }
}

// Used for Deb's Priority field
// This is described in Debian's official documentation here:
// https://www.debian.org/doc/debian-policy/ch-controlfields.html#priority
#[derive(Debug, PartialEq, Eq)]
pub enum DebPriority {
    Required,
    Important,
    Standard,
    Optional,
    Extra,
}

impl DebPriority {
    // Converts DebPriority to &str
    pub fn as_str(&self) -> &str {
        match self {
            DebPriority::Required => "required",
            DebPriority::Important => "important",
            DebPriority::Standard => "standard",
            DebPriority::Optional => "optional",
            DebPriority::Extra => "extra",
        }
    }

    // Converts &str to DebPriority
    pub fn from(input: &str) -> std::io::Result<Self> {
        match input {
            "required" => Ok(DebPriority::Required),
            "important" => Ok(DebPriority::Important),
            "standard" => Ok(DebPriority::Standard),
            "optional" => Ok(DebPriority::Optional),
            "extra" => Ok(DebPriority::Extra),
            &_ => Err(Error::new(ErrorKind::Other, "invalid priority name")),
        }
    }
}

// Used to configure which compression format is used for data and control archives
#[derive(Debug, PartialEq, Eq)]
pub enum DebCompression {
    Xz,
    Zstd,
}

// Used in the abstracted DebPackage struct to represent files in a package's archives
#[derive(Debug)]
pub struct DebFile {
    contents: Vec<u8>, // The contents of the file
    mode: u32,         // The file's permissions in octal form
    path: PathBuf,     // The path the file goes to in the archive
}

impl DebFile {
    // Creates DebFile from AsRef<Path>
    #[cfg(unix)]
    pub fn from_path<F, T>(from: F, to: T) -> std::io::Result<Self>
    where
        F: AsRef<Path>,
        T: AsRef<std::ffi::OsStr>,
    {
        Ok(Self {
            contents: fs::read(&from)?,
            mode: fs::File::open(&from)?.metadata()?.mode(),
            path: PathBuf::from(&to),
        })
    }

    // Same function but for Windows, as file modes are a Unix feature
    #[cfg(windows)]
    pub fn from_path<F, T>(from: F, to: T) -> std::io::Result<Self>
    where
        F: AsRef<Path>,
        T: AsRef<std::ffi::OsStr>,
    {
        Ok(Self {
            contents: fs::read(&from)?,
            mode: 33188,
            path: PathBuf::from(&to),
        })
    }

    // Creates DebFile from Vec<u8>
    pub fn from_buf<T>(buf: Vec<u8>, to: T) -> Self
    where
        T: AsRef<std::ffi::OsStr>,
    {
        Self {
            contents: buf,
            mode: 33188,
            path: PathBuf::from(&to),
        }
    }

    // Sets the file's mode to have executable permissions
    pub fn is_exec(mut self) -> Self {
        self.mode = 33261;
        self
    }

    // Sets the file's mode to have rw- permissions
    pub fn is_conf(mut self) -> Self {
        self.mode = 33188;
        self
    }

    // Sets the file's contents
    pub fn set_contents(mut self, contents: Vec<u8>) -> Self {
        self.contents = contents;
        self
    }

    // Sets the file's mode
    pub fn set_mode(mut self, mode: u32) -> Self {
        self.mode = mode;
        self
    }

    // Sets the file's path
    pub fn set_path<T: AsRef<std::ffi::OsStr>>(mut self, to: T) -> Self {
        self.path = PathBuf::from(&to);
        self
    }

    // Returns the file's contents
    pub fn contents(&self) -> &Vec<u8> {
        &self.contents
    }

    // Returns the file's mode
    pub fn mode(&self) -> &u32 {
        &self.mode
    }

    // Returns the file's path
    pub fn path(&self) -> &PathBuf {
        &self.path
    }
}
