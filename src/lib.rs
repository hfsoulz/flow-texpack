// flow-texpack: A program that will allow you to generate texture atlas.
// zlib License (see LICENSE)

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
