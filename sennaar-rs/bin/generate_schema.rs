use schemars::schema_for;
use sennaar_rs::registry::Registry;

fn main() {
    let schema = schema_for!(Registry);
    println!("{}", serde_json::to_string_pretty(&schema).unwrap());
}
