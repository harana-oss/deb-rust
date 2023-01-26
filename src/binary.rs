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

//! Build and read binary Deb packages.
//!
//! Binary packages contain executable programs, documentation for said executables,
//! configuration files, libraries, etc. Basically, anything that's not source code.
//!
//! # Example
//!
//! ```
//! use std::fs::File;
//! use deb_rust::*;
//! use deb_rust::binary::*;
//!
//! fn main() -> std::io::Result<()> {
//!     let mut package = DebPackage::new("example");
//!
//!     package = package
//!         .set_version("0.1.0")
//!         .set_description("deb-rust example")
//!         .set_architecture(DebArchitecture::Amd64)
//!         .with_depend("bash")
//!         .with_file(DebFile::from_path(
//!             "target/release/example",
//!             "/usr/bin/example",
//!         )?);
//!
//!     package.build()?.write(File::create("example.deb")?)?;
//!
//!     Ok(())
//! ```

use crate::shared::*;

use std::borrow::Cow;
use std::fs;
use std::io::{Error, ErrorKind, Read, Write};
use std::path::{Path, PathBuf};

use regex::Regex;
use xz::read::XzDecoder;
use xz::write::XzEncoder;

// Used in DebPackage to store a package's metadata
// More about these fields here:
// https://www.debian.org/doc/debian-policy/ch-controlfields.html#binary-package-control-files-debian-control
#[derive(Debug)]
struct DebControl {
    name: String,
    version: String,
    priority: DebPriority,
    architecture: DebArchitecture,
    essential: bool,
    depends: Vec<String>,
    pre_depends: Vec<String>,
    recommends: Vec<String>,
    suggests: Vec<String>,
    breaks: Vec<String>,
    conflicts: Vec<String>,
    provides: Vec<String>,
    replaces: Vec<String>,
    enhances: Vec<String>,
    maintainer: String,
    description: String,
    homepage: String,
    built_using: Vec<[String; 2]>,
}

impl DebControl {
    // Converts DebControl into a dpkg-readable control file
    fn serialize(&self) -> Vec<u8> {
        let mut write_out = String::new();
        // Binding temporary values to longer living variables
        let depends = self.depends.join(", ");
        let pre_depends = self.pre_depends.join(", ");
        let recommends = self.recommends.join(", ");
        let suggests = self.suggests.join(", ");
        let breaks = self.breaks.join(", ");
        let conflicts = self.conflicts.join(", ");
        let enhances = self.enhances.join(", ");
        let built_using = {
            let mut output: Vec<String> = Vec::new();
            for build_depend in &self.built_using {
                output.push(format!("{} (= {})", build_depend[0], build_depend[1]));
            }
            output.join(", ")
        };
        let control = vec![
            ["Package", self.name.as_str()],
            ["Version", self.version.as_str()],
            ["Priority", self.priority.as_str()],
            ["Architecture", self.architecture.as_str()],
            [
                "Essential",
                match self.essential {
                    true => "yes",
                    false => "no",
                },
            ],
            ["Depends", depends.as_str()],
            ["Pre-Depends", pre_depends.as_str()],
            ["Recommends", recommends.as_str()],
            ["Suggests", suggests.as_str()],
            ["Breaks", breaks.as_str()],
            ["Conflicts", conflicts.as_str()],
            ["Enhances", enhances.as_str()],
            ["Maintainer", self.maintainer.as_str()],
            ["Description", self.description.as_str()],
            ["Homepage", self.homepage.as_str()],
            ["Built-Using", built_using.as_str()],
        ];
        for field in control {
            if !field[1].is_empty() {
                write_out = format!("{}{}: {}\n", write_out, field[0], field[1]);
            }
        }
        write_out.into_bytes()
    }

    // Converts a dpkg-readable control file into DebControl
    fn deserialize(control: Vec<u8>) -> std::io::Result<Self> {
        // Converts comma-separated lists to Vec<String>
        fn split_to_vec(input: &str) -> Vec<String> {
            input
                .split(',')
                .map(|str| str.trim().to_string())
                .collect::<Vec<String>>()
        }

        let mut output = Self {
            name: String::new(),
            version: String::new(),
            priority: DebPriority::Optional,
            architecture: DebArchitecture::All,
            essential: false,
            depends: Vec::new(),
            pre_depends: Vec::new(),
            recommends: Vec::new(),
            suggests: Vec::new(),
            breaks: Vec::new(),
            conflicts: Vec::new(),
            provides: Vec::new(),
            replaces: Vec::new(),
            enhances: Vec::new(),
            maintainer: String::new(),
            description: String::new(),
            homepage: String::new(),
            built_using: Vec::new(),
        };

        let mut control_string = match String::from_utf8(control) {
            Ok(string) => string,
            Err(e) => return Err(Error::new(ErrorKind::Other, e)),
        };

        // Splits control into Vec<&str> by line, and then splits
        // each line into Vec<&str> by the key-value separating colon,
        // ultimately resulting in Vec<Vec<&str>>
        let iterator = control_string
            .split('\n')
            .collect::<Vec<&str>>()
            .into_iter()
            .map(|str| {
                str.split(':')
                    .collect::<Vec<&str>>()
                    .into_iter()
                    .map(|str| str.trim())
                    .collect::<Vec<&str>>()
            })
            .collect::<Vec<Vec<&str>>>();

        for line in iterator {
            // This is to ensure that the trailing newline at a control file's
            // end doesn't cause an error
            if line.len() == 1 {
                continue;
            } else if line.len() != 2 {
                return Err(Error::new(ErrorKind::Other, "control file is invalid"));
            }
            // Matches the key and writes the value to the appropriate field
            match line[0] {
                "Package" => {
                    output.name = line[1].to_string();
                }
                "Version" => {
                    output.version = line[1].to_string();
                }
                "Priority" => {
                    output.priority = DebPriority::from(line[1])?;
                }
                "Architecture" => {
                    output.architecture = DebArchitecture::from(line[1])?;
                }
                "Essential" => {
                    output.essential = match line[1] {
                        "yes" => true,
                        "no" => false,
                        &_ => {
                            return Err(Error::new(ErrorKind::Other, "control file is invalid"));
                        }
                    }
                }
                "Depends" => {
                    output.depends = split_to_vec(line[1]);
                }
                "Pre-Depends" => {
                    output.pre_depends = split_to_vec(line[1]);
                }
                "Recommends" => {
                    output.recommends = split_to_vec(line[1]);
                }
                "Suggests" => {
                    output.suggests = split_to_vec(line[1]);
                }
                "Breaks" => {
                    output.breaks = split_to_vec(line[1]);
                }
                "Conflicts" => {
                    output.conflicts = split_to_vec(line[1]);
                }
                "Provides" => {
                    output.provides = split_to_vec(line[1]);
                }
                "Replaces" => {
                    output.replaces = split_to_vec(line[1]);
                }
                "Enhances" => {
                    output.enhances = split_to_vec(line[1]);
                }
                "Maintainer" => {
                    output.maintainer = line[1].to_string();
                }
                "Description" => {
                    output.description = line[1].to_string();
                }
                "Homepage" => {
                    output.homepage = line[1].to_string();
                }
                "Built-Using" => {
                    // Pulls the version number out of the `name (= ver)` format
                    // in Built-Using
                    // god i hate regex syntax
                    let ver_regex: Regex = Regex::new(r"\(= ([^()]*)\)$").unwrap();
                    let mut built_using: Vec<[String; 2]> = Vec::new();
                    let source = split_to_vec(line[1]);
                    for entry in source {
                        built_using.push([
                            entry.split(' ').collect::<Vec<&str>>()[0].to_string(),
                            match ver_regex.find(line[1]) {
                                Some(mat) => mat.as_str().to_string(),
                                None => {
                                    return Err(Error::new(
                                        ErrorKind::Other,
                                        "control file is invalid",
                                    ));
                                }
                            },
                        ]);
                    }
                }
                &_ => {
                    return Err(Error::new(ErrorKind::Other, "control file is invalid"));
                }
            }
        }

        Ok(output)
    }
}

/// A high-level structure representing a Deb package.
///
/// For binary package's, it may be helpful to read
/// [Debian's documentation on binary packages' metadata][1].
///
/// As well, you can read Debian's definition for the package's
/// [maintainer scripts][2].
///
/// [1]: https://www.debian.org/doc/debian-policy/ch-controlfields.html#binary-package-control-files-debian-control
/// [2]: https://www.debian.org/doc/debian-policy/ch-binary.html#maintainer-scripts
#[derive(Debug)]
pub struct DebPackage {
    control: DebControl,         // Package's metadata
    data: Vec<DebFile>,          // Package's contents
    config: Option<DebFile>,     // Package's config script
    preinst: Option<DebFile>,    // Package's preinstall script
    postinst: Option<DebFile>,   // Package's postinstall script
    prerm: Option<DebFile>,      // Package's preuninstall script
    postrm: Option<DebFile>,     // Package's postuninstall script
    compression: DebCompression, // Configures the package's compression standard
}

impl DebPackage {
    /// Creates a new DebPackage with `name` as it's name.
    pub fn new(name: &str) -> Self {
        Self {
            control: DebControl {
                name: name.to_string(),
                version: String::new(),
                priority: DebPriority::Optional,
                architecture: DebArchitecture::All,
                essential: false,
                depends: Vec::new(),
                pre_depends: Vec::new(),
                recommends: Vec::new(),
                suggests: Vec::new(),
                breaks: Vec::new(),
                conflicts: Vec::new(),
                provides: Vec::new(),
                replaces: Vec::new(),
                enhances: Vec::new(),
                maintainer: String::new(),
                description: String::new(),
                homepage: String::new(),
                built_using: Vec::new(),
            },
            data: Vec::new(),
            config: None,
            preinst: None,
            postinst: None,
            prerm: None,
            postrm: None,
            compression: DebCompression::Zstd,
        }
    }

    /// Reads a DebPackage from `input`.
    pub fn from<R: Read>(mut input: R) -> std::io::Result<Self> {
        DebArchive::read(input)?.to_package()
    }

    /// Sets the package's name.
    pub fn set_name(mut self, name: &str) -> Self {
        self.control.name = name.to_string();
        self
    }

    /// Sets the package's version.
    pub fn set_version(mut self, version: &str) -> Self {
        self.control.version = version.to_string();
        self
    }

    /// Sets the package's priority.
    pub fn set_priority(mut self, priority: DebPriority) -> Self {
        self.control.priority = priority;
        self
    }

    /// Sets the package's architecture.
    pub fn set_architecture(mut self, architecture: DebArchitecture) -> Self {
        self.control.architecture = architecture;
        self
    }

    /// Sets whether the package is essential.
    pub fn set_essential(mut self, essential: bool) -> Self {
        self.control.essential = essential;
        self
    }

    /// Adds a single dependency from &str.
    pub fn with_depend(mut self, depend: &str) -> Self {
        self.control.depends.push(depend.to_string());
        self
    }

    /// Adds a number of dependencies from Vec<&str>.
    pub fn with_depends(mut self, depends: Vec<&str>) -> Self {
        self.control
            .depends
            .append(&mut depends.iter().map(|str| str.to_string()).collect());
        self
    }

    /// Resets dependencies.
    pub fn no_depends(mut self) -> Self {
        self.control.depends = Vec::new();
        self
    }

    /// Adds a single pre-dependency from &str.
    pub fn with_pre_depend(mut self, depend: &str) -> Self {
        self.control.pre_depends.push(depend.to_string());
        self
    }

    /// Adds a number of pre-dependencies from Vec<&str>.
    pub fn with_pre_depends(mut self, depends: Vec<&str>) -> Self {
        self.control
            .pre_depends
            .append(&mut depends.iter().map(|str| str.to_string()).collect());
        self
    }

    /// Resets pre-dependencies.
    pub fn no_pre_depends(mut self) -> Self {
        self.control.pre_depends = Vec::new();
        self
    }

    /// Adds a single recommend from &str.
    pub fn with_recommend(mut self, recommend: &str) -> Self {
        self.control.recommends.push(recommend.to_string());
        self
    }

    /// Adds a number of recommends from Vec<&str>.
    pub fn with_recommends(mut self, recommends: Vec<&str>) -> Self {
        self.control
            .recommends
            .append(&mut recommends.iter().map(|str| str.to_string()).collect());
        self
    }

    /// Resets recommends.
    pub fn no_recommends(mut self) -> Self {
        self.control.recommends = Vec::new();
        self
    }

    /// Adds a single suggest from &str.
    pub fn with_suggest(mut self, suggest: &str) -> Self {
        self.control.suggests.push(suggest.to_string());
        self
    }

    /// Adds a number of suggests from Vec<&str>.
    pub fn with_suggests(mut self, suggests: Vec<&str>) -> Self {
        self.control
            .suggests
            .append(&mut suggests.iter().map(|str| str.to_string()).collect());
        self
    }

    /// Resets suggests.
    pub fn no_suggests(mut self) -> Self {
        self.control.suggests = Vec::new();
        self
    }

    /// Adds a single break from &str.
    pub fn with_break(mut self, conflict: &str) -> Self {
        self.control.breaks.push(conflict.to_string());
        self
    }

    /// Adds a number of breaks from Vec<&str>.
    pub fn with_breaks(mut self, conflicts: Vec<&str>) -> Self {
        self.control
            .breaks
            .append(&mut conflicts.iter().map(|str| str.to_string()).collect());
        self
    }

    /// Resets breaks.
    pub fn no_breaks(mut self) -> Self {
        self.control.breaks = Vec::new();
        self
    }

    /// Adds a single conflict from &str.
    pub fn with_conflict(mut self, conflict: &str) -> Self {
        self.control.conflicts.push(conflict.to_string());
        self
    }

    /// Adds a number of conflicts from Vec<&str>.
    pub fn with_conflicts(mut self, conflicts: Vec<&str>) -> Self {
        self.control
            .conflicts
            .append(&mut conflicts.iter().map(|str| str.to_string()).collect());
        self
    }

    /// Resets conflicts.
    pub fn no_conflicts(mut self) -> Self {
        self.control.conflicts = Vec::new();
        self
    }

    /// Adds a single provide from &str.
    pub fn with_provide(mut self, provide: &str) -> Self {
        self.control.provides.push(provide.to_string());
        self
    }

    /// Adds a number of provides from Vec<&str>.
    pub fn with_provides(mut self, provides: Vec<&str>) -> Self {
        self.control
            .provides
            .append(&mut provides.iter().map(|str| str.to_string()).collect());
        self
    }

    /// Resets provides.
    pub fn no_provides(mut self) -> Self {
        self.control.provides = Vec::new();
        self
    }

    /// Adds a single replace from &str.
    pub fn with_replace(mut self, replace: &str) -> Self {
        self.control.replaces.push(replace.to_string());
        self
    }

    /// Adds a number of replaces from Vec<&str>.
    pub fn with_replaces(mut self, replaces: Vec<&str>) -> Self {
        self.control
            .replaces
            .append(&mut replaces.iter().map(|str| str.to_string()).collect());
        self
    }

    /// Resets replaces.
    pub fn no_replaces(mut self) -> Self {
        self.control.replaces = Vec::new();
        self
    }

    /// Adds a single enhance from &str.
    pub fn with_enhance(mut self, enhance: &str) -> Self {
        self.control.enhances.push(enhance.to_string());
        self
    }

    /// Adds a number of enhances from Vec<&str>.
    pub fn with_enhances(mut self, enhances: Vec<&str>) -> Self {
        self.control
            .enhances
            .append(&mut enhances.iter().map(|str| str.to_string()).collect());
        self
    }

    /// Resets enhances.
    pub fn no_enhances(mut self) -> Self {
        self.control.enhances = Vec::new();
        self
    }

    /// Sets the package's maintainer.
    pub fn set_maintainer(mut self, maintainer: &str) -> Self {
        self.control.maintainer = maintainer.to_string();
        self
    }

    /// Sets the package's description.
    pub fn set_description(mut self, description: &str) -> Self {
        self.control.description = description.to_string();
        self
    }

    /// Sets the package's homepage.
    pub fn set_homepage(mut self, homepage: &str) -> Self {
        self.control.homepage = homepage.to_string();
        self
    }

    /// Adds a "built using" package.
    pub fn with_built_using(mut self, using: &str, version: &str) -> Self {
        self.control
            .built_using
            .push([using.to_string(), version.to_string()]);
        self
    }

    /// Resets built-using.
    pub fn no_built_using(mut self) -> Self {
        self.control.built_using = Vec::new();
        self
    }

    /// Adds a file to the package.
    pub fn with_file(mut self, file: DebFile) -> Self {
        self.data.push(file);
        self
    }

    /// Recursively adds directory `from` to package as `to`.
    ///
    /// This adds all files and sub-directories to `to`. For example, if you
    /// had a directory `test` containing the files `foo` and `bar`, then
    /// you can add those files as `/usr/bin/foo` and `/usr/bin/bar` with
    /// `with_dir("test", "/usr/bin")?;`
    ///
    /// This function isn't available when compiling on Windows, as it's utility
    /// relies on being able to read the modes of the directory's children,
    /// which is a feature Windows lacks.
    ///
    /// # Errors
    ///
    /// This function may return an error if `from` doesn't exist.
    ///
    /// # Example
    ///
    /// ```
    /// use deb_rust::binary::DebPackage;
    ///
    /// let mut package = DebPackage::new("example")
    ///     .with_dir("test", "/usr/bin").unwrap();
    /// ```
    #[cfg(unix)]
    pub fn with_dir<P>(mut self, from: P, to: P) -> std::io::Result<Self>
    where
        P: AsRef<Path>,
    {
        let mut path_from = PathBuf::new();
        let mut path_to = PathBuf::new();
        path_from.push(from);
        path_to.push(to);
        for file_result in walkdir::WalkDir::new(&path_from) {
            let file = file_result?;
            if file.path().is_file() {
                // Cutting the `from` directory out of the path
                let mut components = file.path().components();
                for _i in path_from.components() {
                    components.next();
                }
                self = self.with_file(DebFile::from_path(file.path(), path_to.join(components))?);
            }
        }
        Ok(self)
    }

    /// Removes all file's from the package.
    pub fn clear_files(mut self) -> Self {
        self.data = Vec::new();
        self
    }

    /// Sets config script from &str.
    pub fn config_from_str(mut self, script: &str) -> Self {
        self.config = Some(DebFile::from_buf(script.as_bytes().to_vec(), "config").is_exec());
        self
    }

    /// Sets config script from Vec<u8>.
    pub fn config_from_buf(mut self, script: Vec<u8>) -> Self {
        self.config = Some(DebFile::from_buf(script, "config").is_exec());
        self
    }

    /// Resets config script.
    pub fn no_config(mut self) -> Self {
        self.config = None;
        self
    }

    /// Sets preinst script from &str.
    pub fn preinst_from_str(mut self, script: &str) -> Self {
        self.preinst = Some(DebFile::from_buf(script.as_bytes().to_vec(), "preinst").is_exec());
        self
    }

    /// Sets preinst script from Vec<u8>.
    pub fn preinst_from_buf(mut self, script: Vec<u8>) -> Self {
        self.preinst = Some(DebFile::from_buf(script, "preinst").is_exec());
        self
    }

    /// Resets preinst script.
    pub fn no_preinst(mut self) -> Self {
        self.preinst = None;
        self
    }

    /// Sets postinst script from &str.
    pub fn postinst_from_str(mut self, script: &str) -> Self {
        self.postinst = Some(DebFile::from_buf(script.as_bytes().to_vec(), "postinst").is_exec());
        self
    }

    /// Sets postinst script from Vec<u8>.
    pub fn postinst_from_buf(mut self, script: Vec<u8>) -> Self {
        self.postinst = Some(DebFile::from_buf(script, "postinst").is_exec());
        self
    }

    /// Resets postinst script.
    pub fn no_postinst(mut self) -> Self {
        self.postinst = None;
        self
    }

    /// Sets prerm script from &str.
    pub fn prerm_from_str(mut self, script: &str) -> Self {
        self.prerm = Some(DebFile::from_buf(script.as_bytes().to_vec(), "prerm").is_exec());
        self
    }

    /// Sets prerm script from Vec<u8>.
    pub fn prerm_from_buf(mut self, script: Vec<u8>) -> Self {
        self.prerm = Some(DebFile::from_buf(script, "prerm").is_exec());
        self
    }

    /// Resets prerm script.
    pub fn no_prerm(mut self) -> Self {
        self.prerm = None;
        self
    }

    /// Sets postrm script from &str.
    pub fn postrm_from_str(mut self, script: &str) -> Self {
        self.postrm = Some(DebFile::from_buf(script.as_bytes().to_vec(), "postrm").is_exec());
        self
    }

    /// Sets postrm script from Vec<u8>.
    pub fn postrm_from_buf(mut self, script: Vec<u8>) -> Self {
        self.postrm = Some(DebFile::from_buf(script, "postrm").is_exec());
        self
    }

    /// Resets postrm script.
    pub fn no_postrm(mut self) -> Self {
        self.postrm = None;
        self
    }

    /// Sets the package's compression standard.
    pub fn set_compression(mut self, compression: DebCompression) -> Self {
        self.compression = compression;
        self
    }

    /// Returns the package's name.
    pub fn name(&self) -> &str {
        &self.control.name
    }

    /// Returns the package's version.
    pub fn version(&self) -> &str {
        &self.control.version
    }

    /// Returns the package's priority.
    pub fn priority(&self) -> &DebPriority {
        &self.control.priority
    }

    /// Returns the package's architecture.
    pub fn architecture(&self) -> &DebArchitecture {
        &self.control.architecture
    }

    /// Returns whether the package is essential.
    pub fn essential(&self) -> bool {
        self.control.essential
    }

    /// Returns the package's depends.
    pub fn depends(&self) -> &Vec<String> {
        &self.control.depends
    }

    /// Returns the package's pre-depends.
    pub fn pre_depends(&self) -> &Vec<String> {
        &self.control.pre_depends
    }

    /// Returns the package's recommends.
    pub fn recommends(&self) -> &Vec<String> {
        &self.control.recommends
    }

    /// Returns the package's suggests.
    pub fn suggests(&self) -> &Vec<String> {
        &self.control.suggests
    }

    /// Returns the package's breaks.
    pub fn breaks(&self) -> &Vec<String> {
        &self.control.breaks
    }

    /// Returns the package's conflicts.
    pub fn conflicts(&self) -> &Vec<String> {
        &self.control.conflicts
    }

    /// Returns the package's provides.
    pub fn provides(&self) -> &Vec<String> {
        &self.control.provides
    }

    /// Returns the package's replaces.
    pub fn replaces(&self) -> &Vec<String> {
        &self.control.replaces
    }

    /// Returns the package's enhances.
    pub fn enhances(&self) -> &Vec<String> {
        &self.control.enhances
    }

    /// Returns the package's maintainer.
    pub fn maintainer(&self) -> &str {
        &self.control.maintainer
    }

    /// Returns the package's description.
    pub fn description(&self) -> &str {
        &self.control.description
    }

    /// Returns the package's homepage.
    pub fn homepage(&self) -> &str {
        &self.control.homepage
    }

    /// Returns the packages this package was built with.
    pub fn built_using(&self) -> &Vec<[String; 2]> {
        &self.control.built_using
    }

    /// Returns a vector of the packages files.
    pub fn files(&self) -> &Vec<DebFile> {
        &self.data
    }

    /// Returns the package's config script.
    pub fn config(&self) -> Option<&Vec<u8>> {
        match &self.config {
            Some(file) => Some(file.contents()),
            None => None,
        }
    }

    /// Returns the package's preinst script.
    pub fn preinst(&self) -> Option<&Vec<u8>> {
        match &self.preinst {
            Some(file) => Some(file.contents()),
            None => None,
        }
    }

    /// Returns the package's postinst script.
    pub fn postinst(&self) -> Option<&Vec<u8>> {
        match &self.postinst {
            Some(file) => Some(file.contents()),
            None => None,
        }
    }

    /// Returns the package's prerm script.
    pub fn prerm(&self) -> Option<&Vec<u8>> {
        match &self.prerm {
            Some(file) => Some(file.contents()),
            None => None,
        }
    }

    /// Returns the package's postrm script.
    pub fn postrm(&self) -> Option<&Vec<u8>> {
        match &self.postrm {
            Some(file) => Some(file.contents()),
            None => None,
        }
    }

    /// Returns the package's compression standard.
    pub fn compression(&self) -> &DebCompression {
        &self.compression
    }

    /// Builds the package into a DebArchive struct.
    pub fn build(&self) -> std::io::Result<DebArchive> {
        let mut output = DebArchive {
            control: Vec::new(),
            data: Vec::new(),
            compression: match self.compression {
                DebCompression::Xz => DebCompression::Xz,
                DebCompression::Zstd => DebCompression::Zstd,
            },
        };

        // Creating tar archives
        let mut control_tar = tar::Builder::new(Vec::new());
        let mut data_tar = tar::Builder::new(Vec::new());

        // Creating DebFile's from control and scripts
        let control_file = Some(DebFile::from_buf(self.control.serialize(), "control"));
        let mut control_vec = vec![
            &control_file,
            &self.config,
            &self.preinst,
            &self.postinst,
            &self.prerm,
            &self.postrm,
        ];

        // Adding files to control tar
        for file in control_vec.into_iter().flatten() {
            let mut file_header = tar::Header::new_gnu();
            // We don't have to worry about the path being absolute here as all
            // scripts can only have relative paths using the struct's methods
            file_header.set_path(file.path())?;
            file_header.set_size(file.contents().len().try_into().unwrap());
            file_header.set_mode(*file.mode());
            file_header.set_cksum();
            control_tar.append(&file_header, file.contents().as_slice())?;
        }

        // Adding files to data tar
        for file in &self.data {
            let mut file_header = tar::Header::new_gnu();
            // We have to strip the root directory if the path is absolute
            // as the tar library doesn't allow absolute paths
            if file.path().is_absolute() {
                match file.path().strip_prefix("/") {
                    Ok(path) => {
                        file_header.set_path(path)?;
                    }
                    Err(e) => {
                        return Err(Error::new(ErrorKind::Other, e));
                    }
                }
            } else {
                file_header.set_path(file.path())?;
            }
            file_header.set_size(file.contents().len().try_into().unwrap());
            file_header.set_mode(*file.mode());
            file_header.set_cksum();
            data_tar.append(&file_header, file.contents().as_slice())?;
        }

        // Compressing tar archives to DebArchive struct
        match self.compression {
            DebCompression::Xz => {
                let mut control_xz = XzEncoder::new(&mut output.control, 9);
                control_xz.write_all(control_tar.into_inner()?.as_slice())?;
                control_xz.finish()?;
                let mut data_xz = XzEncoder::new(&mut output.data, 9);
                data_xz.write_all(data_tar.into_inner()?.as_slice())?;
                data_xz.finish()?;
            }
            DebCompression::Zstd => {
                zstd::stream::copy_encode(
                    control_tar.into_inner()?.as_slice(),
                    &mut output.control,
                    0,
                )?;
                zstd::stream::copy_encode(data_tar.into_inner()?.as_slice(), &mut output.data, 0)?;
            }
        }

        Ok(output)
    }
}

/// An intermediary layer between the DebPackage struct and an actual .deb file.
///
/// This struct allows you to read and write built packages from and to the filesystem.
///
/// The contents of a DebArchive cannot be directly manipulated. To modify a DebArchive,
/// you must first convert it to a DebPackage with the `to_package()` method, or open the
/// file using DebPackage's `from()` function.
pub struct DebArchive {
    control: Vec<u8>,            // Compressed tar archive containing package's metadata
    data: Vec<u8>,               // Compressed tar archive containing the package's contents
    compression: DebCompression, // Configuration for the package's compression standard
}

impl DebArchive {
    /// Writes package to `output`.
    pub fn write<W: Write>(&self, mut output: W) -> std::io::Result<()> {
        // Parsing the name of the control and data archives
        let (control_name, data_name) = match self.compression {
            DebCompression::Xz => ("control.tar.xz", "data.tar.xz"),
            DebCompression::Zstd => ("control.tar.zst", "data.tar.zst"),
        };

        // Creating final archive
        let mut archive = ar::Builder::new(Vec::new());

        // Frankly not quite sure what the "debian-binary" file is for,
        // but it just contains the text "2.0"
        let mut header = ar::Header::new("debian-binary".as_bytes().to_vec(), 4);
        header.set_mode(33188);
        archive.append(&header, "2.0\n".as_bytes())?;

        // Adding control to archive
        let mut header = ar::Header::new(
            control_name.as_bytes().to_vec(),
            self.control.len().try_into().unwrap(),
        );
        header.set_mode(33188);
        archive.append(&header, self.control.as_slice())?;

        // Adding data to archive
        let mut header = ar::Header::new(
            data_name.as_bytes().to_vec(),
            self.data.len().try_into().unwrap(),
        );
        header.set_mode(33188);
        archive.append(&header, self.data.as_slice())?;

        // Writing archive to `out`
        output.write(&archive.into_inner()?);
        Ok(())
    }

    /// Reads package from `input`.
    pub fn read<R: Read>(mut input: R) -> std::io::Result<Self> {
        // Preparing output
        let mut output = Self {
            control: Vec::new(),
            data: Vec::new(),
            compression: DebCompression::Zstd,
        };

        // Creating Archive reader and iterator
        let mut archive = ar::Archive::new(input);

        // Skipping `debian-binary` file
        archive.next_entry().unwrap()?;

        // Reading control archive
        match archive.next_entry() {
            Some(entry) => {
                entry?.read_to_end(&mut output.control)?;
            }
            None => {
                return Err(Error::new(
                    ErrorKind::Other,
                    "deb package is missing archive",
                ));
            }
        }

        // Reading data archive
        let mut data_entry = match archive.next_entry() {
            Some(entry) => entry?,
            None => {
                return Err(Error::new(
                    ErrorKind::Other,
                    "deb package is missing archive",
                ))
            }
        };
        data_entry.read_to_end(&mut output.data)?;

        // Reading data header to parse compression
        let mut data_identifier = String::new();
        match String::from_utf8(data_entry.header().identifier().to_vec()) {
            Ok(id) => {
                data_identifier = id;
            }
            Err(e) => {
                return Err(Error::new(ErrorKind::Other, e));
            }
        }
        if let Some(ext) = Path::new(&data_identifier).extension() {
            if ext.to_str() == Some("xz") {
                output.compression = DebCompression::Xz;
            }
        }

        Ok(output)
    }

    /// Converts DebArchive to DebPackage.
    ///
    /// # Errors
    ///
    /// This function may return an error if the archive's control file (where all
    /// of a package's metadata is stored) contains invalid syntax, or if part of the
    /// package is corrupted and can't be read.
    pub fn to_package(&self) -> std::io::Result<DebPackage> {
        let mut output = DebPackage::new("");
        output.compression = match self.compression {
            DebCompression::Xz => DebCompression::Xz,
            DebCompression::Zstd => DebCompression::Zstd,
        };

        // Decompressing control and data archives
        let mut control_buf: Vec<u8> = Vec::new();
        let mut data_buf: Vec<u8> = Vec::new();
        match self.compression {
            DebCompression::Xz => {
                XzDecoder::new(self.control.as_slice()).read_to_end(&mut control_buf)?;
                XzDecoder::new(self.data.as_slice()).read_to_end(&mut data_buf)?;
            }
            DebCompression::Zstd => {
                zstd::stream::copy_decode(self.control.as_slice(), &mut control_buf)?;
                zstd::stream::copy_decode(self.data.as_slice(), &mut data_buf)?;
            }
        };
        let mut control_tar = tar::Archive::new(control_buf.as_slice());
        let mut data_tar = tar::Archive::new(data_buf.as_slice());

        // Parsing control archive
        for entry_result in control_tar.entries()? {
            let mut entry = entry_result?;
            let mut buf: Vec<u8> = Vec::new();
            entry.read_to_end(&mut buf);
            if entry.path()? == Cow::Borrowed(Path::new("control")) {
                // Converting control file into DebControl struct
                output.control = DebControl::deserialize(buf)?;
            } else if entry.path()? == Cow::Borrowed(Path::new("config")) {
                output = output.config_from_buf(buf);
            } else if entry.path()? == Cow::Borrowed(Path::new("preinst")) {
                output = output.preinst_from_buf(buf);
            } else if entry.path()? == Cow::Borrowed(Path::new("postinst")) {
                output = output.postinst_from_buf(buf);
            } else if entry.path()? == Cow::Borrowed(Path::new("prerm")) {
                output = output.prerm_from_buf(buf);
            } else if entry.path()? == Cow::Borrowed(Path::new("postrm")) {
                output = output.postrm_from_buf(buf);
            }
        }

        // Converting data entries to DebFile structs
        for entry_result in data_tar.entries()? {
            let mut entry = entry_result?;
            let mut buf: Vec<u8> = Vec::new();
            entry.read_to_end(&mut buf);
            if let Cow::Borrowed(path) = entry.path()? {
                output.data.push(
                    DebFile::from_buf(buf, format!("/{}", path.display()))
                        .set_mode(entry.header().mode()?),
                )
            }
        }

        Ok(output)
    }
}
