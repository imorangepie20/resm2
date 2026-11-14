use std::sync::Arc;
use tera::Tera;

pub fn init_templates() -> Arc<Tera> {
    let mut tera = Tera::default();
    tera.load_from_glob("src/forms/templates/**/*").expect("Failed to load templates");
    Arc::new(tera)
}