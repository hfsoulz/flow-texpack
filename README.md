# flow-texpack

`flow-texpack` is a program that will allow you to generate texture atlas from
input images (BMP, HDR, JPG, PNG, TGA, TIFF, WEBP). The application generates
both texture atlas and descriptions file that can be read by a game.

## Installation

Install from Cargo:

```sh
cargo install flow-texpack
```

Build from source:

```sh
cargo install --locked --path .
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

## LICENSE

See the file 'LICENSE' for license information.
