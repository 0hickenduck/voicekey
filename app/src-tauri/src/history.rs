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
    /// zh / ja / en，按字符粗略判断；判断不出为空
    pub lang: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub alts: Vec<Alt>,
}

impl Entry {
    pub fn new(channel: Channel, text: &str, alts: Vec<Alt>) -> Self {
        Self { ts: now_ms(), text: text.into(), channel, lang: lang(text).into(), alts }
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

/// 有假名（占中日文字一成以上）为 ja；汉字不少于英文单词数为 zh；只有拉丁字母为 en
pub fn lang(text: &str) -> &'static str {
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
    if kana > 0 && kana * 10 >= kana + han {
        "ja"
    } else if han > 0 && han >= words {
        "zh"
    } else if words > 0 {
        "en"
    } else {
        ""
    }
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
