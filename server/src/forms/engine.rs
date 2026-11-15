use std::sync::Arc;
use tera::{Tera, Value, Kwargs, State};
use serde_json;

use crate::forms::model::{format_won, format_ymd, fill};

pub fn init_templates() -> Arc<Tera> {
    let mut tera = Tera::default();

    // Register filters FIRST, before loading templates
    tera.register_filter("won", |v: Value, _: Kwargs, _: &State| {
        let opt: Option<serde_json::Value> = v.as_str().map(|s| serde_json::Value::String(s.to_string()));
        let result = format_won(&opt);
        Ok(Value::from_serializable(&result))
    });
    tera.register_filter("ymd", |v: Value, _: Kwargs, _: &State| {
        let opt: Option<serde_json::Value> = v.as_str().map(|s| serde_json::Value::String(s.to_string()));
        let result = format_ymd(&opt);
        Ok(Value::from_serializable(&result))
    });
    tera.register_filter("fill_medium", |v: Value, _: Kwargs, _: &State| {
        let opt: Option<String> = v.as_str().map(|s| s.to_string());
        let result = fill(&opt, "55%");
        Ok(Value::from_serializable(&result))
    });
    tera.register_filter("fill_short", |v: Value, _: Kwargs, _: &State| {
        let opt: Option<String> = v.as_str().map(|s| s.to_string());
        let result = fill(&opt, "40%");
        Ok(Value::from_serializable(&result))
    });
    tera.register_filter("fill_wide", |v: Value, _: Kwargs, _: &State| {
        let opt: Option<String> = v.as_str().map(|s| s.to_string());
        let result = fill(&opt, "80%");
        Ok(Value::from_serializable(&result))
    });
    tera.register_filter("fill", |v: Value, _: Kwargs, _: &State| {
        let opt: Option<String> = v.as_str().map(|s| s.to_string());
        let result = fill(&opt, "100%");
        Ok(Value::from_serializable(&result))
    });

    // Then load templates
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let pattern = format!("{}/src/forms/templates/**/*", manifest_dir);
    tera.load_from_glob(&pattern).expect("Failed to load templates");

    Arc::new(tera)
}