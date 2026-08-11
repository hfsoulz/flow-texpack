// flow-texpack: A program that will allow you to generate texture atlas.
// zlib License (see LICENSE)

use log::info;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use flow_rectpack::FreeRectHeuristic;
use flow_rectpack::RectsBinPack;

use crate::Texture;
use crate::texpack::app::AtlasImage;

/// is the minimum allowed size.
pub const MIN_SIZE: u32 = 64;
/// is the maximum allowed size.
pub const MAX_SIZE: u32 = 8192;

/// Specifies the different error types that can occur.
#[derive(PartialEq, Clone, Debug)]
pub enum PackerError {
    /// Invalid argument
    InvalidArg,
}

/// Specifies the properties of a `Point`.
#[derive(Clone, Debug)]
pub struct Point {
    /// is the x offset.
    pub x: i32,
    /// is the y offset.
    pub y: i32,
    /// is the duplicate ID.
    pub duplicate_id: usize,
    /// is the flag determining whether rotated or not.
    pub rotate: bool,
}

impl Point {
    /// Instantiates a new `Point` instance.
    pub fn new() -> Self {
        Self {
            x: 0,
            y: 0,
            duplicate_id: 0,
            rotate: false,
        }
    }
}

/// Specifies the properties of a `PackerState`.
#[derive(Clone, Debug)]
pub struct PackerState {
    /// is the width.
    pub width: u32,
    /// is the height.
    pub height: u32,
    /// is the padding.
    pub padding: i32,
    /// is the flag determining whether mip maps should be generated (rendering hint).
    pub generate_mipmaps: bool,
    /// is the vector holding the textures to pack.
    pub textures: Vec<Texture>,
    /// is the vector holding the points used for `unique` lookups.
    pub points: Vec<Point>,
    /// is the hash map holding texture hash and duplicate_id.
    pub duplicates: HashMap<u64, usize>,
}

impl PackerState {
    /// Instantiates a new `PackerState` instance.
    ///
    /// # Arguments
    ///
    /// * `width` - is the width.
    /// * `height` - is the height.
    /// * `padding` - is the padding.
    /// * `generate_mipmaps` - is the flag determining whether to generate mip maps (rendering hint)
    /// or not.
    pub fn new(width: u32, height: u32, padding: i32, generate_mipmaps: bool) -> Self {
        Self {
            width,
            height,
            padding,
            generate_mipmaps,
            textures: Vec::new(),
            points: Vec::new(),
            duplicates: HashMap::new(),
        }
    }
}

/// Specifies the properties of a `Packer`.
#[derive(Debug)]
pub struct Packer {
    /// is the `PackerState` protected by a `Mutex`.
    pub state: Mutex<PackerState>,
}

impl Packer {
    /// Instantiates a new `Packer` instance.
    ///
    /// # Arguments
    ///
    /// * `width` - is the width.
    /// * `height` - is the height.
    /// * `padding` - is the padding.
    /// * `generate_mipmaps` - is the flag determining whether to generate mip maps (rendering hint)
    /// or not.
    ///
    /// # Errors
    ///
    /// [`InvalidArg`](crate::texpack::packer::PackerError) error is returned if:
    /// `width != SIZE_IN_POWER_OF_TWO || height != SIZE_IN_POWER_OF_TWO ||
    /// width < 64 || width > 8192 ||
    /// height < 64 || height > 8192`.
    pub fn new(
        width: u32,
        height: u32,
        padding: i32,
        generate_mipmaps: bool,
    ) -> Result<Self, PackerError> {
        // make sure width/height is power-of-two and 64 - 8192 in size:
        if width >= MIN_SIZE
            && width <= MAX_SIZE
            && height >= MIN_SIZE
            && height <= MAX_SIZE
            && (width & (width - 1)) == 0
            && (height & (height - 1)) == 0
        {
            Ok(Self {
                state: Mutex::new(PackerState::new(width, height, padding, generate_mipmaps)),
            })
        } else {
            Err(PackerError::InvalidArg)
        }
    }

    /// Returns a shared `Arc` `Packer` instance.
    pub fn shared(self) -> Arc<Self> {
        Arc::new(self)
    }

    /// Packs all textures that will fit from given `textures` and pops each packed from vector.
    ///
    /// # Arguments
    ///
    /// * `textures` - is the vector holding the textures to pack.
    /// * `unique` - is a flag determining whether to include only unique textures or not (a
    /// unique texture is determined by its combined hash value of `width`, `height` and `buffer`).
    /// * `rotate` - is a flag determining whether rotation of textures is allowed or not.
    /// * `square` - is a flag determining whether packers size must be POWER OF TWO in size for both width
    /// and height.
    /// * `adjust_fit` - is a flag determining whether to adjust fit automatically or not.
    /// * `heuristic` - is the heuristic method to use for determining where to place the texture.
    ///
    /// # Panics
    ///
    /// If packing fails.
    pub fn pack(
        &mut self,
        textures: &mut Vec<Texture>,
        unique: bool,
        rotate: bool,
        square: bool,
        adjust_size: bool,
        heuristic: FreeRectHeuristic,
    ) {
        assert!(textures.len() > 0);

        let mut exists_larger = false;
        if !square {
            exists_larger = self.exists_larger_texture(textures);
        }

        // make sure each texture fit within size:
        if adjust_size {
            self.adjust_size_to_fit(textures);
        }

        let mut lock = self.state.lock().unwrap();
        let mut rbp = RectsBinPack::new(
            lock.width.try_into().unwrap(),
            lock.height.try_into().unwrap(),
            rotate,
        )
        .unwrap();

        let mut ww: u32 = 0;
        let mut hh: u32 = 0;
        while !textures.is_empty() {
            if let Some(texture) = textures.last() {
                if unique {
                    if let Some(value) = lock.duplicates.get(&texture.hash_value) {
                        if let Some(point) = lock.points.get(*value) {
                            info!(
                                "Texture '{}' with hash: {} is not unique (not packed but will be added in descriptor)",
                                texture.file_name, texture.hash_value
                            );
                            let mut p = point.clone();
                            p.duplicate_id = *value;
                            lock.points.push(p);
                            lock.textures.push(texture.clone());
                            textures.pop();
                            continue;
                        }
                    }
                }

                {
                    let tw: i32 = texture.width.try_into().unwrap();
                    let th: i32 = texture.height.try_into().unwrap();
                    let width: i32 = tw + lock.padding;
                    let height: i32 = th + lock.padding;
                    if let Some(rect) = rbp.insert(width, height, heuristic.clone()) {
                        if unique {
                            let num_points = lock.points.len();
                            lock.duplicates.insert(texture.hash_value, num_points);
                        }

                        // check if we rotated:
                        let mut p = Point::new();
                        p.x = rect.x;
                        p.y = rect.y;
                        p.duplicate_id = std::usize::MAX;
                        p.rotate = rotate && tw != rect.width - lock.padding;

                        info!(
                            "Packed '{}' w: {} h: {} rotated: {} hash: {}",
                            texture.file_name,
                            texture.width,
                            texture.height,
                            p.rotate,
                            texture.hash_value
                        );
                        lock.points.push(p);
                        lock.textures.push(texture.clone());
                        textures.pop();

                        ww = std::cmp::max((rect.x + rect.width).try_into().unwrap(), ww);
                        hh = std::cmp::max((rect.y + rect.height).try_into().unwrap(), hh);
                    } else {
                        break;
                    }
                }
            } else {
                panic!("texture.last() failed!");
            }
        }

        // tweak power-of-two size so that it's optimized for largest found width/height:
        if !square && !exists_larger {
            while lock.width / 2 >= ww {
                lock.width /= 2;
            }

            while lock.height / 2 >= hh {
                lock.height /= 2;
            }
        }
    }

    /// Saves the packed textures to disk.
    ///
    /// # Arguments
    ///
    /// * `file_path` - is the output file path.
    /// * `image_type` - is the output image type.
    ///
    /// # Panics
    ///
    /// If save fails.
    pub fn save_image(&self, file_path: &PathBuf, image_type: AtlasImage) {
        let lock = self.state.lock().unwrap();
        let mut texture = Texture::with_details(lock.width, lock.height).unwrap();

        for i in 0..lock.textures.len() {
            if let Some(src) = lock.textures.get(i)
                && let Some(point) = lock.points.get(i)
            {
                if point.duplicate_id == std::usize::MAX {
                    if point.rotate {
                        texture.copy_pixels_rot_90cw(
                            src,
                            point.x.try_into().unwrap(),
                            point.y.try_into().unwrap(),
                        );
                    } else {
                        texture.copy_pixels(
                            src,
                            point.x.try_into().unwrap(),
                            point.y.try_into().unwrap(),
                        );
                    }
                }
            }
        }

        texture.save(file_path, image_type);
    }

    /// Saves the atlas descriptor to disk in JSON format.
    ///
    /// # Arguments
    ///
    /// * `file` - is a reference to already opened for write `File` to use when writing.
    /// * `file_name` - is the name of the atlas image.
    /// * `image_ext` - is the extension of the atlas image ("png", "tga" etc).
    pub fn save_json(&self, file: &mut File, file_name: &str, image_ext: &str) {
        let lock = self.state.lock().unwrap();
        file.write(String::from("\t\t\t{\n").as_bytes()).unwrap();
        file.write(format!("\t\t\t\t\"n\": \"{}.{}\",\n", file_name, image_ext).as_bytes())
            .unwrap();
        file.write(format!("\t\t\t\t\"numImages\": {},\n", lock.textures.len()).as_bytes())
            .unwrap();
        file.write(format!("\t\t\t\t\"width\": {},\n", lock.width).as_bytes())
            .unwrap();
        file.write(format!("\t\t\t\t\"height\": {},\n", lock.height).as_bytes())
            .unwrap();
        file.write(
            format!(
                "\t\t\t\t\"generateMipMaps\": {},\n",
                lock.generate_mipmaps as u8
            )
            .as_bytes(),
        )
        .unwrap();
        file.write(String::from("\t\t\t\t\"img\":\n").as_bytes())
            .unwrap();
        file.write(String::from("\t\t\t\t[\n").as_bytes()).unwrap();

        for i in 0..lock.textures.len() {
            if let Some(texture) = lock.textures.get(i)
                && let Some(point) = lock.points.get(i)
            {
                let mut trimmed = false;
                if texture.frame_w != texture.width || texture.frame_h != texture.height {
                    trimmed = true;
                }

                if i > 0 {
                    file.write(String::from(",\n").as_bytes()).unwrap();
                }

                file.write(String::from("\t\t\t\t\t{\n").as_bytes())
                    .unwrap();
                file.write(format!("\t\t\t\t\t\t\"n\": \"{}\", ", texture.file_name).as_bytes())
                    .unwrap();
                file.write(format!("\"x\": {}, ", point.x).as_bytes())
                    .unwrap();
                file.write(format!("\"y\": {}, ", point.y).as_bytes())
                    .unwrap();
                file.write(format!("\"w\": {}, ", texture.width).as_bytes())
                    .unwrap();
                file.write(format!("\"h\": {}, ", texture.height).as_bytes())
                    .unwrap();
                file.write(format!("\"trimmed\": {}, ", trimmed as u8).as_bytes())
                    .unwrap();
                file.write(format!("\"rotated\": {}, ", point.rotate as u8).as_bytes())
                    .unwrap();
                file.write(format!("\"fx\": {}, ", texture.frame_x).as_bytes())
                    .unwrap();
                file.write(format!("\"fy\": {}, ", texture.frame_y).as_bytes())
                    .unwrap();
                file.write(format!("\"fw\": {}, ", texture.frame_w).as_bytes())
                    .unwrap();
                file.write(format!("\"fh\": {}\n", texture.frame_h).as_bytes())
                    .unwrap();
                file.write(String::from("\t\t\t\t\t}").as_bytes()).unwrap();
            }
        }
        file.write(String::from("\n\t\t\t\t]\n").as_bytes())
            .unwrap();
        file.write(String::from("\t\t\t}").as_bytes()).unwrap();
    }

    /// Saves the atlas descriptor to disk in plain TXT format.
    ///
    /// # Arguments
    ///
    /// * `file` - is a reference to already opened for write `File` to use when writing.
    /// * `file_name` - is the name of the atlas image.
    /// * `image_ext` - is the extension of the atlas image ("png", "tga" etc).
    pub fn save_txt(&self, file: &mut File, file_name: &str, image_ext: &str) {
        let lock = self.state.lock().unwrap();
        file.write(format!("{}.{}", file_name, image_ext).as_bytes())
            .unwrap();
        file.write(format!(",{}", lock.textures.len()).as_bytes())
            .unwrap();
        file.write(format!(",{}", lock.width).as_bytes()).unwrap();
        file.write(format!(",{}", lock.height).as_bytes()).unwrap();
        file.write(format!(",{}\n", lock.generate_mipmaps as u8).as_bytes())
            .unwrap();

        for i in 0..lock.textures.len() {
            if let Some(texture) = lock.textures.get(i)
                && let Some(point) = lock.points.get(i)
            {
                let mut trimmed = false;
                if texture.frame_w != texture.width || texture.frame_h != texture.height {
                    trimmed = true;
                }

                file.write(format!("{}", texture.file_name).as_bytes())
                    .unwrap();
                file.write(format!(",{}", point.x).as_bytes()).unwrap();
                file.write(format!(",{}", point.y).as_bytes()).unwrap();
                file.write(format!(",{}", texture.width).as_bytes())
                    .unwrap();
                file.write(format!(",{}", texture.height).as_bytes())
                    .unwrap();
                file.write(format!(",{}", trimmed as u8).as_bytes())
                    .unwrap();
                file.write(format!(",{}", point.rotate as u8).as_bytes())
                    .unwrap();
                file.write(format!(",{}", texture.frame_x).as_bytes())
                    .unwrap();
                file.write(format!(",{}", texture.frame_y).as_bytes())
                    .unwrap();
                file.write(format!(",{}", texture.frame_w).as_bytes())
                    .unwrap();
                file.write(format!(",{}\n", texture.frame_h).as_bytes())
                    .unwrap();
            }
        }
    }

    /// Adjusts the packer width and height so that given `textures` will fit.
    ///
    /// # Arguments
    ///
    /// * `textures` - is the vector holding the textures.
    ///
    /// # Panics
    ///
    /// If new adjusted width / height is > 8192.
    fn adjust_size_to_fit(&mut self, textures: &Vec<Texture>) -> bool {
        let mut lock = self.state.lock().unwrap();
        let mut adjusted_size = false;
        let padding: u32 = lock.padding.try_into().unwrap();

        for i in 0..textures.len() {
            if let Some(texture) = textures.get(i) {
                if texture.width + padding > lock.width {
                    lock.width *= 2;
                    lock.height = lock.width;
                    adjusted_size = true;
                }

                if texture.height + padding > lock.height {
                    lock.height *= 2;
                    lock.width = lock.height;
                    adjusted_size = true;
                }

                if lock.width > MAX_SIZE || lock.height > MAX_SIZE {
                    panic!(
                        "adjust_size_to_fit failed. Maximum allowed width / height is {}",
                        MAX_SIZE
                    );
                }

                // make sure width is at least minimum size:
                if lock.width < MIN_SIZE {
                    lock.width = MIN_SIZE;
                }

                // make sure height is at least minimum size:
                if lock.height < MIN_SIZE {
                    lock.height = MIN_SIZE;
                }
            }
        }

        if adjusted_size {
            info!(
                "Packer: Adjusted size to {}x{} to fit textures.",
                lock.width, lock.height
            );
        }

        return adjusted_size;
    }

    /// Determines whether there exists a texture in given `textures` which size is larger then
    /// that of packer width and height.
    ///
    /// # Arguments
    ///
    /// * `textures` - is the vector holding the textures.
    fn exists_larger_texture(&self, textures: &Vec<Texture>) -> bool {
        let lock = self.state.lock().unwrap();
        let padding: u32 = lock.padding.try_into().unwrap();

        for i in 0..textures.len() {
            if let Some(texture) = textures.get(i) {
                if texture.width + padding > lock.width || texture.height + padding > lock.height {
                    return true;
                }
            }
        }

        return false;
    }
}

// unit tests:
#[cfg(test)]
mod tests {
    use super::*;
    use crate::texpack::app::{exists_file, get_atlas_image_extension, remove_file};

    #[test]
    fn point_basics() {
        let p = Point::new();

        assert_eq!(p.x, 0);
        assert_eq!(p.y, 0);
        assert_eq!(p.duplicate_id, 0);
        assert_eq!(p.rotate, false);
    }

    #[test]
    fn packer_error() {
        assert_eq!(
            Packer::new(0, 0, 1, false).unwrap_err(),
            PackerError::InvalidArg
        );

        assert_eq!(
            Packer::new(32, 32, 1, false).unwrap_err(),
            PackerError::InvalidArg
        );

        assert_eq!(
            Packer::new(8192, 8193, 1, false).unwrap_err(),
            PackerError::InvalidArg
        );

        assert_eq!(
            Packer::new(32, 64, 1, false).unwrap_err(),
            PackerError::InvalidArg
        );

        assert_eq!(
            Packer::new(64, 32, 1, false).unwrap_err(),
            PackerError::InvalidArg
        );
    }

    fn load_textures() -> Vec<Texture> {
        let mut textures: Vec<Texture> = Vec::new();
        let mut t1 = Texture::new();
        t1.load(
            &PathBuf::from("test_data/white_32x32.png"),
            false,
            false,
            false,
            0,
            64,
        );
        textures.push(t1);

        let mut t2 = Texture::new();
        t2.load(
            &PathBuf::from("test_data/red_32x32.png"),
            false,
            false,
            false,
            0,
            64,
        );
        textures.push(t2);

        let mut t3 = Texture::new();
        t3.load(
            &PathBuf::from("test_data/green_32x32.png"),
            false,
            false,
            false,
            0,
            64,
        );
        textures.push(t3);

        let mut t4 = Texture::new();
        t4.load(
            &PathBuf::from("test_data/blue_32x32.png"),
            false,
            false,
            false,
            0,
            64,
        );
        textures.push(t4);

        return textures;
    }

    #[test]
    fn packer_basics_short_side_fit() {
        let mut textures = load_textures();
        assert_eq!(textures.len(), 4);

        let mut packer = Packer::new(64, 64, 0, true).unwrap();
        packer.pack(
            &mut textures,
            true,
            false,
            false,
            false,
            FreeRectHeuristic::ShortSideFit,
        );
        assert_eq!(textures.len() == 0, true);

        let file_path = PathBuf::from("test_data/atlas_short_side_fit.png");

        packer.save_image(&file_path, AtlasImage::Png);
        assert_eq!(exists_file(&file_path), true);
        remove_file(&file_path);
        assert_eq!(exists_file(&file_path), false);
    }

    #[test]
    fn packer_basics_long_side_fit() {
        let mut textures = load_textures();
        assert_eq!(textures.len(), 4);

        let mut packer = Packer::new(64, 64, 0, true).unwrap();
        packer.pack(
            &mut textures,
            true,
            false,
            false,
            false,
            FreeRectHeuristic::LongSideFit,
        );
        assert_eq!(textures.len() == 0, true);

        let file_path = PathBuf::from("test_data/atlas_long_side_fit.png");

        packer.save_image(&file_path, AtlasImage::Png);
        assert_eq!(exists_file(&file_path), true);
        remove_file(&file_path);
        assert_eq!(exists_file(&file_path), false);
    }

    #[test]
    fn packer_basics_area_fit() {
        let mut textures = load_textures();
        assert_eq!(textures.len(), 4);

        let mut packer = Packer::new(64, 64, 0, true).unwrap();
        packer.pack(
            &mut textures,
            true,
            false,
            false,
            false,
            FreeRectHeuristic::AreaFit,
        );
        assert_eq!(textures.len() == 0, true);

        let file_path = PathBuf::from("test_data/atlas_area_fit.png");

        packer.save_image(&file_path, AtlasImage::Png);
        assert_eq!(exists_file(&file_path), true);
        remove_file(&file_path);
        assert_eq!(exists_file(&file_path), false);
    }

    #[test]
    fn packer_basics_bottom_left() {
        let mut textures = load_textures();
        assert_eq!(textures.len(), 4);

        let mut packer = Packer::new(64, 64, 0, true).unwrap();
        packer.pack(
            &mut textures,
            true,
            false,
            false,
            false,
            FreeRectHeuristic::BottomLeft,
        );
        assert_eq!(textures.len() == 0, true);

        let file_path = PathBuf::from("test_data/atlas_bottom_left.png");

        packer.save_image(&file_path, AtlasImage::Png);
        assert_eq!(exists_file(&file_path), true);
        remove_file(&file_path);
        assert_eq!(exists_file(&file_path), false);
    }

    #[test]
    fn packer_basics_contact_point() {
        let mut textures = load_textures();
        assert_eq!(textures.len(), 4);

        let mut packer = Packer::new(64, 64, 0, true).unwrap();
        packer.pack(
            &mut textures,
            true,
            false,
            false,
            false,
            FreeRectHeuristic::ContactPoint,
        );
        assert_eq!(textures.len() == 0, true);

        let file_path = PathBuf::from("test_data/atlas_contact_point.png");

        packer.save_image(&file_path, AtlasImage::Png);
        assert_eq!(exists_file(&file_path), true);
        remove_file(&file_path);
        assert_eq!(exists_file(&file_path), false);
    }

    #[test]
    fn packer_adjust_size_to_fit() {
        let mut textures: Vec<Texture> = Vec::new();
        let mut t1 = Texture::new();
        t1.load(
            &PathBuf::from("test_data/white_128x128.png"),
            false,
            false,
            true,
            0,
            64,
        );
        textures.push(t1);

        let mut packer = Packer::new(64, 64, 0, true).unwrap();
        packer.pack(
            &mut textures,
            true,
            false,
            false,
            false,
            FreeRectHeuristic::ContactPoint,
        );
        assert_eq!(textures.len() == 0, true);

        let file_path = PathBuf::from("test_data/atlas_adjust_size_to_fit.png");

        packer.save_image(&file_path, AtlasImage::Png);
        assert_eq!(exists_file(&file_path), true);

        let mut output = Texture::new();
        output.load(
            &PathBuf::from("test_data/atlas_adjust_size_to_fit.png"),
            false,
            false,
            false,
            0,
            64,
        );
        assert_eq!(output.width, 64);
        assert_eq!(output.height, 64);

        remove_file(&file_path);
        assert_eq!(exists_file(&file_path), false);
    }

    #[test]
    fn packer_trim() {
        let mut textures: Vec<Texture> = Vec::new();
        let mut t1 = Texture::new();
        t1.load(
            &PathBuf::from("test_data/blue_trimmable_128x128.png"),
            false,
            true,
            false,
            0,
            128,
        );
        textures.push(t1);

        let mut packer = Packer::new(128, 128, 0, true).unwrap();
        packer.pack(
            &mut textures,
            true,
            false,
            false,
            false,
            FreeRectHeuristic::BottomLeft,
        );
        assert_eq!(textures.len() == 0, true);

        let file_path = PathBuf::from("test_data/atlas_trimmed.png");

        packer.save_image(&file_path, AtlasImage::Png);
        assert_eq!(exists_file(&file_path), true);

        let mut output = Texture::new();
        output.load(
            &PathBuf::from("test_data/atlas_trimmed.png"),
            false,
            false,
            false,
            0,
            64,
        );
        assert_eq!(output.width, 32);
        assert_eq!(output.height, 32);

        remove_file(&file_path);
        assert_eq!(exists_file(&file_path), false);
    }

    #[test]
    fn packer_rotated() {
        let mut textures: Vec<Texture> = Vec::new();
        let mut t1 = Texture::new();
        t1.load(
            &PathBuf::from("test_data/white_128x64.png"),
            false,
            false,
            false,
            0,
            64,
        );
        textures.push(t1);
        assert_eq!(textures.len(), 1);

        let mut packer = Packer::new(64, 128, 0, true).unwrap();
        packer.pack(
            &mut textures,
            true,
            true,
            false,
            false,
            FreeRectHeuristic::LongSideFit,
        );
        assert_eq!(textures.len() == 0, true);

        let file_path = PathBuf::from("test_data/atlas_rotated.png");

        packer.save_image(&file_path, AtlasImage::Png);
        assert_eq!(exists_file(&file_path), true);

        let mut output = Texture::new();
        output.load(
            &PathBuf::from("test_data/atlas_rotated.png"),
            false,
            false,
            false,
            0,
            64,
        );
        assert_eq!(output.width, 64);
        assert_eq!(output.height, 128);

        remove_file(&file_path);
        assert_eq!(exists_file(&file_path), false);
    }

    #[test]
    fn packer_save_all_supported_types() {
        let mut textures = load_textures();
        assert_eq!(textures.len(), 4);

        let mut packer = Packer::new(64, 64, 0, true).unwrap();
        packer.pack(
            &mut textures,
            true,
            false,
            false,
            false,
            FreeRectHeuristic::ShortSideFit,
        );
        assert_eq!(textures.len() == 0, true);

        let base_file_path = PathBuf::from("test_data/atlas_save");
        let image_types = vec![
            AtlasImage::Png,
            AtlasImage::Tga,
            AtlasImage::Tiff,
            AtlasImage::Webp,
        ];

        for image_type in image_types {
            let file_path = PathBuf::from(format!(
                "{}.{}",
                base_file_path.display(),
                get_atlas_image_extension(image_type.clone())
            ));

            println!("{}", file_path.display());

            packer.save_image(&file_path, image_type.clone());
            assert_eq!(exists_file(&file_path), true);
            remove_file(&file_path);
            assert_eq!(exists_file(&file_path), false);
        }
    }
}
