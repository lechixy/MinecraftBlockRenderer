# Minecraft Block Renderer

A fast, headless, lightweight software rasterizer written in **Rust**. It parses Minecraft block models, applies GUI/inventory transforms with zero-margin tight fitting, resolves recursive texture references, and renders crisp pixel-art assets directly into `.webp` (including animated textures with `.mcmeta` interpolation).

## ✨ Features

* **Fast & Headless:** 100% CPU-based software rasterizer using `glam` and `image`. No GPU, OpenGL, Vulkan, or window context required.
* **Pixel-Perfect GUI Alignment:** Replicates Minecraft's inventory display transforms with zero-margin viewport clipping (tight-fit bounding boxes).
* **4x SSAA Anti-Aliasing:** Softens diagonal edges using 2x2 subpixel supersampling and alpha-weighted downsampling to match Minecraft's GUI smoothness.
* **Vanilla Face Shading:** Accurately replicates Minecraft's orientation-based lighting coefficients (Down: 0.5, Up: 1.0, North/South: 0.8, West/East: 0.6) with `shade: false` element support.
* **Extended Model Support:** Full hierarchy resolution and multi-element rendering for cubes, columns, stairs, slabs, carpets, and more.
* **Animated Texture Support:** Parses `.png.mcmeta` definitions (frame sequence, delays, and optional linear interpolation) and exports animated WebP files.
* **Lightweight WebP Output:** Exports static and animated blocks in high-performance `.webp` format for web optimization.
* **Robust Texture Resolution:** Recursively resolves model inheritance (`parent`) and texture aliases (`#all`, `#texture`, `force_translucent`, `sprite`).

## 💖 Showcase

You can see the output in the `/output` folder. Here are some sample renders generated directly with the renderer:

| Block Type | Preview | Block Name | Features |
| --- | --- | --- | --- |
| **Animated** | <img src="https://raw.githubusercontent.com/lechixy/MinecraftBlockRenderer/refs/heads/main/output/warped_stem.webp" width=50> | Warped Stem | Animated WebP with frame interpolation |
| **Animated** | <img src="https://raw.githubusercontent.com/lechixy/MinecraftBlockRenderer/refs/heads/main/output/magma_block.webp" width=50> | Magma Block | Multi-frame texture animation |
| **Animated** | <img src="https://raw.githubusercontent.com/lechixy/MinecraftBlockRenderer/refs/heads/main/output/crying_obsidian.webp" width=50> | Crying Obsidian | Dripping particle effect animation |
| **Animated** | <img src="https://raw.githubusercontent.com/lechixy/MinecraftBlockRenderer/refs/heads/main/output/respawn_anchor_4.webp" width=50> | Respawn Anchor | Subtle glowing loop animation |
| **Animated** | <img src="https://raw.githubusercontent.com/lechixy/MinecraftBlockRenderer/refs/heads/main/output/sculk_sensor.webp" width=50> | Sculk Sensor | Wave animation |
| **Translucent** | <img src="https://raw.githubusercontent.com/lechixy/MinecraftBlockRenderer/refs/heads/main/output/light_blue_stained_glass.webp" width=50> | Light Blue Stained Glass | Alpha blending & transparency support |
| **Static** | <img src="https://raw.githubusercontent.com/lechixy/MinecraftBlockRenderer/refs/heads/main/output/crafting_table.webp" width=50> | Crafting Table | Multi-face unique texture mapping |
| **Static** | <img src="https://raw.githubusercontent.com/lechixy/MinecraftBlockRenderer/refs/heads/main/output/tnt.webp" width=50> | TNT | Zero-margin inventory transform |
| **Static** | <img src="https://raw.githubusercontent.com/lechixy/MinecraftBlockRenderer/refs/heads/main/output/diamond_block.webp" width=50> | Diamond Block | Clean cubic rasterization |
| **Static** | <img src="https://raw.githubusercontent.com/lechixy/MinecraftBlockRenderer/refs/heads/main/output/bookshelf.webp" width=50> | Bookshelf | Crisp pixel-art downscaling |

## ⚡ Quick Start (Pre-built Executable)

The easiest way to run the renderer without installing Rust:

1. Download the latest `mc_renderer.exe` from the [Releases](https://www.google.com/search?q=https://github.com/lechixy/MinecraftBlockRenderer/releases) page.
2. Download the required Minecraft assets from **[mcasset.cloud](https://mcasset.cloud/)**:

* Select your target Minecraft version (e.g. `26.2` or any).
* Navigate to `assets/minecraft/models/block/` and download `_all.json` (or generate/merge block models into `_all.json`).
* Navigate to `assets/minecraft/textures/block/` and download the texture files (including `.png` and `.png.mcmeta` files).

1. Organize the files right next to `mc_renderer.exe` in the following layout:

```text
my_render_folder/
├── mc_renderer.exe
└── minecraft/
    ├── models/
    │   └── block/
    │       └── _all.json
    └── textures/
        └── block/
            ├── crafting_table_top.png
            ├── sea_lantern.png
            ├── sea_lantern.png.mcmeta
            └── ...
```

1. Double-click `mc_renderer.exe` (or run it via Command Prompt).
2. Your rendered `.webp` icons will appear directly in `./output/`.

## 📁 Required Asset Directory Structure

The renderer requires the standard `minecraft` asset tree containing `models/block/_all.json` and block textures under `textures/block/`.

```text
minecraft/
├── models/
│   └── block/
│       └── _all.json
└── textures/
    └── block/
        ├── oak_planks.png
        ├── sea_lantern.png
        ├── sea_lantern.png.mcmeta
        └── ...
```

### Obtaining Assets via [mcasset.cloud](https://mcasset.cloud/)

1. Open [mcasset.cloud](https://mcasset.cloud/) in your browser and pick your game version.
2. **Models:** Download `assets/minecraft/models/block/_all.json` and place it under `minecraft/models/block/_all.json`.
3. **Textures:** Download all files from `assets/minecraft/textures/block/` (make sure both `.png` and `.png.mcmeta` files are included) and place them under `minecraft/textures/block/`.

---

## 💻 CLI Usage & Arguments

You can run the program with optional flags to customize paths and render sizes:

### Default Run

```cmd
mc_renderer.exe
```

### Custom Arguments

```cmd
mc_renderer.exe --minecraft "C:/MinecraftAssets/minecraft" --output "./custom_output/" --render-resolution 256 --interpolate
```

### CLI Flags

| Flag | Default | Description |
| --- | --- | --- |
| `--minecraft <path>` | Auto-detect | Path to the `minecraft` asset root directory. |
| `--output <path>` | `./output/` | Destination folder for rendered `.webp` images. |
| `--render-resolution <px>` | `128` | Output resolution in square pixels (e.g. `16`, `64`, `128`,`256`). |
| `--interpolate`, `-i` | `false` | Enable linear frame interpolation for animated textures with `interpolate: true`. |
| `--no-aa` | `false` | Disable 4x SSAA anti-aliasing (enabled by default). |

## 🛠️ Building from Source

### Prerequisites

* [Rust & Cargo](https://www.google.com/search?q=https://www.rust-lang.org/tools/install) (1.70+ recommended)

### Build Steps

1. Clone the repository:

```bash
git clone https://github.com/lechixy/MinecraftBlockRenderer.git
cd MinecraftBlockRenderer
```

1. Download and set up the `assets/minecraft` directory structure as described in the [Asset Directory Structure](https://www.google.com/search?q=%23-required-asset-directory-structure) section using [mcasset.cloud](https://mcasset.cloud/).
2. Build the optimized release binary:

```bash
cargo build --release
```

1. Run the renderer:

```bash
cargo run --release -- --render-resolution 128 --interpolate
```

## 📦 Dependencies

* [`glam`](https://www.google.com/search?q=https://crates.io/crates/glam) - Fast 3D vector and matrix math.
* [`image`](https://www.google.com/search?q=https://crates.io/crates/image) - Image processing and pixel manipulation.
* [`serde`](https://www.google.com/search?q=https://crates.io/crates/serde) / [`serde_json`](https://www.google.com/search?q=https://crates.io/crates/serde_json) - Model JSON and metadata parsing.
* [`webp_animation`](https://www.google.com/search?q=https://crates.io/crates/webp_animation) - Encoding animated WebP files.

## 📄 License

This project is licensed under the MIT License.
