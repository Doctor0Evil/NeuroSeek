use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs::File, io::BufReader, path::Path};

#[derive(Debug, Deserialize)]
struct CfAccessLog {
    timestamp: String,
    email: String,
    ip: String,
    user_agent: String,
    country: Option<String>,
    city: Option<String>,
    outcome: String, // "success" | "failed_login" | ...
    app_uid: String,
}

#[derive(Debug, Deserialize)]
struct S3AccessLog {
    time: String,
    src_ip: String,
    bucket: String,
    key: String,
    op: String, // "GET.OBJECT" | "PUT.OBJECT" | ...
    bytes_sent: u64,
    user_agent: String,
}

#[derive(Debug, Serialize)]
struct AuditEpisode {
    subject_id: String,      // your Bostrom address or email
    source: String,          // "cloudflare" | "s3"
    ts: String,
    event_kind: String,      // "login_success" | "login_failed" | "s3_put" | ...
    ip: String,
    user_agent: String,
    geo: Option<String>,
    anomaly_flags: Vec<String>,
}

fn load_cf_logs<P: AsRef<Path>>(p: P) -> anyhow::Result<Vec<CfAccessLog>> {
    let f = File::open(p)?;
    let rdr = BufReader::new(f);
    let logs = serde_json::from_reader(rdr)?;
    Ok(logs)
}

fn load_s3_logs<P: AsRef<Path>>(p: P) -> anyhow::Result<Vec<S3AccessLog>> {
    let f = File::open(p)?;
    let rdr = BufReader::new(f);
    let logs = serde_json::from_reader(rdr)?;
    Ok(logs)
}

fn build_cf_episodes(logs: &[CfAccessLog], subject_email: &str) -> Vec<AuditEpisode> {
    let mut episodes = Vec::new();
    let mut last_by_device: HashMap<String, (String, String)> = HashMap::new(); // email -> (ip, country)

    for log in logs.iter().filter(|l| l.email == subject_email) {
        let key = log.email.clone();
        let prev = last_by_device.get(&key).cloned();
        let mut flags = Vec::new();

        if let Some((prev_ip, prev_country)) = prev {
            if prev_ip != log.ip {
                flags.push("ip_change".into());
            }
            let curr_country = log.country.clone().unwrap_or_default();
            if !prev_country.is_empty() && !curr_country.is_empty() && prev_country != curr_country {
                flags.push("geo_jump".into());
            }
        }

        if log.outcome == "failed_login" {
            flags.push("auth_failure".into());
        }

        let geo = log
            .country
            .as_ref()
            .map(|c| format!("{}:{}", c, log.city.clone().unwrap_or_default()));

        episodes.push(AuditEpisode {
            subject_id: subject_email.to_string(),
            source: "cloudflare".into(),
            ts: log.timestamp.clone(),
            event_kind: if log.outcome == "success" {
                "login_success".into()
            } else {
                "login_failed".into()
            },
            ip: log.ip.clone(),
            user_agent: log.user_agent.clone(),
            geo,
            anomaly_flags: flags.clone(),
        });

        // update baseline only for successes
        if log.outcome == "success" {
            last_by_device.insert(
                key,
                (
                    log.ip.clone(),
                    log.country.clone().unwrap_or_default(),
                ),
            );
        }
    }
    episodes
}

fn build_s3_episodes(logs: &[S3AccessLog], subject_bucket_prefix: &str) -> Vec<AuditEpisode> {
    let mut episodes = Vec::new();
    for log in logs.iter().filter(|l| l.bucket.starts_with(subject_bucket_prefix)) {
        let mut flags = Vec::new();
        if log.bytes_sent > 5_000_000 {
            flags.push("large_transfer".into());
        }

        episodes.push(AuditEpisode {
            subject_id: subject_bucket_prefix.into(),
            source: "s3".into(),
            ts: log.time.clone(),
            event_kind: log.op.clone(),
            ip: log.src_ip.clone(),
            user_agent: log.user_agent.clone(),
            geo: None,
            anomaly_flags: flags,
        });
    }
    episodes
}

fn main() -> anyhow::Result<()> {
    let cf_logs = load_cf_logs("data/cloudflare_access.json")?;
    let s3_logs = load_s3_logs("data/s3_access.json")?;

    let subject_email = "your-email@example.com";
    let cf_eps = build_cf_episodes(&cf_logs, subject_email);
    let s3_eps = build_s3_episodes(&s3_logs, "ppl-ai-file-upload");

    let all: Vec<AuditEpisode> = cf_eps
        .into_iter()
        .chain(s3_eps.into_iter())
        .collect();

    serde_json::to_writer_pretty(
        File::create("data/audit-episodes.json")?,
        &all,
    )?;

    Ok(())
}
