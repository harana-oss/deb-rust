/*
    deb-rust - Rust library for building and reading Deb packages
    Copyright (C) 2023  NotSludgeBomb

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
#[cfg(unix)]
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

/// Represents the [various architectures Deb supports](https://wiki.debian.org/SupportedArchitectures).
#[derive(Debug, PartialEq, Eq)]
pub enum DebArchitecture {
    /// For architecture independent packages, such as interpreted software
    /// or configuration files.
    All,
    Alpha,
    /// Arm versions 5T and 6
    Armel,
    /// Armv7 (hard float)
    Armhf,
    /// Armv8
    Arm64,
    Hppa,
    /// 32-bit x86
    I386,
    /// 64-bit x86_64
    Amd64,
    Ia64,
    M68k,
    Mips,
    /// Little-endian 32-bit
    Mipsel,
    /// Little-endian 64-bit
    Mips64el,
    PowerPC,
    Ppc64,
    Ppc64el,
    Riscv64,
    S390x,
    Sh4,
    Sparc4,
    X32,
    /// 32-bit x86 for GNU/Hurd
    HurdI386,
    /// 32-bit x86 for FreeBSD
    KFreebsdI386,
    /// 64-bit x86_64 for FreeBSD
    KFreebsdAmd64,
}

impl DebArchitecture {
    /// Converts DebArchitecture to &str.
    pub fn as_str(&self) -> &str {
        match self {
            DebArchitecture::All => "all",
            DebArchitecture::Alpha => "Alpha",
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
            DebArchitecture::Ppc64 => "PPC64",
            DebArchitecture::Ppc64el => "ppc64el",
            DebArchitecture::Riscv64 => "riscv64",
            DebArchitecture::S390x => "s390x",
            DebArchitecture::Sh4 => "SH4",
            DebArchitecture::Sparc4 => "sparc4",
            DebArchitecture::X32 => "x32",
            DebArchitecture::HurdI386 => "hurd-i386",
            DebArchitecture::KFreebsdI386 => "kfreebsd-i386",
            DebArchitecture::KFreebsdAmd64 => "kfreebsd-amd64",
        }
    }

    /// Converts &str to DebArchitecture.
    ///
    /// This function will return an error if the given string doesn't match
    /// any architecture name.
    pub fn from(input: &str) -> std::io::Result<Self> {
        match input {
            "all" => Ok(DebArchitecture::All),
            "Alpha" => Ok(DebArchitecture::Alpha),
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
            "PPC64" => Ok(DebArchitecture::Ppc64),
            "ppc64el" => Ok(DebArchitecture::Ppc64el),
            "riscv64" => Ok(DebArchitecture::Riscv64),
            "s390x" => Ok(DebArchitecture::S390x),
            "SH4" => Ok(DebArchitecture::Sh4),
            "sparc4" => Ok(DebArchitecture::Sparc4),
            "x32" => Ok(DebArchitecture::X32),
            "hurd-i386" => Ok(DebArchitecture::HurdI386),
            "kfreebsd-i386" => Ok(DebArchitecture::KFreebsdI386),
            "kfreebsd-amd64" => Ok(DebArchitecture::KFreebsdAmd64),
            &_ => Err(Error::new(ErrorKind::Other, "invalid architecture name")),
        }
    }
}

/// Used for [Deb's Priority field](https://www.debian.org/doc/debian-policy/ch-archive.html#s-priorities).
#[derive(Debug, PartialEq, Eq)]
pub enum DebPriority {
    Required,
    Important,
    Standard,
    Optional,
    Extra,
}

impl DebPriority {
    /// Converts DebPriority to &str.
    pub fn as_str(&self) -> &str {
        match self {
            DebPriority::Required => "required",
            DebPriority::Important => "important",
            DebPriority::Standard => "standard",
            DebPriority::Optional => "optional",
            DebPriority::Extra => "extra",
        }
    }

    /// Converts &str to DebPriority.
    ///
    /// This function will return in error if the given string doesn't match
    /// any priority name.
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

/// Used to configure which compression format is used for data and control archives.
///
/// Zstd is preferred, though XZ is available as a legacy option.
#[derive(Debug, PartialEq, Eq)]
pub enum DebCompression {
    Xz,
    Zstd,
}

/// Used in the DebPackage struct to represent files in a package's archives.
///
/// This struct contains the file's contents, permissions, and it's path in
/// the final package.
#[derive(Debug)]
pub struct DebFile {
    contents: Vec<u8>, // The contents of the file
    mode: u32,         // The file's permissions in octal form
    path: PathBuf,     // The path the file goes to in the archive
}

impl DebFile {
    /// Creates a DebFile from a path.
    ///
    /// `from` is a path to a file on your system that you're trying to add to the package.
    /// `to` is where the file will go once the package is installed on a user's system.
    ///
    /// On Unix systems, the file's mode will automatically be set based on `from`.
    /// On Windows, the file's mode will be set to `33188`.
    ///
    /// # Errors
    ///
    /// This function will return an error if `from` does not exist.
    ///
    /// # Example
    ///
    /// ```
    /// use deb_rust::DebFile;
    /// use deb_rust::binary::DebPackage;
    ///
    /// let mut package = DebPackage::new("example")
    ///     .with_file(DebFile::from_path(
    ///         "target/release/example",
    ///         "/usr/bin/example",
    ///     ).unwrap());
    /// ```
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

    /// Creates a DebFile from a buffer.
    ///
    /// `buf` is a buffer which will be added as the file's contents.
    /// `to` is where the file will go once the package is installed on a user's system.
    ///
    /// The file's mode is set to 33188. Permission's must be managed manually.
    ///
    /// # Example
    ///
    /// ```
    /// use deb_rust::DebFile;
    /// use deb_rust::binary::DebPackage;
    ///
    /// let mut package = DebPackage::new("example")
    ///     .with_file(DebFile::from_buf(
    ///         "#!/usr/bin/bash\necho Hello world!"
    ///             .as_bytes()
    ///             .to_vec(),
    ///         "/usr/bin/example",
    ///     ).is_exec());
    /// ```
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

    /// Sets the file's mode to have executable permissions.
    pub fn is_exec(mut self) -> Self {
        self.mode = 33261;
        self
    }

    /// Sets the file's mode to have read/write permissions, without executable.
    pub fn is_conf(mut self) -> Self {
        self.mode = 33188;
        self
    }

    /// Sets the file's contents to `contents`.
    pub fn set_contents(mut self, contents: Vec<u8>) -> Self {
        self.contents = contents;
        self
    }

    /// Sets the file's mode to `mode`.
    pub fn set_mode(mut self, mode: u32) -> Self {
        self.mode = mode;
        self
    }

    /// Sets the file's path to `to`.
    pub fn set_path<T: AsRef<std::ffi::OsStr>>(mut self, to: T) -> Self {
        self.path = PathBuf::from(&to);
        self
    }

    /// Returns the file's contents.
    pub fn contents(&self) -> &Vec<u8> {
        &self.contents
    }

    /// Returns the file's mode.
    pub fn mode(&self) -> &u32 {
        &self.mode
    }

    /// Returns the file's path.
    pub fn path(&self) -> &PathBuf {
        &self.path
    }
}
