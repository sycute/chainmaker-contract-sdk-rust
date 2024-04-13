// use serde::{Deserialize, Serialize};
use tiny_keccak::{Hasher, Keccak};

use crate::{
    sim_context::{self, SimContext},
    sim_context_rs,
};
use serde::{Deserialize, Serialize};

const EMPTY_STR: &str = "";
const KEYS_CONNECTOR: &str = "";

#[derive(Serialize, Deserialize)]
pub struct StoreMap {
    name: String,
    depth: u64,
}

impl StoreMap {
    pub fn new(ctx: &impl SimContext, name: &str, depth: u64) -> Result<StoreMap, String> {
        if depth <= 0 {
            return Err("depth must be greater than zero".to_string());
        }

        if name.is_empty() {
            return Err("name cannot be empty".to_string());
        }

        let map_key = get_hash(name.as_bytes());

        // state_key = name + map_key
        let mut state_key = name.to_string();
        state_key.push_str(map_key.as_str());

        let state_bytes = match ctx.get_state(state_key.as_str(), EMPTY_STR) {
            Ok(data) => data,
            Err(_) => return Err("get state faile.".to_string()),
        };

        let store_map;
        if state_bytes.len() != 0 {
            store_map = serde_json::from_slice(&state_bytes).unwrap();
        } else {
            store_map = StoreMap {
                name: name.to_string(),
                depth,
            };
            match store_map.save(ctx, state_key.as_str()) {
                Ok(_) => {}
                Err(e) => return Err(e),
            }
        }

        Ok(store_map)
    }

    pub fn get(&self, ctx: &impl SimContext, key: &Vec<String>) -> Result<Vec<u8>, String> {
        if let Some(e) = self.check_depth(key) {
            return Err(e);
        }

        let (gened_key, field) = self.generate_key(key);

        match ctx.get_state(&gened_key, &field) {
            Ok(res) => Ok(res),
            Err(_) => Err("get state fail".to_string()),
        }
    }

    pub fn set(
        &self,
        ctx: &impl SimContext,
        key: &Vec<String>,
        value: &[u8],
    ) -> Result<(), String> {
        if let Some(e) = self.check_depth(key) {
            return Err(e);
        };

        let (gened_key, field) = self.generate_key(key);

        let re = ctx.put_state(&gened_key, &field, value);
        if re != sim_context::SUCCESS_CODE {
            return Err("failed to put state".to_string());
        }

        Ok(())
    }

    pub fn del(&self, ctx: &impl SimContext, key: &Vec<String>) -> Result<(), String> {
        if let Some(e) = self.check_depth(key) {
            return Err(e);
        }

        let (gened_key, field) = self.generate_key(key);

        if ctx.delete_state(&gened_key, &field) != sim_context::SUCCESS_CODE {
            return Err("failed to del state".to_string());
        }

        Ok(())
    }

    pub fn exist(&self, ctx: &impl SimContext, key: &Vec<String>) -> Result<bool, String> {
        if let Some(e) = self.check_depth(key) {
            return Err(e);
        }

        let (gened_key, field) = self.generate_key(key);
        match ctx.get_state(&gened_key, &field) {
            Ok(data) => {
                if data.len() > 0 {
                    return Ok(true);
                }
            }
            Err(_) => return Err("failed to get state".to_string()),
        }

        Ok(false)
    }

    pub fn new_iterator_prefix_with_key(
        &self,
        ctx: &impl SimContext,
        key: &Vec<String>,
    ) -> Result<Box<dyn sim_context_rs::ResultSet>, sim_context::result_code> {
        let itertor_key = self.generate_itertor_key(key);
        ctx.new_iterator_prefix_with_key(&itertor_key)
    }

    fn save(&self, ctx: &impl SimContext, state_key: &str) -> Result<(), String> {
        let store_map_bytes = serde_json::to_vec(self).unwrap();

        if ctx.put_state(state_key, "", &store_map_bytes) != sim_context::SUCCESS_CODE {
            return Err("failed to put state".to_string());
        }

        Ok(())
    }

    fn check_depth(&self, key: &Vec<String>) -> Option<String> {
        for k in key {
            if k.is_empty() {
                return Some("key cannot be empty".to_string());
            }
        }
        if self.depth != (key.len() as u64) {
            return Some("please check keys".to_string());
        }

        None
    }

    fn generate_key(&self, key: &Vec<String>) -> (String, String) {
        let mut field = get_hash(self.name.as_bytes());
        for k in key {
            field.push_str(k);
            field = get_hash(field.as_bytes());
        }

        let mut gened_key = self.name.clone();
        gened_key.push_str(key.join(KEYS_CONNECTOR).as_str());

        (gened_key, field)
    }

    fn generate_itertor_key(&self, key: &Vec<String>) -> String {
        let mut gened_key = self.name.clone();
        gened_key.push_str(key.join(KEYS_CONNECTOR).as_str());

        gened_key
    }
}

fn get_hash(data: &[u8]) -> String {
    let mut keccak = Keccak::v256();
    keccak.update(data);

    let mut output = [0u8; 32];
    keccak.finalize(&mut output);

    hex::encode(output)
}
