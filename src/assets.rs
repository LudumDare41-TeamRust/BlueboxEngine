//! Constants for easier access to the assets

use texture::{SourcePixelRegion, TextureId, SourceTextureRegion};
use errors::LevelLoadError;
use FastHashMap;

pub const FONT_BIG_SIZE: u32 = 48;
pub const FONT_MEDIUM_SIZE: u32 = 18;
pub const FONT_SMALL_SIZE: u32 = 14;
pub const GAME_TITLE: &str = "Stack Boxes!";

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct LevelId(pub u32);

#[derive(Debug, Default)]
pub struct Level {
    pub font_data: FastHashMap<String, Vec<u8>>,
    pub image_data: FastHashMap<String, (Vec<u8>, Option<SourceTextureRegion>)>,
    pub sound_data: FastHashMap<String, Vec<u8>>,
}

pub(crate) fn load_level(level_id: LevelId) -> Result<Level, LevelLoadError> {
    
    use std::fs::File;
    use std::io::Read;

    let current_exe_path = ::std::env::current_exe()?;
    let mut gamedata_path = current_exe_path.parent()
        .ok_or(LevelLoadError::InvalidParentDirectory)?.to_path_buf();
    gamedata_path.push("gamedata");
    
    if !gamedata_path.is_dir() {
        return Err(LevelLoadError::GameDataIsNotAFolder);
    }

    let searched_dir_name = format!("level{}", level_id.0);
    let mut found_level_directory = None;
    for entry in ::std::fs::read_dir(gamedata_path.as_path())? {
        let entry = entry?;
        let file_name = entry.file_name().into_string().map_err(|_| LevelLoadError::FileNameEncodingError)?;

        if file_name != searched_dir_name {
            continue;
        }

        if !entry.path().is_dir() {
            return Err(LevelLoadError::InvalidLevelToLoad(level_id));
        }

        found_level_directory = Some(entry.path());
    }

    if found_level_directory.is_none() {
        return Err(LevelLoadError::InvalidLevelToLoad(level_id));
    }
    let found_level_directory = found_level_directory.unwrap();

    let mut images_directory = None;
    let mut sounds_directory = None;
    let mut fonts_directory = None;

    for entry in ::std::fs::read_dir(found_level_directory)? {
        let entry = entry?;
        let file_name = entry.file_name().into_string().map_err(|_| LevelLoadError::FileNameEncodingError)?;
        match file_name.as_ref() {
            "images" => images_directory = Some(entry.path()),
            "sounds" => sounds_directory = Some(entry.path()),
            "fonts" => fonts_directory = Some(entry.path()),
            _ => { },
        }
    }

    macro_rules! load_directory {
        ($hashmap:ident, $directory:ident) => ({
            if let Some(dir) = $directory {
                for entry in ::std::fs::read_dir(dir)? {
                    let entry = entry?;
                    if entry.path().is_dir() {
                        continue;
                    }
                    let file_name = entry.file_name().into_string()
                        .map_err(|_| LevelLoadError::FileNameEncodingError)?;
                    let file_size = entry.metadata().and_then(|m| Ok(m.len())).unwrap_or(0);
                    let mut data = Vec::with_capacity(file_size as usize);
                    let mut file = File::open(entry.path())?;
                    file.read_to_end(&mut data).unwrap();
                    $hashmap.insert(file_name, data);
                }
            }
        })
    }

    // load /images directory
    let mut images = FastHashMap::default();
    if let Some(ref dir) = images_directory {
        for entry in ::std::fs::read_dir(dir)? {
            let entry = entry?;
            if entry.path().is_dir() {
                continue;
            }
            let file_name = entry.file_name().into_string()
                .map_err(|_| LevelLoadError::FileNameEncodingError)?;
            let file_size = entry.metadata().and_then(|m| Ok(m.len())).unwrap_or(0);
            let mut data = Vec::with_capacity(file_size as usize);
            let mut file = File::open(entry.path())?;
            file.read_to_end(&mut data).unwrap();
            images.insert(file_name, (data, None));
        }
    }

    if let Some(ref images_directory) = images_directory {
        let mut source_texture_regions_file_path = images_directory.clone();
        source_texture_regions_file_path.push("pixel_regions.toml");
        let mut source_texture_regions_file = File::open(source_texture_regions_file_path.as_path())?;
        let source_texture_regions = load_source_texture_regions_toml(&mut source_texture_regions_file)?;
        for (k, v) in source_texture_regions {
            if let Some(image) = images.get_mut(&k) {
                image.1 = Some(v);
            }
        }        
    }

    let mut sounds = FastHashMap::default();
    load_directory!(sounds, sounds_directory);

    let mut fonts = FastHashMap::default();
    load_directory!(fonts, fonts_directory);

    let level = Level {
        font_data: fonts,
        image_data: images,
        sound_data: sounds,
    };

    Ok(level)
}

use std::fs::File;

fn load_source_texture_regions_toml(texture_region_file: &mut File) 
-> Result<FastHashMap<String, SourceTextureRegion>, LevelLoadError> 
{
    use std::io::Read;
    use toml::Value;

    let mut file_contents = String::new();
    texture_region_file.read_to_string(&mut file_contents)?;
    let toml = file_contents.parse::<Value>()?;
    match toml {
        Value::Table(table) => {
            let texture_regions: Vec<(&String, &Value)> = table.iter().filter(|(k,v)| *k == "textures").collect();
            let mut source_texture_hash_map = FastHashMap::default();

            for (texture_key, texture_description) in texture_regions {
                let texture_description = texture_description.as_table().ok_or(LevelLoadError::InvalidToml)?;
                
                let texture_state = texture_description.get("state")
                    .ok_or(LevelLoadError::InvalidToml)?.as_str().unwrap_or("default".into());
                let texture_bottom_x = texture_description.get("bottom_x")
                    .ok_or(LevelLoadError::InvalidToml)?.as_integer().unwrap_or(0) as u32;
                let texture_bottom_y = texture_description.get("bottom_y")
                    .ok_or(LevelLoadError::InvalidToml)?.as_integer().unwrap_or(0) as u32;
                let texture_width = texture_description.get("width")
                    .ok_or(LevelLoadError::InvalidToml)?.as_integer().unwrap_or(0) as u32;
                let texture_height = texture_description.get("height")
                    .ok_or(LevelLoadError::InvalidToml)?.as_integer().unwrap_or(0) as u32;

                source_texture_hash_map.insert(texture_key.clone(), SourceTextureRegion {
                    texture_id: TextureId { texture_id: format!("{}#{}", texture_key, texture_state) },
                    region: SourcePixelRegion {
                        bottom_x: texture_bottom_x,
                        bottom_y: texture_bottom_y,
                        width: texture_width,
                        height: texture_height,
                    }
                });
            }

            Ok(source_texture_hash_map)
        },
        _ => {
            Err(LevelLoadError::InvalidToml)
        }
    }
}