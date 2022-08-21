use std::{
  io::Write,
  path::Path,
  fs::{File, DirBuilder, remove_file},
};

use dalek_ff_group::EdwardsPoint;

use monero_generators::bulletproofs_generators;

fn serialize(generators_string: &mut String, points: &[EdwardsPoint]) {
  for generator in points {
    generators_string.extend(
      format!(
        "
          dalek_ff_group::EdwardsPoint(
            curve25519_dalek::edwards::CompressedEdwardsY({:?}).decompress().unwrap()
          ),
        ",
        generator.compress().to_bytes()
      )
      .chars(),
    );
  }
}

fn generators(prefix: &'static str, path: &str) {
  let generators = bulletproofs_generators(prefix.as_bytes());
  #[allow(non_snake_case)]
  let mut G_str = "".to_string();
  serialize(&mut G_str, &generators.G);
  #[allow(non_snake_case)]
  let mut H_str = "".to_string();
  serialize(&mut H_str, &generators.H);

  DirBuilder::new().recursive(true).create(".generators").unwrap();
  let path = Path::new(".generators").join(path);
  let _ = remove_file(&path);
  File::create(&path)
    .unwrap()
    .write_all(
      format!(
        "
          lazy_static! {{
            static ref GENERATORS: Generators = Generators {{
              G: [
                {}
              ],
              H: [
                {}
              ],
            }};
          }}
        ",
        G_str, H_str,
      )
      .as_bytes(),
    )
    .unwrap();
}

fn main() {
  // For some reason, filtering off .generators does not work. This prevents re-building overall
  println!("cargo:rerun-if-changed=build.rs");

  generators("bulletproof", "generators.rs");
  generators("bulletproof_plus", "generators_plus.rs");
}