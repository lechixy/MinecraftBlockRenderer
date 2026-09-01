use glam::{vec2, vec3, vec4, Mat4, Vec2, Vec3, Vec4};
use image::{Rgba, RgbaImage};
use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::Instant;
use webp_animation::prelude::*;

// --- Data Models ---
#[derive(Deserialize, Debug, Clone)]
pub struct MinecraftModel {
    pub parent: Option<String>,
    pub textures: Option<HashMap<String, serde_json::Value>>,
    pub elements: Option<Vec<ModelElement>>,
    pub display: Option<HashMap<String, DisplayTransform>>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ModelElement {
    pub from: [f32; 3],
    pub to: [f32; 3],
    pub shade: Option<bool>,
    pub faces: HashMap<String, ModelFace>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ModelFace {
    pub texture: String,
    pub uv: Option<[f32; 4]>,
    pub rotation: Option<i32>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct DisplayTransform {
    pub rotation: Option<[f32; 3]>,
    pub translation: Option<[f32; 3]>,
    pub scale: Option<[f32; 3]>,
}

#[derive(Clone)]
struct Vertex {
    pos: Vec4,
    uv: Vec2,
    brightness: f32,
}

// --- .mcmeta Models ---
#[derive(Deserialize, Debug, Clone)]
pub struct McMeta {
    pub animation: Option<AnimationMeta>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct AnimationMeta {
    pub interpolate: Option<bool>,
    pub frametime: Option<u32>,
    pub frames: Option<Vec<serde_json::Value>>,
}

fn interpolate_frames(img_a: &RgbaImage, img_b: &RgbaImage, t: f32) -> RgbaImage {
    let (w, h) = (img_a.width(), img_a.height());
    let mut out = RgbaImage::new(w, h);

    for y in 0..h {
        for x in 0..w {
            let p_a = img_a.get_pixel(x, y);
            let p_b = img_b.get_pixel(x, y);

            let r = (p_a[0] as f32 * (1.0 - t) + p_b[0] as f32 * t).round() as u8;
            let g = (p_a[1] as f32 * (1.0 - t) + p_b[1] as f32 * t).round() as u8;
            let b = (p_a[2] as f32 * (1.0 - t) + p_b[2] as f32 * t).round() as u8;
            let a = (p_a[3] as f32 * (1.0 - t) + p_b[3] as f32 * t).round() as u8;

            out.put_pixel(x, y, Rgba([r, g, b, a]));
        }
    }
    out
}

#[derive(Clone)]
struct AnimatedTexture {
    pub frames: Vec<RgbaImage>,
    pub frame_delays_ms: Vec<u32>,
}

impl AnimatedTexture {
    pub fn from_file(base_path: &Path, allow_interpolation: bool) -> Option<Self> {
        let png_path = base_path.with_extension("png");
        let meta_path = base_path.with_extension("png.mcmeta");

        let img = image::open(&png_path).ok()?.into_rgba8();
        let width = img.width();
        let height = img.height();

        let is_animated = height > width || meta_path.exists();

        if !is_animated {
            return Some(Self {
                frames: vec![img],
                frame_delays_ms: vec![50],
            });
        }

        let total_slices = height / width;
        let mut base_frametime = 1;
        let mut custom_frames = None;
        let mut should_interpolate = false;

        if let Ok(meta_str) = fs::read_to_string(&meta_path) {
            if let Ok(meta) = serde_json::from_str::<McMeta>(&meta_str) {
                if let Some(anim) = meta.animation {
                    base_frametime = anim.frametime.unwrap_or(1);
                    custom_frames = anim.frames;
                    should_interpolate = anim.interpolate.unwrap_or(false) && allow_interpolation;
                }
            }
        }

        let mut extracted_slices = Vec::new();
        for i in 0..total_slices {
            let slice = image::imageops::crop_imm(&img, 0, i * width, width, width).to_image();
            extracted_slices.push(slice);
        }

        let mut keyframes: Vec<(usize, u32)> = Vec::new();

        if let Some(seq) = custom_frames {
            for item in seq {
                if let Some(idx) = item.as_u64() {
                    let idx = idx as usize;
                    if idx < extracted_slices.len() {
                        keyframes.push((idx, base_frametime));
                    }
                } else if let Some(obj) = item.as_object() {
                    let idx = obj.get("index").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                    let time = obj.get("time").and_then(|v| v.as_u64()).unwrap_or(base_frametime as u64) as u32;
                    if idx < extracted_slices.len() {
                        keyframes.push((idx, time));
                    }
                }
            }
        } else {
            for i in 0..extracted_slices.len() {
                keyframes.push((i, base_frametime));
            }
        }

        if keyframes.is_empty() {
            return None;
        }

        let mut final_frames = Vec::new();
        let mut final_delays = Vec::new();

        if should_interpolate {
            let n = keyframes.len();
            for i in 0..n {
                let (curr_idx, ticks) = keyframes[i];
                let next_idx = keyframes[(i + 1) % n].0;

                let curr_img = &extracted_slices[curr_idx];
                let next_img = &extracted_slices[next_idx];

                for step in 0..ticks {
                    let t = step as f32 / ticks as f32;
                    if t == 0.0 {
                        final_frames.push(curr_img.clone());
                    } else {
                        final_frames.push(interpolate_frames(curr_img, next_img, t));
                    }
                    final_delays.push(50);
                }
            }
        } else {
            for (idx, ticks) in keyframes {
                final_frames.push(extracted_slices[idx].clone());
                final_delays.push(ticks * 50);
            }
        }

        Some(Self {
            frames: final_frames,
            frame_delays_ms: final_delays,
        })
    }

    pub fn get_frame(&self, index: usize) -> &RgbaImage {
        &self.frames[index % self.frames.len()]
    }
}

// --- Model Resolver ---
fn resolve_model(name: &str, models: &HashMap<String, MinecraftModel>) -> MinecraftModel {
    let normalized = name.replace("minecraft:", "").replace("block/", "");
    let mut current = models
        .get(&normalized)
        .cloned()
        .unwrap_or_else(|| MinecraftModel {
            parent: None, textures: None, elements: None, display: None,
        });

    if let Some(parent_name) = &current.parent {
        let parent_model = resolve_model(parent_name, models);
        
        if current.elements.is_none() {
            current.elements = parent_model.elements.clone();
        }
        
        let mut merged_tex = parent_model.textures.unwrap_or_default();
        if let Some(child_tex) = current.textures {
            merged_tex.extend(child_tex);
        }
        current.textures = Some(merged_tex);

        let mut merged_disp = parent_model.display.unwrap_or_default();
        if let Some(child_disp) = current.display {
            merged_disp.extend(child_disp);
        }
        current.display = Some(merged_disp);
    }
    current
}

fn resolve_texture(mut tex: String, textures: &HashMap<String, serde_json::Value>) -> String {
    while tex.starts_with('#') {
        let key = &tex[1..];
        if let Some(next_tex_val) = textures.get(key) {
            if let Some(next_tex) = next_tex_val.as_str() {
                tex = next_tex.to_string();
            } else if let Some(obj) = next_tex_val.as_object() {
                if let Some(sprite_val) = obj.get("sprite").and_then(|v| v.as_str()) {
                    tex = sprite_val.to_string();
                } else {
                    break;
                }
            } else {
                break;
            }
        } else {
            break;
        }
    }
    tex.replace("minecraft:", "").replace("block/", "")
}

// --- Software Rasterizer ---
fn draw_triangle(
    fb: &mut [u32], zb: &mut [f32], w: usize, h: usize,
    v0: Vertex, v1: Vertex, v2: Vertex,
    tex: &RgbaImage,
) {
    let edge = |a: &Vec4, b: &Vec4, c: &Vec4| -> f32 {
        (c.x - a.x) * (b.y - a.y) - (c.y - a.y) * (b.x - a.x)
    };

    let mut area = edge(&v0.pos, &v1.pos, &v2.pos);
    if area.abs() < 1e-5 { return; }

    let (v0, v1, v2) = if area < 0.0 {
        area = -area;
        (v0, v2, v1)
    } else {
        (v0, v1, v2)
    };

    let min_x = (v0.pos.x.min(v1.pos.x).min(v2.pos.x).max(0.0) as usize).min(w - 1);
    let max_x = (v0.pos.x.max(v1.pos.x).max(v2.pos.x).max(0.0) as usize).min(w - 1);
    let min_y = (v0.pos.y.min(v1.pos.y).min(v2.pos.y).max(0.0) as usize).min(h - 1);
    let max_y = (v0.pos.y.max(v1.pos.y).max(v2.pos.y).max(0.0) as usize).min(h - 1);

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let p = vec4(x as f32 + 0.5, y as f32 + 0.5, 0.0, 1.0);
            let w0 = edge(&v1.pos, &v2.pos, &p);
            let w1 = edge(&v2.pos, &v0.pos, &p);
            let w2 = edge(&v0.pos, &v1.pos, &p);

            if w0 >= 0.0 && w1 >= 0.0 && w2 >= 0.0 {
                let w0 = w0 / area;
                let w1 = w1 / area;
                let w2 = w2 / area;

                let z = w0 * v0.pos.z + w1 * v1.pos.z + w2 * v2.pos.z;
                let idx = y * w + x;
                
                if z < zb[idx] {
                    let uv = v0.uv * w0 + v1.uv * w1 + v2.uv * w2;
                    let tx = (uv.x * tex.width() as f32) as u32;
                    let ty = (uv.y * tex.height() as f32) as u32;
                    let tx = tx.clamp(0, tex.width().saturating_sub(1));
                    let ty = ty.clamp(0, tex.height().saturating_sub(1));

                    let pixel = tex.get_pixel(tx, ty);
                    let src_a = pixel[3] as f32 / 255.0;

                    if src_a > 0.01 {
                        let br = v0.brightness;
                        let src_r = pixel[0] as f32 * br;
                        let src_g = pixel[1] as f32 * br;
                        let src_b = pixel[2] as f32 * br;

                        let prev = fb[idx];
                        let dst_a = ((prev >> 24) & 0xFF) as f32 / 255.0;
                        let dst_r = ((prev >> 16) & 0xFF) as f32;
                        let dst_g = ((prev >> 8) & 0xFF) as f32;
                        let dst_b = (prev & 0xFF) as f32;

                        let out_a = src_a + dst_a * (1.0 - src_a);
                        let (out_r, out_g, out_b) = if out_a > 0.0 {
                            (
                                (src_r * src_a + dst_r * dst_a * (1.0 - src_a)) / out_a,
                                (src_g * src_a + dst_g * dst_a * (1.0 - src_a)) / out_a,
                                (src_b * src_a + dst_b * dst_a * (1.0 - src_a)) / out_a,
                            )
                        } else {
                            (0.0, 0.0, 0.0)
                        };

                        if pixel[3] == 255 {
                            zb[idx] = z;
                        }

                        fb[idx] = (((out_a * 255.0) as u32) << 24)
                            | (((out_r.clamp(0.0, 255.0)) as u32) << 16)
                            | (((out_g.clamp(0.0, 255.0)) as u32) << 8)
                            | ((out_b.clamp(0.0, 255.0)) as u32);
                    }
                }
            }
        }
    }
}

// 4x SSAA Render ve Downsampling
fn render_single_frame(
    elements: &[ModelElement],
    textures: &HashMap<String, serde_json::Value>,
    texture_cache: &HashMap<String, AnimatedTexture>,
    mvp: Mat4,
    frame_index: usize,
    target_size: usize,
    use_aa: bool,
) -> RgbaImage {
    let scale = if use_aa { 2 } else { 1 };
    let render_size = target_size * scale;

    let mut frame_buffer = vec![0u32; render_size * render_size];
    let mut z_buffer = vec![f32::MAX; render_size * render_size];

    for el in elements {
        let min_el = Vec3::from(el.from) / 16.0 - 0.5;
        let max_el = Vec3::from(el.to) / 16.0 - 0.5;
        let shade_enabled = el.shade.unwrap_or(true);

        for (face_name, face) in &el.faces {
            let texture_path = resolve_texture(face.texture.clone(), textures);
            let animated_tex = match texture_cache.get(&texture_path) {
                Some(t) => t,
                None => continue,
            };
            let tex = animated_tex.get_frame(frame_index);

            let brightness = if shade_enabled {
                match face_name.as_str() {
                    "down" => 0.5,
                    "up" => 1.0,
                    "north" | "south" => 0.8,
                    "west" | "east" => 0.6,
                    _ => 1.0,
                }
            } else {
                1.0
            };

            let corners = match face_name.as_str() {
                "north" => [vec3(max_el.x, max_el.y, min_el.z), vec3(min_el.x, max_el.y, min_el.z), vec3(max_el.x, min_el.y, min_el.z), vec3(min_el.x, min_el.y, min_el.z)],
                "south" => [vec3(min_el.x, max_el.y, max_el.z), vec3(max_el.x, max_el.y, max_el.z), vec3(min_el.x, min_el.y, max_el.z), vec3(max_el.x, min_el.y, max_el.z)],
                "west"  => [vec3(min_el.x, max_el.y, min_el.z), vec3(min_el.x, max_el.y, max_el.z), vec3(min_el.x, min_el.y, min_el.z), vec3(min_el.x, min_el.y, max_el.z)],
                "east"  => [vec3(max_el.x, max_el.y, max_el.z), vec3(max_el.x, max_el.y, min_el.z), vec3(max_el.x, min_el.y, max_el.z), vec3(max_el.x, min_el.y, min_el.z)],
                "up"    => [vec3(min_el.x, max_el.y, min_el.z), vec3(max_el.x, max_el.y, min_el.z), vec3(min_el.x, max_el.y, max_el.z), vec3(max_el.x, max_el.y, max_el.z)],
                "down"  => [vec3(min_el.x, min_el.y, max_el.z), vec3(max_el.x, min_el.y, max_el.z), vec3(min_el.x, min_el.y, min_el.z), vec3(max_el.x, min_el.y, min_el.z)],
                _ => continue,
            };

            let uv_coords = face.uv.unwrap_or([0.0, 0.0, 16.0, 16.0]);
            let (u1, v1) = (uv_coords[0] / 16.0, uv_coords[1] / 16.0);
            let (u2, v2) = (uv_coords[2] / 16.0, uv_coords[3] / 16.0);

            let mut uvs = [vec2(u1, v1), vec2(u2, v1), vec2(u1, v2), vec2(u2, v2)];
            let rot = (face.rotation.unwrap_or(0) / 90) % 4;
            for _ in 0..rot {
                uvs = [uvs[2], uvs[0], uvs[3], uvs[1]];
            }

            let mut clip_verts = Vec::new();
            for i in 0..4 {
                let clip_pos = mvp * Vec4::new(corners[i].x, corners[i].y, corners[i].z, 1.0);
                
                let screen_x = (clip_pos.x / clip_pos.w * 0.5 + 0.5) * render_size as f32;
                let screen_y = (-clip_pos.y / clip_pos.w * 0.5 + 0.5) * render_size as f32; 
                let screen_z = clip_pos.z / clip_pos.w;

                clip_verts.push(Vertex {
                    pos: vec4(screen_x, screen_y, screen_z, 1.0),
                    uv: uvs[i],
                    brightness,
                });
            }

            draw_triangle(&mut frame_buffer, &mut z_buffer, render_size, render_size, clip_verts[0].clone(), clip_verts[1].clone(), clip_verts[2].clone(), tex);
            draw_triangle(&mut frame_buffer, &mut z_buffer, render_size, render_size, clip_verts[1].clone(), clip_verts[3].clone(), clip_verts[2].clone(), tex);
        }
    }

    let mut out_img = RgbaImage::new(target_size as u32, target_size as u32);

    if use_aa {
        // 2x2 Subpixel Box Filtering (Alpha-Weighted Downsample)
        for y in 0..target_size {
            for x in 0..target_size {
                let mut sum_r = 0.0;
                let mut sum_g = 0.0;
                let mut sum_b = 0.0;
                let mut sum_a = 0.0;

                for dy in 0..2 {
                    for dx in 0..2 {
                        let sx = x * 2 + dx;
                        let sy = y * 2 + dy;
                        let pixel = frame_buffer[sy * render_size + sx];

                        let a = ((pixel >> 24) & 0xFF) as f32 / 255.0;
                        let r = ((pixel >> 16) & 0xFF) as f32;
                        let g = ((pixel >> 8) & 0xFF) as f32;
                        let b = (pixel & 0xFF) as f32;

                        sum_r += r * a;
                        sum_g += g * a;
                        sum_b += b * a;
                        sum_a += a;
                    }
                }

                let final_a = sum_a / 4.0;
                let (final_r, final_g, final_b) = if sum_a > 0.0 {
                    (sum_r / sum_a, sum_g / sum_a, sum_b / sum_a)
                } else {
                    (0.0, 0.0, 0.0)
                };

                out_img.put_pixel(
                    x as u32,
                    y as u32,
                    Rgba([
                        final_r.round() as u8,
                        final_g.round() as u8,
                        final_b.round() as u8,
                        (final_a * 255.0).round() as u8,
                    ]),
                );
            }
        }
    } else {
        for y in 0..target_size {
            for x in 0..target_size {
                let pixel = frame_buffer[y * render_size + x];
                let r = ((pixel >> 16) & 0xFF) as u8;
                let g = ((pixel >> 8) & 0xFF) as u8;
                let b = (pixel & 0xFF) as u8;
                let a = ((pixel >> 24) & 0xFF) as u8;
                out_img.put_pixel(x as u32, y as u32, Rgba([r, g, b, a]));
            }
        }
    }

    out_img
}

fn save_as_animated_webp(
    file_path: &str,
    frames: &[RgbaImage],
    frame_delays_ms: &[u32],
    size: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut encoder = Encoder::new((size, size))?;
    let mut timestamp = 0;

    for (i, frame) in frames.iter().enumerate() {
        encoder.add_frame(frame.as_raw(), timestamp)?;
        let delay = frame_delays_ms.get(i).copied().unwrap_or(250) as i32;
        timestamp += delay;
    }

    let webp_data = encoder.finalize(timestamp)?;
    fs::write(file_path, &*webp_data)?;
    Ok(())
}

fn resolve_minecraft_base(custom_path: Option<&str>) -> Option<(PathBuf, &'static str)> {
    if let Some(path_str) = custom_path {
        let arg_path = PathBuf::from(path_str);
        if arg_path.exists() && arg_path.join("models/block/_all.json").exists() {
            return Some((arg_path, "Provided via --minecraft argument"));
        }
    }

    if let Ok(exe_path) = env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let exe_mc = exe_dir.join("minecraft");
            if exe_mc.exists() && exe_mc.join("models/block/_all.json").exists() {
                return Some((exe_mc, "Detected beside the executable directory"));
            }
        }
    }

    let cur_mc = PathBuf::from("minecraft");
    if cur_mc.exists() && cur_mc.join("models/block/_all.json").exists() {
        return Some((cur_mc, "Detected in current working directory (minecraft/)"));
    }
    let cur_assets_mc = PathBuf::from("assets/minecraft");
    if cur_assets_mc.exists() && cur_assets_mc.join("models/block/_all.json").exists() {
        return Some((cur_assets_mc, "Detected in current working directory (assets/minecraft/)"));
    }

    None
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut custom_mc_path: Option<String> = None;
    let mut save_path = String::from("./output/");
    let mut render_size: usize = 128;
    let mut allow_interpolation = false;
    let mut use_aa = true; // Anti-aliasing varsayılan olarak açık

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--minecraft" => {
                if i + 1 < args.len() {
                    custom_mc_path = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            "--output" => {
                if i + 1 < args.len() {
                    let mut path = args[i + 1].clone();
                    if !path.ends_with('/') && !path.ends_with('\\') {
                        path.push('/');
                    }
                    save_path = path;
                    i += 1;
                }
            }
            "--render-resolution" => {
                if i + 1 < args.len() {
                    if let Ok(val) = args[i + 1].parse::<usize>() {
                        render_size = val;
                    }
                    i += 1;
                }
            }
            "--interpolate" | "-i" => {
                allow_interpolation = true;
            }
            "--no-aa" => {
                use_aa = false;
            }
            _ => {}
        }
        i += 1;
    }

    let (mc_base, source_description) = match resolve_minecraft_base(custom_mc_path.as_deref()) {
        Some(res) => res,
        None => {
            eprintln!("\n=======================================================");
            eprintln!("[ERROR] Minecraft asset directory could not be located!");
            eprintln!("=======================================================");
            eprintln!("Available CLI Arguments:");
            eprintln!("  --minecraft <path>          Path to 'minecraft' folder");
            eprintln!("  --output <path>             Custom output folder (default: ./output/)");
            eprintln!("  --render-resolution <size>  Render resolution in pixels (default: 128)");
            eprintln!("  --interpolate, -i           Enable animation frame interpolation (default: false)");
            eprintln!("  --no-aa                     Disable 4x SSAA anti-aliasing (default: enabled)");
            eprintln!("\nExample usage:");
            eprintln!("  mc_renderer --minecraft \"C:/assets/minecraft\" --output \"./renders/\" --render-resolution 256");
            eprintln!("=======================================================\n");
            std::process::exit(1);
        }
    };

    println!("--------------------------------------------------");
    println!("Minecraft Asset Location : {:?}", fs::canonicalize(&mc_base).unwrap_or(mc_base.clone()));
    println!("Resolution Source        : {}", source_description);
    println!("Render Resolution        : {}x{}", render_size, render_size);
    println!("Anti-Aliasing (4x SSAA)  : {}", use_aa);
    println!("Frame Interpolation      : {}", allow_interpolation);
    println!("Output Directory         : {}", save_path);
    println!("--------------------------------------------------");

    let models_json_path = mc_base.join("models/block/_all.json");
    let textures_base_path = mc_base.join("textures/block");

    let models_json = fs::read_to_string(&models_json_path)
        .unwrap_or_else(|_| panic!("Failed to read '{:?}'", models_json_path));
    let all_models: HashMap<String, MinecraftModel> =
        serde_json::from_str(&models_json).expect("Failed to parse models JSON.");

    if Path::new(&save_path).exists() {
        fs::remove_dir_all(&save_path).expect("Failed to clean previous output directory!");
    }

    fs::create_dir_all(&save_path).expect("Failed to create output directory!");

    let mut created_count = 0;
    let time_start = Instant::now();

    let mut global_texture_cache: HashMap<String, AnimatedTexture> = HashMap::new();

    for (target_block, _) in &all_models {
        let model = resolve_model(target_block, &all_models);
        let elements = model.elements.unwrap_or_default();
        let textures = model.textures.unwrap_or_default();

        if elements.is_empty() || textures.is_empty() || target_block.starts_with("template") {
            continue;
        }

        let only_render_this_parents = vec![
            "block/block",
            "minecraft:block/cube_all",
            "minecraft:block/cube_column",
            "minecraft:block/cube_bottom_top",
            "minecraft:block/cube",
            "minecraft:block/stairs",
            "block/stairs",
            "minecraft:block/slab",
            "block/slab",
            "minecraft:block/carpet",
            "block/carpet",
        ];
        if !only_render_this_parents.contains(&model.parent.as_deref().unwrap_or("")) {
            continue;
        }

        let mut max_animation_frames = 1;
        let mut frame_delays = Vec::new();
        let mut all_textures_found = true;

        for el in &elements {
            for face in el.faces.values() {
                let texture_path = resolve_texture(face.texture.clone(), &textures);
                if !global_texture_cache.contains_key(&texture_path) {
                    let full_path = textures_base_path.join(&texture_path);
                    if let Some(anim_tex) = AnimatedTexture::from_file(&full_path, allow_interpolation) {
                        global_texture_cache.insert(texture_path.clone(), anim_tex);
                    } else {
                        all_textures_found = false;
                        break;
                    }
                }

                if let Some(anim_tex) = global_texture_cache.get(&texture_path) {
                    if anim_tex.frames.len() > max_animation_frames {
                        max_animation_frames = anim_tex.frames.len();
                        frame_delays = anim_tex.frame_delays_ms.clone();
                    }
                }
            }
            if !all_textures_found {
                break;
            }
        }

        if !all_textures_found {
            continue;
        }

        let gui_display = model.display.unwrap_or_default()
            .get("gui").cloned().unwrap_or_else(|| DisplayTransform {
                rotation: Some([30.0, 225.0, 0.0]),
                translation: Some([0.0, 0.0, 0.0]),
                scale: Some([0.625, 0.625, 0.625]),
            });

        let rot = gui_display.rotation.unwrap_or([30.0, 225.0, 0.0]);
        let trans = gui_display.translation.unwrap_or([0.0, 0.0, 0.0]);
        let scale = gui_display.scale.unwrap_or([0.625, 0.625, 0.625]);

        let model_matrix = Mat4::from_translation(Vec3::new(trans[0] / 16.0, trans[1] / 16.0, trans[2] / 16.0))
            * Mat4::from_euler(glam::EulerRot::XYZ, rot[0].to_radians(), rot[1].to_radians(), rot[2].to_radians())
            * Mat4::from_scale(Vec3::new(scale[0], scale[1], scale[2]));

        let view = Mat4::look_at_rh(Vec3::new(0.0, 0.0, 5.0), Vec3::ZERO, Vec3::Y);
        let view_model = view * model_matrix;

        let mut min_bound = Vec2::splat(f32::MAX);
        let mut max_bound = Vec2::splat(f32::MIN);

        for el in &elements {
            let min_el = Vec3::from(el.from) / 16.0 - 0.5;
            let max_el = Vec3::from(el.to) / 16.0 - 0.5;
            for i in 0..8 {
                let x = if i & 1 == 0 { min_el.x } else { max_el.x };
                let y = if i & 2 == 0 { min_el.y } else { max_el.y };
                let z = if i & 4 == 0 { min_el.z } else { max_el.z };
                
                let v_pos = view_model * Vec4::new(x, y, z, 1.0);
                min_bound = min_bound.min(Vec2::new(v_pos.x, v_pos.y));
                max_bound = max_bound.max(Vec2::new(v_pos.x, v_pos.y));
            }
        }

        let center = (min_bound + max_bound) * 0.5;
        let size = max_bound - min_bound;
        let half_size = size.x.max(size.y) / 2.0;

        let proj = Mat4::orthographic_rh(
            center.x - half_size, center.x + half_size,
            center.y - half_size, center.y + half_size,
            0.01, 100.0,
        );
        let mvp = proj * view_model;

        let out_path = format!("{}{}.webp", save_path, target_block);

        let success = if max_animation_frames <= 1 {
            let single_frame = render_single_frame(&elements, &textures, &global_texture_cache, mvp, 0, render_size, use_aa);
            single_frame.save(&out_path).is_ok()
        } else {
            let mut rendered_frames = Vec::new();
            for f in 0..max_animation_frames {
                let frame = render_single_frame(&elements, &textures, &global_texture_cache, mvp, f, render_size, use_aa);
                rendered_frames.push(frame);
            }
            save_as_animated_webp(&out_path, &rendered_frames, &frame_delays, render_size as u32).is_ok()
        };

        if success {
            created_count += 1;
            print!("\rImages created: {}", created_count);
            let _ = io::stdout().flush();
        }
    }

    println!();
    println!(
        "Successfully generated {} images out of {} models!",
        created_count,
        all_models.len()
    );
    println!("Total execution time: {:?}", time_start.elapsed());
}