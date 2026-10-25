//! 识别历史：每次上屏的定稿追加到数据目录的 history.jsonl（一行一条），设置页可搜索、复制
use crate::settings::Channel;
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use voicekey_core::util::{data_file, now_ms};

/// 多渠道模式下没有选中的其他定稿
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Alt {
    pub channel: Channel,
    pub text: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Entry {
    /// Unix 毫秒
    pub ts: u64,
    pub text: String,
    pub channel: Channel,
    /// zh / ja / en，按字符粗略判断的主要语言；判断不出为空
    pub lang: String,
    /// 混说时各语言按占比排列，如 "en+zh"；单一语言时省略
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub mix: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub alts: Vec<Alt>,
}

impl Entry {
    pub fn new(channel: Channel, text: &str, alts: Vec<Alt>) -> Self {
        let langs = langs(text);
        let mix = if langs.len() > 1 { langs.join("+") } else { String::new() };
        Self { ts: now_ms(), text: text.into(), channel, lang: langs.first().copied().unwrap_or("").into(), mix, alts }
    }
}

pub fn path() -> PathBuf {
    data_file("history.jsonl")
}

pub fn append(e: &Entry) -> std::io::Result<()> {
    let mut line = serde_json::to_string(e)?;
    line.push('\n');
    std::fs::OpenOptions::new().create(true).append(true).open(path())?.write_all(line.as_bytes())
}

/// 最近的在前；query 非空时按文字过滤（不分大小写）
pub fn list(query: &str, limit: usize) -> Vec<Entry> {
    let Ok(f) = std::fs::File::open(path()) else { return Vec::new() };
    let q = query.trim().to_lowercase();
    let mut v: Vec<Entry> = BufReader::new(f)
        .lines()
        .map_while(Result::ok)
        .filter_map(|l| serde_json::from_str::<Entry>(&l).ok())
        .filter(|e| q.is_empty() || e.text.to_lowercase().contains(&q))
        .collect();
    v.reverse();
    v.truncate(limit);
    v
}

/// 主要语言：zh / ja / en，判断不出为空
pub fn lang(text: &str) -> &'static str {
    langs(text).first().copied().unwrap_or("")
}

/// 句中出现的语言，按占比从高到低；占比不足 15% 的忽略。
/// 粗略折算成“词”：英文按单词，中文约 1.5 字一词，日文（有假名时汉字也算日文）约 2 字一词。
pub fn langs(text: &str) -> Vec<&'static str> {
    let (mut kana, mut han, mut words, mut in_word) = (0usize, 0usize, 0usize, false);
    for c in text.chars() {
        let latin = c.is_ascii_alphabetic();
        if latin && !in_word {
            words += 1;
        }
        in_word = latin;
        match c as u32 {
            0x3041..=0x30FA | 0x30FD..=0x30FF | 0x31F0..=0x31FF | 0xFF66..=0xFF9D => kana += 1,
            0x3400..=0x4DBF | 0x4E00..=0x9FFF | 0xF900..=0xFAFF => han += 1,
            _ => {}
        }
    }
    let japanese = kana > 0 && kana * 10 >= kana + han;
    let (ja, zh) = if japanese { ((kana + han) as f64 / 2.0, 0.0) } else { (0.0, han as f64 / 1.5) };
    let en = words as f64;
    let total = ja + zh + en;
    if total == 0.0 {
        return Vec::new();
    }
    let mut v: Vec<(&'static str, f64)> = [("zh", zh), ("ja", ja), ("en", en)]
        .into_iter()
        .filter(|&(_, n)| n > 0.0 && n / total >= 0.15)
        .collect();
    v.sort_by(|a, b| b.1.total_cmp(&a.1));
    v.into_iter().map(|(l, _)| l).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_language() {
        assert_eq!(lang("今天天气不错。"), "zh");
        assert_eq!(lang("我觉得这个 API 的 design 不太好"), "zh");
        assert_eq!(lang("明日は雨が降るそうです。"), "ja");
        assert_eq!(lang("コーヒーをください"), "ja");
        assert_eq!(lang("Actually, I think it works."), "en");
        assert_eq!(lang("Let's use 豆包 for this one"), "en");
        assert_eq!(lang("123，。"), "");
    }

    #[test]
    fn detects_mixed_speech() {
        assert_eq!(langs("Let's use 豆包 for this one"), vec!["en", "zh"]);
        assert_eq!(langs("这个 deadline 太赶了"), vec!["zh", "en"]);
        assert_eq!(langs("明日の meeting は online でお願いします"), vec!["ja", "en"]);
        assert_eq!(langs("Actually, I think it works."), vec!["en"]);
        assert_eq!(Entry::new(Channel::Doubao, "今天天气不错。", Vec::new()).mix, "");
        assert_eq!(Entry::new(Channel::Doubao, "这个 deadline 太赶了", Vec::new()).mix, "zh+en");
    }

    #[test]
    fn entry_round_trips_and_skips_empty_alts() {
        let e = Entry::new(Channel::Doubao, "你好", Vec::new());
        let s = serde_json::to_string(&e).unwrap();
        assert!(s.contains(r#""channel":"doubao""#) && s.contains(r#""lang":"zh""#) && !s.contains("alts"));
        assert_eq!(serde_json::from_str::<Entry>(&s).unwrap(), e);
        let p = Entry::new(Channel::Qwen, "hi", vec![Alt { channel: Channel::Doubao, text: "high".into() }]);
        let back: Entry = serde_json::from_str(&serde_json::to_string(&p).unwrap()).unwrap();
        assert_eq!(back.alts[0].channel, Channel::Doubao);
    }
}
