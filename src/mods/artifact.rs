use std::path::Path;

use fancy_regex::Regex;

pub struct ArtifactId {
    pub name: String,
    pub version: String,
    pub classifier: Option<String>,
}

pub fn parse(path: impl AsRef<Path>) -> Option<ArtifactId> {
    let path = path.as_ref();

    let name = path.file_name()?.to_str()?;

    let id_pattern = Regex::new(r#"(.+?)-([\d.]+)(?:\+.+?(?=(?:\.jar)|-))?-?(.*?)\.jar"#).unwrap();
    let id_captures = id_pattern.captures(name).unwrap()?;

    let name = id_captures.get(1)?.as_str().to_owned();
    let version = id_captures.get(2)?.as_str().to_owned();
    let classifier = id_captures.get(3)
        .map(|m| m.as_str().to_owned())
        .filter(|s| !s.is_empty());

    Some(ArtifactId {
        name,
        version,
        classifier,
    })
}
