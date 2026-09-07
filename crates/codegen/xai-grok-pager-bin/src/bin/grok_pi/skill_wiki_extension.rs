use anyhow::{Context, Result};
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Materialized skill-wiki extension source (single module).
pub(super) struct SkillWikiExtension {
    _source_dir: TempDir,
    source_path: PathBuf,
}

impl SkillWikiExtension {
    pub(super) fn source_path(&self) -> &Path {
        &self.source_path
    }
}

pub(super) fn write_skill_wiki_extension() -> Result<SkillWikiExtension> {
    let source_dir = tempfile::Builder::new()
        .prefix("pi-grok-skill-wiki-")
        .tempdir()
        .context("create Pi skill-wiki extension source directory")?;
    let source_path = source_dir.path().join("index.ts");
    let mut file = File::create(&source_path).context("create Pi skill-wiki extension source")?;
    file.write_all(include_str!("../../../../../../extensions/pi-grok-skill-wiki/index.ts").as_bytes())
        .context("write Pi skill-wiki extension source")?;
    file.flush().context("flush Pi skill-wiki extension source")?;
    file.sync_all().ok();
    Ok(SkillWikiExtension {
        _source_dir: source_dir,
        source_path,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skill_wiki_extension_source_loads() {
        let ext = write_skill_wiki_extension().expect("write skill-wiki extension");
        let source = std::fs::read_to_string(ext.source_path()).expect("read source");
        assert!(source.contains("skill_wiki_record"));
        assert!(source.contains("skill_wiki_read"));
        assert!(source.contains("skill_wiki_search"));
        assert!(source.contains("before_agent_start"));
        assert!(source.contains("registerCommand(\"skill-note\""));
    }
}
