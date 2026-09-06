//! Fixed current-wave bilingual model comparison; run explicitly, outside unit tests.
use fastembed::{EmbeddingModel, TextEmbedding, TextInitOptions};
use nous_memory_retrieval::residual::normalize;
use std::{path::Path, time::Instant};

fn bytes(path: &Path) -> u64 {
    std::fs::read_dir(path)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| {
            if e.path().is_dir() {
                bytes(&e.path())
            } else {
                e.metadata().map(|m| m.len()).unwrap_or(0)
            }
        })
        .sum()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let candidate = std::env::args()
        .nth(1)
        .ok_or("expected small, base or m3")?;
    let model = match candidate.as_str() {
        "small" => EmbeddingModel::MultilingualE5Small,
        "base" => EmbeddingModel::MultilingualE5Base,
        "m3" => EmbeddingModel::BGEM3,
        _ => return Err("unknown fixed candidate".into()),
    };
    let info = TextEmbedding::get_model_info(&model)?;
    let cache = std::path::PathBuf::from(".local/bakeoff").join(&candidate);
    let pairs = [
        (
            "绒岬实验站把备用门钥匙放在紫色陶罐里。",
            "绒岬实验站的备用钥匙藏在哪里？",
        ),
        (
            "澄鹭号调查船每周四清晨向西航行去测量盐度。",
            "澄鹭号哪一天去海上调查？",
        ),
        (
            "工程师陶蔚发现赤砂泵的异响来自松动的铜环。",
            "赤砂泵为何发出异常噪声？",
        ),
        (
            "雾穗图书馆的借阅证丢失后，应找管理员宁簇补办。",
            "在雾穗遗失借书卡应该联系谁？",
        ),
        (
            "琥铃果园在霜冻夜用温水循环管保护幼苗。",
            "琥铃果园怎样让幼苗免遭低温损害？",
        ),
        (
            "蓝砾仓库把需要冷藏的样品放在地下二层。",
            "蓝砾仓库的低温样本存在哪一层？",
        ),
        (
            "At Velmora observatory, the backup battery is behind the cedar cabinet.",
            "Where is Velmora's spare power supply?",
        ),
        (
            "The Nembrel ferry suspends crossings when the river gauge exceeds four metres.",
            "What water level stops the Nembrel ferry?",
        ),
        (
            "Dr Talsen uses a violet filter to detect cracks in the Orveth lens.",
            "How does Talsen inspect the Orveth lens for damage?",
        ),
        (
            "The Kelbrin archive sends restoration requests to curator Soven.",
            "Who handles repairs to Kelbrin archival material?",
        ),
        (
            "The Zurnel greenhouse ventilates at dawn to prevent fungal growth.",
            "Why are Zurnel's vents opened early in the morning?",
        ),
        (
            "At the Peldra workshop, amber labels identify instruments awaiting calibration.",
            "Which Peldra instruments carry amber labels?",
        ),
        (
            "诺汀工坊的消防集合点在北侧石桥。",
            "Where should people gather during a fire at the Nuoting workshop?",
        ),
        (
            "The Mirven expedition stores its emergency radio in a red waterproof case.",
            "米尔文探险队的应急无线电装在哪里？",
        ),
    ];
    let mut documents: Vec<String> = pairs.iter().map(|(d, _)| d.to_string()).collect();
    for topic in [
        "library loans",
        "greenhouse irrigation",
        "warehouse delivery",
        "river travel",
        "laboratory instruments",
        "battery maintenance",
        "图书修复",
        "果园病虫害",
        "船舶维修",
        "实验站测量",
        "样品保存",
        "光学镜片",
    ] {
        for suffix in [
            "general training manual",
            "monthly purchasing report",
            "staff schedule",
        ] {
            documents.push(format!("General unrelated reference material about {topic}: {suffix}. 本条目是一般背景，没有私人经历的具体事实。"));
        }
    }
    eprintln!("loading {}", info.model_code);
    let mut engine = TextEmbedding::try_new(
        TextInitOptions::new(model.clone())
            .with_cache_dir(cache.clone())
            .with_show_download_progress(false)
            .with_intra_threads(2),
    )?;
    let e5 = candidate != "m3";
    let passages: Vec<_> = documents
        .iter()
        .map(|s| {
            if e5 {
                format!("passage: {s}")
            } else {
                s.clone()
            }
        })
        .collect();
    let vectors = engine
        .embed(passages, Some(8))?
        .iter()
        .map(|v| normalize(v).ok_or("invalid document vector"))
        .collect::<Result<Vec<_>, _>>()?;
    let mut ranks = vec![];
    let mut latencies = vec![];
    for (expected, (_, query)) in pairs.iter().enumerate() {
        let query = if e5 {
            format!("query: {query}")
        } else {
            query.to_string()
        };
        let start = Instant::now();
        let result = engine.embed(vec![query], Some(1))?;
        latencies.push(start.elapsed().as_secs_f64() * 1000.0);
        let query = normalize(&result[0]).ok_or("invalid query vector")?;
        let mut ranked: Vec<_> = vectors
            .iter()
            .enumerate()
            .map(|(id, v)| (id, v.iter().zip(&query).map(|(a, b)| a * b).sum::<f32>()))
            .collect();
        ranked.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
        ranks.push(ranked.iter().position(|(id, _)| *id == expected).unwrap() + 1);
    }
    latencies.sort_by(f64::total_cmp);
    let count = ranks.len() as f64;
    let rss = if cfg!(windows) {
        std::process::Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                &format!("(Get-Process -Id {}).WorkingSet64", std::process::id()),
            ])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .and_then(|s| s.trim().parse::<u64>().ok())
    } else {
        None
    };
    let result = serde_json::json!({
        "candidate": info.model_code, "fastembed": "6.0.2", "dimension": info.dim,
        "preprocessing": if e5 { "e5-query-passage-l2-v1" } else { "identity-l2-v1" },
        "fixture": "private-bilingual-v1", "documents": documents.len(), "queries": pairs.len(), "ranks": ranks,
        "recall_at_10": ranks.iter().filter(|r| **r <= 10).count() as f64/count,
        "mrr": ranks.iter().map(|r| 1.0/(*r as f64)).sum::<f64>()/count,
        "ndcg_at_10": ranks.iter().map(|r| if *r <= 10 { 1.0/((*r+1) as f64).log2() } else { 0.0 }).sum::<f64>()/count,
        "p50_embedding_ms": latencies[latencies.len()/2], "p95_embedding_ms": latencies[(latencies.len()*95/100).min(latencies.len()-1)],
        "steady_rss_bytes": rss, "model_asset_bytes": bytes(&cache),
        "correctness_pass": ranks.iter().all(|r| *r <= 10),
    });
    std::fs::create_dir_all(".local/bakeoff")?;
    std::fs::write(
        format!(".local/bakeoff/{candidate}.json"),
        serde_json::to_vec_pretty(&result)?,
    )?;
    println!("{result}");
    Ok(())
}
