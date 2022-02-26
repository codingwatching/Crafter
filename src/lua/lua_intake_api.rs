use std::{path::Path, fmt::Error};

use image::{GenericImageView, DynamicImage};
use mlua::{Lua, Table, prelude};
use texture_packer::{importer::ImageImporter, TexturePackerConfig, TexturePacker, exporter::ImageExporter};

use crate::{blocks::blocks::{BlockComponentSystem, DrawType}, graphics::mesh_component_system::MeshComponentSystem, helper::helper_functions::with_path};


fn get_texture_size(path_string: String) -> (u32, u32) {
    let path = Path::new(&path_string);
    let texture_option = ImageImporter::import_from_file(path);

    let texture: DynamicImage;

    match texture_option {
        Ok(texture_wrapped) =>  texture = texture_wrapped,
        Err(error) => {
            panic!("COULD NOT LOAD TEXTURE: {}! Error String: {}", path_string, error)
        }
    }
    (texture.width(), texture.height())
}

fn configure_texture_atlas(module_name: &str, texture_name: &str, number_of_textures: &mut u32, biggest_width: &mut u32, biggest_height: &mut u32) {
    let (width, height) = get_texture_size(with_path( &("/mods/".to_owned() + module_name + "/textures/" + texture_name) ));

    if width > *biggest_width {
        *biggest_width = width;
    }
    if height > *biggest_height {
        *biggest_height = height;
    }

    *number_of_textures += 1;
}

fn create_texture(module_name: &str, texture_name: &str) -> DynamicImage {
    let string_path: String = with_path( &("/mods/".to_owned() + module_name + "/textures/" + texture_name) );
    let path: &Path = Path::new(&string_path);
    ImageImporter::import_from_file(path).expect("UNABLE TO LOAD TEXTURE")
}


pub fn intake_api_values(lua: &Lua, mcs: &mut MeshComponentSystem, bcs: &mut BlockComponentSystem) {

    // this follows the same pattern as lua
    let crafter: Table = lua.globals().get("crafter").unwrap();
    let texture_cache: Table = crafter.get("texture_cache").unwrap();


    println!("-------BEGINNING TEST OF API TRANSLATION ------------");

    // this is done imperatively because it's easier to understand the program flow

    // first we must configure the texture atlas using the modules defined in lua
    let mut cached_table_values: Vec<(String, String)> = Vec::new();

    // iterate the base of the texture cache table - crafter.texture_cache
    for module_name in texture_cache.pairs::<String, Table>() {

        // the module_name_unwrapped.0 is the module name
        let module_name_unwrapped: (String, Table) = module_name.unwrap();

        // iterate through textures in module table - crafter.texture_cache[texture]
        for texture_name in module_name_unwrapped.1.pairs::<u32, String>() {

            // 0 - index | 1 - texture name (.png)
            let texture_name_unwrapped: (u32, String) = texture_name.unwrap();

            // insert the values into the vector tuple
            cached_table_values.push((module_name_unwrapped.0.clone(), texture_name_unwrapped.1));

        }
    }

    println!("{:#?}", cached_table_values);

    // find the biggest size, and number of textures

    let mut biggest_width = 0;
    let mut biggest_height = 0;
    let mut number_of_textures = 0;

    for (module_name, texture_name) in cached_table_values.iter() {
        configure_texture_atlas(
            &module_name, 
            &texture_name,
            &mut number_of_textures,
            &mut biggest_width,
            &mut biggest_height
        )
    }

    // automatically configure the texture atlas with the supplied information

    println!("width: {} | height: {}, number of textures: {}", biggest_width, biggest_height, number_of_textures);

    // configged width is the number of textures it can fit on that axis
    let configged_width: u32 = (number_of_textures + 2) / 2;
    let configged_height: u32 = ((number_of_textures + 2) / 2) + 1;

    println!("{configged_width}");
    println!("{configged_height}");

    let config = TexturePackerConfig {
        max_width: biggest_width * configged_width,
        max_height: biggest_height * configged_height,
        allow_rotation: false,
        texture_outlines: false,
        border_padding: 0,
        texture_padding: 0,
        texture_extrusion: 0,
        trim: false,
    };
    
    // this is the actual texture packer
    let mut packer: TexturePacker<DynamicImage, String> = TexturePacker::new_skyline(config);

    for (module_name, texture_name) in cached_table_values.iter() {
        let created_texture: DynamicImage = create_texture(
            &module_name, 
            &texture_name
        );

        packer.pack_own(texture_name.to_string(), created_texture).expect("Unable to pack texture!");
    }

    let atlas: DynamicImage = ImageExporter::export(&packer).unwrap();

    /*
    for value in packer.get_frames() {
        println!("{:#?}", value);
    }
    */

    println!("atlas width: {} | atlas height: {}", atlas.width(), atlas.height());

    // iterate blocks to be put into Block Component System

    // iterating crafter.blocks
    let blocks: Table = crafter.get("blocks").unwrap();

    // intake all data from lua
    for blocks in blocks.pairs::<String, Table>() {

        let (_, lua_table) = blocks.unwrap();

        // these are required
        let block_name: String = lua_table.get("name").unwrap();
        let block_mod: String = lua_table.get("mod").unwrap();

        println!("{}, {}", block_name, block_mod);

        // pull lua texture table into Rust String vector
        let lua_block_textures: Table = lua_table.get("textures").unwrap();
        let mut block_textures: Vec<String> = Vec::new();

        for value in lua_block_textures.pairs::<String, String>(){
            block_textures.push(value.unwrap().1);
        }

        println!("{:?}", block_textures);

        // begin the optional values
        let draw_type_option: Result<String, prelude::LuaError> = lua_table.get("draw_type");

        let draw_type: DrawType;

        match draw_type_option {
            Ok(draw_type_string) => {
                match draw_type_string.as_str() {
                    "normal" => draw_type = DrawType::Normal,
                    "airlike" => draw_type = DrawType::None,
                    _ => draw_type = DrawType::Normal
                }
            },
            Err(_) => todo!(),
        }

        bcs.register_block(
            block_name,
            block_textures,
            None,
            draw_type
        )
    }


    

    // texture atlas will always be id 1
    let value_test = mcs.new_texture_from_memory(atlas.as_rgba8().unwrap().to_owned());

    println!("TEXTURE ATLAS IS VALUE: {}", value_test);
    
    println!("-------------- done -----------------");
}