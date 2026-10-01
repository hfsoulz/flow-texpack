// Copyright (C) 2026-2026 Andreas Widen <aw@luflow.net>
//
// This software is provided 'as-is', without any express or implied
// warranty.  In no event will the authors be held liable for any damages
// arising from the use of this software.
//
// Permission is granted to anyone to use this software for any purpose,
// including commercial applications, and to alter it and redistribute it
// freely, subject to the following restrictions:
//
// 1. The origin of this software must not be misrepresented; you must not
//    claim that you wrote the original software. If you use this software
//    in a product, an acknowledgment in the product documentation would be
//    appreciated but is not required.
// 2. Altered source versions must be plainly marked as such, and must not be
//    misrepresented as being the original software.
// 3. This notice may not be removed or altered from any source distribution.

//! `flow-texpack` is a program that will allow you to generate texture atlas from input images (BMP,
//! HDR, JPG, PNG, TGA, TIFF, WEBP). The application generates both texture atlas and descriptions
//! file that can be read by a game.
//!
//! ## Usage
//! Show available options:
//! ```sh
//! flow-texpack -h
//! ```
//!
//! or
//!
//! ```sh
//! flow-texpack --help
//! ```
//!
//! ## Examples
//!
//! Generate from input `data/characters` and `data/tiles`, write output to `out/atlas` and enable
//! the options: `premultiply` pixels by their alpha channel, `trim` excess transparency off the
//! textures, `remove duplicate textures` from the atlas, enable `rotation` of textures 90 degrees
//! clockwise, `pad` each texture by 2 pixels and finally enable `verbose` output mode.
//!
//! ```sh
//! flow-texpack -i data/characters data/tiles -o out/atlas -m -t -u -r -p 2 -v
//! ```

#[doc(hidden)]
pub mod texpack;

// re-export types:
#[doc(hidden)]
pub use crate::texpack::app::App;

#[doc(hidden)]
pub use crate::texpack::app::get_atlas_image_extension;

#[doc(hidden)]
pub use crate::texpack::app::create_dir_all;

#[doc(hidden)]
pub use crate::texpack::app::remove_dir_all;

#[doc(hidden)]
pub use crate::texpack::app::remove_file;

#[doc(hidden)]
pub use crate::texpack::app::exists_dir;

#[doc(hidden)]
pub use crate::texpack::app::exists_file;

#[doc(hidden)]
pub use crate::texpack::app::write_file_sync;

#[doc(hidden)]
pub use crate::texpack::packer::Packer;

#[doc(hidden)]
pub use crate::texpack::packer::PackerError;

#[doc(hidden)]
pub use crate::texpack::texture::Texture;

#[doc(hidden)]
pub use crate::texpack::texture::TextureError;
