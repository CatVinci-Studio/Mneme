use crate::domain::*;
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Clone)]
pub struct Vault {
    pub root: PathBuf,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct EntityFrontmatter {
    slug: String,
    title: String,
    kind: String,
    #[serde(default)]
    aliases: Vec<String>,
    #[serde(default)]
    sources: Vec<String>,
    updated: String,
}

impl Vault {
    pub fn new(root: PathBuf) -> Result<Self, String> {
        let vault = Self { root };
        vault.init()?;
        Ok(vault)
    }

    pub fn init(&self) -> Result<(), String> {
        fs::create_dir_all(self.root.join("raw")).map_err(err)?;
        fs::create_dir_all(self.root.join("wiki/entities")).map_err(err)?;
        fs::create_dir_all(self.root.join("wiki/notes")).map_err(err)?;
        if !self.root.join("log.md").exists() {
            atomic_write(&self.root.join("log.md"), b"# Mneme Log\n\n")?;
        }
        let ignore_path = self.root.join(".gitignore");
        let mut ignore = if ignore_path.exists() {
            fs::read_to_string(&ignore_path).map_err(err)?
        } else {
            String::new()
        };
        for rule in ["config.json", ".index/"] {
            if !ignore.lines().any(|line| line.trim() == rule) {
                if !ignore.is_empty() && !ignore.ends_with('\n') {
                    ignore.push('\n');
                }
                ignore.push_str(rule);
                ignore.push('\n');
            }
        }
        atomic_write(&ignore_path, ignore.as_bytes())?;
        if !self.root.join(".git").exists() {
            let _ = self.git(&["init"]);
            let _ = self.git(&["config", "user.name", "Mneme"]);
            let _ = self.git(&["config", "user.email", "mneme@localhost"]);
            let _ = self.git(&["add", "-A"]);
            let _ = self.git(&["commit", "-m", "init mneme vault", "--quiet"]);
        }
        Ok(())
    }

    pub fn raw_dir(&self, id: &str) -> PathBuf {
        self.root.join("raw").join(id)
    }
    fn entity_path(&self, slug: &str) -> PathBuf {
        self.root.join("wiki/entities").join(format!("{slug}.md"))
    }
    fn note_path(&self, id: &str) -> PathBuf {
        self.root.join("wiki/notes").join(format!("{id}.md"))
    }

    pub fn write_raw(
        &self,
        meta: &SourceMeta,
        content: &str,
        original: Option<&str>,
    ) -> Result<(), String> {
        let dir = self.raw_dir(&meta.id);
        if dir.exists() {
            let content_path = dir.join("content.md");
            let meta_path = dir.join("meta.json");
            if !content_path.exists() || !meta_path.exists() {
                return Err("incomplete immutable source snapshot".into());
            }
            let existing = fs::read_to_string(content_path).map_err(err)?;
            let existing_meta: SourceMeta =
                serde_json::from_slice(&fs::read(meta_path).map_err(err)?).map_err(err)?;
            if existing != content || existing_meta.content_hash != meta.content_hash {
                return Err("immutable source id collision".into());
            }
            return Ok(());
        }
        let raw_root = self.root.join("raw");
        let staging = tempfile::Builder::new()
            .prefix(".mneme-source-")
            .tempdir_in(&raw_root)
            .map_err(err)?;
        atomic_write(&staging.path().join("content.md"), content.as_bytes())?;
        atomic_write(
            &staging.path().join("meta.json"),
            &serde_json::to_vec_pretty(meta).map_err(err)?,
        )?;
        if let Some(value) = original {
            atomic_write(&staging.path().join("original.html"), value.as_bytes())?;
        }
        let staging_path = staging.keep();
        fs::rename(&staging_path, &dir).map_err(err)?;
        Ok(())
    }

    pub fn read_raw(&self, id: &str) -> Result<String, String> {
        safe_component(id)?;
        fs::read_to_string(self.raw_dir(id).join("content.md")).map_err(err)
    }

    pub fn read_raw_utf16_span(
        &self,
        id: &str,
        start: usize,
        end: usize,
    ) -> Result<String, String> {
        let content = self.read_raw(id)?;
        let units: Vec<u16> = content.encode_utf16().collect();
        if start > end || end > units.len() {
            return Err("provenance range out of bounds".into());
        }
        String::from_utf16(&units[start..end]).map_err(err)
    }

    pub fn write_note(&self, id: &str, markdown: &str) -> Result<(), String> {
        safe_component(id)?;
        atomic_write(&self.note_path(id), markdown.as_bytes())
    }
    pub fn read_note(&self, id: &str) -> Result<Option<String>, String> {
        safe_component(id)?;
        let path = self.note_path(id);
        if !path.exists() {
            return Ok(None);
        }
        fs::read_to_string(path).map(Some).map_err(err)
    }

    pub fn read_meta(&self, id: &str) -> Result<Option<SourceMeta>, String> {
        safe_component(id)?;
        let path = self.raw_dir(id).join("meta.json");
        if !path.exists() {
            return Ok(None);
        }
        serde_json::from_slice(&fs::read(path).map_err(err)?)
            .map(Some)
            .map_err(err)
    }

    pub fn source_ids(&self) -> Result<Vec<String>, String> {
        list_dirs(&self.root.join("raw"))
    }
    pub fn entity_slugs(&self) -> Result<Vec<String>, String> {
        let mut out = Vec::new();
        for entry in fs::read_dir(self.root.join("wiki/entities")).map_err(err)? {
            let path = entry.map_err(err)?.path();
            if path.extension().and_then(|x| x.to_str()) == Some("md") {
                if let Some(name) = path.file_stem().and_then(|x| x.to_str()) {
                    out.push(name.to_string());
                }
            }
        }
        out.sort();
        Ok(out)
    }

    pub fn list_sources(&self) -> Result<Vec<SourceMeta>, String> {
        let mut items = Vec::new();
        for id in self.source_ids()? {
            if let Some(mut meta) = self.read_meta(&id)? {
                meta.summarized = Some(self.note_path(&id).exists());
                items.push(meta);
            }
        }
        items.sort_by(|a, b| b.fetched_at.cmp(&a.fetched_at));
        Ok(items)
    }

    pub fn read_entity(&self, slug: &str) -> Result<Option<EntityPage>, String> {
        safe_component(slug)?;
        let path = self.entity_path(slug);
        if !path.exists() {
            return Ok(None);
        }
        parse_entity(slug, &fs::read_to_string(path).map_err(err)?).map(Some)
    }

    pub fn entity_hash(&self, slug: &str) -> Result<String, String> {
        let path = self.entity_path(slug);
        if !path.exists() {
            return Ok(String::new());
        }
        Ok(hash_bytes(&fs::read(path).map_err(err)?))
    }

    pub fn write_entity(&self, page: &EntityPage) -> Result<String, String> {
        safe_component(&page.slug)?;
        let body = serialize_entity(page)?;
        atomic_write(&self.entity_path(&page.slug), body.as_bytes())?;
        Ok(hash_bytes(body.as_bytes()))
    }

    pub fn list_entities(&self) -> Result<Vec<EntitySummary>, String> {
        let mut out = Vec::new();
        for slug in self.entity_slugs()? {
            if let Some(p) = self.read_entity(&slug)? {
                out.push(EntitySummary {
                    slug,
                    title: p.title,
                    kind: p.kind,
                    summary: p.summary,
                    updated: p.updated,
                });
            }
        }
        out.sort_by(|a, b| a.title.cmp(&b.title));
        Ok(out)
    }

    pub fn backlinks(&self, target: &str) -> Result<Vec<String>, String> {
        let mut out = Vec::new();
        for slug in self.entity_slugs()? {
            if slug == target {
                continue;
            }
            if self
                .read_entity(&slug)?
                .is_some_and(|p| p.links.iter().any(|l| l.target_slug == target))
            {
                out.push(slug);
            }
        }
        Ok(out)
    }

    pub fn graph(&self) -> Result<GraphData, String> {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        let mut seen = HashSet::new();
        for slug in self.entity_slugs()? {
            if let Some(page) = self.read_entity(&slug)? {
                nodes.push(GraphNode {
                    slug: slug.clone(),
                    title: page.title,
                    kind: page.kind,
                });
                for link in page.links {
                    let mut pair = [slug.clone(), link.target_slug.clone()];
                    pair.sort();
                    let key = pair.join("|");
                    if seen.insert(key) {
                        edges.push(GraphEdge {
                            source: pair[0].clone(),
                            target: pair[1].clone(),
                        });
                    }
                }
            }
        }
        let valid: HashSet<_> = nodes.iter().map(|n| n.slug.as_str()).collect();
        edges.retain(|e| valid.contains(e.source.as_str()) && valid.contains(e.target.as_str()));
        Ok(GraphData { nodes, edges })
    }

    pub fn append_log(&self, line: &str) -> Result<(), String> {
        use std::io::Write;
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.root.join("log.md"))
            .map_err(err)?;
        writeln!(file, "{}", line.trim_end()).map_err(err)
    }

    pub fn read_config(&self) -> Result<AppConfig, String> {
        let path = self.root.join("config.json");
        if !path.exists() {
            return Ok(AppConfig::default());
        }
        serde_json::from_slice(&fs::read(path).map_err(err)?).map_err(err)
    }
    pub fn write_config(&self, config: &AppConfig) -> Result<(), String> {
        atomic_write(
            &self.root.join("config.json"),
            &serde_json::to_vec_pretty(config).map_err(err)?,
        )
    }

    pub fn git(&self, args: &[&str]) -> Result<String, String> {
        let output = Command::new("git")
            .arg("-C")
            .arg(&self.root)
            .args(args)
            .output()
            .map_err(err)?;
        let text = format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
        .trim()
        .to_string();
        if output.status.success() {
            Ok(text)
        } else {
            Err(text)
        }
    }
}

pub fn hash_bytes(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}
pub fn normalize_space(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn serialize_entity(p: &EntityPage) -> Result<String, String> {
    let fm = EntityFrontmatter {
        slug: p.slug.clone(),
        title: p.title.clone(),
        kind: p.kind.clone(),
        aliases: p.aliases.clone(),
        sources: p.sources.clone(),
        updated: p.updated.clone(),
    };
    let yaml = serde_yaml::to_string(&fm).map_err(err)?;
    let mut out = format!(
        "---\n{}---\n\n## Summary\n{}\n\n## Facts\n",
        yaml,
        if p.summary.is_empty() {
            "(pending)"
        } else {
            &p.summary
        }
    );
    let mut footnotes = Vec::new();
    for fact in &p.facts {
        out.push_str(&fact_line(fact, "", &mut footnotes));
        out.push('\n');
    }
    if p.facts.is_empty() {
        out.push_str("(none)\n");
    }
    if !p.history.is_empty() {
        out.push_str("\n## History\n");
        for h in &p.history {
            let fact = FactEntry {
                id: h.id.clone(),
                text: h.text.clone(),
                prov: h.prov.clone(),
            };
            out.push_str(&fact_line(
                &fact,
                &format!("(superseded {}) ", h.superseded_at),
                &mut footnotes,
            ));
            out.push('\n');
        }
    }
    if !p.contradictions.is_empty() {
        out.push_str("\n## Contradictions\n");
        for c in &p.contradictions {
            out.push_str(&format!("- ⚠ {}\n", c.note));
        }
    }
    if !p.links.is_empty() {
        out.push_str("\n## Related\n");
        out.push_str(
            &p.links
                .iter()
                .map(|l| {
                    format!(
                        "[[{}]]{}",
                        l.target_slug,
                        l.relation
                            .as_ref()
                            .map(|r| format!(" ({r})"))
                            .unwrap_or_default()
                    )
                })
                .collect::<Vec<_>>()
                .join(" · "),
        );
        out.push('\n');
    }
    out.push_str("\n## Sources\n");
    out.push_str(
        &p.sources
            .iter()
            .map(|s| format!("[[{s}]]"))
            .collect::<Vec<_>>()
            .join(" · "),
    );
    out.push_str("\n\n");
    out.push_str(&footnotes.join("\n"));
    out.push('\n');
    Ok(out)
}

fn fact_line(f: &FactEntry, prefix: &str, defs: &mut Vec<String>) -> String {
    defs.push(format!(
        "[^p_{}]: src:{}@{}-{}",
        f.id, f.prov.source_id, f.prov.start, f.prov.end
    ));
    format!("- {prefix}{} [^p_{}] <!--fact:{}-->", f.text, f.id, f.id)
}

fn parse_entity(slug: &str, body: &str) -> Result<EntityPage, String> {
    let rest = body
        .strip_prefix("---\n")
        .ok_or("invalid entity frontmatter")?;
    let split = rest.find("\n---\n").ok_or("invalid entity frontmatter")?;
    let fm: EntityFrontmatter = serde_yaml::from_str(&rest[..split]).map_err(err)?;
    let content = &rest[split + 5..];
    let mut prov = HashMap::new();
    let foot_re =
        regex::Regex::new(r"(?m)^\[\^p_([^\]]+)\]: src:([^@]+)@(\d+)-(\d+)$").map_err(err)?;
    for c in foot_re.captures_iter(content) {
        prov.insert(
            c[1].to_string(),
            Provenance {
                source_id: c[2].trim().to_string(),
                start: c[3].parse().unwrap_or(0),
                end: c[4].parse().unwrap_or(0),
            },
        );
    }
    let fact_re = regex::Regex::new(
        r"^- (?:(?:\(superseded ([^)]+)\) )?)(.*?) \[\^p_([^\]]+)\] <!--fact:([^>]+)-->",
    )
    .map_err(err)?;
    let link_re = regex::Regex::new(r"\[\[([^\]]+)\]\](?: \(([^)]+)\))?").map_err(err)?;
    let mut section = "";
    let mut summary = String::new();
    let mut facts = Vec::new();
    let mut history = Vec::new();
    let mut contradictions = Vec::new();
    let mut links = Vec::new();
    for line in content.lines() {
        if let Some(name) = line.strip_prefix("## ") {
            section = name.trim();
            continue;
        }
        if section == "Summary"
            && summary.is_empty()
            && !line.trim().is_empty()
            && line.trim() != "(pending)"
        {
            summary = line.trim().to_string();
        }
        if let Some(c) = fact_re.captures(line) {
            let id = c[4].to_string();
            let p = prov.get(&id).cloned().unwrap_or(Provenance {
                source_id: "?".into(),
                start: 0,
                end: 0,
            });
            if section == "Facts" {
                facts.push(FactEntry {
                    id,
                    text: c[2].trim().to_string(),
                    prov: p,
                });
            } else if section == "History" {
                history.push(HistoryEntry {
                    id,
                    text: c[2].trim().to_string(),
                    prov: p,
                    superseded_at: c.get(1).map(|x| x.as_str().to_string()).unwrap_or_default(),
                });
            }
        }
        if section == "Contradictions" {
            if let Some(note) = line.strip_prefix("- ⚠ ") {
                contradictions.push(Contradiction { note: note.into() });
            }
        }
        if section == "Related" {
            for c in link_re.captures_iter(line) {
                links.push(LinkEntry {
                    target_slug: c[1].into(),
                    relation: c.get(2).map(|x| x.as_str().into()),
                });
            }
        }
    }
    Ok(EntityPage {
        slug: slug.into(),
        title: fm.title,
        kind: fm.kind,
        aliases: fm.aliases,
        sources: fm.sources,
        updated: fm.updated,
        summary,
        facts,
        history,
        contradictions,
        links,
    })
}

fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let parent = path.parent().ok_or("file has no parent directory")?;
    fs::create_dir_all(parent).map_err(err)?;
    let mut temporary = tempfile::NamedTempFile::new_in(parent).map_err(err)?;
    temporary.write_all(bytes).map_err(err)?;
    temporary.as_file().sync_all().map_err(err)?;
    temporary.persist(path).map_err(err)?;
    Ok(())
}

fn list_dirs(path: &Path) -> Result<Vec<String>, String> {
    let mut out = Vec::new();
    for entry in fs::read_dir(path).map_err(err)? {
        let entry = entry.map_err(err)?;
        if entry.path().is_dir() {
            if let Some(s) = entry.file_name().to_str() {
                out.push(s.into());
            }
        }
    }
    out.sort();
    Ok(out)
}
fn safe_component(value: &str) -> Result<(), String> {
    if value.is_empty()
        || value.contains('/')
        || value.contains('\\')
        || value == "."
        || value == ".."
    {
        Err("invalid path component".into())
    } else {
        Ok(())
    }
}
fn err<E: std::fmt::Display>(e: E) -> String {
    e.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn entity_round_trip_preserves_provenance() {
        let page = EntityPage {
            slug: "测试".into(),
            title: "测试".into(),
            kind: "concept".into(),
            aliases: vec!["Test".into()],
            sources: vec!["S1".into()],
            updated: "2026-01-01".into(),
            summary: "摘要".into(),
            facts: vec![FactEntry {
                id: "f1".into(),
                text: "事实".into(),
                prov: Provenance {
                    source_id: "S1".into(),
                    start: 1,
                    end: 3,
                },
            }],
            history: vec![],
            contradictions: vec![],
            links: vec![],
        };
        let body = serialize_entity(&page).unwrap();
        let parsed = parse_entity("测试", &body).unwrap();
        assert_eq!(parsed.facts[0].prov.start, 1);
        assert_eq!(parsed.title, "测试");
    }
    #[test]
    fn initialization_preserves_custom_gitignore_rules() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join(".gitignore"), "private-notes/\n").unwrap();
        Vault::new(dir.path().into()).unwrap();
        let ignore = fs::read_to_string(dir.path().join(".gitignore")).unwrap();
        assert!(ignore.contains("private-notes/"));
        assert!(ignore.contains("config.json"));
    }

    #[test]
    fn incomplete_source_snapshot_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let vault = Vault::new(dir.path().into()).unwrap();
        fs::create_dir_all(vault.raw_dir("Sbroken")).unwrap();
        fs::write(vault.raw_dir("Sbroken").join("content.md"), "partial").unwrap();
        let meta = SourceMeta {
            id: "Sbroken".into(),
            kind: "text".into(),
            title: "x".into(),
            author: None,
            url: None,
            fetched_at: "x".into(),
            content_hash: hash_bytes(b"partial"),
            word_count: 1,
            doi: None,
            authors: None,
            published: None,
            summarized: None,
        };
        assert!(vault
            .write_raw(&meta, "partial", None)
            .unwrap_err()
            .contains("incomplete"));
    }

    #[test]
    fn utf16_provenance_handles_emoji() {
        let dir = tempfile::tempdir().unwrap();
        let v = Vault::new(dir.path().into()).unwrap();
        let meta = SourceMeta {
            id: "S1".into(),
            kind: "text".into(),
            title: "x".into(),
            author: None,
            url: None,
            fetched_at: "x".into(),
            content_hash: "x".into(),
            word_count: 1,
            doi: None,
            authors: None,
            published: None,
            summarized: None,
        };
        v.write_raw(&meta, "a😀中", None).unwrap();
        assert_eq!(v.read_raw_utf16_span("S1", 1, 3).unwrap(), "😀");
    }
}
