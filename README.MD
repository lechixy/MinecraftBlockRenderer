# Minecraft Block Software Renderer

A fast, headless, lightweight software rasterizer written in **Rust**. It parses Minecraft block models, applies GUI/inventory transforms with zero-margin tight fitting, resolves recursive texture references, and renders crisp pixel-art assets directly into `.webp` (including animated textures with `.mcmeta` interpolation).

## ✨ Features

* **⚡ Fast & Headless:** 100% CPU-based software rasterizer using `glam` and `image`. No GPU, OpenGL, Vulkan, or window context required.
* **📐 Pixel-Perfect GUI Alignment:** Replicates Minecraft's inventory display transforms with zero-margin viewport clipping (tight-fit bounding boxes).
* **🔄 Animated Texture Support:** Parses `.png.mcmeta` definitions (frame sequence, delays, and linear interpolation) and exports animated WebP files.
* **🗜️ Lightweight WebP Output:** Exports static and animated blocks in high-performance `.webp` format for web optimization.
* **🛡️ Robust Texture Resolution:** Recursively resolves model inheritance (`parent`) and texture aliases (`#all`, `#texture`, `force_translucent`, `sprite`).



## ⚡ Quick Start (Pre-built Executable)

The easiest way to run the renderer without installing Rust:

1. Download the latest `mc_renderer.exe` from the [Releases](https://github.com/lechixy/MinecraftBlockRenderer/releases) page.
2. Download the required Minecraft assets from **[mcasset.cloud](https://mcasset.cloud/)**:
   - Select your target Minecraft version (e.g. `26.2` or any).
   - Navigate to `assets/minecraft/models/block/` and download `_all.json` (or generate/merge block models into `_all.json`).
   - Navigate to `assets/minecraft/textures/block/` and download the texture files (including `.png` and `.png.mcmeta` files).
3. Organize the files right next to `mc_renderer.exe` in the following layout:

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

4. Double-click `mc_renderer.exe` (or run it via Command Prompt).
5. Your rendered `.webp` icons will appear directly in `./output/`.


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
mc_renderer.exe --minecraft "C:/MinecraftAssets/minecraft" --output "./custom_output/" --render-resolution 256

```

### CLI Flags

| Flag | Default | Description |
| --- | --- | --- |
| `--minecraft <path>` | Auto-detect | Path to the `minecraft` asset root directory. |
| `--output <path>` | `./output/` | Destination folder for rendered `.webp` images. |
| `--render-resolution <px>` | `128` | Output resolution in square pixels (e.g. `16`, `64`, `128`, `256`). |

---

## 🛠️ Building from Source

### Prerequisites

* [Rust & Cargo](https://www.google.com/search?q=https://www.rust-lang.org/tools/install) (1.70+ recommended)

### Build Steps

1. Clone the repository:
```bash
git clone [https://github.com/lechixy/MinecraftBlockRenderer.git](https://github.com/lechixy/MinecraftBlockRenderer.git)
cd MinecraftBlockRenderer

```


2. Download and set up the `assets/minecraft` directory structure as described in the [Asset Directory Structure](https://www.google.com/search?q=%23-required-asset-directory-structure) section using [mcasset.cloud](https://mcasset.cloud/).
3. Build the optimized release binary:
```bash
cargo build --release

```


4. Run the renderer:
```bash
cargo run --release -- --render-resolution 128

```

## 📦 Dependencies

* [`glam`](https://www.google.com/search?q=https://crates.io/crates/glam) - Fast 3D vector and matrix math.
* [`image`](https://www.google.com/search?q=https://crates.io/crates/image) - Image processing and pixel manipulation.
* [`serde`](https://www.google.com/search?q=https://crates.io/crates/serde) / [`serde_json`](https://www.google.com/search?q=https://crates.io/crates/serde_json) - Model JSON and metadata parsing.
* [`webp_animation`](https://www.google.com/search?q=https://crates.io/crates/webp_animation) - Encoding animated WebP files.


## 📄 License

This project is licensed under the MIT License.