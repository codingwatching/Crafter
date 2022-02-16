use std::collections::HashMap;

use glam::{Vec2, IVec2};

use super::chunk::Chunk;

pub struct World {
    map: HashMap<String, Chunk>
}

impl World {

    // adds a chunk to the map - returns success
    pub fn add(&mut self, chunk: Chunk) -> bool {

        let key: String = chunk.get_key();

        let pos_x = &chunk.get_pos().x;
        let pos_y = &chunk.get_pos().y;

        if !self.map.contains_key(&key) {
            self.map.insert(key, chunk);
            println!("SET {}, {}!", pos_x, pos_y);
            return true;
        }
        false
    }

    pub fn clean_up(&mut self){
        self.map.iter_mut().for_each(| this_chunk |{
            match this_chunk.1.get_mesh_mut() {
                Some(mesh) => mesh.clean_up(false),
                None => (),
            }
        });
    }
    
    // returns a map iterator
    pub fn iter_map(&self) -> std::collections::hash_map::Iter<String, Chunk> {
        self.map.iter()
    }

    // removes a chunk from the world
    pub fn remove(&mut self, key: String) {
        self.map.remove(&key);
    }

    // gets a chunk
    pub fn get_chunk(&self, key: String) -> &Chunk {
        &self.map.get(&key).unwrap()
    }

    // gets a mutable chunk
    pub fn get_chunk_mut(&mut self, key: String) -> &mut Chunk {
        self.map.get_mut(&key).unwrap()
    }
}

pub fn new() -> World {
    World {
        map: HashMap::new(),
    }
}