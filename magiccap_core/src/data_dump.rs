use std::fs::File;

// Create a data dump from MagicCap's database. The option is an error if set.
pub fn dump_data(fp: String) -> Option<String> {
    let mut file = match File::create(fp) {
        Ok(f) => f,
        Err(e) => return Some(e.to_string()),
    };

    None
}
