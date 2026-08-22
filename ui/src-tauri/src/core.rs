use crate::ai::{jaccard, Llm, MockProvider, OpenAiProvider, Provider};
use crate::domain::*;
use crate::vault::{hash_bytes, normalize_space, Vault};
use chrono::Utc;
use scraper::{Html, Selector};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;
use std::sync::{Mutex, RwLock};
use std::time::Duration;
use url::Url;
use uuid::Uuid;

pub struct MnemeCore {
    pub vault: Vault,
    config: RwLock<AppConfig>,
    writer: Mutex<()>,
}

impl MnemeCore {
    pub fn new(vault_root: PathBuf, _config_dir: PathBuf) -> Result<Self, String> {
        let vault = Vault::new(vault_root)?;
        let config = vault.read_config()?;
        Ok(Self {
            vault,
            config: RwLock::new(config),
            writer: Mutex::new(()),
        })
    }

    pub fn config_view(&self) -> Result<AppConfigView, String> {
        let c = self
            .config
            .read()
            .map_err(|_| "config lock poisoned")?
            .clone();
        let provider = c.provider.unwrap_or_else(|| "mock".into());
        Ok(AppConfigView {
            has_key: self.read_key(&provider)?.is_some(),
            provider,
            model: c.model,
            base_url: c.base_url,
            retriever: "builtin".into(),
            sync_remote: c.sync_remote,
            vault: self.vault.root.to_string_lossy().into(),
            providers: vec![
                "openai".into(),
                "chatglm".into(),
                "deepseek".into(),
                "qwen".into(),
                "llamacpp".into(),
                "custom".into(),
                "mock".into(),
            ],
        })
    }

    pub fn set_config(&self, patch: ConfigPatch) -> Result<AppConfigView, String> {
        let _requested_retriever = patch.retriever.as_deref();
        let provider_for_key = patch
            .provider
            .clone()
            .or_else(|| self.config.read().ok()?.provider.clone())
            .unwrap_or_else(|| "mock".into());
        if let Some(key) = patch.api_key {
            let entry = keyring::Entry::new("studio.catvinci.mneme", &provider_for_key)
                .map_err(|e| e.to_string())?;
            if key.is_empty() {
                if let Err(error) = entry.delete_credential() {
                    if !matches!(error, keyring::Error::NoEntry) {
                        return Err(error.to_string());
                    }
                }
            } else {
                entry.set_password(&key).map_err(|e| e.to_string())?;
            }
        }
        {
            let mut c = self.config.write().map_err(|_| "config lock poisoned")?;
            if patch.provider.is_some() {
                c.provider = patch.provider;
            }
            if let Some(value) = patch.model {
                c.model = non_empty(value);
            }
            if let Some(value) = patch.base_url {
                c.base_url = non_empty(value);
            }
            if let Some(value) = patch.sync_remote {
                c.sync_remote = non_empty(value);
            }
            c.retriever = Some("builtin".into());
            self.vault.write_config(&c)?;
        }
        self.config_view()
    }

    fn provider(&self) -> Result<Provider, String> {
        let c = self
            .config
            .read()
            .map_err(|_| "config lock poisoned")?
            .clone();
        let name = c.provider.unwrap_or_else(|| "mock".into());
        if name == "mock" {
            return Ok(Provider::Mock(MockProvider));
        }
        let presets: HashMap<&str, (&str, &str)> = HashMap::from([
            ("openai", ("https://api.openai.com/v1", "gpt-4o")),
            ("deepseek", ("https://api.deepseek.com/v1", "deepseek-chat")),
            (
                "chatglm",
                ("https://open.bigmodel.cn/api/paas/v4", "glm-4-plus"),
            ),
            (
                "qwen",
                (
                    "https://dashscope.aliyuncs.com/compatible-mode/v1",
                    "qwen-plus",
                ),
            ),
            ("llamacpp", ("http://127.0.0.1:8080/v1", "local")),
            ("custom", ("", "")),
        ]);
        let (default_url, default_model) = presets
            .get(name.as_str())
            .copied()
            .ok_or("unknown provider")?;
        let url = c
            .base_url
            .filter(|x| !x.trim().is_empty())
            .unwrap_or_else(|| default_url.into());
        let model = c
            .model
            .filter(|x| !x.trim().is_empty())
            .unwrap_or_else(|| default_model.into());
        if url.is_empty() || model.is_empty() {
            return Err("provider base URL and model are required".into());
        }
        let key = self.read_key(&name)?;
        if name != "llamacpp" && name != "custom" && key.is_none() {
            return Err(format!("{name} API key is not configured"));
        }
        Ok(Provider::OpenAi(OpenAiProvider::new(
            name, url, key, model,
        )?))
    }

    pub async fn add_source(&self, input: AddSourceInput) -> Result<AddSourceResult, String> {
        let normalized = normalize_input(&input).await?;
        let content_hash = hash_bytes(normalized.content.as_bytes());
        let identity = normalized.url.as_deref().unwrap_or(&normalized.title);
        let digest = hash_bytes(format!("{identity}:{content_hash}").as_bytes());
        let id = format!("S{}", &digest[..10]);
        let meta = SourceMeta {
            id: id.clone(),
            kind: normalized.kind,
            title: normalized.title,
            author: normalized.author,
            url: normalized.url,
            fetched_at: today(),
            content_hash,
            word_count: normalized.content.split_whitespace().count(),
            doi: normalized.doi,
            authors: normalized.authors,
            published: normalized.published,
            summarized: None,
        };
        self.vault
            .write_raw(&meta, &normalized.content, normalized.original.as_deref())?;
        let provider = self.provider()?;
        let note = provider.note(&meta.title, &normalized.content).await?;
        self.vault.write_note(&id, &render_note(&meta, &note))?;
        self.vault.append_log(&format!(
            "[ingest] {id} \"{}\" ({}) — {} entities",
            meta.title,
            meta.kind,
            note.candidate_entities.len()
        ))?;
        let report = self
            .wikify_with(&id, &note.candidate_entities, &provider)
            .await?;
        self.commit_git(&format!("ingest: {}", meta.title));
        Ok(AddSourceResult {
            source_id: id,
            meta,
            report,
        })
    }

    pub async fn wikify_source(&self, id: &str) -> Result<WikifyReport, String> {
        if self.vault.read_meta(id)?.is_none() {
            return Err("source not found".into());
        }
        let p = self.provider()?;
        self.wikify_with(id, &[], &p).await
    }

    async fn wikify_with(
        &self,
        id: &str,
        candidates: &[CandidateEntity],
        provider: &Provider,
    ) -> Result<WikifyReport, String> {
        let content = self.vault.read_raw(id)?;
        let extracted = provider.extract(&content, candidates).await?;
        let kind_of: HashMap<_, _> = extracted
            .entities
            .iter()
            .map(|e| (e.name.clone(), e.kind.clone()))
            .collect();
        let aliases: HashMap<_, _> = extracted
            .entities
            .iter()
            .map(|e| (e.name.clone(), e.aliases.clone()))
            .collect();
        let mut groups: BTreeMap<String, Vec<Claim>> = BTreeMap::new();
        for item in extracted.claims {
            if let Some(byte_start) = content.find(&item.span) {
                let start = content[..byte_start].encode_utf16().count();
                let end = start + item.span.encode_utf16().count();
                let cid = format!(
                    "c{}",
                    &hash_bytes(format!("{id}:{}", item.span).as_bytes())[..12]
                );
                groups.entry(item.entity_name).or_default().push(Claim {
                    id: cid,
                    text: item.text,
                    span: item.span,
                    source_id: id.into(),
                    char_start: start,
                    char_end: end,
                    confidence: item.confidence,
                });
            }
        }
        let mut report = WikifyReport {
            source_id: id.into(),
            entities: vec![],
        };
        let mut touched = Vec::new();
        for (name, claims) in groups {
            let slug = self
                .locate(&name, aliases.get(&name).map(Vec::as_slice).unwrap_or(&[]))?
                .unwrap_or_else(|| slugify(&name));
            let mut status = "noop".to_string();
            let mut op_count = 0;
            for _ in 0..3 {
                let page = self.vault.read_entity(&slug)?;
                let related = self.related_facts(&claims, &slug)?;
                let input = ReconcileInput {
                    slug: slug.clone(),
                    title: name.clone(),
                    kind: kind_of
                        .get(&name)
                        .cloned()
                        .unwrap_or_else(|| "concept".into()),
                    exists: page.is_some(),
                    current_facts: page
                        .as_ref()
                        .map(|p| {
                            p.facts
                                .iter()
                                .map(|f| (f.id.clone(), f.text.clone()))
                                .collect()
                        })
                        .unwrap_or_default(),
                    new_claims: claims.clone(),
                    related_facts: related,
                };
                let mut ops = provider.reconcile(&input).await?;
                if let Some(PatchOp::CreatePage { entity, .. }) = ops
                    .iter_mut()
                    .find(|o| matches!(o, PatchOp::CreatePage { .. }))
                {
                    entity.aliases = aliases.get(&name).cloned().unwrap_or_default();
                }
                op_count = ops.len();
                if ops.is_empty() {
                    status = "dedup/noop".into();
                    break;
                }
                match self.commit_routed(&slug, ops, &today())? {
                    CommitStatus::Ok => {
                        status = "committed".into();
                        touched.push(slug.clone());
                        break;
                    }
                    CommitStatus::Stale => status = "retry-stale".into(),
                    CommitStatus::Invalid(x) => {
                        status = format!("invalid-claim {x}");
                        break;
                    }
                }
            }
            self.vault.append_log(&format!(
                "[wikify] {id} → [[{slug}]] ({op_count} ops, {status})"
            ))?;
            report.entities.push(WikifyEntityReport {
                slug,
                ops: op_count,
                result: status,
            });
        }
        touched.sort();
        touched.dedup();
        self.cross_link(&touched, &today())?;
        self.commit_git(&format!("wikify: {id}"));
        Ok(report)
    }

    fn commit_routed(
        &self,
        current: &str,
        ops: Vec<PatchOp>,
        now: &str,
    ) -> Result<CommitStatus, String> {
        let mut groups: BTreeMap<String, Vec<PatchOp>> = BTreeMap::new();
        for op in ops {
            let target = match &op {
                PatchOp::UpdateFact { target_slug, .. }
                | PatchOp::SupersedeFact { target_slug, .. }
                | PatchOp::FlagContradiction { target_slug, .. } => target_slug.clone(),
                _ => None,
            }
            .unwrap_or_else(|| current.into());
            groups.entry(target).or_default().push(op);
        }
        for (slug, ops) in groups {
            let base = self.vault.entity_hash(&slug)?;
            match self.commit_page(&slug, &base, ops, now)? {
                CommitStatus::Ok => {}
                other => return Ok(other),
            }
        }
        Ok(CommitStatus::Ok)
    }

    fn commit_page(
        &self,
        slug: &str,
        base: &str,
        ops: Vec<PatchOp>,
        now: &str,
    ) -> Result<CommitStatus, String> {
        let _guard = self.writer.lock().map_err(|_| "writer lock poisoned")?;
        if self.vault.entity_hash(slug)? != base {
            return Ok(CommitStatus::Stale);
        }
        for op in &ops {
            if let Some(c) = claim_of(op) {
                let raw = self
                    .vault
                    .read_raw_utf16_span(&c.source_id, c.char_start, c.char_end)?;
                if normalize_space(&raw) != normalize_space(&c.span) {
                    return Ok(CommitStatus::Invalid(c.id.clone()));
                }
            }
        }
        let mut page = self.vault.read_entity(slug)?.unwrap_or_else(|| EntityPage {
            slug: slug.into(),
            title: slug.into(),
            kind: "concept".into(),
            aliases: vec![],
            sources: vec![],
            updated: now.into(),
            summary: String::new(),
            facts: vec![],
            history: vec![],
            contradictions: vec![],
            links: vec![],
        });
        for op in ops {
            apply_op(&mut page, op, now);
        }
        if page.summary.is_empty() || page.summary.to_lowercase().contains("creating page") {
            page.summary = page
                .facts
                .first()
                .map(|f| f.text.clone())
                .unwrap_or_default();
        }
        page.updated = now.into();
        self.vault.write_entity(&page)?;
        Ok(CommitStatus::Ok)
    }

    fn cross_link(&self, slugs: &[String], now: &str) -> Result<(), String> {
        if slugs.len() < 2 {
            return Ok(());
        }
        for slug in slugs {
            let Some(page) = self.vault.read_entity(slug)? else {
                continue;
            };
            let existing: HashSet<_> = page.links.iter().map(|l| l.target_slug.as_str()).collect();
            let ops = slugs
                .iter()
                .filter(|s| *s != slug && !existing.contains(s.as_str()))
                .map(|s| PatchOp::AddLink {
                    target_slug: s.clone(),
                    relation: None,
                })
                .collect::<Vec<_>>();
            if !ops.is_empty() {
                let base = self.vault.entity_hash(slug)?;
                let _ = self.commit_page(slug, &base, ops, now)?;
            }
        }
        Ok(())
    }

    fn locate(&self, name: &str, aliases: &[String]) -> Result<Option<String>, String> {
        let target = slugify(name);
        let slugs = self.vault.entity_slugs()?;
        if slugs.contains(&target) {
            return Ok(Some(target));
        }
        for alias in aliases {
            let s = slugify(alias);
            if slugs.contains(&s) {
                return Ok(Some(s));
            }
        }
        for slug in slugs {
            if let Some(p) = self.vault.read_entity(&slug)? {
                if p.title.eq_ignore_ascii_case(name)
                    || p.aliases.iter().any(|a| a.eq_ignore_ascii_case(name))
                {
                    return Ok(Some(slug));
                }
            }
        }
        Ok(None)
    }
    fn related_facts(&self, claims: &[Claim], exclude: &str) -> Result<Vec<RelatedFact>, String> {
        let query = claims
            .iter()
            .map(|c| c.text.as_str())
            .collect::<Vec<_>>()
            .join(" ");
        let mut out = Vec::new();
        for hit in self.search(&query, 6)? {
            if hit.slug == exclude {
                continue;
            }
            if let Some(p) = self.vault.read_entity(&hit.slug)? {
                for f in p.facts {
                    out.push(RelatedFact {
                        slug: hit.slug.clone(),
                        fact_id: f.id,
                        text: f.text,
                    });
                }
            }
        }
        if out.is_empty() {
            for slug in self
                .vault
                .entity_slugs()?
                .into_iter()
                .filter(|s| s != exclude)
            {
                if let Some(p) = self.vault.read_entity(&slug)? {
                    for f in p.facts {
                        out.push(RelatedFact {
                            slug: slug.clone(),
                            fact_id: f.id,
                            text: f.text,
                        });
                    }
                }
            }
        }
        out.truncate(12);
        Ok(out)
    }

    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchHit>, String> {
        let q = query.trim();
        if q.is_empty() {
            return Ok(vec![]);
        }
        let lower = q.to_lowercase();
        let mut hits = Vec::new();
        for e in self.vault.list_entities()? {
            let p = self.vault.read_entity(&e.slug)?.unwrap();
            let blob = format!(
                "{} {} {}",
                p.title,
                p.summary,
                p.facts
                    .iter()
                    .map(|f| f.text.as_str())
                    .collect::<Vec<_>>()
                    .join(" ")
            );
            let mut score = jaccard(q, &blob);
            if blob.to_lowercase().contains(&lower) {
                score += 1.0
            }
            if score > 0.0 {
                hits.push(SearchHit {
                    slug: e.slug,
                    title: e.title,
                    snippet: if e.summary.is_empty() {
                        p.facts.first().map(|f| f.text.clone()).unwrap_or_default()
                    } else {
                        e.summary
                    },
                    score,
                });
            }
        }
        hits.sort_by(|a, b| b.score.total_cmp(&a.score));
        hits.truncate(limit);
        Ok(hits)
    }

    pub async fn research(&self, question: &str) -> Result<ResearchResult, String> {
        let hits = self.search(question, 6)?;
        let mut context = Vec::new();
        for h in &hits {
            if let Some(p) = self.vault.read_entity(&h.slug)? {
                let facts = p
                    .facts
                    .iter()
                    .map(|f| {
                        format!(
                            "- {} (src:{}@{}-{})",
                            f.text, f.prov.source_id, f.prov.start, f.prov.end
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                context.push(format!(
                    "[[{}]] {}\n{}\n{}",
                    h.slug, p.title, p.summary, facts
                ));
            }
        }
        let answer = self
            .provider()?
            .answer(question, &context.join("\n\n"))
            .await?;
        self.vault
            .append_log(&format!("[query] \"{}\" — {} hits", question, hits.len()))?;
        Ok(ResearchResult {
            answer,
            sources: hits
                .into_iter()
                .map(|h| ResearchSource {
                    slug: h.slug,
                    title: h.title,
                })
                .collect(),
        })
    }

    pub fn lint(&self) -> Result<LintReport, String> {
        let slugs = self.vault.entity_slugs()?;
        let mut linked = HashSet::new();
        let mut pages = Vec::new();
        let mut facts = 0;
        let mut empty = Vec::new();
        let mut contradictions = Vec::new();
        for slug in slugs {
            if let Some(p) = self.vault.read_entity(&slug)? {
                facts += p.facts.len();
                if p.facts.is_empty() {
                    empty.push(slug.clone())
                }
                if !p.contradictions.is_empty() {
                    contradictions.push(LintContradiction {
                        slug: slug.clone(),
                        count: p.contradictions.len(),
                    })
                }
                for l in &p.links {
                    linked.insert(l.target_slug.clone());
                }
                pages.push((slug, p));
            }
        }
        let orphans = pages
            .iter()
            .filter(|(s, p)| p.links.is_empty() && !linked.contains(s))
            .map(|x| x.0.clone())
            .collect();
        let r = LintReport {
            entities: pages.len(),
            sources: self.vault.source_ids()?.len(),
            facts,
            orphans,
            empty_facts: empty,
            contradictions,
        };
        self.vault.append_log(&format!(
            "[lint] {} entities, {} orphans",
            r.entities,
            r.orphans.len()
        ))?;
        Ok(r)
    }

    pub fn sync(&self) -> Result<SyncResult, String> {
        let remote = self
            .config
            .read()
            .map_err(|_| "config lock poisoned")?
            .sync_remote
            .clone();
        let Some(remote) = remote.filter(|x| !x.trim().is_empty()) else {
            return Ok(SyncResult {
                ok: false,
                message: "未配置远端".into(),
            });
        };
        if let Err(error) = self.vault.git(&["add", "-A"]) {
            return Ok(SyncResult {
                ok: false,
                message: format!("git add failed: {error}"),
            });
        }
        let pending = match self.vault.git(&["status", "--porcelain"]) {
            Ok(value) => value,
            Err(error) => {
                return Ok(SyncResult {
                    ok: false,
                    message: format!("git status failed: {error}"),
                })
            }
        };
        if !pending.is_empty() {
            if let Err(error) =
                self.vault
                    .git(&["commit", "-m", &format!("sync {}", today()), "--quiet"])
            {
                return Ok(SyncResult {
                    ok: false,
                    message: format!("git commit failed: {error}"),
                });
            }
        }
        let _ = self.vault.git(&["remote", "remove", "origin"]);
        if let Err(e) = self.vault.git(&["remote", "add", "origin", &remote]) {
            return Ok(SyncResult {
                ok: false,
                message: e,
            });
        }
        match self.vault.git(&["push", "-u", "origin", "HEAD", "--quiet"]) {
            Ok(_) => Ok(SyncResult {
                ok: true,
                message: "已推送备份".into(),
            }),
            Err(e) => Ok(SyncResult {
                ok: false,
                message: e.chars().take(300).collect(),
            }),
        }
    }

    fn read_key(&self, provider: &str) -> Result<Option<String>, String> {
        let entry =
            keyring::Entry::new("studio.catvinci.mneme", provider).map_err(|e| e.to_string())?;
        match entry.get_password() {
            Ok(value) => Ok(Some(value)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(error) => Err(error.to_string()),
        }
    }
    fn commit_git(&self, message: &str) {
        let _ = self.vault.git(&["add", "-A"]);
        let _ = self.vault.git(&["commit", "-m", message, "--quiet"]);
    }
}

enum CommitStatus {
    Ok,
    Stale,
    Invalid(String),
}
fn claim_of(op: &PatchOp) -> Option<&Claim> {
    match op {
        PatchOp::AppendFact { claim, .. }
        | PatchOp::UpdateFact { claim, .. }
        | PatchOp::SupersedeFact { claim, .. } => Some(claim),
        PatchOp::FlagContradiction { conflicting, .. } => Some(conflicting),
        _ => None,
    }
}
fn apply_op(page: &mut EntityPage, op: PatchOp, now: &str) {
    match op {
        PatchOp::CreatePage { entity, summary } => {
            page.title = entity.name;
            page.kind = entity.kind;
            for a in entity.aliases {
                if !page.aliases.contains(&a) {
                    page.aliases.push(a)
                }
            }
            if page.summary.is_empty() {
                page.summary = summary
            }
        }
        PatchOp::AppendFact { fact, claim } => {
            page.facts.push(mk_fact(&claim, Some(fact)));
            add_source(page, &claim.source_id)
        }
        PatchOp::UpdateFact {
            fact_id,
            new_fact,
            claim,
            ..
        } => {
            if let Some(f) = page.facts.iter_mut().find(|f| f.id == fact_id) {
                f.text = new_fact;
                f.prov = prov(&claim)
            } else {
                page.facts.push(mk_fact(&claim, Some(new_fact)))
            }
            add_source(page, &claim.source_id)
        }
        PatchOp::SupersedeFact {
            fact_id,
            new_fact,
            claim,
            ..
        } => {
            if let Some(i) = page.facts.iter().position(|f| f.id == fact_id) {
                let f = page.facts.remove(i);
                page.history.push(HistoryEntry {
                    id: f.id,
                    text: f.text,
                    prov: f.prov,
                    superseded_at: now.into(),
                })
            }
            page.facts.push(mk_fact(&claim, Some(new_fact)));
            add_source(page, &claim.source_id)
        }
        PatchOp::FlagContradiction {
            fact_id,
            conflicting,
            note,
            ..
        } => {
            let existing = page
                .facts
                .iter()
                .find(|f| f.id == fact_id)
                .map(|f| format!("\"{}\"", f.text))
                .unwrap_or_else(|| format!("fact {fact_id}"));
            page.contradictions.push(Contradiction {
                note: format!(
                    "{existing} ↔ \"{note}\" (src:{}). 待人工裁决。",
                    conflicting.source_id
                ),
            });
            add_source(page, &conflicting.source_id)
        }
        PatchOp::AddLink {
            target_slug,
            relation,
        } => {
            if !page.links.iter().any(|l| l.target_slug == target_slug) {
                page.links.push(LinkEntry {
                    target_slug,
                    relation,
                })
            }
        }
    }
}
fn prov(c: &Claim) -> Provenance {
    Provenance {
        source_id: c.source_id.clone(),
        start: c.char_start,
        end: c.char_end,
    }
}
fn mk_fact(c: &Claim, text: Option<String>) -> FactEntry {
    FactEntry {
        id: format!("f{}", Uuid::new_v4().simple()),
        text: text.unwrap_or_else(|| c.text.clone()),
        prov: prov(c),
    }
}
fn add_source(p: &mut EntityPage, id: &str) {
    if !p.sources.iter().any(|x| x == id) {
        p.sources.push(id.into())
    }
}
fn slugify(s: &str) -> String {
    let mut out = String::new();
    let mut dash = false;
    for c in s.to_lowercase().chars() {
        if c.is_alphanumeric() {
            out.push(c);
            dash = false
        } else if !dash && !out.is_empty() {
            out.push('-');
            dash = true
        }
    }
    let slug = out.trim_matches('-').to_string();
    if slug.is_empty() {
        format!("entity-{}", &hash_bytes(s.as_bytes())[..10])
    } else {
        slug
    }
}
fn non_empty(value: String) -> Option<String> {
    let trimmed = value.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}
fn today() -> String {
    Utc::now().format("%Y-%m-%d").to_string()
}

struct Normalized {
    kind: String,
    title: String,
    author: Option<String>,
    url: Option<String>,
    content: String,
    original: Option<String>,
    doi: Option<String>,
    authors: Option<Vec<String>>,
    published: Option<String>,
}
async fn normalize_input(input: &AddSourceInput) -> Result<Normalized, String> {
    if let Some(url) = input
        .url
        .as_deref()
        .map(str::trim)
        .filter(|x| !x.is_empty())
    {
        fetch_url(url, input.title.clone()).await
    } else if let Some(text) = input
        .text
        .as_deref()
        .map(str::trim)
        .filter(|x| !x.is_empty())
    {
        Ok(Normalized {
            kind: "text".into(),
            title: input
                .title
                .clone()
                .filter(|x| !x.trim().is_empty())
                .unwrap_or_else(|| "Untitled".into()),
            author: None,
            url: None,
            content: text.into(),
            original: None,
            doi: None,
            authors: None,
            published: None,
        })
    } else {
        Err("url or text is required".into())
    }
}
async fn fetch_url(value: &str, title: Option<String>) -> Result<Normalized, String> {
    let mut url = Url::parse(value).map_err(|e| e.to_string())?;
    let mut response = {
        let mut final_response = None;
        for _ in 0..=5 {
            let (host, address) = resolve_public_url(&url).await?;
            // Pin this request to the exact address that passed the public-IP check.
            // This prevents a second DNS lookup from being swapped to a private address.
            let client = reqwest::Client::builder()
                .timeout(Duration::from_secs(30))
                .redirect(reqwest::redirect::Policy::none())
                .resolve(&host, address)
                .build()
                .map_err(|e| e.to_string())?;
            let current = client
                .get(url.clone())
                .send()
                .await
                .map_err(|e| e.to_string())?;
            if current.status().is_redirection() {
                let location = current
                    .headers()
                    .get(reqwest::header::LOCATION)
                    .and_then(|value| value.to_str().ok())
                    .ok_or("redirect has no Location header")?;
                url = url.join(location).map_err(|e| e.to_string())?;
                continue;
            }
            final_response = Some(current);
            break;
        }
        final_response.ok_or("too many redirects")?
    };
    if !response.status().is_success() {
        return Err(format!("fetch failed: {}", response.status()));
    }
    let host = url.host_str().ok_or("URL has no host")?.to_string();
    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();
    const MAX_SOURCE_BYTES: usize = 10 * 1024 * 1024;
    if response
        .content_length()
        .is_some_and(|size| size > MAX_SOURCE_BYTES as u64)
    {
        return Err("source exceeds 10 MB".into());
    }
    let mut bytes = Vec::with_capacity(
        response
            .content_length()
            .unwrap_or(0)
            .min(MAX_SOURCE_BYTES as u64) as usize,
    );
    while let Some(chunk) = response.chunk().await.map_err(|e| e.to_string())? {
        if bytes.len() + chunk.len() > MAX_SOURCE_BYTES {
            return Err("source exceeds 10 MB".into());
        }
        bytes.extend_from_slice(&chunk);
    }
    if content_type.contains("pdf") || url.path().to_lowercase().ends_with(".pdf") {
        let content = pdf_extract::extract_text_from_mem(&bytes).map_err(|e| e.to_string())?;
        return Ok(Normalized {
            kind: "pdf".into(),
            title: title.unwrap_or_else(|| {
                url.path_segments()
                    .and_then(|mut x| x.next_back())
                    .unwrap_or("PDF")
                    .into()
            }),
            author: None,
            url: Some(value.into()),
            content,
            original: None,
            doi: None,
            authors: None,
            published: None,
        });
    }
    let html = String::from_utf8_lossy(&bytes).to_string();
    let doc = Html::parse_document(&html);
    let title_text = title
        .or_else(|| {
            Selector::parse("title").ok().and_then(|s| {
                doc.select(&s)
                    .next()
                    .map(|x| x.text().collect::<String>().trim().to_string())
            })
        })
        .filter(|x| !x.is_empty())
        .unwrap_or(host);
    let selector = Selector::parse("article, main, body").map_err(|e| e.to_string())?;
    let content = doc
        .select(&selector)
        .next()
        .map(|x| {
            x.text()
                .map(str::trim)
                .filter(|x| !x.is_empty())
                .collect::<Vec<_>>()
                .join("\n\n")
        })
        .unwrap_or_default();
    if content.trim().is_empty() {
        return Err("web page contained no readable text".into());
    }
    Ok(Normalized {
        kind: "web".into(),
        title: title_text,
        author: None,
        url: Some(value.into()),
        content,
        original: Some(html),
        doi: None,
        authors: None,
        published: None,
    })
}
async fn resolve_public_url(url: &Url) -> Result<(String, SocketAddr), String> {
    if !matches!(url.scheme(), "http" | "https") {
        return Err("only http and https URLs are allowed".into());
    }
    let host = url.host_str().ok_or("URL has no host")?.to_string();
    let port = url.port_or_known_default().ok_or("URL has no known port")?;
    let addresses = tokio::net::lookup_host((host.as_str(), port))
        .await
        .map_err(|e| e.to_string())?
        .collect::<Vec<_>>();
    if addresses.is_empty() {
        return Err("URL host did not resolve".into());
    }
    if addresses.iter().any(|address| private_ip(address.ip())) {
        return Err("private or local network URLs are not allowed".into());
    }
    Ok((host, addresses[0]))
}
fn private_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(x) => {
            let octets = x.octets();
            x.is_private() || x.is_loopback() || x.is_link_local() || x.is_unspecified()
                || x.is_broadcast() || x.is_multicast() || x.is_documentation()
                || (octets[0] == 100 && (64..=127).contains(&octets[1])) // shared address space
                || (octets[0] == 198 && matches!(octets[1], 18 | 19)) // benchmark networks
        }
        IpAddr::V6(x) => {
            if let Some(mapped) = x.to_ipv4_mapped() {
                return private_ip(IpAddr::V4(mapped));
            }
            let segments = x.segments();
            let first = segments[0];
            x.is_loopback() || x.is_unspecified() || x.is_multicast()
                || first & 0xfe00 == 0xfc00 // fc00::/7 unique-local
                || first & 0xffc0 == 0xfe80 // fe80::/10 link-local
                || (segments[0] == 0x2001 && segments[1] == 0x0db8) // documentation
        }
    }
}
fn render_note(meta: &SourceMeta, n: &Note) -> String {
    format!("---\nsource: {}\ntitle: {}\nkind: {}\n---\n\n## TL;DR\n{}\n\n## Key points\n{}\n\n## Candidate entities\n{}\n\n## Tags\n{}\n",meta.id,meta.title,meta.kind,n.tldr,n.key_points.iter().map(|x|format!("- {x}")).collect::<Vec<_>>().join("\n"),n.candidate_entities.iter().map(|e|format!("- {} ({}) — {}",e.name,e.kind,e.why)).collect::<Vec<_>>().join("\n"),n.tags.iter().map(|x|format!("#{x}")).collect::<Vec<_>>().join(" "))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slug_keeps_unicode() {
        assert_eq!(slugify("人工 智能"), "人工-智能");
    }

    #[test]
    fn blocks_private_networks() {
        assert!(private_ip("127.0.0.1".parse().unwrap()));
        assert!(private_ip("192.168.1.1".parse().unwrap()));
        assert!(private_ip("::ffff:127.0.0.1".parse().unwrap()));
        assert!(!private_ip("2606:4700:4700::1111".parse().unwrap()));
    }

    #[test]
    fn empty_config_values_clear_overrides() {
        assert_eq!(non_empty("  ".into()), None);
        assert_eq!(non_empty(" model ".into()), Some("model".into()));
    }

    #[tokio::test]
    async fn mock_ingest_wikify_search_round_trip() {
        let vault = tempfile::tempdir().unwrap();
        let config = tempfile::tempdir().unwrap();
        let core = MnemeCore::new(vault.path().into(), config.path().into()).unwrap();
        let result = core
            .add_source(AddSourceInput {
                title: Some("Tesla timeline".into()),
                text: Some("Tesla was founded in 2003. Elon Musk became CEO in 2008.".into()),
                url: None,
            })
            .await
            .unwrap();
        assert!(!result.report.entities.is_empty());
        assert_eq!(core.vault.list_sources().unwrap().len(), 1);
        assert!(!core.search("Tesla", 10).unwrap().is_empty());
        assert!(core.lint().unwrap().facts > 0);
    }
}
