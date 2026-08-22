use crate::domain::*;
use async_trait::async_trait;
use once_cell::sync::Lazy;
use regex::Regex;
use serde::Deserialize;
use serde_json::json;
use std::collections::{HashMap, HashSet};

const NOTE_SYSTEM: &str = r#"You are Mneme's ingest agent. Produce a concise faithful note in the article's language. Return ONLY JSON: {"tldr":string,"key_points":[string],"candidate_entities":[{"name":string,"kind":"person|org|concept|tech|topic|event|place","why":string}],"tags":[string]}. Do not add outside knowledge."#;
const EXTRACT_SYSTEM: &str = r#"You are Mneme's claim extractor. Extract atomic verifiable facts. Every claim must contain an exact contiguous article substring in span; never infer. Preserve source language. Return ONLY JSON: {"entities":[{"name":string,"kind":"person|org|concept|tech|topic|event|place","aliases":[string]}],"claims":[{"entity_name":string,"text":string,"span":string,"confidence":number}]}"#;
const RECONCILE_SYSTEM: &str = r#"You are Mneme's reconciler. Integrate new sourced claims without destroying information. APPEND new facts, emit nothing for DEDUP, update_fact for a compatible refinement, supersede_fact for time-varying facts, flag_contradiction when both cannot be true. Never delete. Return ONLY JSON {"ops":[...]}. Ops: {"op":"create_page","summary":string}; {"op":"append_fact","claim_id":string}; {"op":"update_fact","fact_id":string,"new_fact":string,"claim_id":string,"page":string?}; {"op":"supersede_fact","fact_id":string,"new_fact":string,"claim_id":string,"page":string?}; {"op":"flag_contradiction","fact_id":string,"claim_id":string,"note":string,"page":string?}."#;
const RESEARCH_SYSTEM: &str = r#"Answer only from the supplied Mneme wiki context. Cite wiki pages as [[slug]] and source ids as (src:ID). If evidence is insufficient, say so. Answer in the question's language."#;

#[async_trait]
pub trait Llm: Send + Sync {
    async fn note(&self, title: &str, content: &str) -> Result<Note, String>;
    async fn extract(
        &self,
        content: &str,
        candidates: &[CandidateEntity],
    ) -> Result<ExtractResult, String>;
    async fn reconcile(&self, input: &ReconcileInput) -> Result<Vec<PatchOp>, String>;
    async fn answer(&self, question: &str, context: &str) -> Result<String, String>;
}

pub enum Provider {
    Mock(MockProvider),
    OpenAi(OpenAiProvider),
}
#[async_trait]
impl Llm for Provider {
    async fn note(&self, t: &str, c: &str) -> Result<Note, String> {
        match self {
            Self::Mock(p) => p.note(t, c).await,
            Self::OpenAi(p) => p.note(t, c).await,
        }
    }
    async fn extract(&self, c: &str, x: &[CandidateEntity]) -> Result<ExtractResult, String> {
        match self {
            Self::Mock(p) => p.extract(c, x).await,
            Self::OpenAi(p) => p.extract(c, x).await,
        }
    }
    async fn reconcile(&self, i: &ReconcileInput) -> Result<Vec<PatchOp>, String> {
        match self {
            Self::Mock(p) => p.reconcile(i).await,
            Self::OpenAi(p) => p.reconcile(i).await,
        }
    }
    async fn answer(&self, q: &str, c: &str) -> Result<String, String> {
        match self {
            Self::Mock(p) => p.answer(q, c).await,
            Self::OpenAi(p) => p.answer(q, c).await,
        }
    }
}

pub struct OpenAiProvider {
    name: String,
    base_url: String,
    key: Option<String>,
    model: String,
    client: reqwest::Client,
}
impl OpenAiProvider {
    pub fn new(
        name: String,
        base_url: String,
        key: Option<String>,
        model: String,
    ) -> Result<Self, String> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .map_err(|e| e.to_string())?;
        Ok(Self {
            name,
            base_url: base_url.trim_end_matches('/').into(),
            key,
            model,
            client,
        })
    }
    async fn chat(&self, system: &str, user: &str, structured: bool) -> Result<String, String> {
        let mut body = json!({
            "model": self.model,
            "temperature": 0,
            "messages": [
                {"role": "system", "content": system},
                {"role": "user", "content": user}
            ]
        });
        if structured {
            body["response_format"] = json!({"type": "json_object"});
        }
        let mut request = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .json(&body);
        if let Some(key) = &self.key {
            request = request.bearer_auth(key);
        }
        let response = request
            .send()
            .await
            .map_err(|e| format!("{}: {e}", self.name))?;
        let status = response.status();
        let text = response.text().await.map_err(|e| e.to_string())?;
        if !status.is_success() {
            return Err(format!(
                "{} {}: {}",
                self.name,
                status,
                text.chars().take(500).collect::<String>()
            ));
        }
        let value: serde_json::Value =
            serde_json::from_str(&text).map_err(|e| format!("invalid provider response: {e}"))?;
        value
            .pointer("/choices/0/message/content")
            .and_then(|v| v.as_str())
            .map(str::to_string)
            .ok_or("provider returned no content".into())
    }
    fn json<T: serde::de::DeserializeOwned>(text: &str) -> Result<T, String> {
        let trimmed = text
            .trim()
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim();
        serde_json::from_str(trimmed).map_err(|e| format!("invalid structured model output: {e}"))
    }
}

#[async_trait]
impl Llm for OpenAiProvider {
    async fn note(&self, title: &str, content: &str) -> Result<Note, String> {
        Self::json(
            &self
                .chat(NOTE_SYSTEM, &format!("TITLE: {title}\n\n{content}"), true)
                .await?,
        )
    }
    async fn extract(
        &self,
        content: &str,
        candidates: &[CandidateEntity],
    ) -> Result<ExtractResult, String> {
        let list = candidates
            .iter()
            .map(|c| format!("{} ({})", c.name, c.kind))
            .collect::<Vec<_>>()
            .join(", ");
        Self::json(
            &self
                .chat(
                    EXTRACT_SYSTEM,
                    &format!(
                        "CANDIDATES: {}\n\nARTICLE:\n{content}",
                        if list.is_empty() { "(none)" } else { &list }
                    ),
                    true,
                )
                .await?,
        )
    }
    async fn reconcile(&self, input: &ReconcileInput) -> Result<Vec<PatchOp>, String> {
        let user = reconcile_prompt(input);
        let wire: WireResponse = Self::json(&self.chat(RECONCILE_SYSTEM, &user, true).await?)?;
        map_wire_ops(wire.ops, input)
    }
    async fn answer(&self, question: &str, context: &str) -> Result<String, String> {
        self.chat(
            RESEARCH_SYSTEM,
            &format!("QUESTION: {question}\n\nCONTEXT:\n{context}"),
            false,
        )
        .await
    }
}

#[derive(Default)]
pub struct MockProvider;
#[async_trait]
impl Llm for MockProvider {
    async fn note(&self, _: &str, content: &str) -> Result<Note, String> {
        let entities = proper_nouns(content);
        let sentences = sentences(content);
        Ok(Note {
            tldr: sentences
                .first()
                .cloned()
                .unwrap_or_else(|| content.chars().take(120).collect()),
            key_points: sentences.into_iter().take(5).collect(),
            candidate_entities: entities
                .iter()
                .take(6)
                .map(|(name, kind, _)| CandidateEntity {
                    name: name.clone(),
                    kind: kind.clone(),
                    why: "mentioned in source".into(),
                })
                .collect(),
            tags: entities
                .iter()
                .take(3)
                .map(|x| x.0.to_lowercase())
                .collect(),
        })
    }
    async fn extract(
        &self,
        content: &str,
        candidates: &[CandidateEntity],
    ) -> Result<ExtractResult, String> {
        let ranked = proper_nouns(content);
        let entities = if candidates.is_empty() {
            ranked
                .iter()
                .map(|(n, k, _)| Entity {
                    name: n.clone(),
                    kind: k.clone(),
                    aliases: vec![],
                })
                .collect()
        } else {
            candidates
                .iter()
                .map(|e| Entity {
                    name: e.name.clone(),
                    kind: e.kind.clone(),
                    aliases: vec![],
                })
                .collect()
        };
        let mut claims = Vec::new();
        for span in sentences(content)
            .into_iter()
            .filter(|s| s.chars().count() >= 15)
        {
            if let Some((name, _, _)) = ranked
                .iter()
                .find(|(name, _, _)| span.to_lowercase().contains(&name.to_lowercase()))
            {
                claims.push(ExtractClaim {
                    entity_name: name.clone(),
                    text: span.clone(),
                    span,
                    confidence: 0.8,
                });
            }
        }
        Ok(ExtractResult { entities, claims })
    }
    async fn reconcile(&self, input: &ReconcileInput) -> Result<Vec<PatchOp>, String> {
        let mut ops = Vec::new();
        if !input.exists {
            ops.push(PatchOp::CreatePage {
                entity: Entity {
                    name: input.title.clone(),
                    kind: input.kind.clone(),
                    aliases: vec![],
                },
                summary: input
                    .new_claims
                    .first()
                    .map(|c| c.text.clone())
                    .unwrap_or_else(|| input.title.clone()),
            });
        }
        for claim in &input.new_claims {
            let best = input
                .current_facts
                .iter()
                .map(|(id, text)| (id, text, jaccard(&claim.text, text)))
                .max_by(|a, b| a.2.total_cmp(&b.2));
            let contra = cue(
                &claim.text,
                &[
                    "actually",
                    "however",
                    "but",
                    "dispute",
                    "reportedly",
                    "allegedly",
                ],
            );
            let status = cue(
                &claim.text,
                &[
                    "stepped down",
                    "resigned",
                    "former",
                    "no longer",
                    "until",
                    "left",
                    "replaced",
                    "departed",
                ],
            );
            match best {
                None => ops.push(PatchOp::AppendFact {
                    fact: claim.text.clone(),
                    claim: claim.clone(),
                }),
                Some((_, _, score)) if score < 0.18 => ops.push(PatchOp::AppendFact {
                    fact: claim.text.clone(),
                    claim: claim.clone(),
                }),
                Some((id, text, _)) if contra && numbers_differ(&claim.text, text) => {
                    ops.push(PatchOp::FlagContradiction {
                        fact_id: id.clone(),
                        conflicting: claim.clone(),
                        note: claim.text.clone(),
                        target_slug: None,
                    })
                }
                Some((id, _, _)) if status => ops.push(PatchOp::SupersedeFact {
                    fact_id: id.clone(),
                    new_fact: claim.text.clone(),
                    claim: claim.clone(),
                    target_slug: None,
                }),
                Some((id, text, _)) if claim.text.len() > text.len() * 115 / 100 => {
                    ops.push(PatchOp::UpdateFact {
                        fact_id: id.clone(),
                        new_fact: claim.text.clone(),
                        claim: claim.clone(),
                        target_slug: None,
                    })
                }
                _ => {}
            }
        }
        Ok(ops)
    }
    async fn answer(&self, question: &str, context: &str) -> Result<String, String> {
        let mut lines = context
            .lines()
            .filter(|l| l.trim().len() > 10)
            .map(|l| (jaccard(question, l), l.trim()))
            .collect::<Vec<_>>();
        lines.sort_by(|a, b| b.0.total_cmp(&a.0));
        Ok(format!(
            "(mock) Based on your wiki:\n{}",
            lines
                .into_iter()
                .take(4)
                .map(|(_, l)| format!("- {l}"))
                .collect::<Vec<_>>()
                .join("\n")
        ))
    }
}

#[derive(Deserialize)]
struct WireResponse {
    #[serde(default)]
    ops: Vec<WireOp>,
}
#[derive(Deserialize)]
struct WireOp {
    op: String,
    summary: Option<String>,
    fact_id: Option<String>,
    new_fact: Option<String>,
    claim_id: Option<String>,
    note: Option<String>,
    page: Option<String>,
}
fn map_wire_ops(wire: Vec<WireOp>, input: &ReconcileInput) -> Result<Vec<PatchOp>, String> {
    let claims: HashMap<_, _> = input
        .new_claims
        .iter()
        .map(|c| (c.id.as_str(), c))
        .collect();
    let mut out = Vec::new();
    for op in wire {
        match op.op.as_str() {
            "create_page" => out.push(PatchOp::CreatePage {
                entity: Entity {
                    name: input.title.clone(),
                    kind: input.kind.clone(),
                    aliases: vec![],
                },
                summary: op.summary.unwrap_or_else(|| input.title.clone()),
            }),
            "append_fact" => {
                let c = claims
                    .get(op.claim_id.as_deref().unwrap_or(""))
                    .ok_or("unknown claim id")?;
                out.push(PatchOp::AppendFact {
                    fact: c.text.clone(),
                    claim: (*c).clone(),
                });
            }
            "update_fact" => {
                let c = claims
                    .get(op.claim_id.as_deref().unwrap_or(""))
                    .ok_or("unknown claim id")?;
                out.push(PatchOp::UpdateFact {
                    fact_id: op.fact_id.ok_or("missing fact_id")?,
                    new_fact: op.new_fact.unwrap_or_else(|| c.text.clone()),
                    claim: (*c).clone(),
                    target_slug: op.page,
                });
            }
            "supersede_fact" => {
                let c = claims
                    .get(op.claim_id.as_deref().unwrap_or(""))
                    .ok_or("unknown claim id")?;
                out.push(PatchOp::SupersedeFact {
                    fact_id: op.fact_id.ok_or("missing fact_id")?,
                    new_fact: op.new_fact.unwrap_or_else(|| c.text.clone()),
                    claim: (*c).clone(),
                    target_slug: op.page,
                });
            }
            "flag_contradiction" => {
                let c = claims
                    .get(op.claim_id.as_deref().unwrap_or(""))
                    .ok_or("unknown claim id")?;
                out.push(PatchOp::FlagContradiction {
                    fact_id: op.fact_id.ok_or("missing fact_id")?,
                    conflicting: (*c).clone(),
                    note: op.note.unwrap_or_else(|| c.text.clone()),
                    target_slug: op.page,
                });
            }
            _ => {}
        }
    }
    Ok(out)
}

fn reconcile_prompt(i: &ReconcileInput) -> String {
    let facts = if i.current_facts.is_empty() {
        "(none)".into()
    } else {
        i.current_facts
            .iter()
            .map(|(id, t)| format!("[{id}] {t}"))
            .collect::<Vec<_>>()
            .join("\n")
    };
    let claims = i
        .new_claims
        .iter()
        .map(|c| format!("[{}] {}", c.id, c.text))
        .collect::<Vec<_>>()
        .join("\n");
    let related = i
        .related_facts
        .iter()
        .map(|r| format!("[{}] page={} {}", r.fact_id, r.slug, r.text))
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "ENTITY {} ({}) slug={} exists={}\nCURRENT:\n{}\nNEW:\n{}\nRELATED:\n{}",
        i.title, i.kind, i.slug, i.exists, facts, claims, related
    )
}

static SENTENCE_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"[^.!?。！？]*[.!?。！？]+").unwrap());
static PROPER_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\b([A-Z][a-zA-Z.]+(?:\s+[A-Z][a-zA-Z.]+)*)\b").unwrap());
fn sentences(s: &str) -> Vec<String> {
    SENTENCE_RE
        .find_iter(s)
        .map(|m| m.as_str().trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}
fn proper_nouns(content: &str) -> Vec<(String, String, usize)> {
    let stop: HashSet<_> = [
        "The", "A", "An", "In", "On", "It", "This", "That", "Some", "He", "She", "They",
    ]
    .into_iter()
    .collect();
    let mut counts = HashMap::new();
    for cap in PROPER_RE.captures_iter(content) {
        let n = cap[1].trim_end_matches('.').trim().to_string();
        if n.len() > 1 && !stop.contains(n.as_str()) {
            *counts.entry(n).or_insert(0) += 1;
        }
    }
    let mut out = counts
        .into_iter()
        .map(|(n, c)| {
            let kind = if [
                "Inc",
                "Corp",
                "LLC",
                "Ltd",
                "Company",
                "Motors",
                "Labs",
                "Foundation",
            ]
            .iter()
            .any(|x| n.split_whitespace().any(|w| w == *x))
            {
                "org"
            } else if n.split_whitespace().count() >= 2 {
                "person"
            } else {
                "concept"
            };
            (n, kind.into(), c)
        })
        .collect::<Vec<_>>();
    out.sort_by_key(|item| std::cmp::Reverse(item.2));
    out
}
fn tokenize(s: &str) -> HashSet<String> {
    s.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|x| !x.is_empty())
        .map(str::to_string)
        .collect()
}
pub fn jaccard(a: &str, b: &str) -> f64 {
    let a = tokenize(a);
    let b = tokenize(b);
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }
    let inter = a.intersection(&b).count();
    let union = a.union(&b).count();
    inter as f64 / union as f64
}
fn cue(s: &str, words: &[&str]) -> bool {
    let lower = s.to_lowercase();
    words.iter().any(|w| lower.contains(w))
}
fn numbers_differ(a: &str, b: &str) -> bool {
    static YEAR: Lazy<Regex> = Lazy::new(|| Regex::new(r"\b\d{4}\b").unwrap());
    let aa = YEAR.find_iter(a).map(|x| x.as_str()).collect::<Vec<_>>();
    let bb = YEAR.find_iter(b).map(|x| x.as_str()).collect::<Vec<_>>();
    !aa.is_empty() && !bb.is_empty() && !aa.iter().any(|x| bb.contains(x))
}
