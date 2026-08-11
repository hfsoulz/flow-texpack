// flow-texpack: A program that will allow you to generate texture atlas.
// zlib License (see LICENSE)

use crate::texpack::app::AtlasImage;
use crate::texpack::packer::MAX_SIZE;

use image::GenericImageView;
use image::{DynamicImage, ImageBuffer, ImageReader, Rgba, RgbaImage, imageops::FilterType};
use log::info;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::PathBuf;

/// Specifies the different error types that can occur.
#[derive(PartialEq, Clone, Debug)]
pub enum TextureError {
    /// Invalid argument
    InvalidArg,
}

/// Specifies the properties of a `Texture`.
#[derive(Clone, Debug)]
pub struct Texture {
    /// is the file path.
    pub file_path: PathBuf,
    /// is the file name.
    pub file_name: String,
    /// is the width.
    pub width: u32,
    /// is the height.
    pub height: u32,
    /// is the orignal x position (valid if trimmed).
    pub frame_x: i32,
    /// is the orignal y position (valid if trimmed).
    pub frame_y: i32,
    /// is the orignal width (valid if trimmed).
    pub frame_w: u32,
    /// is the orignal height (valid if trimmed).
    pub frame_h: u32,
    /// is the hash value (width, height and buffer combined).
    pub hash_value: u64,
    /// is the raw buffer in RGBA format.
    pub buffer: RgbaImage,
}

impl Texture {
    /// Instantiates a new `Texture` instance.
    pub fn new() -> Self {
        Self {
            file_path: PathBuf::new(),
            file_name: String::new(),
            width: 0,
            height: 0,
            frame_x: 0,
            frame_y: 0,
            frame_w: 0,
            frame_h: 0,
            hash_value: 0,
            buffer: RgbaImage::new(1, 1),
        }
    }

    /// Instantiates a new `Texture` instance based on given input params.
    ///
    /// # Arguments
    ///
    /// * `width` - is the `Texture` width.
    /// * `height` - is the `Texture` height.
    ///
    /// # Errors
    ///
    /// [`InvalidArg`](crate::texpack::texture::TextureError) error is returned if:
    /// `width == 0 || width > 8192 ||
    /// height == 0 || height > 8192`.
    pub fn with_details(width: u32, height: u32) -> Result<Self, TextureError> {
        if width > 0 && width <= MAX_SIZE && height > 0 && height <= MAX_SIZE {
            Ok(Self {
                file_path: PathBuf::new(),
                file_name: String::new(),
                width,
                height,
                frame_x: 0,
                frame_y: 0,
                frame_w: 0,
                frame_h: 0,
                hash_value: 0,
                buffer: RgbaImage::new(width, height),
            })
        } else {
            Err(TextureError::InvalidArg)
        }
    }

    /// Load texture.
    ///
    /// # Arguments
    ///
    /// * `file_path` - is the texture file path.
    /// * `premultiply` - is a flag determining whether to premultiply RBG by alpha channel or not.
    /// * `trim` - is a flag determining whether to trim excess transparent pixels or not.
    /// * `adjust_fit` - is a flag determining whether to adjust fit automatically or not.
    /// * `padding` - is the padding to use between textures.
    /// * `atlas_size` - is the atlas size.
    ///
    /// # Panics
    ///
    /// If loading fails.
    pub fn load(
        &mut self,
        file_path: &PathBuf,
        premultiply: bool,
        trim: bool,
        adjust_fit: bool,
        padding: u32,
        atlas_size: u32,
    ) {
        // remember file path and file name:
        self.file_path = file_path.clone();
        if let Some(file_name) = file_path.file_name() {
            if let Some(file_name_str) = file_name.to_str() {
                self.file_name = String::from(file_name_str);
            }
        }

        // load the image:
        let image = ImageReader::open(file_path).unwrap().decode().unwrap();
        let (width, height) = image.dimensions();
        self.update_initial_size(width, height);

        // trim excess transparent pixels off the texture:
        if trim {
            self.buffer = self.trim(&image.to_rgba8());
        } else {
            self.buffer = image.to_rgba8();
        }

        // premultiply all the pixels by their alpha value:
        if premultiply {
            self.premultiply();
        }

        // check if needing to adjust / scale texture size to fit atlas size:
        if adjust_fit
            && (((self.width + padding) > atlas_size) || ((self.height + padding) > atlas_size))
        {
            self.buffer = self.resize_to_fit(padding, atlas_size);
        }

        let mut hasher = DefaultHasher::new();
        self.width.hash(&mut hasher);
        self.height.hash(&mut hasher);
        self.buffer.hash(&mut hasher);
        self.hash_value = hasher.finish();

        info!(
            "Loaded texture: '{}' w: {} h: {}, hash_value: {}",
            self.file_name, self.width, self.height, self.hash_value
        );
    }

    /// Saves texture to disk.
    ///
    /// # Arguments
    ///
    /// * `file_path` - is the output file path.
    /// * `image_type` - is the output image type.
    ///
    /// # Panics
    ///
    /// If save fails.
    pub fn save(&self, file_path: &PathBuf, image_type: AtlasImage) {
        let dst_image = DynamicImage::ImageRgba8(self.buffer.clone());

        // make sure extension supplied is valid to help out with 'guessing' of type:
        if let Some(extension) = file_path.extension() {
            let ext_lc = extension.to_ascii_lowercase();
            if image_type == AtlasImage::Png && ext_lc == "png"
                || image_type == AtlasImage::Tga && ext_lc == "tga"
                || image_type == AtlasImage::Tiff && ext_lc == "tiff"
                || image_type == AtlasImage::Webp && ext_lc == "webp"
            {
                dst_image.save(file_path).unwrap();
            } else {
                panic!(
                    "Supplied file_path: {} does not have a valid extension that matches image type: {:?}!",
                    file_path.display(),
                    image_type
                );
            }
        }
    }

    /// Copy pixels from given `src` texture into this texture.
    ///
    /// # Arguments
    ///
    /// * `src` - is the source texture to copy from.
    /// * `tx` - is the x offset to use when copying.
    /// * `ty` - is the y offset to use when copying.
    ///
    /// # Panics
    ///
    /// If pixel is out of bounds.
    pub fn copy_pixels(&mut self, src: &Texture, tx: u32, ty: u32) {
        let (src_width, src_height) = src.buffer.dimensions();

        for y in 0..src_height {
            for x in 0..src_width {
                let pixel = src.buffer.get_pixel(x, y);
                self.buffer.put_pixel(x + tx, y + ty, *pixel);
            }
        }
    }

    /// Copy pixels from given `src` texture into this texture rotated 90 degrees clockwise.
    ///
    /// # Arguments
    ///
    /// * `src` - is the source texture to copy from.
    /// * `tx` - is the x offset to use when copying.
    /// * `ty` - is the y offset to use when copying.
    pub fn copy_pixels_rot_90cw(&mut self, src: &Texture, tx: u32, ty: u32) {
        let (src_width, src_height) = src.buffer.dimensions();
        let r = src_height - 1;

        for y in 0..src_height {
            for x in 0..src_width {
                let pixel = src.buffer.get_pixel(x, y);
                self.buffer.put_pixel(r - y + tx, x + ty, *pixel);
            }
        }
    }

    /// Get the texture area (width * height).
    pub fn get_area(&self) -> u32 {
        return self.width * self.height;
    }

    /// Updates initial size properties.
    fn update_initial_size(&mut self, width: u32, height: u32) {
        self.frame_x = 0;
        self.frame_y = 0;
        self.frame_w = width;
        self.frame_h = height;
        self.width = width;
        self.height = height;
    }

    /// Trims out excess pixels and returns new RBGA buffer.
    fn trim(&mut self, img: &RgbaImage) -> RgbaImage {
        let (width, height) = img.dimensions();

        if width == 0 || height == 0 {
            return ImageBuffer::new(1, 1);
        }

        // bounding box of non-transparent pixels:
        let mut min_x = width;
        let mut min_y = height;
        let mut max_x = 0u32;
        let mut max_y = 0u32;

        for y in 0..height {
            for x in 0..width {
                let pixel = img.get_pixel(x, y);
                if pixel[3] > 0 {
                    // non-transparent pixel:
                    min_x = min_x.min(x);
                    min_y = min_y.min(y);
                    max_x = max_x.max(x);
                    max_y = max_y.max(y);
                }
            }
        }

        // no non-transparent pixels found, return 1x1 transparent buffer:
        if max_x < min_x || max_y < min_y {
            return ImageBuffer::from_pixel(1, 1, Rgba([0, 0, 0, 0]));
        }

        // calc new dimensions (add 1 for bounds):
        let new_width = (max_x - min_x) + 1;
        let new_height = (max_y - min_y) + 1;

        // no trimming needed -> just clone it:
        if new_width == width && new_height == height {
            return img.clone();
        }

        // crop the image:
        let mut trimmed = ImageBuffer::new(new_width, new_height);
        for y in 0..new_height {
            for x in 0..new_width {
                let src_pixel = img.get_pixel(min_x + x, min_y + y);
                trimmed.put_pixel(x, y, *src_pixel);
            }
        }

        let x: i32 = min_x.try_into().unwrap();
        let y: i32 = min_y.try_into().unwrap();
        self.frame_x = -x;
        self.frame_y = -y;
        self.frame_w = width;
        self.frame_h = height;
        self.width = new_width;
        self.height = new_height;

        return trimmed;
    }

    /// Premultiply destination pixel by alpha.
    fn premultiply(&mut self) {
        let (width, height) = self.buffer.dimensions();

        for y in 0..height {
            for x in 0..width {
                // get source pixel:
                let src_pixel = self.buffer.get_pixel(x, y);

                // premultiply destination pixel by alpha:
                let alpha = src_pixel[3] as f32 / 255.0;
                let dst_pixel = Rgba([
                    (src_pixel[0] as f32 * alpha) as u8,
                    (src_pixel[1] as f32 * alpha) as u8,
                    (src_pixel[2] as f32 * alpha) as u8,
                    src_pixel[3],
                ]);

                // set new pixel value:
                self.buffer.put_pixel(x, y, dst_pixel);
            }
        }
    }

    /// Resize buffer so that it fits given `atlas_size + padding`.
    fn resize_to_fit(&mut self, padding: u32, atlas_size: u32) -> RgbaImage {
        let (src_width, src_height) = self.buffer.dimensions();

        if src_width == 0 || src_height == 0 || atlas_size == 0 {
            panic!("Invalid internal buffer state or atlas_size is 0");
        }

        // calculate scale factor:
        let mut _scale_factor = 0.0;
        if src_width > src_height {
            _scale_factor = src_width as f32 / atlas_size as f32;
        } else if src_height > src_width {
            _scale_factor = src_height as f32 / atlas_size as f32;
        } else {
            _scale_factor = src_width as f32 / atlas_size as f32;
        }
        info!("scale_factor is: {}", _scale_factor);

        // calculate new size of texture:
        let mut new_width = (src_width as f32 / _scale_factor).floor() as u32;
        let mut new_height = (src_height as f32 / _scale_factor).floor() as u32;

        // adjust for padding too:
        new_width -= padding;
        new_height -= padding;

        // make sure width and height is at least 1 pixel after scaling and padding:
        if new_width <= 0 {
            new_width = 1;
        }
        if new_height <= 0 {
            new_height = 1;
        }

        let src_image = DynamicImage::ImageRgba8(self.buffer.clone());
        let dst_image = src_image.resize(new_width, new_height, FilterType::Lanczos3);
        info!(
            "Resized image from {}x{} to {}x{}",
            src_width, src_height, new_width, new_height
        );

        // reset:
        self.frame_x = 0;
        self.frame_y = 0;
        self.frame_w = new_width;
        self.frame_h = new_height;
        self.width = new_width;
        self.height = new_height;

        return dst_image.to_rgba8();
    }
}

// unit tests:
#[cfg(test)]
mod tests {
    use super::*;
    use crate::texpack::app::{exists_file, get_atlas_image_extension, remove_file};
    use std::ffi::OsString;

    #[test]
    fn texture_error() {
        assert_eq!(
            Texture::with_details(0, 0).unwrap_err(),
            TextureError::InvalidArg
        );

        assert_eq!(
            Texture::with_details(32, 0).unwrap_err(),
            TextureError::InvalidArg
        );

        assert_eq!(
            Texture::with_details(0, 32).unwrap_err(),
            TextureError::InvalidArg
        );

        assert_eq!(
            Texture::with_details(8192, 8193).unwrap_err(),
            TextureError::InvalidArg
        );

        assert_eq!(
            Texture::with_details(8193, 8192).unwrap_err(),
            TextureError::InvalidArg
        );
    }

    #[test]
    fn texture_basics() {
        let t1 = Texture::new();
        assert_eq!(t1.width, 0);
        assert_eq!(t1.height, 0);
        assert_eq!(t1.frame_x, 0);
        assert_eq!(t1.frame_y, 0);
        assert_eq!(t1.frame_w, 0);
        assert_eq!(t1.frame_h, 0);
        assert_eq!(t1.hash_value, 0);

        let t2 = Texture::with_details(32, 32).unwrap();
        assert_eq!(t2.width, 32);
        assert_eq!(t2.height, 32);
    }

    #[test]
    fn texture_load_all_supported_formats() {
        let supported_extensions = vec![
            OsString::from("bmp"),
            OsString::from("hdr"),
            OsString::from("jpg"),
            OsString::from("jpeg"),
            OsString::from("png"),
            OsString::from("tga"),
            OsString::from("tiff"),
            OsString::from("webp"),
        ];

        for ext in &supported_extensions {
            let base_file_path = "test_data/white_32x32";
            let file_path = PathBuf::from(format!("{}.{}", base_file_path, ext.display()));

            let mut t = Texture::new();
            t.load(&file_path, false, false, false, 0, 64);
            assert_eq!(t.width, 32);
            assert_eq!(t.height, 32);
            assert_eq!(t.file_path, file_path);
            if let Some(file_name) = file_path.file_name() {
                if let Some(file_name_str) = file_name.to_str() {
                    assert_eq!(t.file_name, file_name_str);
                }
            }
            assert_eq!(t.frame_x, 0);
            assert_eq!(t.frame_y, 0);
            assert_eq!(t.frame_w, 32);
            assert_eq!(t.frame_h, 32);
            assert_eq!(t.hash_value > 0, true);
        }
    }

    #[test]
    fn texture_save_all_supported_formats() {
        let atlas_image_types = vec![
            AtlasImage::Png,
            AtlasImage::Tga,
            AtlasImage::Tiff,
            AtlasImage::Webp,
        ];

        let mut t1 = Texture::new();
        t1.load(
            &PathBuf::from("test_data/white_32x32.png"),
            false,
            false,
            false,
            0,
            64,
        );

        let mut output = Texture::with_details(64, 64).unwrap();
        output.copy_pixels(&t1, 16, 16);

        let base_file_path = "test_data/save_64x64";
        for atlas_image_type in &atlas_image_types {
            let file_path = PathBuf::from(format!(
                "{}.{}",
                base_file_path,
                get_atlas_image_extension(atlas_image_type.clone())
            ));
            output.save(&file_path, atlas_image_type.clone());

            assert_eq!(exists_file(&file_path), true);
            remove_file(&file_path);
            assert_eq!(exists_file(&file_path), false);
        }
    }

    #[test]
    fn texture_copy_pixels() {
        let mut t1 = Texture::new();
        t1.load(
            &PathBuf::from("test_data/white_32x32.png"),
            false,
            false,
            false,
            0,
            64,
        );

        let mut t2 = Texture::new();
        t2.load(
            &PathBuf::from("test_data/red_32x32.png"),
            false,
            false,
            false,
            0,
            64,
        );

        let mut t3 = Texture::new();
        t3.load(
            &PathBuf::from("test_data/green_32x32.png"),
            false,
            false,
            false,
            0,
            64,
        );

        let mut t4 = Texture::new();
        t4.load(
            &PathBuf::from("test_data/blue_32x32.png"),
            false,
            false,
            false,
            0,
            64,
        );

        let mut output = Texture::with_details(64, 64).unwrap();
        let file_path = PathBuf::from("test_data/copy_pixels_64x64.png");

        output.copy_pixels(&t1, 0, 0);
        output.copy_pixels(&t2, 32, 0);
        output.copy_pixels(&t3, 32, 32);
        output.copy_pixels(&t4, 0, 32);

        output.save(&file_path, AtlasImage::Png);
        assert_eq!(exists_file(&file_path), true);
        remove_file(&file_path);
        assert_eq!(exists_file(&file_path), false);
    }

    #[test]
    fn texture_copy_pixels_rot_90cw() {
        let mut t1 = Texture::new();
        t1.load(
            &PathBuf::from("test_data/white_32x16.png"),
            false,
            false,
            false,
            0,
            64,
        );

        let mut output = Texture::with_details(64, 64).unwrap();
        let file_path = PathBuf::from("test_data/copy_pixels_rot_90cw_64x64.png");

        output.copy_pixels_rot_90cw(&t1, 0, 0);
        output.copy_pixels_rot_90cw(&t1, 32, 0);

        output.save(&file_path, AtlasImage::Png);
        assert_eq!(exists_file(&file_path), true);
        remove_file(&file_path);
        assert_eq!(exists_file(&file_path), false);
    }

    #[test]
    fn texture_get_area() {
        let t1 = Texture::with_details(32, 32).unwrap();
        assert_eq!(t1.get_area(), 1024);
    }
}
