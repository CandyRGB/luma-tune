use serde::{Deserialize, Serialize};

use crate::params::FilterParams;

pub const SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct PostProcessParams {
    pub hdr_toning: f32,
    pub bloom: f32,
    pub saturation: f32,
    pub clarity: f32,
    pub vibrance: f32,
}

impl PostProcessParams {
    pub fn normalized(self) -> Self {
        Self {
            hdr_toning: self.hdr_toning.clamp(-1.0, 1.0),
            bloom: self.bloom.clamp(0.0, 1.0),
            saturation: self.saturation.clamp(-1.0, 1.0),
            clarity: self.clarity.clamp(-1.0, 1.0),
            vibrance: self.vibrance.clamp(0.0, 1.0),
        }
    }

    pub fn is_neutral(&self) -> bool {
        *self == Self::default()
    }
}

#[derive(Deserialize, Serialize)]
struct StoredPostProcess {
    enabled: Option<bool>,
    #[serde(flatten)]
    params: PostProcessParams,
}

impl StoredPostProcess {
    fn read<'de, D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> Result<PostProcessParams, D::Error> {
        let stored = Self::deserialize(deserializer)?;
        Ok(if stored.enabled == Some(false) {
            PostProcessParams::default()
        } else {
            stored.params
        })
    }

    fn write<S: serde::Serializer>(
        params: &PostProcessParams,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        // Older versions require this flag to render a preset with effects.
        Self {
            enabled: Some(true),
            params: *params,
        }
        .serialize(serializer)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Preset {
    pub schema_version: u32,
    pub name: String,
    #[serde(default)]
    pub display: Option<String>,
    #[serde(default)]
    pub bindings: Vec<String>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub favorite: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cover: Option<String>,
    #[serde(rename = "operators")]
    pub params: FilterParams,
    #[serde(
        default,
        skip_serializing_if = "PostProcessParams::is_neutral",
        deserialize_with = "StoredPostProcess::read",
        serialize_with = "StoredPostProcess::write"
    )]
    pub post_process: PostProcessParams,
}

impl Preset {
    pub fn new(name: impl Into<String>, params: FilterParams) -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            name: name.into(),
            display: None,
            bindings: Vec::new(),
            favorite: false,
            cover: None,
            params,
            post_process: PostProcessParams::default(),
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
        let mut preset = Preset::new("暗部增强", params);
        preset.favorite = true;
        preset.cover = Some("cover-example.png".into());
        preset.post_process = PostProcessParams {
            hdr_toning: -0.4,
            bloom: 0.65,
            saturation: -0.25,
            clarity: -0.35,
            vibrance: 0.55,
        };

        let text = serde_json::to_string_pretty(&preset).unwrap();
        assert_eq!(
            serde_json::to_value(&preset).unwrap()["postProcess"]["enabled"],
            true
        );
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
        assert!(!preset.favorite);
        assert!(preset.cover.is_none());
        assert_eq!(preset.post_process, PostProcessParams::default());
        assert!(
            serde_json::to_value(preset)
                .unwrap()
                .get("postProcess")
                .is_none()
        );
    }

    #[test]
    fn existing_post_process_settings_default_missing_effects_to_zero() {
        let params: PostProcessParams =
            serde_json::from_str(r#"{"enabled":true,"hdrToning":-0.4,"bloom":0.65}"#).unwrap();
        assert_eq!(
            params,
            PostProcessParams {
                hdr_toning: -0.4,
                bloom: 0.65,
                ..PostProcessParams::default()
            }
        );
    }

    #[test]
    fn reads_legacy_disabled_and_flagless_post_process_presets() {
        let base = Preset::new("test", FilterParams::default());
        let mut stored = serde_json::to_value(base).unwrap();

        stored["postProcess"] = serde_json::json!({"enabled": false, "vibrance": 0.75});
        let legacy: Preset = serde_json::from_value(stored.clone()).unwrap();
        assert!(legacy.post_process.is_neutral());

        stored["postProcess"] = serde_json::json!({"vibrance": 0.75});
        let flagless: Preset = serde_json::from_value(stored).unwrap();
        assert_eq!(flagless.post_process.vibrance, 0.75);
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
