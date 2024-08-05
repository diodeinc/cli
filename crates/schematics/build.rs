#[cfg(feature = "edatasheet")]
use std::{env, fs, path::Path};

#[cfg(feature = "edatasheet")]
use typify::{TypeSpace, TypeSpaceSettings};

fn main() {
    #[cfg(feature = "edatasheet")]
    generate_edatasheet();
}

#[cfg(feature = "edatasheet")]
fn generate_edatasheet() {
    let part_spec_path = std::env::var("PART_SPEC_PATH").unwrap();
    let path = Path::new(&part_spec_path);
    let content = std::fs::read_to_string(path).unwrap();
    let schema = serde_json::from_str::<schemars::schema::RootSchema>(&content).unwrap();

    let mut type_space = TypeSpace::new(TypeSpaceSettings::default().with_struct_builder(true));
    type_space.add_root_schema(schema).unwrap();

    let contents = format!(
        "{}\n{}",
        "use serde::{Deserialize, Serialize};",
        prettyplease::unparse(&syn::parse2::<syn::File>(type_space.to_stream()).unwrap())
    );

    let mut out_file = Path::new(&env::var("OUT_DIR").unwrap()).to_path_buf();
    out_file.push(format!(
        "{}.rs",
        path.file_stem().unwrap().to_str().unwrap()
    ));
    fs::write(out_file.clone(), contents).unwrap();
    println!("cargo:rerun-if-changed={}", part_spec_path);
    println!("wrote {}", out_file.display());
}
