use std::collections::HashSet;

/// Result of feeding one key into [`RomajiInput`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyResult {
    /// The key can still lead to a valid romanization.
    Accepted,
    /// The key completed the target text.
    Completed,
    /// The key does not match any valid romanization from the current state.
    Rejected,
}

/// Incremental romaji input matcher for typing games.
///
/// The matcher accepts common Hepburn and Kunrei-style alternatives such as
/// `shi`/`si`, `chi`/`ti`, `tsu`/`tu`, `kya`, small-kana spellings like `xya`,
/// doubled consonants for `っ`, and context-aware `ん`.
#[derive(Debug, Clone)]
pub struct RomajiInput {
    target: String,
    target_byte_indices: Vec<usize>,
    graph: Vec<Vec<Edge>>,
    states: HashSet<State>,
    typed: String,
}

impl RomajiInput {
    /// Builds a matcher from hiragana or katakana text.
    pub fn new(target: impl Into<String>) -> Self {
        let target = normalize_kana(&target.into());
        let target_byte_indices = target_byte_indices(&target);
        let graph = compile_graph(&target);
        let mut states = HashSet::new();
        states.insert(State::Node(0));

        Self {
            target,
            target_byte_indices,
            graph,
            states,
            typed: String::new(),
        }
    }

    /// Returns the normalized kana target used by this matcher.
    pub fn target(&self) -> &str {
        &self.target
    }

    /// Returns all accepted ASCII keys typed so far.
    pub fn typed(&self) -> &str {
        &self.typed
    }

    /// Clears typed input and returns to the beginning.
    pub fn reset(&mut self) {
        self.typed.clear();
        self.states.clear();
        self.states.insert(State::Node(0));
    }

    /// Returns true when the current input has completed the target.
    pub fn is_completed(&self) -> bool {
        self.states.contains(&State::Node(self.graph.len() - 1))
    }

    /// Returns how many normalized target characters are confirmed.
    ///
    /// When the current romaji is still inside one kana unit, the unit is not
    /// counted as confirmed yet.
    pub fn confirmed_target_chars(&self) -> usize {
        self.candidate_target_positions()
            .into_iter()
            .min()
            .unwrap_or(0)
    }

    /// Returns the byte index that splits [`Self::target`] at the confirmed
    /// target character position.
    pub fn confirmed_target_byte_index(&self) -> usize {
        self.target_byte_indices[self.confirmed_target_chars()]
    }

    /// Splits [`Self::target`] into confirmed and unconfirmed parts.
    pub fn target_parts(&self) -> (&str, &str) {
        self.target.split_at(self.confirmed_target_byte_index())
    }

    /// Returns sorted unique target character positions for all current
    /// candidates.
    ///
    /// Candidates that are still matching a multi-key romaji edge report the
    /// edge's starting target position.
    pub fn candidate_target_positions(&self) -> Vec<usize> {
        let mut positions = self
            .states
            .iter()
            .map(|state| state.target_position())
            .collect::<Vec<_>>();
        positions.sort_unstable();
        positions.dedup();
        positions
    }

    /// Tries to consume one keyboard character.
    ///
    /// Rejected keys do not mutate the matcher, which makes it convenient to
    /// count misses in a game loop while keeping the current progress intact.
    pub fn input(&mut self, key: char) -> KeyResult {
        let Some(key) = normalize_key(key) else {
            return KeyResult::Rejected;
        };

        let Some(next_states) = advance_states(&self.graph, &self.states, key) else {
            return KeyResult::Rejected;
        };

        self.states = next_states;
        self.typed.push(key);

        if self.is_completed() {
            KeyResult::Completed
        } else {
            KeyResult::Accepted
        }
    }

    /// Tries to consume a whole string and returns the first rejected character,
    /// if any.
    pub fn input_str(&mut self, input: &str) -> Result<KeyResult, char> {
        let mut last = if self.is_completed() {
            KeyResult::Completed
        } else {
            KeyResult::Accepted
        };

        for key in input.chars() {
            last = self.input(key);
            if last == KeyResult::Rejected {
                return Err(key);
            }
        }

        Ok(last)
    }

    /// Returns a sorted list of keys that are valid from the current state.
    pub fn next_keys(&self) -> Vec<char> {
        let mut keys = HashSet::new();

        for state in &self.states {
            match *state {
                State::Node(node) => {
                    for edge in &self.graph[node] {
                        if let Some(key) = edge.label.chars().next() {
                            keys.insert(key);
                        }
                    }
                }
                State::Edge { edge, offset, .. } => {
                    if let Some(key) = self.graph_edge(edge).label.chars().nth(offset) {
                        keys.insert(key);
                    }
                }
            }
        }

        let mut keys = keys.into_iter().collect::<Vec<_>>();
        keys.sort_unstable();
        keys
    }

    fn graph_edge(&self, edge: EdgeId) -> &Edge {
        &self.graph[edge.from][edge.index]
    }
}

/// Returns true when `input` is one complete romanization of `target`.
pub fn matches_romaji(target: &str, input: &str) -> bool {
    let mut matcher = RomajiInput::new(target);
    matcher
        .input_str(input)
        .is_ok_and(|result| result == KeyResult::Completed)
}

#[derive(Debug, Clone)]
struct Edge {
    to: usize,
    label: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct EdgeId {
    from: usize,
    index: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum State {
    Node(usize),
    Edge { edge: EdgeId, offset: usize },
}

impl State {
    fn target_position(&self) -> usize {
        match *self {
            Self::Node(node) => node,
            Self::Edge { edge, .. } => edge.from,
        }
    }
}

fn advance_states(
    graph: &[Vec<Edge>],
    current: &HashSet<State>,
    key: char,
) -> Option<HashSet<State>> {
    let mut next = HashSet::new();

    for state in current {
        match *state {
            State::Node(node) => {
                for (index, edge) in graph[node].iter().enumerate() {
                    if edge.label.starts_with(key) {
                        if edge.label.len() == 1 {
                            next.insert(State::Node(edge.to));
                        } else {
                            next.insert(State::Edge {
                                edge: EdgeId { from: node, index },
                                offset: 1,
                            });
                        }
                    }
                }
            }
            State::Edge { edge, offset } => {
                let graph_edge = &graph[edge.from][edge.index];
                if graph_edge.label.chars().nth(offset) == Some(key) {
                    if offset + 1 == graph_edge.label.len() {
                        next.insert(State::Node(graph_edge.to));
                    } else {
                        next.insert(State::Edge {
                            edge,
                            offset: offset + 1,
                        });
                    }
                }
            }
        }
    }

    (!next.is_empty()).then_some(next)
}

fn compile_graph(target: &str) -> Vec<Vec<Edge>> {
    let kana = target.chars().collect::<Vec<_>>();
    let mut graph = vec![Vec::new(); kana.len() + 1];
    let mut index = 0;

    while index < kana.len() {
        let ch = kana[index];

        if ch == 'っ' {
            add_edges(
                &mut graph,
                index,
                index + 1,
                &["xtu", "xtsu", "ltu", "ltsu"],
            );

            if let Some(next_unit) = read_unit(&kana, index + 1) {
                for roma in unit_romaji(&next_unit, kana.get(index + next_unit.len)) {
                    if let Some(first) = first_doubleable_consonant(&roma) {
                        let mut doubled = String::new();
                        doubled.push(first);
                        doubled.push_str(&roma);
                        add_edge(&mut graph, index, index + 1 + next_unit.len, doubled);
                    }
                }
            }

            index += 1;
            continue;
        }

        if let Some(unit) = read_unit(&kana, index) {
            let next = kana.get(index + unit.len);
            let romaji = unit_romaji(&unit, next);
            add_edges_owned(&mut graph, index, index + unit.len, romaji);
            index += unit.len;
        } else {
            add_edge(&mut graph, index, index + 1, ch.to_string());
            index += 1;
        }
    }

    graph
}

fn add_edges(graph: &mut [Vec<Edge>], from: usize, to: usize, labels: &[&str]) {
    for label in labels {
        add_edge(graph, from, to, *label);
    }
}

fn add_edges_owned(graph: &mut [Vec<Edge>], from: usize, to: usize, labels: Vec<String>) {
    for label in labels {
        add_edge(graph, from, to, label);
    }
}

fn add_edge(graph: &mut [Vec<Edge>], from: usize, to: usize, label: impl Into<String>) {
    let label = label.into();
    if !graph[from]
        .iter()
        .any(|edge| edge.to == to && edge.label == label)
    {
        graph[from].push(Edge { to, label });
    }
}

#[derive(Debug, Clone)]
struct KanaUnit {
    text: String,
    len: usize,
}

fn read_unit(kana: &[char], index: usize) -> Option<KanaUnit> {
    let ch = *kana.get(index)?;

    if index + 1 < kana.len() && is_small_y(kana[index + 1]) {
        let pair = format!("{}{}", ch, kana[index + 1]);
        if base_romaji(&pair).is_some() {
            return Some(KanaUnit { text: pair, len: 2 });
        }
    }

    Some(KanaUnit {
        text: ch.to_string(),
        len: 1,
    })
}

fn unit_romaji(unit: &KanaUnit, next: Option<&char>) -> Vec<String> {
    if unit.text == "ん" {
        return n_romaji(next).into_iter().map(String::from).collect();
    }

    base_romaji(&unit.text)
        .unwrap_or_else(|| vec![unit.text.as_str()])
        .into_iter()
        .map(String::from)
        .collect()
}

fn base_romaji(kana: &str) -> Option<Vec<&'static str>> {
    let romaji = match kana {
        "あ" => vec!["a"],
        "い" => vec!["i", "yi"],
        "う" => vec!["u", "wu"],
        "え" => vec!["e"],
        "お" => vec!["o"],
        "ぁ" => vec!["xa", "la"],
        "ぃ" => vec!["xi", "li", "xyi", "lyi"],
        "ぅ" => vec!["xu", "lu"],
        "ぇ" => vec!["xe", "le", "xye", "lye"],
        "ぉ" => vec!["xo", "lo"],
        "か" => vec!["ka", "ca"],
        "き" => vec!["ki"],
        "く" => vec!["ku", "cu", "qu"],
        "け" => vec!["ke"],
        "こ" => vec!["ko", "co"],
        "さ" => vec!["sa"],
        "し" => vec!["shi", "si", "ci"],
        "す" => vec!["su"],
        "せ" => vec!["se", "ce"],
        "そ" => vec!["so"],
        "た" => vec!["ta"],
        "ち" => vec!["chi", "ti"],
        "つ" => vec!["tsu", "tu"],
        "て" => vec!["te"],
        "と" => vec!["to"],
        "な" => vec!["na"],
        "に" => vec!["ni"],
        "ぬ" => vec!["nu"],
        "ね" => vec!["ne"],
        "の" => vec!["no"],
        "は" => vec!["ha"],
        "ひ" => vec!["hi"],
        "ふ" => vec!["fu", "hu"],
        "へ" => vec!["he"],
        "ほ" => vec!["ho"],
        "ま" => vec!["ma"],
        "み" => vec!["mi"],
        "む" => vec!["mu"],
        "め" => vec!["me"],
        "も" => vec!["mo"],
        "や" => vec!["ya"],
        "ゆ" => vec!["yu"],
        "よ" => vec!["yo"],
        "ゃ" => vec!["xya", "lya"],
        "ゅ" => vec!["xyu", "lyu"],
        "ょ" => vec!["xyo", "lyo"],
        "ら" => vec!["ra"],
        "り" => vec!["ri"],
        "る" => vec!["ru"],
        "れ" => vec!["re"],
        "ろ" => vec!["ro"],
        "わ" => vec!["wa"],
        "を" => vec!["wo"],
        "ゎ" => vec!["xwa", "lwa"],
        "ん" => vec!["n", "nn", "xn"],
        "が" => vec!["ga"],
        "ぎ" => vec!["gi"],
        "ぐ" => vec!["gu"],
        "げ" => vec!["ge"],
        "ご" => vec!["go"],
        "ざ" => vec!["za"],
        "じ" => vec!["ji", "zi"],
        "ず" => vec!["zu"],
        "ぜ" => vec!["ze"],
        "ぞ" => vec!["zo"],
        "だ" => vec!["da"],
        "ぢ" => vec!["di"],
        "づ" => vec!["du"],
        "で" => vec!["de"],
        "ど" => vec!["do"],
        "ば" => vec!["ba"],
        "び" => vec!["bi"],
        "ぶ" => vec!["bu"],
        "べ" => vec!["be"],
        "ぼ" => vec!["bo"],
        "ぱ" => vec!["pa"],
        "ぴ" => vec!["pi"],
        "ぷ" => vec!["pu"],
        "ぺ" => vec!["pe"],
        "ぽ" => vec!["po"],
        "ゔ" => vec!["vu"],
        "きゃ" => vec!["kya"],
        "きぃ" => vec!["kyi"],
        "きゅ" => vec!["kyu"],
        "きぇ" => vec!["kye"],
        "きょ" => vec!["kyo"],
        "しゃ" => vec!["sha", "sya"],
        "しぃ" => vec!["syi"],
        "しゅ" => vec!["shu", "syu"],
        "しぇ" => vec!["she", "sye"],
        "しょ" => vec!["sho", "syo"],
        "ちゃ" => vec!["cha", "tya", "cya"],
        "ちぃ" => vec!["tyi", "cyi"],
        "ちゅ" => vec!["chu", "tyu", "cyu"],
        "ちぇ" => vec!["che", "tye", "cye"],
        "ちょ" => vec!["cho", "tyo", "cyo"],
        "にゃ" => vec!["nya"],
        "にぃ" => vec!["nyi"],
        "にゅ" => vec!["nyu"],
        "にぇ" => vec!["nye"],
        "にょ" => vec!["nyo"],
        "ひゃ" => vec!["hya"],
        "ひぃ" => vec!["hyi"],
        "ひゅ" => vec!["hyu"],
        "ひぇ" => vec!["hye"],
        "ひょ" => vec!["hyo"],
        "みゃ" => vec!["mya"],
        "みぃ" => vec!["myi"],
        "みゅ" => vec!["myu"],
        "みぇ" => vec!["mye"],
        "みょ" => vec!["myo"],
        "りゃ" => vec!["rya"],
        "りぃ" => vec!["ryi"],
        "りゅ" => vec!["ryu"],
        "りぇ" => vec!["rye"],
        "りょ" => vec!["ryo"],
        "ぎゃ" => vec!["gya"],
        "ぎぃ" => vec!["gyi"],
        "ぎゅ" => vec!["gyu"],
        "ぎぇ" => vec!["gye"],
        "ぎょ" => vec!["gyo"],
        "じゃ" => vec!["ja", "jya", "zya"],
        "じぃ" => vec!["jyi", "zyi"],
        "じゅ" => vec!["ju", "jyu", "zyu"],
        "じぇ" => vec!["je", "jye", "zye"],
        "じょ" => vec!["jo", "jyo", "zyo"],
        "びゃ" => vec!["bya"],
        "びぃ" => vec!["byi"],
        "びゅ" => vec!["byu"],
        "びぇ" => vec!["bye"],
        "びょ" => vec!["byo"],
        "ぴゃ" => vec!["pya"],
        "ぴぃ" => vec!["pyi"],
        "ぴゅ" => vec!["pyu"],
        "ぴぇ" => vec!["pye"],
        "ぴょ" => vec!["pyo"],
        "ふぁ" => vec!["fa", "fwa"],
        "ふぃ" => vec!["fi", "fwi"],
        "ふぇ" => vec!["fe", "fwe"],
        "ふぉ" => vec!["fo", "fwo"],
        "てぃ" => vec!["thi"],
        "てゅ" => vec!["thu"],
        "でぃ" => vec!["dhi"],
        "でゅ" => vec!["dhu"],
        "うぃ" => vec!["wi", "whi"],
        "うぇ" => vec!["we", "whe"],
        "うぉ" => vec!["who"],
        "ゔぁ" => vec!["va"],
        "ゔぃ" => vec!["vi"],
        "ゔぇ" => vec!["ve"],
        "ゔぉ" => vec!["vo"],
        " " => vec![" "],
        "ー" => vec!["-"],
        "−" => vec!["-"],
        "、" => vec![","],
        "。" => vec!["."],
        "・" => vec!["/"],
        "！" => vec!["!"],
        "？" => vec!["?"],
        "（" => vec!["("],
        "）" => vec![")"],
        "［" | "「" | "『" => vec!["["],
        "］" | "」" | "』" => vec!["]"],
        "｛" => vec!["{"],
        "｝" => vec!["}"],
        "：" => vec![":"],
        "；" => vec![";"],
        "”" | "“" | "＂" => vec!["\""],
        "’" | "‘" | "＇" => vec!["'"],
        "￥" => vec!["\\"],
        "〜" | "～" => vec!["~"],
        _ => return None,
    };

    Some(romaji)
}

fn n_romaji(next: Option<&char>) -> Vec<&'static str> {
    if next.is_none() {
        return vec!["nn", "xn"];
    }

    let next_requires_escape = next.is_some_and(|ch| {
        let normalized = normalize_kana(&ch.to_string());
        let mut chars = normalized.chars();
        matches!(
            chars.next(),
            Some(
                'あ' | 'い'
                    | 'う'
                    | 'え'
                    | 'お'
                    | 'な'
                    | 'に'
                    | 'ぬ'
                    | 'ね'
                    | 'の'
                    | 'や'
                    | 'ゆ'
                    | 'よ'
            )
        )
    });

    if next_requires_escape {
        vec!["nn", "n'", "xn"]
    } else {
        vec!["n", "nn", "n'", "xn"]
    }
}

fn first_doubleable_consonant(romaji: &str) -> Option<char> {
    let first = romaji.chars().next()?;
    (!matches!(
        first,
        'a' | 'i' | 'u' | 'e' | 'o' | 'n' | '-' | '\'' | ',' | '.' | '/' | '!' | '?'
    ))
    .then_some(first)
}

fn is_small_y(ch: char) -> bool {
    matches!(ch, 'ゃ' | 'ゅ' | 'ょ' | 'ぁ' | 'ぃ' | 'ぅ' | 'ぇ' | 'ぉ')
}

fn normalize_kana(input: &str) -> String {
    input
        .chars()
        .map(|ch| normalize_target_char(ch).unwrap_or(ch))
        .collect()
}

fn target_byte_indices(target: &str) -> Vec<usize> {
    let mut indices = target
        .char_indices()
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    indices.push(target.len());
    indices
}

fn normalize_key(key: char) -> Option<char> {
    normalize_ascii_like(key).or_else(|| key.is_ascii().then_some(key.to_ascii_lowercase()))
}

fn normalize_target_char(ch: char) -> Option<char> {
    match ch {
        'ヵ' => Some('か'),
        'ヶ' => Some('け'),
        'ヴ' => Some('ゔ'),
        'ァ'..='ヶ' => char::from_u32(ch as u32 - 0x60),
        _ => normalize_ascii_like(ch),
    }
}

fn normalize_ascii_like(ch: char) -> Option<char> {
    let ascii = match ch {
        'Ａ'..='Ｚ' => char::from_u32(ch as u32 - 'Ａ' as u32 + 'a' as u32)?,
        'ａ'..='ｚ' => char::from_u32(ch as u32 - 'ａ' as u32 + 'a' as u32)?,
        '０'..='９' => char::from_u32(ch as u32 - '０' as u32 + '0' as u32)?,
        '！'..='～' => char::from_u32(ch as u32 - 0xfee0)?.to_ascii_lowercase(),
        '　' => ' ',
        '−' | 'ー' => '-',
        '‐' | '‑' | '‒' | '–' | '—' | '―' => '-',
        '、' => ',',
        '。' => '.',
        '・' => '/',
        '「' | '『' => '[',
        '」' | '』' => ']',
        '“' | '”' => '"',
        '‘' | '’' => '\'',
        '￥' => '\\',
        '〜' => '~',
        _ if ch.is_ascii() => ch.to_ascii_lowercase(),
        _ => return None,
    };

    Some(ascii)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_common_alternatives() {
        assert!(matches_romaji("しちつふじ", "shichitsufuji"));
        assert!(matches_romaji("しちつふじ", "sitituhuzi"));
    }

    #[test]
    fn accepts_yoon_and_foreign_sounds() {
        assert!(matches_romaji("しゃしゅしょ", "shashusho"));
        assert!(matches_romaji("しゃしゅしょ", "syasyusyo"));
        assert!(matches_romaji("ふぁいる", "fairu"));
        assert!(matches_romaji("ティー", "thi-"));
    }

    #[test]
    fn handles_small_tsu() {
        assert!(matches_romaji("かった", "katta"));
        assert!(matches_romaji("かった", "kaxtsuta"));
        assert!(matches_romaji("マッチ", "macchi"));
        assert!(matches_romaji("マッチ", "matti"));
    }

    #[test]
    fn handles_context_aware_n() {
        assert!(matches_romaji("ほんだ", "honda"));
        assert!(matches_romaji("ほんあ", "honna"));
        assert!(matches_romaji("ほんあ", "hon'a"));
        assert!(!matches_romaji("ほんあ", "hona"));
        assert!(matches_romaji("ほん", "honn"));
        assert!(matches_romaji("ほん", "hoxn"));
        assert!(!matches_romaji("ほん", "hon"));
    }

    #[test]
    fn accepts_fullwidth_digits_letters_and_symbols() {
        assert!(matches_romaji("バージョン２．０！", "ba-jon2.0!"));
        assert!(matches_romaji("Ｙｅｗ　＋　Ｔｒｕｎｋ", "yew + trunk"));
        assert!(matches_romaji("「Rust」", "[rust]"));
    }

    #[test]
    fn normalizes_fullwidth_key_input() {
        let mut input = RomajiInput::new("abc123!?");

        assert_eq!(
            input.input_str("ＡＢＣ１２３！？"),
            Ok(KeyResult::Completed)
        );
        assert_eq!(input.typed(), "abc123!?");
    }

    #[test]
    fn accepts_common_ascii_punctuation_in_targets() {
        assert!(matches_romaji("Ready? Go!", "ready? go!"));
        assert!(matches_romaji("（テスト）", "(tesuto)"));
        assert!(matches_romaji("ねだん￥１００", "nedan\\100"));
    }

    #[test]
    fn rejects_without_mutating_progress() {
        let mut input = RomajiInput::new("かき");

        assert_eq!(input.input('k'), KeyResult::Accepted);
        assert_eq!(input.input('x'), KeyResult::Rejected);
        assert_eq!(input.typed(), "k");
        assert_eq!(input.input('a'), KeyResult::Accepted);
        assert_eq!(input.input('k'), KeyResult::Accepted);
        assert_eq!(input.input('i'), KeyResult::Completed);
    }

    #[test]
    fn exposes_confirmed_target_parts() {
        let mut input = RomajiInput::new("しゃしん");

        assert_eq!(input.confirmed_target_chars(), 0);
        assert_eq!(input.confirmed_target_byte_index(), 0);
        assert_eq!(input.target_parts(), ("", "しゃしん"));
        assert_eq!(input.candidate_target_positions(), vec![0]);

        assert_eq!(input.input('s'), KeyResult::Accepted);
        assert_eq!(input.confirmed_target_chars(), 0);
        assert_eq!(input.target_parts(), ("", "しゃしん"));
        assert_eq!(input.candidate_target_positions(), vec![0]);

        assert_eq!(input.input('h'), KeyResult::Accepted);
        assert_eq!(input.confirmed_target_chars(), 0);
        assert_eq!(input.target_parts(), ("", "しゃしん"));

        assert_eq!(input.input('a'), KeyResult::Accepted);
        assert_eq!(input.confirmed_target_chars(), 2);
        assert_eq!(input.confirmed_target_byte_index(), "しゃ".len());
        assert_eq!(input.target_parts(), ("しゃ", "しん"));
        assert_eq!(input.candidate_target_positions(), vec![2]);

        assert_eq!(input.input('s'), KeyResult::Accepted);
        assert_eq!(input.confirmed_target_chars(), 2);

        assert_eq!(input.input('i'), KeyResult::Accepted);
        assert_eq!(input.confirmed_target_chars(), 3);
        assert_eq!(input.target_parts(), ("しゃし", "ん"));

        assert_eq!(input.input('n'), KeyResult::Accepted);
        assert_eq!(input.confirmed_target_chars(), 3);
        assert_eq!(input.target_parts(), ("しゃし", "ん"));

        assert_eq!(input.input('n'), KeyResult::Completed);
        assert_eq!(input.confirmed_target_chars(), 4);
        assert_eq!(input.target_parts(), ("しゃしん", ""));
        assert_eq!(input.candidate_target_positions(), vec![4]);
    }

    #[test]
    fn confirmed_target_position_ignores_rejected_keys() {
        let mut input = RomajiInput::new("かな");

        assert_eq!(input.input('k'), KeyResult::Accepted);
        assert_eq!(input.confirmed_target_chars(), 0);
        assert_eq!(input.input('x'), KeyResult::Rejected);
        assert_eq!(input.confirmed_target_chars(), 0);
        assert_eq!(input.input('a'), KeyResult::Accepted);
        assert_eq!(input.confirmed_target_chars(), 1);
    }

    #[test]
    fn exposes_next_keys() {
        let mut input = RomajiInput::new("し");

        assert_eq!(input.next_keys(), vec!['c', 's']);
        assert_eq!(input.input('s'), KeyResult::Accepted);
        assert_eq!(input.next_keys(), vec!['h', 'i']);
    }

    #[test]
    fn reset_returns_to_start() {
        let mut input = RomajiInput::new("あ");

        assert_eq!(input.input('a'), KeyResult::Completed);
        input.reset();

        assert!(!input.is_completed());
        assert_eq!(input.typed(), "");
        assert_eq!(input.input('a'), KeyResult::Completed);
    }
}
