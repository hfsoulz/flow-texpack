# flow-texpack

`flow-texpack` is a program that will allow you to generate texture atlas from
input images (BMP, HDR, JPG, PNG, TGA, TIFF, WEBP). The application generates
both texture atlas and descriptions file that can be read by a game.

## Installation

Install `Rust` from your package manager or by downloading from here:
[https://rust-lang.org/](https://rust-lang.org/).

## Build from source

Build using release mode and install locally (on GNU/Linux this is `~/.cargo/bin`):

```sh
cargo install --locked --path .
```

## Install from Cargo

Install directly from Cargo:

```sh
cargo install flow-texpack
```

## Usage

Show available options:

```sh
flow-texpack -h
```

or

```sh
flow-texpack --help
```

## Examples

Generate from input `data/characters` and `data/tiles`, write output to
`out/atlas` and enable the options: `premultiply` pixels by their alpha
channel, `trim` excess transparency off the textures, `remove duplicate
textures` from the atlas, enable `rotation` of textures 90 degrees clockwise,
`pad` each texture by 2 pixels and finally enable `verbose` output mode.

```sh
flow-texpack -i data/characters data/tiles -o out/atlas -m -t -u -r -p 2 -v
```

Enable `load filter` so that only `TGA` images are included in the texture atlas:

```sh
flow-texpack -i data/tiles -o out/atlas --load-filter tga -v
```

Enable rect heuristic `AreaFit`:

```sh
flow-texpack -i data/tiles -o out/atlas --rect-heuristic area-fit -v
```

Enable output `atlas size` of **2048x2048**:

```sh
flow-texpack -i data/tiles -o out/atlas --atlas-size pot2048 -v
```

Read input files/directories from `input.txt` but exclude all in `exclude.txt`:

```sh
flow-texpack --input-file input.txt --exlude-file exclude.txt -o out/atlas -v
```

`Adjust atlas size` automatically so that texture will fit:

```sh
flow-texpack -i data/characters -o out/atlas --adjust-size -v
```

`Adjust texture size` so that it will fit given atlas size:

```sh
flow-texpack -i data/characters -o out/atlas --adjust-fit -v
```

## LICENSE

See the file 'LICENSE' for license information.
