//! Constants for easier access to the assets

use texture::{SourcePixelRegion, TextureId, SourceTextureRegion, TextureInstanceId};
use errors::LevelLoadError;
use FastHashMap;

pub const FONT_BIG_SIZE: u32 = 48;
pub const FONT_MEDIUM_SIZE: u32 = 18;
pub const FONT_SMALL_SIZE: u32 = 14;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct LevelId(pub u32);

#[derive(Debug, Default)]
pub struct Level {
    pub font_data: FastHashMap<String, Vec<u8>>,
    pub image_data: FastHashMap<String, Vec<u8>>,
    pub sound_data: FastHashMap<String, Vec<u8>>,
}

pub(crate) fn load_level(level_id: LevelId) -> Result<Level, LevelLoadError> {
    
    use std::fs::{self, File};
    use std::io::Read;
    use zip::ZipArchive;

    let current_exe_path = ::std::env::current_exe()?;
    let mut gamedata_path = current_exe_path.parent().ok_or(LevelLoadError::InvalidParentDirectory)?.to_path_buf();
    gamedata_path.push("gamedata");

    let gamedata_folder = File::open(gamedata_path)?;
    
    if !gamedata_path.is_dir() {
        return Err(LevelLoadError::GameDataIsNotAFolder);
    }

    let searched_zip_file = format!("level{}.zip", level_id.0);
    let mut found_zip_file = None;
    for entry in ::std::fs::read_dir(gamedata_path.as_path())? {

        let entry = entry?;
        let file_name = entry.file_name().into_string().map_err(|_| LevelLoadError::FileNameEncodingError)?;

        if file_name != searched_zip_file {
            continue;
        }

        if entry.path().is_dir() {
            return Err(LevelLoadError::InvalidLevelToLoad(level_id));
        }

        found_zip_file = Some((entry.path(), entry.path().metadata().and_then(|m| Ok(m.len())).unwrap_or(0)));
    }

    if found_zip_file.is_none() {
        // level.zip file not found
        return Err(LevelLoadError::InvalidLevelToLoad(level_id));
    }

    let found_zip_file = found_zip_file.unwrap();
    let level_zip_file = File::open(found_zip_file.0)?;

    let mut level = Level::default();
    Ok(())
}

// start screen
pub const START_SCREEN_BUTTON_00_ID: &str = "../assets/images/ui/PNG/yellow_button04.png";
pub const START_SCREEN_BUTTON_00_TX_STR: SourceTextureRegion = SourceTextureRegion {
    texture_id: TextureId { texture_id: START_SCREEN_BUTTON_00_ID },
    region: SourcePixelRegion {
        bottom_x: 0,
        bottom_y: 0,
        width: 190,
        height: 49,
    }
};

// hero texture
pub const HERO_TEXTURE_ID: &str = "../assets/images/hero.png";
/* todo: add source pixel regions*/
pub const HERO_TX_NORMAL_STR: SourceTextureRegion = SourceTextureRegion {
    texture_id: TextureId { texture_id: HERO_TEXTURE_ID },
    region: SourcePixelRegion {
        bottom_x: 0,
        bottom_y: 64,
        width: 16,
        height: 16,
    }
};

// crate texture
pub const CRATE_TEXTURE_ID: &str = "../assets/images/crate.png";
pub const CRATE_TEXTURE_TX_STR: SourceTextureRegion = SourceTextureRegion {
    texture_id: TextureId { texture_id: CRATE_TEXTURE_ID },
    region: SourcePixelRegion {
        bottom_x: 0,
        bottom_y: 0,
        width: 32,
        height: 32,
    }
};

// background texture
pub const BACKGROUND_3_TEXTURE_ID: &str = "../assets/images/background/3.png";
pub const BACKGROUND_3_TEXTURE_TX_STR: SourceTextureRegion = SourceTextureRegion {
    texture_id: TextureId { texture_id: BACKGROUND_3_TEXTURE_ID },
    region: SourcePixelRegion {
        bottom_x: 0,
        bottom_y: 0,
        width: 256,
        height: 182,
    }
};
