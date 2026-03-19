//! 易经排卦与符号映射。
//!
//! 这里把 UI 表达和卦象算法分离出来，保证后续无论要接 CLI、Web 还是更多解释文本，
//! 都可以在不碰状态机的情况下复用这一层。

/// 六个爻位，自下而上排列。
pub const LINE_POSITIONS: [&str; 6] = ["初爻", "二爻", "三爻", "四爻", "五爻", "上爻"];

const HEXAGRAM_NAMES: [&str; 64] = [
    "坤为地",
    "地雷复",
    "地水师",
    "地泽临",
    "地山谦",
    "地火明夷",
    "地风升",
    "地天泰",
    "雷地豫",
    "震为雷",
    "雷水解",
    "雷泽归妹",
    "雷山小过",
    "雷火丰",
    "雷风恒",
    "雷天大壮",
    "水地比",
    "水雷屯",
    "坎为水",
    "水泽节",
    "水山蹇",
    "水火既济",
    "水风井",
    "水天需",
    "泽地萃",
    "泽雷随",
    "泽水困",
    "兑为泽",
    "泽山咸",
    "泽火革",
    "泽风大过",
    "泽天夬",
    "山地剥",
    "山雷颐",
    "山水蒙",
    "山泽损",
    "艮为山",
    "山火贲",
    "山风蛊",
    "山天大畜",
    "火地晋",
    "火雷噬嗑",
    "火水未济",
    "火泽睽",
    "火山旅",
    "离为火",
    "火风鼎",
    "火天大有",
    "风地观",
    "风雷益",
    "风水涣",
    "风泽中孚",
    "风山渐",
    "风火家人",
    "巽为风",
    "风天小畜",
    "天地否",
    "天雷无妄",
    "天水讼",
    "天泽履",
    "天山遁",
    "天火同人",
    "天风姤",
    "乾为天",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HexagramIdentity {
    pub index: usize,
    pub name: &'static str,
}

/// 一次完整起卦的核心结果。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HexagramResult {
    pub primary: HexagramIdentity,
    pub relating: Option<HexagramIdentity>,
    pub changing_lines: Vec<String>,
    pub transformed_lines: [u8; 6],
}

/// 将六爻和数值映射为六十四卦索引与变爻说明。
pub fn analyze_hexagram(line_sums: &[u8]) -> HexagramResult {
    assert_eq!(
        line_sums.len(),
        6,
        "hexagram analysis requires exactly 6 lines"
    );

    let primary = hexagram_identity(line_sums);
    let changing_lines = line_sums
        .iter()
        .copied()
        .enumerate()
        .filter(|(_, value)| matches!(value, 6 | 9))
        .map(|(idx, value)| format_changing_line(idx + 1, value))
        .collect::<Vec<_>>();
    let transformed_lines = std::array::from_fn(|idx| transformed_line_value(line_sums[idx]));
    let relating = (!changing_lines.is_empty()).then(|| hexagram_identity(&transformed_lines));

    HexagramResult {
        primary,
        relating,
        changing_lines,
        transformed_lines,
    }
}

/// 阳爻以 `1` 计，阴爻以 `0` 计。
pub fn is_yang(value: u8) -> bool {
    matches!(value, 7 | 9)
}

/// 适合终端渲染的简化爻图。
pub fn line_symbol(value: u8) -> &'static str {
    match value {
        6 => "--x--",
        7 => "-----",
        8 => "-- --",
        9 => "--o--",
        _ => "?????",
    }
}

/// 六、七、八、九的人类可读说明。
pub fn line_label(value: u8) -> &'static str {
    match value {
        6 => "老阴 / 变爻",
        7 => "少阳 / 静爻",
        8 => "少阴 / 静爻",
        9 => "老阳 / 变爻",
        _ => "未知",
    }
}

/// 将本卦中的某一爻转换为之卦中的静爻。
pub fn transformed_line_value(value: u8) -> u8 {
    match value {
        6 => 7,
        7 => 7,
        8 => 8,
        9 => 8,
        _ => value,
    }
}

fn hexagram_identity(line_sums: &[u8]) -> HexagramIdentity {
    let mut lower = 0usize;
    let mut upper = 0usize;

    for (idx, value) in line_sums.iter().copied().enumerate() {
        let bit = usize::from(is_yang(value));
        if idx < 3 {
            lower |= bit << idx;
        } else {
            upper |= bit << (idx - 3);
        }
    }

    let index = (upper << 3) | lower;
    HexagramIdentity {
        index,
        name: HEXAGRAM_NAMES[index],
    }
}

/// 把变爻位置转成传统写法。
pub fn format_changing_line(position: usize, value: u8) -> String {
    let yin_or_yang = if value == 9 { "九" } else { "六" };
    match position {
        1 => format!("初{}", yin_or_yang),
        2 => format!("{}二", yin_or_yang),
        3 => format!("{}三", yin_or_yang),
        4 => format!("{}四", yin_or_yang),
        5 => format!("{}五", yin_or_yang),
        6 => format!("上{}", yin_or_yang),
        _ => "未知变爻".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        analyze_hexagram, format_changing_line, line_label, line_symbol, transformed_line_value,
    };

    #[test]
    fn maps_all_yang_to_qian() {
        let result = analyze_hexagram(&[7, 7, 7, 7, 7, 7]);
        assert_eq!(result.primary.index, 63);
        assert_eq!(result.primary.name, "乾为天");
        assert!(result.changing_lines.is_empty());
        assert_eq!(result.relating, None);
    }

    #[test]
    fn maps_water_thunder_zhun_correctly() {
        let result = analyze_hexagram(&[7, 8, 8, 8, 7, 8]);
        assert_eq!(result.primary.index, 17);
        assert_eq!(result.primary.name, "水雷屯");
    }

    #[test]
    fn changing_lines_use_traditional_labels() {
        let result = analyze_hexagram(&[6, 7, 8, 9, 8, 7]);
        assert_eq!(
            result.changing_lines,
            vec!["初六".to_string(), "九四".to_string()]
        );
        assert_eq!(format_changing_line(6, 6), "上六");
        assert_eq!(result.primary.name, "火水未济");
        assert_eq!(result.relating.expect("relating").name, "山泽损");
        assert_eq!(result.transformed_lines, [7, 7, 8, 8, 8, 7]);
    }

    #[test]
    fn line_descriptions_match_symbols() {
        assert_eq!(line_symbol(6), "--x--");
        assert_eq!(line_symbol(9), "--o--");
        assert_eq!(line_label(7), "少阳 / 静爻");
        assert_eq!(line_label(8), "少阴 / 静爻");
    }

    #[test]
    fn transformed_lines_collapse_to_static_yin_yang() {
        assert_eq!(transformed_line_value(6), 7);
        assert_eq!(transformed_line_value(7), 7);
        assert_eq!(transformed_line_value(8), 8);
        assert_eq!(transformed_line_value(9), 8);
    }
}
