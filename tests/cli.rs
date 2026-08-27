// flow-texpack: A program that will allow you to generate texture atlas.
// public domain License

use assert_cmd::cargo::*;
use std::path::PathBuf;

use flow_texpack::{Texture, exists_file, remove_dir_all, remove_file};

fn verify_texture_size(image_path: &PathBuf, width: u32, height: u32) {
    let mut t1 = Texture::default();
    t1.load(image_path, false, false, false, 0, 1024);
    assert_eq!(t1.width, width);
    assert_eq!(t1.height, height);
}

fn validate_descriptor_json(json_path: &PathBuf) {
    let contents = std::fs::read_to_string(json_path).unwrap();
    let mut lines = contents.lines();

    assert_eq!(Some("{"), lines.next());
    assert_eq!(Some("\t\"ImageAtlas\":"), lines.next());
    assert_eq!(Some("\t{"), lines.next());
    assert_eq!(Some("\t\t\"info\":"), lines.next());
    assert_eq!(Some("\t\t{"), lines.next());
    assert_eq!(Some("\t\t\t\"numberOfAtlasImages\": 1,"), lines.next());
    assert_eq!(
        Some("\t\t\t\"generatedWith\": \"https://git.luflow.net/hfsoulz/flow-texpack.git\""),
        lines.next()
    );
    assert_eq!(Some("\t\t},"), lines.next());
    assert_eq!(Some("\t\t\"AtlasImage\":"), lines.next());
    assert_eq!(Some("\t\t["), lines.next());

    // files part:
    assert_eq!(Some("\t\t\t{"), lines.next());
    assert_eq!(Some("\t\t\t\t\"n\": \"atlas0.png\","), lines.next());
    assert_eq!(Some("\t\t\t\t\"numImages\": 2,"), lines.next());
    assert_eq!(Some("\t\t\t\t\"width\": 64,"), lines.next());
    assert_eq!(Some("\t\t\t\t\"height\": 64,"), lines.next());
    assert_eq!(Some("\t\t\t\t\"generateMipMaps\": 1,"), lines.next());
    assert_eq!(Some("\t\t\t\t\"img\":"), lines.next());
    assert_eq!(Some("\t\t\t\t["), lines.next());
    assert_eq!(Some("\t\t\t\t\t{"), lines.next());
    assert_eq!(
        Some(
            "\t\t\t\t\t\t\"n\": \"white_32x32.png\", \"x\": 0, \"y\": 0, \"w\": 32, \"h\": 32, \"trimmed\": 0, \"rotated\": 0, \"fx\": 0, \"fy\": 0, \"fw\": 32, \"fh\": 32"
        ),
        lines.next()
    );
    assert_eq!(Some("\t\t\t\t\t},"), lines.next());
    assert_eq!(Some("\t\t\t\t\t{"), lines.next());
    assert_eq!(
        Some(
            "\t\t\t\t\t\t\"n\": \"white_32x16.png\", \"x\": 0, \"y\": 36, \"w\": 32, \"h\": 16, \"trimmed\": 0, \"rotated\": 0, \"fx\": 0, \"fy\": 0, \"fw\": 32, \"fh\": 16"
        ),
        lines.next()
    );
    assert_eq!(Some("\t\t\t\t\t}"), lines.next());
    assert_eq!(Some("\t\t\t\t]"), lines.next());
    assert_eq!(Some("\t\t\t}"), lines.next());

    assert_eq!(Some("\t\t]"), lines.next());
    assert_eq!(Some("\t}"), lines.next());
    assert_eq!(Some("}"), lines.next());
}

fn validate_descriptor_txt(txt_path: &PathBuf, with_header: bool) {
    let contents = std::fs::read_to_string(txt_path).unwrap();
    let mut lines = contents.lines();

    if with_header {
        assert_eq!(Some("/*"), lines.next());
        assert_eq!(
            Some("\t ************************************************"),
            lines.next()
        );
        assert_eq!(
            Some("\t * Generated with: https://git.luflow.net/hfsoulz/flow-texpack.git"),
            lines.next()
        );
        assert_eq!(
            Some("\t ************************************************"),
            lines.next()
        );
        assert_eq!(Some(""), lines.next());
        assert_eq!(
            Some("\t ************************************************"),
            lines.next()
        );
        assert_eq!(Some("\t * Format description:"), lines.next());
        assert_eq!(
            Some("\t ************************************************"),
            lines.next()
        );
        assert_eq!(Some("\t [info]"), lines.next());
        assert_eq!(Some("\t numberOfAtlasImages,generatedWith"), lines.next());
        assert_eq!(Some(""), lines.next());
        assert_eq!(
            Some("\t [AtlasImage (repeated numberOfAtlasImages)]"),
            lines.next()
        );
        assert_eq!(
            Some(
                "\t atlasImageName,numberOfImages,atlasImageWidth,atlasImageHeight,generateMipMaps"
            ),
            lines.next()
        );
        assert_eq!(Some(""), lines.next());
        assert_eq!(Some("\t [Image (repeated numberOfImages)]"), lines.next());
        assert_eq!(
            Some(
                "\t name,x,y,w,h,trimmed,rotated,fx,fy,fw,fh (NOTE: fx,fy,fw,fh valid if trimmed==1)"
            ),
            lines.next()
        );
        assert_eq!(Some(""), lines.next());
        assert_eq!(Some("\t Text format example:"), lines.next());
        assert_eq!(Some("\t [info]"), lines.next());
        assert_eq!(Some("\t [AtlasImage]"), lines.next());
        assert_eq!(Some("\t [Image]"), lines.next());
        assert_eq!(Some("\t [Image]"), lines.next());
        assert_eq!(Some("\t ..."), lines.next());
        assert_eq!(Some("\t [AtlasImage]"), lines.next());
        assert_eq!(Some("\t [Image]"), lines.next());
        assert_eq!(Some("\t [Image]"), lines.next());
        assert_eq!(Some("\t ..."), lines.next());
        assert_eq!(Some("*/@"), lines.next());
    }

    assert_eq!(
        Some("1,https://git.luflow.net/hfsoulz/flow-texpack.git"),
        lines.next()
    );
    assert_eq!(Some("atlas0.png,2,64,64,1"), lines.next());
    assert_eq!(
        Some("white_32x32.png,0,0,32,32,0,0,0,0,32,32"),
        lines.next()
    );
    assert_eq!(
        Some("white_32x16.png,0,36,32,16,0,0,0,0,32,16"),
        lines.next()
    );
}

fn get_hash_value(hash_path: &PathBuf) -> u64 {
    let result = std::fs::read_to_string(hash_path);

    match result {
        Ok(old_hash_value) => old_hash_value.parse().unwrap(),
        Err(_) => 0,
    }
}

// *************************************************
// Integration tests that should fail:
// *************************************************
#[test]
fn cli_error_no_arguments() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = cargo_bin_cmd!("flow-texpack");

    cmd.assert().failure();

    Ok(())
}

#[test]
fn cli_error_no_input() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = cargo_bin_cmd!("flow-texpack");

    cmd.arg("-i");
    cmd.assert().failure();

    Ok(())
}

#[test]
fn cli_error_no_input_v2() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = cargo_bin_cmd!("flow-texpack");

    cmd.arg("--input");
    cmd.assert().failure();

    Ok(())
}

#[test]
fn cli_error_no_input_v3() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = cargo_bin_cmd!("flow-texpack");

    cmd.arg("-i").arg("test_data2");
    cmd.assert().failure();

    Ok(())
}

#[test]
fn cli_error_no_input_file() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = cargo_bin_cmd!("flow-texpack");

    cmd.arg("--input-file");
    cmd.assert().failure();

    Ok(())
}

#[test]
fn cli_error_no_input_file_v2() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = cargo_bin_cmd!("flow-texpack");

    cmd.arg("--input-file").arg("test_data/input_not_found.txt");
    cmd.assert().failure();

    Ok(())
}

#[test]
fn cli_error_no_exclude_file() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = cargo_bin_cmd!("flow-texpack");

    cmd.arg("-i").arg("test_data").arg("--exclude-file");
    cmd.assert().failure();

    Ok(())
}

#[test]
fn cli_error_no_exclude_file_v2() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = cargo_bin_cmd!("flow-texpack");

    cmd.arg("-i")
        .arg("test_data")
        .arg("--exclude-file")
        .arg("test_data/exclude_not_found.txt");
    cmd.assert().failure();

    Ok(())
}

#[test]
fn cli_error_no_output() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = cargo_bin_cmd!("flow-texpack");

    cmd.arg("-i").arg("test_data").arg("-o");
    cmd.assert().failure();

    Ok(())
}

#[test]
fn cli_error_no_output_v2() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = cargo_bin_cmd!("flow-texpack");

    cmd.arg("-i").arg("test_data").arg("--output");
    cmd.assert().failure();

    Ok(())
}

#[test]
fn cli_error_invalid_atlas_descriptor() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = cargo_bin_cmd!("flow-texpack");

    cmd.arg("-i")
        .arg("test_data")
        .arg("-o")
        .arg("test_data/out/atlas")
        .arg("--atlas-descriptor")
        .arg("invalid");
    cmd.assert().failure();

    Ok(())
}

#[test]
fn cli_error_invalid_atlas_image() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = cargo_bin_cmd!("flow-texpack");

    cmd.arg("-i")
        .arg("test_data")
        .arg("-o")
        .arg("test_data/out/atlas")
        .arg("--atlas-image")
        .arg("invalid");
    cmd.assert().failure();

    Ok(())
}

#[test]
fn cli_error_invalid_atlas_size() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = cargo_bin_cmd!("flow-texpack");

    cmd.arg("-i")
        .arg("test_data")
        .arg("-o")
        .arg("test_data/out/atlas")
        .arg("--atlas-size")
        .arg("invalid");
    cmd.assert().failure();

    Ok(())
}

#[test]
fn cli_error_invalid_load_filter() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = cargo_bin_cmd!("flow-texpack");

    cmd.arg("-i")
        .arg("test_data")
        .arg("-o")
        .arg("test_data/out/atlas")
        .arg("--load-filter")
        .arg("invalid");
    cmd.assert().failure();

    Ok(())
}

#[test]
fn cli_error_invalid_max_atlases() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = cargo_bin_cmd!("flow-texpack");

    cmd.arg("-i")
        .arg("test_data")
        .arg("-o")
        .arg("test_data/out/atlas")
        .arg("--max-atlases")
        .arg("4097");
    cmd.assert().failure();

    Ok(())
}

#[test]
fn cli_error_invalid_padding() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = cargo_bin_cmd!("flow-texpack");

    cmd.arg("-i")
        .arg("test_data")
        .arg("-o")
        .arg("test_data/out/atlas")
        .arg("-p")
        .arg("17");
    cmd.assert().failure();

    Ok(())
}

#[test]
fn cli_error_invalid_padding_v2() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = cargo_bin_cmd!("flow-texpack");

    cmd.arg("-i")
        .arg("test_data")
        .arg("-o")
        .arg("test_data/out/atlas")
        .arg("--pad")
        .arg("17");
    cmd.assert().failure();

    Ok(())
}

#[test]
fn cli_error_invalid_rect_heuristic() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = cargo_bin_cmd!("flow-texpack");

    cmd.arg("-i")
        .arg("test_data")
        .arg("-o")
        .arg("test_data/out/atlas")
        .arg("--rect-heuristic")
        .arg("invalid");
    cmd.assert().failure();

    Ok(())
}

// *************************************************
// Integration tests that should succeed:
// *************************************************
#[test]
fn cli_success_input_defaults() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = cargo_bin_cmd!("flow-texpack");
    let out_file_path = "test_data/cli_success_input_defaults/atlas";
    let parent_path = PathBuf::from(PathBuf::from(out_file_path).parent().unwrap());

    cmd.arg("-i")
        .arg("test_data/white_32x32.png")
        .arg("-o")
        .arg(out_file_path);
    cmd.assert().success();

    let hash_path = PathBuf::from(format!("{}.hash", out_file_path));
    let json_path = PathBuf::from(format!("{}.json", out_file_path));
    let image_path = PathBuf::from(format!("{}0.png", out_file_path));

    assert!(exists_file(&hash_path));
    assert!(exists_file(&json_path));
    assert!(exists_file(&image_path));
    assert!(get_hash_value(&hash_path) > 0);

    verify_texture_size(&image_path, 64, 64);

    remove_dir_all(&parent_path);

    assert!(!exists_file(&hash_path));
    assert!(!exists_file(&json_path));
    assert!(!exists_file(&image_path));

    Ok(())
}

#[test]
fn cli_success_input_defaults_v2() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = cargo_bin_cmd!("flow-texpack");
    let out_file_path = "test_data/cli_success_input_defaults_v2/atlas";
    let parent_path = PathBuf::from(PathBuf::from(out_file_path).parent().unwrap());

    cmd.arg("--input")
        .arg("test_data/white_32x32.png")
        .arg("-o")
        .arg(out_file_path);
    cmd.assert().success();

    let hash_path = PathBuf::from(format!("{}.hash", out_file_path));
    let json_path = PathBuf::from(format!("{}.json", out_file_path));
    let image_path = PathBuf::from(format!("{}0.png", out_file_path));

    assert!(exists_file(&hash_path));
    assert!(exists_file(&json_path));
    assert!(exists_file(&image_path));
    assert!(get_hash_value(&hash_path) > 0);

    verify_texture_size(&image_path, 64, 64);

    remove_dir_all(&parent_path);

    assert!(!exists_file(&hash_path));
    assert!(!exists_file(&json_path));
    assert!(!exists_file(&image_path));

    Ok(())
}

#[test]
fn cli_success_input_with_exclude_defaults() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = cargo_bin_cmd!("flow-texpack");
    let out_file_path = "test_data/cli_success_input_with_exclude_defaults/atlas";
    let parent_path = PathBuf::from(PathBuf::from(out_file_path).parent().unwrap());

    cmd.arg("-i")
        .arg("test_data/white_32x32.png")
        .arg("test_data/blue_32x32.png")
        .arg("-e")
        .arg("test_data/white_32x32.png")
        .arg("-o")
        .arg(out_file_path);
    cmd.assert().success();

    let hash_path = PathBuf::from(format!("{}.hash", out_file_path));
    let json_path = PathBuf::from(format!("{}.json", out_file_path));
    let image_path = PathBuf::from(format!("{}0.png", out_file_path));

    assert!(exists_file(&hash_path));
    assert!(exists_file(&json_path));
    assert!(exists_file(&image_path));
    assert!(get_hash_value(&hash_path) > 0);

    verify_texture_size(&image_path, 64, 64);

    remove_dir_all(&parent_path);

    assert!(!exists_file(&hash_path));
    assert!(!exists_file(&json_path));
    assert!(!exists_file(&image_path));

    Ok(())
}

#[test]
fn cli_success_input_with_exclude_defaults_v2() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = cargo_bin_cmd!("flow-texpack");
    let out_file_path = "test_data/cli_success_input_with_exclude_defaults_v2/atlas";
    let parent_path = PathBuf::from(PathBuf::from(out_file_path).parent().unwrap());

    cmd.arg("-i")
        .arg("test_data/white_32x32.png")
        .arg("test_data/blue_32x32.png")
        .arg("--exclude")
        .arg("test_data/white_32x32.png")
        .arg("-o")
        .arg(out_file_path);
    cmd.assert().success();

    let hash_path = PathBuf::from(format!("{}.hash", out_file_path));
    let json_path = PathBuf::from(format!("{}.json", out_file_path));
    let image_path = PathBuf::from(format!("{}0.png", out_file_path));

    assert!(exists_file(&hash_path));
    assert!(exists_file(&json_path));
    assert!(exists_file(&image_path));
    assert!(get_hash_value(&hash_path) > 0);

    verify_texture_size(&image_path, 64, 64);

    remove_dir_all(&parent_path);

    assert!(!exists_file(&hash_path));
    assert!(!exists_file(&json_path));
    assert!(!exists_file(&image_path));

    Ok(())
}

#[test]
fn cli_success_input_defaults_verbose() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = cargo_bin_cmd!("flow-texpack");
    let out_file_path = "test_data/cli_success_input_defaults_verbose/atlas";
    let parent_path = PathBuf::from(PathBuf::from(out_file_path).parent().unwrap());

    cmd.arg("-i")
        .arg("test_data/white_32x32.png")
        .arg("-o")
        .arg(out_file_path)
        .arg("-v");
    cmd.assert().success();

    let hash_path = PathBuf::from(format!("{}.hash", out_file_path));
    let json_path = PathBuf::from(format!("{}.json", out_file_path));
    let image_path = PathBuf::from(format!("{}0.png", out_file_path));

    assert!(exists_file(&hash_path));
    assert!(exists_file(&json_path));
    assert!(exists_file(&image_path));
    assert!(get_hash_value(&hash_path) > 0);

    verify_texture_size(&image_path, 64, 64);

    remove_dir_all(&parent_path);

    assert!(!exists_file(&hash_path));
    assert!(!exists_file(&json_path));
    assert!(!exists_file(&image_path));

    Ok(())
}

#[test]
fn cli_success_input_defaults_verbose_v2() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = cargo_bin_cmd!("flow-texpack");
    let out_file_path = "test_data/cli_success_input_defaults_verbose_v2/atlas";
    let parent_path = PathBuf::from(PathBuf::from(out_file_path).parent().unwrap());

    cmd.arg("-i")
        .arg("test_data/white_32x32.png")
        .arg("-o")
        .arg(out_file_path)
        .arg("--verbose");
    cmd.assert().success();

    let hash_path = PathBuf::from(format!("{}.hash", out_file_path));
    let json_path = PathBuf::from(format!("{}.json", out_file_path));
    let image_path = PathBuf::from(format!("{}0.png", out_file_path));

    assert!(exists_file(&hash_path));
    assert!(exists_file(&json_path));
    assert!(exists_file(&image_path));
    assert!(get_hash_value(&hash_path) > 0);

    verify_texture_size(&image_path, 64, 64);

    remove_dir_all(&parent_path);

    assert!(!exists_file(&hash_path));
    assert!(!exists_file(&json_path));
    assert!(!exists_file(&image_path));

    Ok(())
}

#[test]
fn cli_success_input_defaults_log() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = cargo_bin_cmd!("flow-texpack");
    let out_file_path = "test_data/cli_success_input_defaults_log/atlas";
    let parent_path = PathBuf::from(PathBuf::from(out_file_path).parent().unwrap());

    cmd.arg("-i")
        .arg("test_data/white_32x32.png")
        .arg("-o")
        .arg(out_file_path)
        .arg("-l");
    cmd.assert().success();

    let hash_path = PathBuf::from(format!("{}.hash", out_file_path));
    let json_path = PathBuf::from(format!("{}.json", out_file_path));
    let image_path = PathBuf::from(format!("{}0.png", out_file_path));
    let log_path = PathBuf::from("flow-texpack.log");

    assert!(exists_file(&hash_path));
    assert!(exists_file(&json_path));
    assert!(exists_file(&image_path));
    assert!(get_hash_value(&hash_path) > 0);
    assert!(exists_file(&log_path));

    verify_texture_size(&image_path, 64, 64);

    remove_dir_all(&parent_path);
    remove_file(&log_path);

    assert!(!exists_file(&hash_path));
    assert!(!exists_file(&json_path));
    assert!(!exists_file(&image_path));
    assert!(!exists_file(&log_path));

    Ok(())
}

#[test]
fn cli_success_input_defaults_version() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = cargo_bin_cmd!("flow-texpack");

    cmd.arg("-V");
    cmd.assert().success();

    Ok(())
}

#[test]
fn cli_success_input_defaults_version_v2() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = cargo_bin_cmd!("flow-texpack");

    cmd.arg("--version");
    cmd.assert().success();

    Ok(())
}

#[test]
fn cli_success_input_defaults_help() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = cargo_bin_cmd!("flow-texpack");

    cmd.arg("-h");
    cmd.assert().success();

    Ok(())
}

#[test]
fn cli_success_input_defaults_help_v2() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = cargo_bin_cmd!("flow-texpack");

    cmd.arg("--help");
    cmd.assert().success();

    Ok(())
}

#[test]
fn cli_success_input_defaults_force() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = cargo_bin_cmd!("flow-texpack");
    let out_file_path = "test_data/cli_success_input_defaults_force/atlas";
    let parent_path = PathBuf::from(PathBuf::from(out_file_path).parent().unwrap());

    cmd.arg("-i")
        .arg("test_data/white_32x32.png")
        .arg("-o")
        .arg(out_file_path)
        .arg("-f");
    cmd.assert().success();

    let hash_path = PathBuf::from(format!("{}.hash", out_file_path));
    let json_path = PathBuf::from(format!("{}.json", out_file_path));
    let image_path = PathBuf::from(format!("{}0.png", out_file_path));

    assert!(exists_file(&hash_path));
    assert!(exists_file(&json_path));
    assert!(exists_file(&image_path));
    assert!(get_hash_value(&hash_path) == 0);

    verify_texture_size(&image_path, 64, 64);

    remove_dir_all(&parent_path);

    assert!(!exists_file(&hash_path));
    assert!(!exists_file(&json_path));
    assert!(!exists_file(&image_path));

    Ok(())
}

#[test]
fn cli_success_input_defaults_force_v2() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = cargo_bin_cmd!("flow-texpack");
    let out_file_path = "test_data/cli_success_input_defaults_force_v2/atlas";
    let parent_path = PathBuf::from(PathBuf::from(out_file_path).parent().unwrap());

    cmd.arg("-i")
        .arg("test_data/white_32x32.png")
        .arg("-o")
        .arg(out_file_path)
        .arg("--force");
    cmd.assert().success();

    let hash_path = PathBuf::from(format!("{}.hash", out_file_path));
    let json_path = PathBuf::from(format!("{}.json", out_file_path));
    let image_path = PathBuf::from(format!("{}0.png", out_file_path));

    assert!(exists_file(&hash_path));
    assert!(exists_file(&json_path));
    assert!(exists_file(&image_path));
    assert!(get_hash_value(&hash_path) == 0);

    verify_texture_size(&image_path, 64, 64);

    remove_dir_all(&parent_path);

    assert!(!exists_file(&hash_path));
    assert!(!exists_file(&json_path));
    assert!(!exists_file(&image_path));

    Ok(())
}

#[test]
fn cli_success_input_defaults_force_square() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = cargo_bin_cmd!("flow-texpack");
    let out_file_path = "test_data/cli_success_input_defaults_force_square/atlas";
    let parent_path = PathBuf::from(PathBuf::from(out_file_path).parent().unwrap());

    cmd.arg("-i")
        .arg("test_data/white_32x16.png")
        .arg("-o")
        .arg(out_file_path)
        .arg("--force-square");
    cmd.assert().success();

    let hash_path = PathBuf::from(format!("{}.hash", out_file_path));
    let json_path = PathBuf::from(format!("{}.json", out_file_path));
    let image_path = PathBuf::from(format!("{}0.png", out_file_path));

    assert!(exists_file(&hash_path));
    assert!(exists_file(&json_path));
    assert!(exists_file(&image_path));
    assert!(get_hash_value(&hash_path) > 0);

    verify_texture_size(&image_path, 1024, 1024);

    remove_dir_all(&parent_path);

    assert!(!exists_file(&hash_path));
    assert!(!exists_file(&json_path));
    assert!(!exists_file(&image_path));

    Ok(())
}

#[test]
fn cli_success_input_defaults_atlas_image_png() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = cargo_bin_cmd!("flow-texpack");
    let out_file_path = "test_data/cli_success_input_defaults_atlas_image_png/atlas";
    let parent_path = PathBuf::from(PathBuf::from(out_file_path).parent().unwrap());

    cmd.arg("-i")
        .arg("test_data/green_32x32.png")
        .arg("-o")
        .arg(out_file_path)
        .arg("--atlas-image")
        .arg("png");
    cmd.assert().success();

    let hash_path = PathBuf::from(format!("{}.hash", out_file_path));
    let json_path = PathBuf::from(format!("{}.json", out_file_path));
    let image_path = PathBuf::from(format!("{}0.png", out_file_path));

    assert!(exists_file(&hash_path));
    assert!(exists_file(&json_path));
    assert!(exists_file(&image_path));
    assert!(get_hash_value(&hash_path) > 0);

    verify_texture_size(&image_path, 64, 64);

    remove_dir_all(&parent_path);

    assert!(!exists_file(&hash_path));
    assert!(!exists_file(&json_path));
    assert!(!exists_file(&image_path));

    Ok(())
}

#[test]
fn cli_success_input_defaults_atlas_image_tga() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = cargo_bin_cmd!("flow-texpack");
    let out_file_path = "test_data/cli_success_input_defaults_atlas_image_tga/atlas";
    let parent_path = PathBuf::from(PathBuf::from(out_file_path).parent().unwrap());

    cmd.arg("-i")
        .arg("test_data/green_32x32.png")
        .arg("-o")
        .arg(out_file_path)
        .arg("--atlas-image")
        .arg("tga");
    cmd.assert().success();

    let hash_path = PathBuf::from(format!("{}.hash", out_file_path));
    let json_path = PathBuf::from(format!("{}.json", out_file_path));
    let image_path = PathBuf::from(format!("{}0.tga", out_file_path));

    assert!(exists_file(&hash_path));
    assert!(exists_file(&json_path));
    assert!(exists_file(&image_path));
    assert!(get_hash_value(&hash_path) > 0);

    verify_texture_size(&image_path, 64, 64);

    remove_dir_all(&parent_path);

    assert!(!exists_file(&hash_path));
    assert!(!exists_file(&json_path));
    assert!(!exists_file(&image_path));

    Ok(())
}

#[test]
fn cli_success_input_defaults_atlas_image_tiff() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = cargo_bin_cmd!("flow-texpack");
    let out_file_path = "test_data/cli_success_input_defaults_atlas_image_tiff/atlas";
    let parent_path = PathBuf::from(PathBuf::from(out_file_path).parent().unwrap());

    cmd.arg("-i")
        .arg("test_data/green_32x32.png")
        .arg("-o")
        .arg(out_file_path)
        .arg("--atlas-image")
        .arg("tiff");
    cmd.assert().success();

    let hash_path = PathBuf::from(format!("{}.hash", out_file_path));
    let json_path = PathBuf::from(format!("{}.json", out_file_path));
    let image_path = PathBuf::from(format!("{}0.tiff", out_file_path));

    assert!(exists_file(&hash_path));
    assert!(exists_file(&json_path));
    assert!(exists_file(&image_path));
    assert!(get_hash_value(&hash_path) > 0);

    verify_texture_size(&image_path, 64, 64);

    remove_dir_all(&parent_path);

    assert!(!exists_file(&hash_path));
    assert!(!exists_file(&json_path));
    assert!(!exists_file(&image_path));

    Ok(())
}

#[test]
fn cli_success_input_defaults_atlas_image_webp() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = cargo_bin_cmd!("flow-texpack");
    let out_file_path = "test_data/cli_success_input_defaults_atlas_image_webp/atlas";
    let parent_path = PathBuf::from(PathBuf::from(out_file_path).parent().unwrap());

    cmd.arg("-i")
        .arg("test_data/green_32x32.png")
        .arg("-o")
        .arg(out_file_path)
        .arg("--atlas-image")
        .arg("webp");
    cmd.assert().success();

    let hash_path = PathBuf::from(format!("{}.hash", out_file_path));
    let json_path = PathBuf::from(format!("{}.json", out_file_path));
    let image_path = PathBuf::from(format!("{}0.webp", out_file_path));

    assert!(exists_file(&hash_path));
    assert!(exists_file(&json_path));
    assert!(exists_file(&image_path));
    assert!(get_hash_value(&hash_path) > 0);

    verify_texture_size(&image_path, 64, 64);

    remove_dir_all(&parent_path);

    assert!(!exists_file(&hash_path));
    assert!(!exists_file(&json_path));
    assert!(!exists_file(&image_path));

    Ok(())
}

#[test]
fn cli_success_input_defaults_atlas_descriptor_json() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = cargo_bin_cmd!("flow-texpack");
    let out_file_path = "test_data/cli_success_input_defaults_atlas_descriptor_json/atlas";
    let parent_path = PathBuf::from(PathBuf::from(out_file_path).parent().unwrap());

    cmd.arg("-i")
        .arg("test_data/green_32x32.png")
        .arg("-o")
        .arg(out_file_path)
        .arg("--atlas-descriptor")
        .arg("json");
    cmd.assert().success();

    let hash_path = PathBuf::from(format!("{}.hash", out_file_path));
    let json_path = PathBuf::from(format!("{}.json", out_file_path));
    let image_path = PathBuf::from(format!("{}0.png", out_file_path));

    assert!(exists_file(&hash_path));
    assert!(exists_file(&json_path));
    assert!(exists_file(&image_path));
    assert!(get_hash_value(&hash_path) > 0);

    verify_texture_size(&image_path, 64, 64);

    remove_dir_all(&parent_path);

    assert!(!exists_file(&hash_path));
    assert!(!exists_file(&json_path));
    assert!(!exists_file(&image_path));

    Ok(())
}

#[test]
fn cli_success_input_defaults_atlas_descriptor_txt() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = cargo_bin_cmd!("flow-texpack");
    let out_file_path = "test_data/cli_success_input_defaults_atlas_descriptor_txt/atlas";
    let parent_path = PathBuf::from(PathBuf::from(out_file_path).parent().unwrap());

    cmd.arg("-i")
        .arg("test_data/green_32x32.png")
        .arg("-o")
        .arg(out_file_path)
        .arg("--atlas-descriptor")
        .arg("txt");
    cmd.assert().success();

    let hash_path = PathBuf::from(format!("{}.hash", out_file_path));
    let json_path = PathBuf::from(format!("{}.txt", out_file_path));
    let image_path = PathBuf::from(format!("{}0.png", out_file_path));

    assert!(exists_file(&hash_path));
    assert!(exists_file(&json_path));
    assert!(exists_file(&image_path));
    assert!(get_hash_value(&hash_path) > 0);

    verify_texture_size(&image_path, 64, 64);

    remove_dir_all(&parent_path);

    assert!(!exists_file(&hash_path));
    assert!(!exists_file(&json_path));
    assert!(!exists_file(&image_path));

    Ok(())
}

#[test]
fn cli_success_input_defaults_atlas_descriptor_txt_desc() -> Result<(), Box<dyn std::error::Error>>
{
    let mut cmd = cargo_bin_cmd!("flow-texpack");
    let out_file_path = "test_data/cli_success_input_defaults_atlas_descriptor_txt_desc/atlas";
    let parent_path = PathBuf::from(PathBuf::from(out_file_path).parent().unwrap());

    cmd.arg("-i")
        .arg("test_data/green_32x32.png")
        .arg("-o")
        .arg(out_file_path)
        .arg("--atlas-descriptor")
        .arg("txt");
    cmd.assert().success();

    let hash_path = PathBuf::from(format!("{}.hash", out_file_path));
    let json_path = PathBuf::from(format!("{}.txt", out_file_path));
    let image_path = PathBuf::from(format!("{}0.png", out_file_path));

    assert!(exists_file(&hash_path));
    assert!(exists_file(&json_path));
    assert!(exists_file(&image_path));
    assert!(get_hash_value(&hash_path) > 0);

    verify_texture_size(&image_path, 64, 64);

    remove_dir_all(&parent_path);

    assert!(!exists_file(&hash_path));
    assert!(!exists_file(&json_path));
    assert!(!exists_file(&image_path));

    Ok(())
}

#[test]
fn cli_success_input_defaults_atlas_size_64() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = cargo_bin_cmd!("flow-texpack");
    let out_file_path = "test_data/cli_success_input_defaults_atlas_size_64/atlas";
    let parent_path = PathBuf::from(PathBuf::from(out_file_path).parent().unwrap());

    cmd.arg("-i")
        .arg("test_data/white_32x32.png")
        .arg("-o")
        .arg(out_file_path)
        .arg("--atlas-size")
        .arg("pot64")
        .arg("--force-square");
    cmd.assert().success();

    let hash_path = PathBuf::from(format!("{}.hash", out_file_path));
    let json_path = PathBuf::from(format!("{}.json", out_file_path));
    let image_path = PathBuf::from(format!("{}0.png", out_file_path));

    assert!(exists_file(&hash_path));
    assert!(exists_file(&json_path));
    assert!(exists_file(&image_path));
    assert!(get_hash_value(&hash_path) > 0);

    verify_texture_size(&image_path, 64, 64);

    remove_dir_all(&parent_path);

    assert!(!exists_file(&hash_path));
    assert!(!exists_file(&json_path));
    assert!(!exists_file(&image_path));

    Ok(())
}

#[test]
fn cli_success_input_defaults_atlas_size_128() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = cargo_bin_cmd!("flow-texpack");
    let out_file_path = "test_data/cli_success_input_defaults_atlas_size_128/atlas";
    let parent_path = PathBuf::from(PathBuf::from(out_file_path).parent().unwrap());

    cmd.arg("-i")
        .arg("test_data/white_32x32.png")
        .arg("-o")
        .arg(out_file_path)
        .arg("--atlas-size")
        .arg("pot128")
        .arg("--force-square");
    cmd.assert().success();

    let hash_path = PathBuf::from(format!("{}.hash", out_file_path));
    let json_path = PathBuf::from(format!("{}.json", out_file_path));
    let image_path = PathBuf::from(format!("{}0.png", out_file_path));

    assert!(exists_file(&hash_path));
    assert!(exists_file(&json_path));
    assert!(exists_file(&image_path));
    assert!(get_hash_value(&hash_path) > 0);

    verify_texture_size(&image_path, 128, 128);

    remove_dir_all(&parent_path);

    assert!(!exists_file(&hash_path));
    assert!(!exists_file(&json_path));
    assert!(!exists_file(&image_path));

    Ok(())
}

#[test]
fn cli_success_input_defaults_atlas_size_256() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = cargo_bin_cmd!("flow-texpack");
    let out_file_path = "test_data/cli_success_input_defaults_atlas_size_256/atlas";
    let parent_path = PathBuf::from(PathBuf::from(out_file_path).parent().unwrap());

    cmd.arg("-i")
        .arg("test_data/white_32x32.png")
        .arg("-o")
        .arg(out_file_path)
        .arg("--atlas-size")
        .arg("pot256")
        .arg("--force-square");
    cmd.assert().success();

    let hash_path = PathBuf::from(format!("{}.hash", out_file_path));
    let json_path = PathBuf::from(format!("{}.json", out_file_path));
    let image_path = PathBuf::from(format!("{}0.png", out_file_path));

    assert!(exists_file(&hash_path));
    assert!(exists_file(&json_path));
    assert!(exists_file(&image_path));
    assert!(get_hash_value(&hash_path) > 0);

    verify_texture_size(&image_path, 256, 256);

    remove_dir_all(&parent_path);

    assert!(!exists_file(&hash_path));
    assert!(!exists_file(&json_path));
    assert!(!exists_file(&image_path));

    Ok(())
}

#[test]
fn cli_success_input_defaults_atlas_size_512() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = cargo_bin_cmd!("flow-texpack");
    let out_file_path = "test_data/cli_success_input_defaults_atlas_size_512/atlas";
    let parent_path = PathBuf::from(PathBuf::from(out_file_path).parent().unwrap());

    cmd.arg("-i")
        .arg("test_data/white_32x32.png")
        .arg("-o")
        .arg(out_file_path)
        .arg("--atlas-size")
        .arg("pot512")
        .arg("--force-square");
    cmd.assert().success();

    let hash_path = PathBuf::from(format!("{}.hash", out_file_path));
    let json_path = PathBuf::from(format!("{}.json", out_file_path));
    let image_path = PathBuf::from(format!("{}0.png", out_file_path));

    assert!(exists_file(&hash_path));
    assert!(exists_file(&json_path));
    assert!(exists_file(&image_path));
    assert!(get_hash_value(&hash_path) > 0);

    verify_texture_size(&image_path, 512, 512);

    remove_dir_all(&parent_path);

    assert!(!exists_file(&hash_path));
    assert!(!exists_file(&json_path));
    assert!(!exists_file(&image_path));

    Ok(())
}

#[test]
fn cli_success_input_defaults_atlas_size_1024() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = cargo_bin_cmd!("flow-texpack");
    let out_file_path = "test_data/cli_success_input_defaults_atlas_size_1024/atlas";
    let parent_path = PathBuf::from(PathBuf::from(out_file_path).parent().unwrap());

    cmd.arg("-i")
        .arg("test_data/white_32x32.png")
        .arg("-o")
        .arg(out_file_path)
        .arg("--atlas-size")
        .arg("pot1024")
        .arg("--force-square");
    cmd.assert().success();

    let hash_path = PathBuf::from(format!("{}.hash", out_file_path));
    let json_path = PathBuf::from(format!("{}.json", out_file_path));
    let image_path = PathBuf::from(format!("{}0.png", out_file_path));

    assert!(exists_file(&hash_path));
    assert!(exists_file(&json_path));
    assert!(exists_file(&image_path));
    assert!(get_hash_value(&hash_path) > 0);

    verify_texture_size(&image_path, 1024, 1024);

    remove_dir_all(&parent_path);

    assert!(!exists_file(&hash_path));
    assert!(!exists_file(&json_path));
    assert!(!exists_file(&image_path));

    Ok(())
}

#[test]
fn cli_success_input_defaults_atlas_size_2048() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = cargo_bin_cmd!("flow-texpack");
    let out_file_path = "test_data/cli_success_input_defaults_atlas_size_2048/atlas";
    let parent_path = PathBuf::from(PathBuf::from(out_file_path).parent().unwrap());

    cmd.arg("-i")
        .arg("test_data/white_32x32.png")
        .arg("-o")
        .arg(out_file_path)
        .arg("--atlas-size")
        .arg("pot2048")
        .arg("--force-square");
    cmd.assert().success();

    let hash_path = PathBuf::from(format!("{}.hash", out_file_path));
    let json_path = PathBuf::from(format!("{}.json", out_file_path));
    let image_path = PathBuf::from(format!("{}0.png", out_file_path));

    assert!(exists_file(&hash_path));
    assert!(exists_file(&json_path));
    assert!(exists_file(&image_path));
    assert!(get_hash_value(&hash_path) > 0);

    verify_texture_size(&image_path, 2048, 2048);

    remove_dir_all(&parent_path);

    assert!(!exists_file(&hash_path));
    assert!(!exists_file(&json_path));
    assert!(!exists_file(&image_path));

    Ok(())
}

#[test]
fn cli_success_input_defaults_atlas_size_4096() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = cargo_bin_cmd!("flow-texpack");
    let out_file_path = "test_data/cli_success_input_defaults_atlas_size_4096/atlas";
    let parent_path = PathBuf::from(PathBuf::from(out_file_path).parent().unwrap());

    cmd.arg("-i")
        .arg("test_data/white_32x32.png")
        .arg("-o")
        .arg(out_file_path)
        .arg("--atlas-size")
        .arg("pot4096")
        .arg("--force-square");
    cmd.assert().success();

    let hash_path = PathBuf::from(format!("{}.hash", out_file_path));
    let json_path = PathBuf::from(format!("{}.json", out_file_path));
    let image_path = PathBuf::from(format!("{}0.png", out_file_path));

    assert!(exists_file(&hash_path));
    assert!(exists_file(&json_path));
    assert!(exists_file(&image_path));
    assert!(get_hash_value(&hash_path) > 0);

    verify_texture_size(&image_path, 4096, 4096);

    remove_dir_all(&parent_path);

    assert!(!exists_file(&hash_path));
    assert!(!exists_file(&json_path));
    assert!(!exists_file(&image_path));

    Ok(())
}

#[test]
fn cli_success_input_defaults_atlas_size_8192() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = cargo_bin_cmd!("flow-texpack");
    let out_file_path = "test_data/cli_success_input_defaults_atlas_size_8192/atlas";
    let parent_path = PathBuf::from(PathBuf::from(out_file_path).parent().unwrap());

    cmd.arg("-i")
        .arg("test_data/white_32x32.png")
        .arg("-o")
        .arg(out_file_path)
        .arg("--atlas-size")
        .arg("pot8192")
        .arg("--force-square");
    cmd.assert().success();

    let hash_path = PathBuf::from(format!("{}.hash", out_file_path));
    let json_path = PathBuf::from(format!("{}.json", out_file_path));
    let image_path = PathBuf::from(format!("{}0.png", out_file_path));

    assert!(exists_file(&hash_path));
    assert!(exists_file(&json_path));
    assert!(exists_file(&image_path));
    assert!(get_hash_value(&hash_path) > 0);

    verify_texture_size(&image_path, 8192, 8192);

    remove_dir_all(&parent_path);

    assert!(!exists_file(&hash_path));
    assert!(!exists_file(&json_path));
    assert!(!exists_file(&image_path));

    Ok(())
}

#[test]
fn cli_success_input_defaults_rect_heuristic_short_side_fit()
-> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = cargo_bin_cmd!("flow-texpack");
    let out_file_path = "test_data/cli_success_input_defaults_rect_heuristic_short_side_fit/atlas";
    let parent_path = PathBuf::from(PathBuf::from(out_file_path).parent().unwrap());

    cmd.arg("-i")
        .arg("test_data/white_32x32.png")
        .arg("test_data/red_32x32.png")
        .arg("test_data/green_32x32.png")
        .arg("test_data/blue_32x32.png")
        .arg("-o")
        .arg(out_file_path)
        .arg("-p")
        .arg("0")
        .arg("--atlas-size")
        .arg("pot64")
        .arg("--rect-heuristic")
        .arg("short-side-fit");
    cmd.assert().success();

    let hash_path = PathBuf::from(format!("{}.hash", out_file_path));
    let json_path = PathBuf::from(format!("{}.json", out_file_path));
    let image_path = PathBuf::from(format!("{}0.png", out_file_path));

    assert!(exists_file(&hash_path));
    assert!(exists_file(&json_path));
    assert!(exists_file(&image_path));
    assert!(get_hash_value(&hash_path) > 0);

    verify_texture_size(&image_path, 64, 64);

    remove_dir_all(&parent_path);

    assert!(!exists_file(&hash_path));
    assert!(!exists_file(&json_path));
    assert!(!exists_file(&image_path));

    Ok(())
}

#[test]
fn cli_success_input_defaults_rect_heuristic_long_side_fit()
-> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = cargo_bin_cmd!("flow-texpack");
    let out_file_path = "test_data/cli_success_input_defaults_rect_heuristic_long_side_fit/atlas";
    let parent_path = PathBuf::from(PathBuf::from(out_file_path).parent().unwrap());

    cmd.arg("-i")
        .arg("test_data/white_32x32.png")
        .arg("test_data/red_32x32.png")
        .arg("test_data/green_32x32.png")
        .arg("test_data/blue_32x32.png")
        .arg("-o")
        .arg(out_file_path)
        .arg("-p")
        .arg("0")
        .arg("--atlas-size")
        .arg("pot64")
        .arg("--rect-heuristic")
        .arg("long-side-fit");
    cmd.assert().success();

    let hash_path = PathBuf::from(format!("{}.hash", out_file_path));
    let json_path = PathBuf::from(format!("{}.json", out_file_path));
    let image_path = PathBuf::from(format!("{}0.png", out_file_path));

    assert!(exists_file(&hash_path));
    assert!(exists_file(&json_path));
    assert!(exists_file(&image_path));
    assert!(get_hash_value(&hash_path) > 0);

    verify_texture_size(&image_path, 64, 64);

    remove_dir_all(&parent_path);

    assert!(!exists_file(&hash_path));
    assert!(!exists_file(&json_path));
    assert!(!exists_file(&image_path));

    Ok(())
}

#[test]
fn cli_success_input_defaults_rect_heuristic_area_fit() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = cargo_bin_cmd!("flow-texpack");
    let out_file_path = "test_data/cli_success_input_defaults_rect_heuristic_area_fit/atlas";
    let parent_path = PathBuf::from(PathBuf::from(out_file_path).parent().unwrap());

    cmd.arg("-i")
        .arg("test_data/white_32x32.png")
        .arg("test_data/red_32x32.png")
        .arg("test_data/green_32x32.png")
        .arg("test_data/blue_32x32.png")
        .arg("-o")
        .arg(out_file_path)
        .arg("-p")
        .arg("0")
        .arg("--atlas-size")
        .arg("pot64")
        .arg("--rect-heuristic")
        .arg("area-fit");
    cmd.assert().success();

    let hash_path = PathBuf::from(format!("{}.hash", out_file_path));
    let json_path = PathBuf::from(format!("{}.json", out_file_path));
    let image_path = PathBuf::from(format!("{}0.png", out_file_path));

    assert!(exists_file(&hash_path));
    assert!(exists_file(&json_path));
    assert!(exists_file(&image_path));
    assert!(get_hash_value(&hash_path) > 0);

    verify_texture_size(&image_path, 64, 64);

    remove_dir_all(&parent_path);

    assert!(!exists_file(&hash_path));
    assert!(!exists_file(&json_path));
    assert!(!exists_file(&image_path));

    Ok(())
}

#[test]
fn cli_success_input_defaults_rect_heuristic_bottom_left() -> Result<(), Box<dyn std::error::Error>>
{
    let mut cmd = cargo_bin_cmd!("flow-texpack");
    let out_file_path = "test_data/cli_success_input_defaults_rect_heuristic_bottom_left/atlas";
    let parent_path = PathBuf::from(PathBuf::from(out_file_path).parent().unwrap());

    cmd.arg("-i")
        .arg("test_data/white_32x32.png")
        .arg("test_data/red_32x32.png")
        .arg("test_data/green_32x32.png")
        .arg("test_data/blue_32x32.png")
        .arg("-o")
        .arg(out_file_path)
        .arg("-p")
        .arg("0")
        .arg("--atlas-size")
        .arg("pot64")
        .arg("--rect-heuristic")
        .arg("bottom-left");
    cmd.assert().success();

    let hash_path = PathBuf::from(format!("{}.hash", out_file_path));
    let json_path = PathBuf::from(format!("{}.json", out_file_path));
    let image_path = PathBuf::from(format!("{}0.png", out_file_path));

    assert!(exists_file(&hash_path));
    assert!(exists_file(&json_path));
    assert!(exists_file(&image_path));
    assert!(get_hash_value(&hash_path) > 0);

    verify_texture_size(&image_path, 64, 64);

    remove_dir_all(&parent_path);

    assert!(!exists_file(&hash_path));
    assert!(!exists_file(&json_path));
    assert!(!exists_file(&image_path));

    Ok(())
}

#[test]
fn cli_success_input_defaults_rect_heuristic_contact_point()
-> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = cargo_bin_cmd!("flow-texpack");
    let out_file_path = "test_data/cli_success_input_defaults_rect_heuristic_contact_point/atlas";
    let parent_path = PathBuf::from(PathBuf::from(out_file_path).parent().unwrap());

    cmd.arg("-i")
        .arg("test_data/white_32x32.png")
        .arg("test_data/red_32x32.png")
        .arg("test_data/green_32x32.png")
        .arg("test_data/blue_32x32.png")
        .arg("-o")
        .arg(out_file_path)
        .arg("-p")
        .arg("0")
        .arg("--atlas-size")
        .arg("pot64")
        .arg("--rect-heuristic")
        .arg("contact-point");
    cmd.assert().success();

    let hash_path = PathBuf::from(format!("{}.hash", out_file_path));
    let json_path = PathBuf::from(format!("{}.json", out_file_path));
    let image_path = PathBuf::from(format!("{}0.png", out_file_path));

    assert!(exists_file(&hash_path));
    assert!(exists_file(&json_path));
    assert!(exists_file(&image_path));
    assert!(get_hash_value(&hash_path) > 0);

    verify_texture_size(&image_path, 64, 64);

    remove_dir_all(&parent_path);

    assert!(!exists_file(&hash_path));
    assert!(!exists_file(&json_path));
    assert!(!exists_file(&image_path));

    Ok(())
}

#[test]
fn cli_success_input_defaults_premultiply() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = cargo_bin_cmd!("flow-texpack");
    let out_file_path = "test_data/cli_success_input_defaults_premultiply/atlas";
    let parent_path = PathBuf::from(PathBuf::from(out_file_path).parent().unwrap());

    cmd.arg("-i")
        .arg("test_data/white_32x32.png")
        .arg("-o")
        .arg(out_file_path)
        .arg("-p")
        .arg("0");
    cmd.assert().success();

    let hash_path = PathBuf::from(format!("{}.hash", out_file_path));
    let json_path = PathBuf::from(format!("{}.json", out_file_path));
    let image_path = PathBuf::from(format!("{}0.png", out_file_path));

    assert!(exists_file(&hash_path));
    assert!(exists_file(&json_path));
    assert!(exists_file(&image_path));
    assert!(get_hash_value(&hash_path) > 0);

    verify_texture_size(&image_path, 32, 32);

    remove_dir_all(&parent_path);

    assert!(!exists_file(&hash_path));
    assert!(!exists_file(&json_path));
    assert!(!exists_file(&image_path));

    Ok(())
}

#[test]
fn cli_success_input_defaults_trim() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = cargo_bin_cmd!("flow-texpack");
    let out_file_path = "test_data/cli_success_input_defaults_trim/atlas";
    let parent_path = PathBuf::from(PathBuf::from(out_file_path).parent().unwrap());

    cmd.arg("-i")
        .arg("test_data/blue_trimmable_128x128.png")
        .arg("-o")
        .arg(out_file_path)
        .arg("-p")
        .arg("0")
        .arg("-t");
    cmd.assert().success();

    let hash_path = PathBuf::from(format!("{}.hash", out_file_path));
    let json_path = PathBuf::from(format!("{}.json", out_file_path));
    let image_path = PathBuf::from(format!("{}0.png", out_file_path));

    assert!(exists_file(&hash_path));
    assert!(exists_file(&json_path));
    assert!(exists_file(&image_path));
    assert!(get_hash_value(&hash_path) > 0);

    verify_texture_size(&image_path, 32, 32);

    remove_dir_all(&parent_path);

    assert!(!exists_file(&hash_path));
    assert!(!exists_file(&json_path));
    assert!(!exists_file(&image_path));

    Ok(())
}

#[test]
fn cli_success_input_defaults_unique() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = cargo_bin_cmd!("flow-texpack");
    let out_file_path = "test_data/cli_success_input_defaults_unique/atlas";
    let parent_path = PathBuf::from(PathBuf::from(out_file_path).parent().unwrap());

    cmd.arg("-i")
        .arg("test_data/white_128x128.png")
        .arg("test_data/white_128x128_v2.png")
        .arg("-o")
        .arg(out_file_path)
        .arg("-p")
        .arg("0")
        .arg("-u");
    cmd.assert().success();

    let hash_path = PathBuf::from(format!("{}.hash", out_file_path));
    let json_path = PathBuf::from(format!("{}.json", out_file_path));
    let image_path = PathBuf::from(format!("{}0.png", out_file_path));

    assert!(exists_file(&hash_path));
    assert!(exists_file(&json_path));
    assert!(exists_file(&image_path));
    assert!(get_hash_value(&hash_path) > 0);

    verify_texture_size(&image_path, 128, 128);

    remove_dir_all(&parent_path);

    assert!(!exists_file(&hash_path));
    assert!(!exists_file(&json_path));
    assert!(!exists_file(&image_path));

    Ok(())
}

#[test]
fn cli_success_input_defaults_rotate() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = cargo_bin_cmd!("flow-texpack");
    let out_file_path = "test_data/cli_success_input_defaults_rotate/atlas";
    let parent_path = PathBuf::from(PathBuf::from(out_file_path).parent().unwrap());

    cmd.arg("-i")
        .arg("test_data/white_128x64.png")
        .arg("test_data/white_32x16.png")
        .arg("-o")
        .arg(out_file_path)
        .arg("--rect-heuristic")
        .arg("long-side-fit")
        .arg("-r");
    cmd.assert().success();

    let hash_path = PathBuf::from(format!("{}.hash", out_file_path));
    let json_path = PathBuf::from(format!("{}.json", out_file_path));
    let image_path = PathBuf::from(format!("{}0.png", out_file_path));

    assert!(exists_file(&hash_path));
    assert!(exists_file(&json_path));
    assert!(exists_file(&image_path));
    assert!(get_hash_value(&hash_path) > 0);

    verify_texture_size(&image_path, 256, 128);

    remove_dir_all(&parent_path);

    assert!(!exists_file(&hash_path));
    assert!(!exists_file(&json_path));
    assert!(!exists_file(&image_path));

    Ok(())
}

#[test]
fn cli_success_input_defaults_adjust_size() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = cargo_bin_cmd!("flow-texpack");
    let out_file_path = "test_data/cli_success_input_defaults_adjust_size/atlas";
    let parent_path = PathBuf::from(PathBuf::from(out_file_path).parent().unwrap());

    cmd.arg("-i")
        .arg("test_data/white_128x128.png")
        .arg("-o")
        .arg(out_file_path)
        .arg("--atlas-size")
        .arg("pot64")
        .arg("-p")
        .arg("0")
        .arg("--adjust-size");
    cmd.assert().success();

    let hash_path = PathBuf::from(format!("{}.hash", out_file_path));
    let json_path = PathBuf::from(format!("{}.json", out_file_path));
    let image_path = PathBuf::from(format!("{}0.png", out_file_path));

    assert!(exists_file(&hash_path));
    assert!(exists_file(&json_path));
    assert!(exists_file(&image_path));
    assert!(get_hash_value(&hash_path) > 0);

    verify_texture_size(&image_path, 128, 128);

    remove_dir_all(&parent_path);

    assert!(!exists_file(&hash_path));
    assert!(!exists_file(&json_path));
    assert!(!exists_file(&image_path));

    Ok(())
}

#[test]
fn cli_success_input_defaults_adjust_fit() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = cargo_bin_cmd!("flow-texpack");
    let out_file_path = "test_data/cli_success_input_defaults_adjust_fit/atlas";
    let parent_path = PathBuf::from(PathBuf::from(out_file_path).parent().unwrap());

    cmd.arg("-i")
        .arg("test_data/white_128x128.png")
        .arg("-o")
        .arg(out_file_path)
        .arg("--atlas-size")
        .arg("pot64")
        .arg("-p")
        .arg("0")
        .arg("--adjust-fit");
    cmd.assert().success();

    let hash_path = PathBuf::from(format!("{}.hash", out_file_path));
    let json_path = PathBuf::from(format!("{}.json", out_file_path));
    let image_path = PathBuf::from(format!("{}0.png", out_file_path));

    assert!(exists_file(&hash_path));
    assert!(exists_file(&json_path));
    assert!(exists_file(&image_path));
    assert!(get_hash_value(&hash_path) > 0);

    verify_texture_size(&image_path, 64, 64);

    remove_dir_all(&parent_path);

    assert!(!exists_file(&hash_path));
    assert!(!exists_file(&json_path));
    assert!(!exists_file(&image_path));

    Ok(())
}

#[test]
fn cli_success_input_defaults_validate_descriptor_json() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = cargo_bin_cmd!("flow-texpack");
    let out_file_path = "test_data/cli_success_input_defaults_validate_descriptor_json/atlas";
    let parent_path = PathBuf::from(PathBuf::from(out_file_path).parent().unwrap());

    cmd.arg("-i")
        .arg("test_data/white_32x32.png")
        .arg("test_data/white_32x16.png")
        .arg("-o")
        .arg(out_file_path)
        .arg("--atlas-size")
        .arg("pot64")
        .arg("--generate-mipmaps")
        .arg("-p")
        .arg("4");
    cmd.assert().success();

    let hash_path = PathBuf::from(format!("{}.hash", out_file_path));
    let json_path = PathBuf::from(format!("{}.json", out_file_path));
    let image_path = PathBuf::from(format!("{}0.png", out_file_path));

    assert!(exists_file(&hash_path));
    assert!(exists_file(&json_path));
    assert!(exists_file(&image_path));
    assert!(get_hash_value(&hash_path) > 0);

    verify_texture_size(&image_path, 64, 64);
    validate_descriptor_json(&json_path);

    remove_dir_all(&parent_path);

    assert!(!exists_file(&hash_path));
    assert!(!exists_file(&json_path));
    assert!(!exists_file(&image_path));

    Ok(())
}

#[test]
fn cli_success_input_defaults_validate_descriptor_txt() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = cargo_bin_cmd!("flow-texpack");
    let out_file_path = "test_data/cli_success_input_defaults_validate_descriptor_txt/atlas";
    let parent_path = PathBuf::from(PathBuf::from(out_file_path).parent().unwrap());

    cmd.arg("-i")
        .arg("test_data/white_32x32.png")
        .arg("test_data/white_32x16.png")
        .arg("-o")
        .arg(out_file_path)
        .arg("--atlas-size")
        .arg("pot64")
        .arg("--atlas-descriptor")
        .arg("txt")
        .arg("--generate-mipmaps")
        .arg("-p")
        .arg("4");
    cmd.assert().success();

    let hash_path = PathBuf::from(format!("{}.hash", out_file_path));
    let txt_path = PathBuf::from(format!("{}.txt", out_file_path));
    let image_path = PathBuf::from(format!("{}0.png", out_file_path));

    assert!(exists_file(&hash_path));
    assert!(exists_file(&txt_path));
    assert!(exists_file(&image_path));
    assert!(get_hash_value(&hash_path) > 0);

    verify_texture_size(&image_path, 64, 64);
    validate_descriptor_txt(&txt_path, false);

    remove_dir_all(&parent_path);

    assert!(!exists_file(&hash_path));
    assert!(!exists_file(&txt_path));
    assert!(!exists_file(&image_path));

    Ok(())
}

#[test]
fn cli_success_input_defaults_validate_descriptor_txt_header()
-> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = cargo_bin_cmd!("flow-texpack");
    let out_file_path = "test_data/cli_success_input_defaults_validate_descriptor_txt_header/atlas";
    let parent_path = PathBuf::from(PathBuf::from(out_file_path).parent().unwrap());

    cmd.arg("-i")
        .arg("test_data/white_32x32.png")
        .arg("test_data/white_32x16.png")
        .arg("-o")
        .arg(out_file_path)
        .arg("--atlas-size")
        .arg("pot64")
        .arg("--atlas-descriptor")
        .arg("txt-desc")
        .arg("--generate-mipmaps")
        .arg("--pad")
        .arg("4");
    cmd.assert().success();

    let hash_path = PathBuf::from(format!("{}.hash", out_file_path));
    let txt_path = PathBuf::from(format!("{}.txt", out_file_path));
    let image_path = PathBuf::from(format!("{}0.png", out_file_path));

    assert!(exists_file(&hash_path));
    assert!(exists_file(&txt_path));
    assert!(exists_file(&image_path));
    assert!(get_hash_value(&hash_path) > 0);

    verify_texture_size(&image_path, 64, 64);
    validate_descriptor_txt(&txt_path, true);

    remove_dir_all(&parent_path);

    assert!(!exists_file(&hash_path));
    assert!(!exists_file(&txt_path));
    assert!(!exists_file(&image_path));

    Ok(())
}

#[test]
fn cli_success_input_defaults_load_filter_bmp() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = cargo_bin_cmd!("flow-texpack");
    let out_file_path = "test_data/cli_success_input_defaults_load_filter_bmp/atlas";
    let parent_path = PathBuf::from(PathBuf::from(out_file_path).parent().unwrap());

    cmd.arg("-i")
        .arg("test_data/white_32x32.bmp")
        .arg("test_data/white_128x128.png")
        .arg("-o")
        .arg(out_file_path)
        .arg("--load-filter")
        .arg("bmp");
    cmd.assert().success();

    let hash_path = PathBuf::from(format!("{}.hash", out_file_path));
    let json_path = PathBuf::from(format!("{}.json", out_file_path));
    let image_path = PathBuf::from(format!("{}0.png", out_file_path));

    assert!(exists_file(&hash_path));
    assert!(exists_file(&json_path));
    assert!(exists_file(&image_path));
    assert!(get_hash_value(&hash_path) > 0);

    verify_texture_size(&image_path, 64, 64);

    remove_dir_all(&parent_path);

    assert!(!exists_file(&hash_path));
    assert!(!exists_file(&json_path));
    assert!(!exists_file(&image_path));

    Ok(())
}

#[test]
fn cli_success_input_defaults_load_filter_hdr() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = cargo_bin_cmd!("flow-texpack");
    let out_file_path = "test_data/cli_success_input_defaults_load_filter_hdr/atlas";
    let parent_path = PathBuf::from(PathBuf::from(out_file_path).parent().unwrap());

    cmd.arg("-i")
        .arg("test_data/white_32x32.hdr")
        .arg("test_data/white_128x128.png")
        .arg("-o")
        .arg(out_file_path)
        .arg("--load-filter")
        .arg("hdr");
    cmd.assert().success();

    let hash_path = PathBuf::from(format!("{}.hash", out_file_path));
    let json_path = PathBuf::from(format!("{}.json", out_file_path));
    let image_path = PathBuf::from(format!("{}0.png", out_file_path));

    assert!(exists_file(&hash_path));
    assert!(exists_file(&json_path));
    assert!(exists_file(&image_path));
    assert!(get_hash_value(&hash_path) > 0);

    verify_texture_size(&image_path, 64, 64);

    remove_dir_all(&parent_path);

    assert!(!exists_file(&hash_path));
    assert!(!exists_file(&json_path));
    assert!(!exists_file(&image_path));

    Ok(())
}

#[test]
fn cli_success_input_defaults_load_filter_jpg() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = cargo_bin_cmd!("flow-texpack");
    let out_file_path = "test_data/cli_success_input_defaults_load_filter_jpg/atlas";
    let parent_path = PathBuf::from(PathBuf::from(out_file_path).parent().unwrap());

    cmd.arg("-i")
        .arg("test_data/white_32x32.jpg")
        .arg("test_data/white_128x128.png")
        .arg("-o")
        .arg(out_file_path)
        .arg("--load-filter")
        .arg("jpg");
    cmd.assert().success();

    let hash_path = PathBuf::from(format!("{}.hash", out_file_path));
    let json_path = PathBuf::from(format!("{}.json", out_file_path));
    let image_path = PathBuf::from(format!("{}0.png", out_file_path));

    assert!(exists_file(&hash_path));
    assert!(exists_file(&json_path));
    assert!(exists_file(&image_path));
    assert!(get_hash_value(&hash_path) > 0);

    verify_texture_size(&image_path, 64, 64);

    remove_dir_all(&parent_path);

    assert!(!exists_file(&hash_path));
    assert!(!exists_file(&json_path));
    assert!(!exists_file(&image_path));

    Ok(())
}

#[test]
fn cli_success_input_defaults_load_filter_png() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = cargo_bin_cmd!("flow-texpack");
    let out_file_path = "test_data/cli_success_input_defaults_load_filter_png/atlas";
    let parent_path = PathBuf::from(PathBuf::from(out_file_path).parent().unwrap());

    cmd.arg("-i")
        .arg("test_data/white_32x32.png")
        .arg("test_data/white_128x128.png")
        .arg("-o")
        .arg(out_file_path)
        .arg("--load-filter")
        .arg("png");
    cmd.assert().success();

    let hash_path = PathBuf::from(format!("{}.hash", out_file_path));
    let json_path = PathBuf::from(format!("{}.json", out_file_path));
    let image_path = PathBuf::from(format!("{}0.png", out_file_path));

    assert!(exists_file(&hash_path));
    assert!(exists_file(&json_path));
    assert!(exists_file(&image_path));
    assert!(get_hash_value(&hash_path) > 0);

    verify_texture_size(&image_path, 256, 256);

    remove_dir_all(&parent_path);

    assert!(!exists_file(&hash_path));
    assert!(!exists_file(&json_path));
    assert!(!exists_file(&image_path));

    Ok(())
}

#[test]
fn cli_success_input_defaults_load_filter_tga() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = cargo_bin_cmd!("flow-texpack");
    let out_file_path = "test_data/cli_success_input_defaults_load_filter_tga/atlas";
    let parent_path = PathBuf::from(PathBuf::from(out_file_path).parent().unwrap());

    cmd.arg("-i")
        .arg("test_data/white_32x32.tga")
        .arg("test_data/white_128x128.png")
        .arg("-o")
        .arg(out_file_path)
        .arg("--load-filter")
        .arg("tga");
    cmd.assert().success();

    let hash_path = PathBuf::from(format!("{}.hash", out_file_path));
    let json_path = PathBuf::from(format!("{}.json", out_file_path));
    let image_path = PathBuf::from(format!("{}0.png", out_file_path));

    assert!(exists_file(&hash_path));
    assert!(exists_file(&json_path));
    assert!(exists_file(&image_path));
    assert!(get_hash_value(&hash_path) > 0);

    verify_texture_size(&image_path, 64, 64);

    remove_dir_all(&parent_path);

    assert!(!exists_file(&hash_path));
    assert!(!exists_file(&json_path));
    assert!(!exists_file(&image_path));

    Ok(())
}

#[test]
fn cli_success_input_defaults_load_filter_tiff() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = cargo_bin_cmd!("flow-texpack");
    let out_file_path = "test_data/cli_success_input_defaults_load_filter_tiff/atlas";
    let parent_path = PathBuf::from(PathBuf::from(out_file_path).parent().unwrap());

    cmd.arg("-i")
        .arg("test_data/white_32x32.tiff")
        .arg("test_data/white_128x128.png")
        .arg("-o")
        .arg(out_file_path)
        .arg("--load-filter")
        .arg("tiff");
    cmd.assert().success();

    let hash_path = PathBuf::from(format!("{}.hash", out_file_path));
    let json_path = PathBuf::from(format!("{}.json", out_file_path));
    let image_path = PathBuf::from(format!("{}0.png", out_file_path));

    assert!(exists_file(&hash_path));
    assert!(exists_file(&json_path));
    assert!(exists_file(&image_path));
    assert!(get_hash_value(&hash_path) > 0);

    verify_texture_size(&image_path, 64, 64);

    remove_dir_all(&parent_path);

    assert!(!exists_file(&hash_path));
    assert!(!exists_file(&json_path));
    assert!(!exists_file(&image_path));

    Ok(())
}

#[test]
fn cli_success_input_defaults_load_filter_webp() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = cargo_bin_cmd!("flow-texpack");
    let out_file_path = "test_data/cli_success_input_defaults_load_filter_webp/atlas";
    let parent_path = PathBuf::from(PathBuf::from(out_file_path).parent().unwrap());

    cmd.arg("-i")
        .arg("test_data/white_32x32.webp")
        .arg("test_data/white_128x128.png")
        .arg("-o")
        .arg(out_file_path)
        .arg("--load-filter")
        .arg("webp");
    cmd.assert().success();

    let hash_path = PathBuf::from(format!("{}.hash", out_file_path));
    let json_path = PathBuf::from(format!("{}.json", out_file_path));
    let image_path = PathBuf::from(format!("{}0.png", out_file_path));

    assert!(exists_file(&hash_path));
    assert!(exists_file(&json_path));
    assert!(exists_file(&image_path));
    assert!(get_hash_value(&hash_path) > 0);

    verify_texture_size(&image_path, 64, 64);

    remove_dir_all(&parent_path);

    assert!(!exists_file(&hash_path));
    assert!(!exists_file(&json_path));
    assert!(!exists_file(&image_path));

    Ok(())
}
