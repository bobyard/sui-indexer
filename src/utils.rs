use serde_json::Value;
use std::collections::HashMap;

pub fn json_to_kv_map(fields: &Value) -> HashMap<String, String> {
    let mut kv_set = HashMap::new();
    if fields.is_array() {
        for v in fields.as_array().unwrap().iter() {
            let name = v["key"].as_str().unwrap().to_string();
            let value = v["value"].as_str().unwrap().to_string();
            // .to_string()
            // .strip_prefix('"')
            // .unwrap()
            // .to_string();
            kv_set.insert(name, value);
        }
    } else if fields.is_object() {
        fields.as_object().unwrap().iter().for_each(|(k, v)| {
            if k == &"id" {
                return;
            }
            kv_set.insert(k.as_str().parse().unwrap(), v.as_str().unwrap().to_string());
        });
    }
    kv_set
}
