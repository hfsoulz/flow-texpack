// flow-texpack: A program that will allow you to generate texture atlas.
// zlib License (see LICENSE)

use crate::Packer;
use crate::Texture;

use flow_rectpack::FreeRectHeuristic;
use rich_rust::console::Console;
use rich_rust::interactive::Status;

use clap::{Parser, ValueEnum};

use tokio::io;
use tokio::task;

use std::time::Instant;

use log::info;

use std::collections::HashSet;
use std::ffi::OsString;
use std::fs;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// name of the application:
const NAME: &str = "https://github.com/hfsoulz/flow-texpack.git";

/// short about description shown for option '-h':
const ABOUT: &str = "
flow-texpack is a program that will allow you to generate texture atlas from input images (BMP, HDR,
JPG, PNG, TGA, TIFF, WEBP). The application generates both texture atlas and descriptions file that
can be read by a game.";

/// long about description shown for option '--help':
const LONG_ABOUT: &str = "
flow-texpack is a program that will allow you to generate texture atlas from input images (BMP, HDR,
JPG, PNG, TGA, TIFF, WEBP). The application generates both texture atlas and descriptions file that
can be read by a game.

Examples:
flow-texpack -i data/characters data/tiles -o out/atlas -m -t -u -r -p 2 -v
flow-texpack -i data/characters data/tiles -o out/atlas -m -t -u -r -p 2 -v --load-filter png tga
flow-texpack -i data -e data/tiles -o out/atlas -m -t -u -r --atlas-size pot2048 --rect-heuristic area-fit -v
flow-texpack --input-file input.txt --exlude-file exclude.txt -o out/atlas -v
flow-texpack -i data/characters data/tiles -o out/atlas -m -t -u -r --adjust-size -v
flow-texpack -i data/characters data/tiles -o out/atlas -m -t -u -r --adjust-fit -v";

/// Specifies the different atlas descriptor types.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum AtlasDescriptor {
    /// atlas descriptor type: JSON
    Json,
    /// atlas descriptor type: Txt
    Txt,
    /// atlas descriptor type: Txt (with description header)
    TxtDesc,
}

/// Specifies the different atlas image types.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum AtlasImage {
    /// atlas image type: PNG
    Png,
    /// atlas image type: TGA
    Tga,
    /// atlas image type: TIFF
    Tiff,
    /// atlas image type: Webp
    Webp,
}

/// Specifies the different load filter types.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum LoadFilter {
    /// load filter type: BMP
    Bmp,
    /// load filter type: HDR
    Hdr,
    /// load filter type: JPG
    Jpg,
    /// load filter type: PNG
    Png,
    /// load filter type: TGA
    Tga,
    /// load filter type: TIFF
    Tiff,
    /// load filter type: Webp
    Webp,
}

/// Specifies the different atlas output sizes.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum AtlasSize {
    /// power-of-two 64x64 size:
    Pot64 = 64,
    /// power-of-two 128x128 size:
    Pot128 = 128,
    /// power-of-two 256x256 size:
    Pot256 = 256,
    /// power-of-two 512x512 size:
    Pot512 = 512,
    /// power-of-two 1024x1024 size:
    Pot1024 = 1024,
    /// power-of-two 2048x2048 size:
    Pot2048 = 2048,
    /// power-of-two 4096x4096 size:
    Pot4096 = 4096,
    /// power-of-two 8192x8192 size:
    Pot8192 = 8192,
}

/// Specifies the different heuristic types.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum RectHeuristic {
    /// Choose to pack `R` into such `Fi` that `min(wf - w, hf - h)` is the smallest. In other words, we
    /// minimize the length of the shorter leftover side.
    ShortSideFit,

    /// Pack `R` into an `Fi` such that `max(wf - w, hf - h)` is the smallest. That is, we minimize
    /// the length of the longer leftover side.
    LongSideFit,

    /// Pick the `Fi ∈ F` that is smallest in area to place the next rectangle `R` into. If there is a
    /// tie, we use the `ShortSideFit` rule to break it.
    AreaFit,

    /// Orient and place each rectangle to the position where the y-coordinate of the top side of the
    /// rectangle is the smallest and if there are several such valid positions, pick the one that has
    /// the smallest x-coordinate value.
    BottomLeft,

    /// Place `R` into a position where the length of the perimeter of `R` that is touched by the bin
    /// edge or by a previously packed rectangle is maximized.
    ContactPoint,
}

#[derive(Parser, Debug)]
#[command(version, about = ABOUT, long_about = LONG_ABOUT)]
struct CliArgs {
    /// input files/directories separated by space (' ')
    #[arg(short = 'i', long = "input", value_delimiter = ' ', num_args = 1.., group = "input_group")]
    input: Option<Vec<PathBuf>>,

    /// input file containing files/directories
    /// (each entry needs to be on a new line)
    #[arg(long = "input-file", group = "input_group", verbatim_doc_comment)]
    input_file: Option<PathBuf>,

    /// exclude files/directories separated by space (' ')
    #[arg(short = 'e', long = "exclude", value_delimiter = ' ', num_args = 1.., group = "exclude_group")]
    exclude: Option<Vec<PathBuf>>,

    /// exclude file containing files/directories
    /// (each entry needs to be on a new line)
    #[arg(long = "exclude-file", group = "exclude_group", verbatim_doc_comment)]
    exclude_file: Option<PathBuf>,

    /// output file
    #[arg(short = 'o', long = "output", requires = "input_group")]
    output: PathBuf,

    /// atlas descriptor
    #[arg(long = "atlas-descriptor", value_enum, default_value_t = AtlasDescriptor::Json)]
    atlas_descriptor: AtlasDescriptor,

    /// atlas image
    #[arg(long = "atlas-image", value_enum, default_value_t = AtlasImage::Png)]
    atlas_image: AtlasImage,

    /// max atlas output size POT
    #[arg(long = "atlas-size", value_enum, default_value_t = AtlasSize::Pot1024)]
    atlas_size: AtlasSize,

    /// load filter
    #[arg(long = "load-filter", value_enum,  value_delimiter = ' ', num_args = 1..)]
    load_filter: Option<Vec<LoadFilter>>,

    /// max atlases that can be created (value in the range 1 - 4096)
    #[arg(long = "max-atlases", default_value_t = 64, value_parser = clap::value_parser!(u16).range(1..=4096))]
    max_atlases: u16,

    /// enable premultiply the pixels of the textures by their alpha channel
    #[arg(short = 'm', long = "premultiply")]
    premultiply: bool,

    /// enable trim excess transparency off the textures
    #[arg(short = 't', long = "trim")]
    trim: bool,

    /// enable force packer to re-pack (ignore stored hashes)
    #[arg(short = 'f', long = "force")]
    force: bool,

    /// enable remove duplicate textures from the atlas
    #[arg(short = 'u', long = "unique")]
    unique: bool,

    /// enable rotation of textures (90 degrees clockwise)
    #[arg(short = 'r', long = "rotate")]
    rotate: bool,

    /// enable force atlas POT square size
    #[arg(long = "force-square")]
    force_square: bool,

    /// enable adjust atlas size automatically so that texture will fit
    #[arg(long = "adjust-size")]
    adjust_size: bool,

    /// enable adjust texture size so that it will fit given atlas size
    #[arg(long = "adjust-fit")]
    adjust_fit: bool,

    /// enable generation of mipmaps (rendering hint)
    #[arg(long = "generate-mipmaps")]
    generate_mipmaps: bool,

    /// padding between textures (value in the range 0 - 16)
    #[arg(short = 'p', long = "pad", default_value_t = 1, value_parser = clap::value_parser!(u8).range(0..=16) )]
    pad: u8,

    /// heuristic rule to use when deciding where to place a new rectangle
    #[arg(long = "rect-heuristic", value_enum, default_value_t = RectHeuristic::LongSideFit)]
    rect_heuristic: RectHeuristic,

    /// enable verbose output of progress
    #[arg(short = 'v', long = "verbose")]
    verbose: bool,

    /// enable logging to log file 'flow-texpack.log'
    #[arg(short = 'l', long = "log")]
    log: bool,
}

/// Specifies the properties of a App.
pub struct App {
    /// is the cli arguments program is called with:
    cli_args: CliArgs,
    /// is the console used for colored text output and progress bars (verbose mode enabled):
    console: Arc<Console>,
    /// is the set holding all input files (excluding those in exclude_files if any):
    input_files: HashSet<PathBuf>,
    /// is the set holding all exclude files:
    exclude_files: HashSet<PathBuf>,
    /// is the set holding supported image extensions for load:
    supported_extensions_load: HashSet<OsString>,
    /// is the set holding supported image extensions for save:
    supported_extensions_save: HashSet<OsString>,
    /// is the vector holding the loaded textures:
    textures: Vec<Texture>,
    /// is the vector holding the packers:
    packers: Vec<Arc<Packer>>,
    /// is the default hasher:
    hasher: DefaultHasher,
    /// is the hash value:
    hash_value: u64,
}

impl Default for App {
    fn default() -> Self {
        Self {
            cli_args: CliArgs::parse(),
            console: Console::new().shared(),
            input_files: HashSet::new(),
            exclude_files: HashSet::new(),
            supported_extensions_load: HashSet::from([
                OsString::from("bmp"),
                OsString::from("hdr"),
                OsString::from("jpg"),
                OsString::from("png"),
                OsString::from("tga"),
                OsString::from("tiff"),
                OsString::from("webp"),
            ]),
            supported_extensions_save: HashSet::from([
                OsString::from("png"),
                OsString::from("tga"),
                OsString::from("tiff"),
                OsString::from("webp"),
            ]),
            textures: Vec::new(),
            packers: Vec::new(),
            hasher: DefaultHasher::new(),
            hash_value: 0,
        }
    }
}

impl App {
    /// Execute the main application loop.
    ///
    /// # Panics
    ///
    /// If something unexpected happens.
    pub async fn run(&mut self) {
        // start timer:
        let start_time = Instant::now();

        // initialize:
        self.initialize().await;

        if !self.identical_hash().await {
            if !self.input_files.is_empty() {
                // remove old atlas files if any:
                self.remove_old_files();

                // load all input textures:
                self.load_textures().await;

                // sort textures by area (largest area first):
                self.sort_textures();

                // make sure out directory exists:
                if !exists_dir(&self.cli_args.output)
                    && let Some(parent_dir) = self.cli_args.output.parent()
                {
                    create_dir_all(&parent_dir.to_path_buf());
                }

                // pack the textures:
                self.pack_textures();

                // save atlas image/s:
                self.save_atlas_images().await;

                // save atlas descriptor:
                self.save_atlas_descriptor();

                // save new hash value:
                self.save_input_hash();
            } else {
                if self.cli_args.verbose {
                    self.console.print("[dim]No input files...[/]");
                }
            }
        } else {
            if self.cli_args.verbose {
                self.console
                    .print("[dim]Identical hash value. No need to continue...[/]");
            }
        }

        // we're done:
        if self.cli_args.verbose {
            self.console.print("");
            self.console.print(&format!(
                "[dim]Completed in {:.1}s[/]",
                start_time.elapsed().as_secs_f64()
            ));
            self.console.print("[green]Done![/]");
        }
    }

    /// initialize logger and prepare input files vector:
    async fn initialize(&mut self) {
        // initialize logger:
        self.init_logger();

        // prepare input files:
        self.prepare().await;

        // log cli_args options:
        self.log_options();
    }

    /// determine whether current hash is the same as previous:
    async fn identical_hash(&mut self) -> bool {
        if !self.cli_args.force {
            // identical hash from prev run = no need to continue:
            if self.cli_args.verbose {
                if let Ok(_status) = Status::new(&self.console, "Checking input hash...") {
                    return self.check_input_hash().await;
                }
            } else {
                return self.check_input_hash().await;
            }
        }

        false
    }

    /// initialize logger:
    fn init_logger(&self) {
        if self.cli_args.log {
            fern::Dispatch::new()
                .format(|out, message, record| {
                    out.finish(format_args!("[{}] {}", record.level(), message))
                })
                .level(log::LevelFilter::Debug)
                .chain(
                    std::fs::OpenOptions::new()
                        .create(true)
                        .write(true)
                        .truncate(true)
                        .append(false)
                        .open("flow-texpack.log")
                        .unwrap(),
                )
                .apply()
                .unwrap();
        }
    }

    /// log command line arguments program was called with:
    fn log_options(&self) {
        info!("Options that will be used:");

        info!("Exclude files:");
        for path in &self.exclude_files {
            info!("\t {}", path.display());
        }

        info!("Input files (does not include excludes above):");
        for path in &self.input_files {
            info!("\t {}", path.display());
        }

        info!("Load filters:");
        for ext in &self.supported_extensions_load {
            info!("\t {}", ext.display());
        }

        info!("Output dir: {}", self.cli_args.output.display());
        info!("AtlasDescriptor: {:?}", self.cli_args.atlas_descriptor);
        info!("AtlasImage: {:?}", self.cli_args.atlas_image);
        info!("AtlasSize: {:?}", self.cli_args.atlas_size);
        info!("LoadFilter: {:?}", self.cli_args.load_filter);
        info!("Max atlases: {}", self.cli_args.max_atlases);
        info!("Premultiply: {}", self.cli_args.premultiply);
        info!("Trim: {}", self.cli_args.trim);
        info!("Force: {}", self.cli_args.force);
        info!("Unique: {}", self.cli_args.unique);
        info!("Rotate: {}", self.cli_args.rotate);
        info!("Force square: {}", self.cli_args.force_square);
        info!("Adjust size: {}", self.cli_args.adjust_size);
        info!("Adjust fit: {}", self.cli_args.adjust_fit);
        info!("Generate mipmaps: {}", self.cli_args.generate_mipmaps);
        info!("Pad: {}", self.cli_args.pad);
        info!("Rect heuristic: {:?}", self.cli_args.rect_heuristic);
        info!("Verbose: {:?}", self.cli_args.verbose);
    }

    /// prepare input files vector (exclude those in exclude/exclude_file if any):
    async fn prepare(&mut self) {
        // prepare load filters (default includes all if no args set):
        self.prepare_load_filter();

        // prepare files to exlude:
        if let Some(exclude_vec) = self.cli_args.exclude.clone() {
            for path in &exclude_vec {
                self.prepare_exclude_files(path).await;
            }
        } else if let Some(exclude_file) = self.cli_args.exclude_file.clone()
            && exclude_file.is_file()
        {
            self.read_exclude_file(&exclude_file).await;
        }

        // prepare input files and exclude those above if any:
        if let Some(input_vec) = self.cli_args.input.clone() {
            for path in &input_vec {
                self.prepare_input_files(path).await;
            }
        } else if let Some(input_file) = self.cli_args.input_file.clone()
            && input_file.is_file()
        {
            self.read_input_file(&input_file).await;
        }
    }

    fn prepare_load_filter(&mut self) {
        if let Some(load_filters) = self.cli_args.load_filter.clone() {
            self.supported_extensions_load.clear();
            for load_filter in load_filters {
                let ext = get_load_filter_extension(load_filter);
                self.supported_extensions_load.insert(OsString::from(ext));
            }
        }
    }

    /// prepare exclude files vector:
    #[async_recursion::async_recursion]
    async fn prepare_exclude_files(&mut self, path: &PathBuf) {
        if path.is_file() {
            self.add_exclude_file(path);
        } else {
            let mut reader = tokio::fs::read_dir(path).await.unwrap();
            while let Some(f) = reader.next_entry().await.unwrap() {
                if f.path().is_dir() {
                    self.prepare_exclude_files(&f.path()).await;
                } else if f.path().is_file() {
                    self.add_exclude_file(&f.path());
                }
            }
        }
    }

    fn add_exclude_file(&mut self, path: &Path) {
        if let Some(extension) = path.extension() {
            let ext = OsString::from(extension);
            if self.supported_extensions_load.contains(&ext) {
                self.exclude_files.insert(path.to_path_buf());
            }
        }
    }

    /// read exclude file and parse each line and put dir/file into exclude vector:
    async fn read_exclude_file(&mut self, exclude_file: &PathBuf) {
        let contents = tokio::fs::read_to_string(exclude_file).await.unwrap();

        for line in contents.lines() {
            let path = PathBuf::from(line);

            if path.is_file() {
                self.exclude_files.insert(path.clone());
            } else {
                self.prepare_exclude_files(&path).await;
            }
        }
    }

    /// prepare input files vector (exclude those in exclude_files if any and also filter out so
    /// that only the supported extensions is included):
    #[async_recursion::async_recursion]
    async fn prepare_input_files(&mut self, path: &PathBuf) {
        if path.is_file() {
            self.add_input_file(path);
        } else {
            let mut reader = tokio::fs::read_dir(path).await.unwrap();
            while let Some(f) = reader.next_entry().await.unwrap() {
                if f.path().is_dir() {
                    self.prepare_input_files(&f.path()).await;
                } else if f.path().is_file() {
                    self.add_input_file(&f.path());
                }
            }
        }
    }

    /// add input file if it's extension is valid and not to be excluded:
    fn add_input_file(&mut self, path: &PathBuf) {
        if let Some(extension) = path.extension() {
            let ext = OsString::from(extension);

            if self.supported_extensions_load.contains(&ext) && !self.exclude_files.contains(path) {
                self.input_files.insert(path.clone());
                // hash each value if hashing mode:
                if !self.cli_args.force {
                    let data = std::fs::read(path).unwrap();
                    data.hash(&mut self.hasher);
                }
            }
        }
    }

    /// read include file and parse each line and put dir/file into include vector:
    async fn read_input_file(&mut self, input_file: &PathBuf) {
        let contents = tokio::fs::read_to_string(input_file).await.unwrap();

        for line in contents.lines() {
            let path = PathBuf::from(line);

            if path.is_file() {
                self.input_files.insert(path.clone());
            } else {
                self.prepare_input_files(&path).await;
            }
        }
    }

    /// compares current hash value with previous value if any:
    async fn check_input_hash(&mut self) -> bool {
        self.hash_value = self.hasher.finish();
        let old_hash_value = self.get_old_input_hash().await;

        info!(
            "Hash value new: {} old: {}",
            self.hash_value, old_hash_value
        );

        if !self.cli_args.force && self.cli_args.verbose {
            self.console.print("[dim]Checked input hash[/]");
        }

        if self.hash_value == old_hash_value {
            info!("Identical hash value. No need to continue...");
            return true;
        }

        false
    }

    /// returns the hash value stored on disk if any exists:
    async fn get_old_input_hash(&self) -> u64 {
        let file_path = format!("{}.hash", self.cli_args.output.display());
        let result = tokio::fs::read_to_string(file_path).await;

        match result {
            Ok(old_hash_value) => old_hash_value.parse().unwrap(),
            Err(_) => 0,
        }
    }

    /// removes all atlas related files from previous run with same name if any exists:
    fn remove_old_files(&self) {
        let hash_file_path = PathBuf::from(format!("{}.hash", self.cli_args.output.display()));
        let json_file_path = PathBuf::from(format!("{}.json", self.cli_args.output.display()));
        let txt_file_path = PathBuf::from(format!("{}.txt", self.cli_args.output.display()));

        if exists_file(&hash_file_path) {
            remove_file(&hash_file_path);
        }

        if exists_file(&json_file_path) {
            remove_file(&json_file_path);
        }

        if exists_file(&txt_file_path) {
            remove_file(&txt_file_path);
        }

        let mut removed = false;
        for i in 0..4096 {
            for img_ext in &self.supported_extensions_save {
                let atlas_file_path = PathBuf::from(format!(
                    "{}{}.{}",
                    self.cli_args.output.display(),
                    i,
                    img_ext.display()
                ));
                if exists_file(&atlas_file_path) {
                    remove_file(&atlas_file_path);
                    removed = true;
                }
            }
            if !removed {
                break;
            }
        }
    }

    /// loads all textures in input_files vector as separate async tasks and then store each in
    /// textures vector:
    async fn load_textures(&mut self) {
        let mut join_handles: Vec<task::JoinHandle<Texture>> = Vec::new();
        for path in &self.input_files {
            join_handles.push(tokio::spawn(load_texture(
                path.clone(),
                self.cli_args.premultiply,
                self.cli_args.trim,
                self.cli_args.adjust_fit,
                self.cli_args.pad.into(),
                self.cli_args.atlas_size as u32,
            )));
        }

        if self.cli_args.verbose {
            if let Ok(_status) = Status::new(&self.console, "Loading textures...") {
                for join_handle in join_handles {
                    self.textures.push(join_handle.await.unwrap());
                }
            }
        } else {
            for join_handle in join_handles {
                self.textures.push(join_handle.await.unwrap());
            }
        }

        if self.cli_args.verbose {
            self.console.print("[dim]Loaded textures[/]");
        }
    }

    /// sort all textures by area (from largest to smallest):
    fn sort_textures(&mut self) {
        if self.cli_args.verbose {
            if let Ok(_status) = Status::new(&self.console, "Sorting textures by area...") {
                self.textures.sort_by_key(|a| a.get_area());
            }
        } else {
            self.textures.sort_by_key(|a| a.get_area());
        }

        if self.cli_args.verbose {
            self.console.print("[dim]Sorted textures[/]");
        }
    }

    /// pack all loaded textures into 1-n bins of atlas size and store each packer in packers
    /// vector as those will be used to save atlas images/descriptor later as separate async tasks:
    fn pack_textures(&mut self) {
        if self.cli_args.verbose {
            if let Ok(_status) = Status::new(&self.console, "Packing textures...") {
                self.pack();
            }
        } else {
            self.pack();
        }

        if self.cli_args.verbose {
            self.console.print("[dim]Packed textures[/]");
        }
    }

    /// pack textures into each packer of atlas size and remove each packed texture from `textures`
    /// vector and create as many packers as needed up until `max_atlases` or if texture doesn't fit
    /// in given `atlas size`:
    fn pack(&mut self) {
        while !self.textures.is_empty() {
            let width: u32 = self.cli_args.atlas_size as u32;
            let height = width;

            let mut packer = Packer::new(
                width,
                height,
                self.cli_args.pad as i32,
                self.cli_args.generate_mipmaps,
            )
            .unwrap();

            let heuristic = self.convert_rect_heuristic(&self.cli_args.rect_heuristic);
            packer.pack(
                &mut self.textures,
                self.cli_args.unique,
                self.cli_args.rotate,
                self.cli_args.force_square,
                self.cli_args.adjust_size,
                heuristic,
            );

            self.packers.push(packer.shared());

            if self.packers.len() > self.cli_args.max_atlases as usize {
                panic!(
                    "Packing failed. There is a limit of {} atlases being created. Use a larger atlas output size (--atlas-size SIZE)",
                    self.cli_args.max_atlases
                );
            }

            let lock = self.packers.last().unwrap().state.lock().unwrap();
            if lock.textures.is_empty() {
                panic!(
                    "Packing failed: Could not fit texture {}",
                    self.textures.last().unwrap().file_name
                );
            }
        }
    }

    /// saves all atlas images to disk:
    async fn save_atlas_images(&self) {
        if self.cli_args.verbose {
            if let Ok(_status) = Status::new(&self.console, "Writing atlas images...") {
                self.save_images().await;
            }
        } else {
            self.save_images().await;
        }
    }

    /// saves all atlas images (an async task for each save operation to speed things up):
    async fn save_images(&self) {
        let image_extension = get_atlas_image_extension(self.cli_args.atlas_image);
        let mut join_handles: Vec<task::JoinHandle<PathBuf>> = Vec::new();

        for i in 0..self.packers.len() {
            let file_path = PathBuf::from(format!(
                "{}{}.{}",
                self.cli_args.output.display(),
                i,
                image_extension
            ));

            if let Some(packer) = self.packers.get(i) {
                join_handles.push(tokio::spawn(save_image(
                    file_path.clone(),
                    packer.clone(),
                    self.cli_args.atlas_image,
                )));
            }
        }

        for join_handle in join_handles {
            let file_path = join_handle.await.unwrap();
            if self.cli_args.verbose {
                let msg = format!("Wrote '{}'", file_path.display());
                info!("{}", msg);
                self.console.print(&format!("[dim]{}[/]", msg));
            }
        }
    }

    /// saves atlas descriptor to disk:
    fn save_atlas_descriptor(&self) {
        match self.cli_args.atlas_descriptor {
            AtlasDescriptor::Json => self.save_atlas_json(),
            AtlasDescriptor::Txt => self.save_atlas_txt(),
            AtlasDescriptor::TxtDesc => self.save_atlas_txt(),
        }
    }

    /// saves atlas descriptor in JSON format:
    fn save_atlas_json(&self) {
        let file_path = PathBuf::from(format!("{}.json", self.cli_args.output.display()));
        let mut file = std::fs::File::create(file_path.clone()).unwrap();

        file.write_all(String::from("{\n").as_bytes()).unwrap();
        file.write_all(String::from("\t\"ImageAtlas\":\n").as_bytes())
            .unwrap();
        file.write_all(String::from("\t{\n").as_bytes()).unwrap();

        // info part:
        file.write_all(String::from("\t\t\"info\":\n").as_bytes())
            .unwrap();
        file.write_all(String::from("\t\t{\n").as_bytes()).unwrap();
        file.write_all(
            format!("\t\t\t\"numberOfAtlasImages\": {},\n", self.packers.len()).as_bytes(),
        )
        .unwrap();
        file.write_all(format!("\t\t\t\"generatedWith\": \"{}\"\n", NAME).as_bytes())
            .unwrap();
        file.write_all(String::from("\t\t},\n").as_bytes()).unwrap();
        file.write_all(String::from("\t\t\"AtlasImage\":\n").as_bytes())
            .unwrap();
        file.write_all(String::from("\t\t[\n").as_bytes()).unwrap();

        for i in 0..self.packers.len() {
            let img_ext = get_atlas_image_extension(self.cli_args.atlas_image);
            let file_path_stripped =
                PathBuf::from(format!("{}{}", self.cli_args.output.display(), i));

            if let Some(packer) = self.packers.get(i) {
                if i > 0 {
                    file.write_all(String::from(",\n").as_bytes()).unwrap();
                }

                if let Some(file_name) = file_path_stripped.file_name()
                    && let Some(file_name_str) = file_name.to_str()
                {
                    packer.save_json(&mut file, file_name_str, &img_ext);
                }
            }
        }
        file.write_all(String::from("\n\t\t]\n").as_bytes())
            .unwrap();
        file.write_all(String::from("\t}\n").as_bytes()).unwrap();
        file.write_all(String::from("}\n").as_bytes()).unwrap();

        if self.cli_args.verbose {
            let msg = format!("Wrote '{}'", file_path.display());
            info!("{}", msg);
            self.console.print(&format!("[dim]{}[/]", msg));
        }
    }

    /// saves atlas descriptor in TXT format:
    fn save_atlas_txt(&self) {
        let file_path = PathBuf::from(format!("{}.txt", self.cli_args.output.display()));
        let mut file = std::fs::File::create(file_path.clone()).unwrap();

        if self.cli_args.atlas_descriptor == AtlasDescriptor::TxtDesc {
            self.write_txt_header(&mut file);
        }

        // info part:
        file.write_all(format!("{},{}\n", self.packers.len(), NAME).as_bytes())
            .unwrap();

        for i in 0..self.packers.len() {
            let img_ext = get_atlas_image_extension(self.cli_args.atlas_image);
            let file_path_stripped =
                PathBuf::from(format!("{}{}", self.cli_args.output.display(), i));

            if let Some(packer) = self.packers.get(i)
                && let Some(file_name) = file_path_stripped.file_name()
                && let Some(file_name_str) = file_name.to_str()
            {
                packer.save_txt(&mut file, file_name_str, &img_ext);
            }
        }

        if self.cli_args.verbose {
            let msg = format!("Wrote '{}'", file_path.display());
            info!("{}", msg);
            self.console.print(&format!("[dim]{}[/]", msg));
        }
    }

    /// writes the description header for TXT atlas descriptor:
    fn write_txt_header(&self, file: &mut fs::File) {
        file.write_all(String::from("/*\n").as_bytes()).unwrap();
        file.write_all(
            String::from("\t ************************************************\n").as_bytes(),
        )
        .unwrap();
        file.write_all(format!("\t * Generated with: {}\n", NAME).as_bytes())
            .unwrap();
        file.write_all(
            String::from("\t ************************************************\n").as_bytes(),
        )
        .unwrap();
        file.write_all(String::from("\n").as_bytes()).unwrap();
        file.write_all(
            String::from("\t ************************************************\n").as_bytes(),
        )
        .unwrap();
        file.write_all(String::from("\t * Format description:\n").as_bytes())
            .unwrap();
        file.write_all(
            String::from("\t ************************************************\n").as_bytes(),
        )
        .unwrap();
        file.write_all(String::from("\t [info]\n").as_bytes())
            .unwrap();
        file.write_all(String::from("\t numberOfAtlasImages,generatedWith\n").as_bytes())
            .unwrap();
        file.write_all(String::from("\n").as_bytes()).unwrap();
        file.write_all(String::from("\t [AtlasImage (repeated numberOfAtlasImages)]\n").as_bytes())
            .unwrap();
        file.write_all(String::from("\t atlasImageName,numberOfImages,atlasImageWidth,atlasImageHeight,generateMipMaps\n").as_bytes())
            .unwrap();
        file.write_all(String::from("\n").as_bytes()).unwrap();
        file.write_all(String::from("\t [Image (repeated numberOfImages)]\n").as_bytes())
            .unwrap();
        file.write_all(String::from("\t name,x,y,w,h,trimmed,rotated,fx,fy,fw,fh (NOTE: fx,fy,fw,fh valid if trimmed==1)\n").as_bytes())
            .unwrap();
        file.write_all(String::from("\n").as_bytes()).unwrap();
        file.write_all(String::from("\t Text format example:\n").as_bytes())
            .unwrap();
        file.write_all(String::from("\t [info]\n").as_bytes())
            .unwrap();
        file.write_all(String::from("\t [AtlasImage]\n").as_bytes())
            .unwrap();
        file.write_all(String::from("\t [Image]\n").as_bytes())
            .unwrap();
        file.write_all(String::from("\t [Image]\n").as_bytes())
            .unwrap();
        file.write_all(String::from("\t ...\n").as_bytes()).unwrap();
        file.write_all(String::from("\t [AtlasImage]\n").as_bytes())
            .unwrap();
        file.write_all(String::from("\t [Image]\n").as_bytes())
            .unwrap();
        file.write_all(String::from("\t [Image]\n").as_bytes())
            .unwrap();
        file.write_all(String::from("\t ...\n").as_bytes()).unwrap();
        file.write_all(String::from("*/@\n").as_bytes()).unwrap();
    }

    /// converts rect heuristic to the enum used in flow_rectpack:
    fn convert_rect_heuristic(&self, heuristic: &RectHeuristic) -> FreeRectHeuristic {
        match heuristic {
            RectHeuristic::ShortSideFit => FreeRectHeuristic::ShortSideFit,
            RectHeuristic::LongSideFit => FreeRectHeuristic::LongSideFit,
            RectHeuristic::AreaFit => FreeRectHeuristic::AreaFit,
            RectHeuristic::BottomLeft => FreeRectHeuristic::BottomLeft,
            RectHeuristic::ContactPoint => FreeRectHeuristic::ContactPoint,
        }
    }

    /// saves input hash value to disk:
    fn save_input_hash(&self) {
        let file_path = PathBuf::from(format!("{}.hash", self.cli_args.output.display()));
        let data = format!("{}", self.hash_value);

        write_file_sync(&file_path, data.as_bytes()).unwrap();

        if self.cli_args.verbose {
            self.console
                .print(&format!("[dim]Wrote '{}'[/]", file_path.display()));
        }
    }
}

/// Get the atlas image extension for given `atlas_image`.
///
/// # Arguments
///
/// * `atlas_image` - is the atlas image type.
pub fn get_atlas_image_extension(atlas_image: AtlasImage) -> String {
    match atlas_image {
        AtlasImage::Png => String::from("png"),
        AtlasImage::Tga => String::from("tga"),
        AtlasImage::Tiff => String::from("tiff"),
        AtlasImage::Webp => String::from("webp"),
    }
}

/// Get the load filter extension for given `load_filter`.
///
/// # Arguments
///
/// * `load_filter` - is the load filter.
pub fn get_load_filter_extension(load_filter: LoadFilter) -> String {
    match load_filter {
        LoadFilter::Bmp => String::from("bmp"),
        LoadFilter::Hdr => String::from("hdr"),
        LoadFilter::Jpg => String::from("jpg"),
        LoadFilter::Png => String::from("png"),
        LoadFilter::Tga => String::from("tga"),
        LoadFilter::Tiff => String::from("tiff"),
        LoadFilter::Webp => String::from("webp"),
    }
}

/// Create given `dir` recursively.
///
/// # Arguments
///
/// * `dir` - is the directory to create recursively.
///
/// # Panics
///
/// If failed to create given directory.
pub fn create_dir_all(dir: &PathBuf) {
    match fs::create_dir_all(dir) {
        Ok(()) => info!("Created dir: '{}'", dir.display()),
        Err(err) => panic!(
            "Failed to create dir: '{}'. Error msg: '{}'",
            dir.display(),
            err
        ),
    };
}

/// Remove given `dir` recursively.
///
/// # Arguments
///
/// * `dir` - is the directory to remove recursively.
///
/// # Panics
///
/// If failed to remove given directory.
pub fn remove_dir_all(dir: &PathBuf) {
    match fs::remove_dir_all(dir) {
        Ok(()) => info!("Removed dir: '{}'", dir.display()),
        Err(err) => panic!(
            "Failed to remove dir: '{}'. Error msg: '{}'",
            dir.display(),
            err
        ),
    };
}

/// Remove given `file_path`.
///
/// # Arguments
///
/// * `file_path` - is the path to file to remove.
///
/// # Panics
///
/// If failed to remove given `file_path`.
pub fn remove_file(file_path: &PathBuf) {
    match fs::remove_file(file_path) {
        Ok(()) => info!("Removed file: '{}'", file_path.display()),
        Err(err) => panic!(
            "Failed to remove file: '{}'. Error msg: '{}'",
            file_path.display(),
            err
        ),
    };
}

/// Determine whether given `dir` exists.
///
/// # Arguments
///
/// * `dir` - is the directory.
pub fn exists_dir(dir: &Path) -> bool {
    dir.exists()
}

/// Determine whether given `file_path` exists.
///
/// # Arguments
///
/// * `file_path` - is the file path.
pub fn exists_file(file_path: &Path) -> bool {
    file_path.is_file()
}

/// Write `data` to given `file_path`.
///
/// # Arguments
///
/// * `file_path` - is the file path.
/// * `data` - is the data to write.
pub fn write_file_sync(file_path: &PathBuf, data: &[u8]) -> io::Result<()> {
    // create output file:
    let mut file = std::fs::File::create(file_path)?;

    // write data to file:
    file.write_all(data)?;

    info!("Wrote '{}' successfully", file_path.display());
    Ok(())
}

/// load an individual texture from disk and return it (used for async tasks):
async fn load_texture(
    file_path: PathBuf,
    premultiply: bool,
    trim: bool,
    adjust_fit: bool,
    pad: u32,
    atlas_size: u32,
) -> Texture {
    let mut texture = Texture::default();

    texture.load(&file_path, premultiply, trim, adjust_fit, pad, atlas_size);

    texture
}

/// saves an individual atlas image to disk (used for async tasks):
async fn save_image(file_path: PathBuf, packer: Arc<Packer>, image_type: AtlasImage) -> PathBuf {
    packer.save_image(&file_path, image_type);
    file_path
}
