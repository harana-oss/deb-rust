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

//! A pure Rust library for building and reading Deb packages.
//!
//! As of now, deb-rust provides one interface for binary Deb packages, with planned
//! support for source and source-control packages.

#[allow(unused)]
pub mod binary;
mod shared;
#[cfg(test)]
mod test;

pub use shared::*;
