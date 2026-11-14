use serde::{Deserialize, Serialize};

use crate::params::FilterParams;

pub const SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Preset {
    pub schema_version: u32,
    pub name: String,
    #[serde(default)]
    pub display: Option<String>,
    #[serde(default)]
    pub bindings: Vec<String>,
    #[serde(rename = "operators")]
    pub params: FilterParams,
}

impl Preset {
    pub fn new(name: impl Into<String>, params: FilterParams) -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            name: name.into(),
            display: None,
            bindings: Vec::new(),
            params,
        }
    }

    pub fn is_compatible(&self) -> bool {
        self.schema_version <= SCHEMA_VERSION
    }

    pub fn bind_to(&mut self, exe: impl Into<String>) {
        let exe = exe.into();
        if !self.bindings.iter().any(|b| b.eq_ignore_ascii_case(&exe)) {
            self.bindings.push(exe);
        }
    }

    pub fn unbind(&mut self, exe: &str) {
        self.bindings.retain(|b| !b.eq_ignore_ascii_case(exe));
    }

    pub fn matches_exe(&self, exe: &str) -> bool {
        self.bindings.iter().any(|b| b.eq_ignore_ascii_case(exe))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrips_through_json() {
        let mut params = FilterParams::default();
        params.gamma = 2.2;
        params.temperature = 7200;
        params.curve = vec![[0.0, 0.0], [0.5, 0.45], [1.0, 1.0]];
        let preset = Preset::new("暗部增强", params);

        let text = serde_json::to_string_pretty(&preset).unwrap();
        let back: Preset = serde_json::from_str(&text).unwrap();
        assert_eq!(preset, back);
    }

    #[test]
    fn accepts_missing_optional_fields() {
        let text = r#"{"schemaVersion":1,"name":"x","operators":{
            "levels":{"inBlack":0.0,"inWhite":1.0,"outBlack":0.0,"outWhite":1.0},
            "brightness":0.0,"contrast":1.0,"gamma":1.0,
            "temperature":6500,"tint":0.0,"gains":[1.0,1.0,1.0],"curve":[]}}"#;
        let preset: Preset = serde_json::from_str(text).unwrap();
        assert!(preset.bindings.is_empty());
        assert!(preset.display.is_none());
    }

    #[test]
    fn binding_is_case_insensitive_and_deduped() {
        let mut preset = Preset::new("x", FilterParams::default());
        preset.bind_to("CS2.exe");
        preset.bind_to("cs2.exe");
        assert_eq!(preset.bindings.len(), 1);
    }

    #[test]
    fn matches_only_bound_processes() {
        let mut preset = Preset::new("暗部增强", FilterParams::default());
        preset.bind_to("CS2.exe");
        assert!(preset.matches_exe("cs2.EXE"));
        assert!(!preset.matches_exe("dota2.exe"));
    }

    #[test]
    fn unbinding_ignores_case() {
        let mut preset = Preset::new("x", FilterParams::default());
        preset.bind_to("CS2.exe");
        preset.unbind("cs2.exe");
        assert!(preset.bindings.is_empty());
    }
}
