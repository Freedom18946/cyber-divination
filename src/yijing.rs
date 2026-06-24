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
    pub changing_indices: Vec<usize>,
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
    let (changing_lines, changing_indices): (Vec<String>, Vec<usize>) = line_sums
        .iter()
        .copied()
        .enumerate()
        .filter(|(_, value)| matches!(value, 6 | 9))
        .map(|(idx, value)| (format_changing_line(idx + 1, value), idx))
        .unzip();
    let transformed_lines = std::array::from_fn(|idx| transformed_line_value(line_sums[idx]));
    let relating = (!changing_lines.is_empty()).then(|| hexagram_identity(&transformed_lines));

    HexagramResult {
        primary,
        relating,
        changing_lines,
        changing_indices,
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

// ── 卦辞 / 爻辞数据库 ───────────────────────────────────
//
// 按 `HEXAGRAM_NAMES` 同序排列。每卦含 1 条卦辞与 6 条爻辞；
// 爻辞仅存正文，不带"初九/六二"等位置前缀，渲染时由 `format_changing_line`
// 复用占法标签。原文取通行本《周易》（王弼注本系统），释义力求简白。
//
// 注意：448 条古文由人工据通行本誊录，建议上线前再人工复核一次。

/// 一条卦辞或爻辞：原文 + 简明白话释义。
pub struct LineText {
    pub classic: &'static str,
    pub gloss: &'static str,
}

/// 单卦的全部文本：1 条卦辞 + 6 条爻辞（自下而上，对应 line index 0..6）。
pub struct HexagramText {
    pub judgment: LineText,
    pub lines: [LineText; 6],
}

pub static HEXAGRAM_TEXTS: [HexagramText; 64] = [
    // 0 坤为地
    HexagramText {
        judgment: LineText { classic: "坤：元，亨，利牝马之贞。君子有攸往，先迷后得主，利。西南得朋，东北丧朋。安贞，吉。", gloss: "坤卦象征大地：至为亨通，宜如牝马般柔顺守正。君子有所前往，先则迷途，后必得主导而有利。西南方得同类，东北方失同类。安顺守正则吉。" },
        lines: [
            LineText { classic: "履霜，坚冰至。", gloss: "脚下踩到霜，坚冰将至。见微知著，阴气始凝。" },
            LineText { classic: "直，方，大，不习无不利。", gloss: "正直、方正、宏大，不必修习亦无往不利。自然之德。" },
            LineText { classic: "含章可贞。或从王事，无成有终。", gloss: "含蓄章美而能守正。或参与王事，不居功而有善终。" },
            LineText { classic: "括囊，无咎无誉。", gloss: "扎紧袋口，无咎也无誉。慎言谨行，收敛自守。" },
            LineText { classic: "黄裳，元吉。", gloss: "黄色下裳，至为吉祥。中道柔顺之德。" },
            LineText { classic: "龙战于野，其血玄黄。", gloss: "龙在郊野交战，流出玄黄之血。阴盛至极而抗阳。" },
        ],
    },
    // 1 地雷复
    HexagramText {
        judgment: LineText { classic: "复：亨。出入无疾，朋来无咎。反复其道，七日来复，利有攸往。", gloss: "复卦亨通。出入无碍，朋来无咎。循道反复，七日一回归，宜有所前往。" },
        lines: [
            LineText { classic: "不远复，无祗悔，元吉。", gloss: "走得不远便回复，不致大悔，至吉。早知早改。" },
            LineText { classic: "休复，吉。", gloss: "安休而复，吉。停下回归正道。" },
            LineText { classic: "频复，厉无咎。", gloss: "屡复屡退，有危厉而无咎。频复须自勉。" },
            LineText { classic: "中行独复。", gloss: "独自从正道而行。与众偕行而能自返。" },
            LineText { classic: "敦复，无悔。", gloss: "敦厚而复，无悔。笃诚复正。" },
            LineText { classic: "迷复，凶，有灾眚。", gloss: "迷而不复，凶，有灾患。执迷不返则凶。" },
        ],
    },
    // 2 地水师
    HexagramText {
        judgment: LineText { classic: "师：贞，丈人吉无咎。", gloss: "师卦宜守正，任用老成持重之人则吉无咎。" },
        lines: [
            LineText { classic: "师出以律，否臧，凶。", gloss: "出师须凭严格的军纪，纪律不善则凶。" },
            LineText { classic: "在师中，吉无咎，王三锡命。", gloss: "居中统师，吉无咎，君王三次赐命嘉奖。" },
            LineText { classic: "师或舆尸，凶。", gloss: "军队或致舆尸而归，凶。败军之象。" },
            LineText { classic: "师左次，无咎。", gloss: "军队驻扎左方据守，无咎。稳守无失。" },
            LineText { classic: "田有禽，利执言，无咎。长子帅师，弟子舆尸，贞凶。", gloss: "田中有禽兽宜驱捕，无咎。长子统师得当，弟子则致舆尸，守正亦凶。" },
            LineText { classic: "大君有命，开国承家，小人勿用。", gloss: "天子颁命，开国封家，但小人不可任用。" },
        ],
    },
    // 3 地泽临
    HexagramText {
        judgment: LineText { classic: "临：元，亨，利，贞。至于八月有凶。", gloss: "临卦：元亨利贞。然至八月将有凶。居高思危。" },
        lines: [
            LineText { classic: "咸临，贞吉。", gloss: "以感应之道监临，守正吉。" },
            LineText { classic: "咸临，吉无不利。", gloss: "以感应之道监临，吉无不利。" },
            LineText { classic: "甘临，无攸利。既忧之，无咎。", gloss: "以甜言甘辞监临，无所利。若能忧惧警省，则无咎。" },
            LineText { classic: "至临，无咎。", gloss: "以切身至诚监临，无咎。" },
            LineText { classic: "知临，大君之宜，吉。", gloss: "以明智监临，乃大国君之相宜，吉。" },
            LineText { classic: "敦临，吉无咎。", gloss: "以敦厚之道监临，吉无咎。" },
        ],
    },
    // 4 地山谦
    HexagramText {
        judgment: LineText { classic: "谦：亨，君子有终。", gloss: "谦卦亨通，君子能得其善终。" },
        lines: [
            LineText { classic: "谦谦君子，用涉大川，吉。", gloss: "谦而又谦的君子，可涉大川，吉。" },
            LineText { classic: "鸣谦，贞吉。", gloss: "鸣其谦德而守正，吉。" },
            LineText { classic: "劳谦，君子有终，吉。", gloss: "勤劳而谦逊，君子有终，吉。" },
            LineText { classic: "无不利，捴谦。", gloss: "无所不利，举事而谦。" },
            LineText { classic: "不富，以其邻，利用侵伐，无不利。", gloss: "不富而能得其邻，宜于征伐，无所不利。" },
            LineText { classic: "鸣谦，利用行师，征邑国。", gloss: "鸣谦之德，宜用兵征讨己之邑国。" },
        ],
    },
    // 5 地火明夷
    HexagramText {
        judgment: LineText { classic: "明夷：利艰贞。", gloss: "明夷卦利于在艰难中守正。韬光养晦之象。" },
        lines: [
            LineText { classic: "明夷于飞，垂其翼。君子于行，三日不食，有攸往，主人有言。", gloss: "明夷时飞而垂翼。君子行役，三日不食，有所前往而遭主人讥议。" },
            LineText { classic: "明夷，夷于左股，用拯马壮，吉。", gloss: "明夷时伤在左腿，乘壮马相救则吉。" },
            LineText { classic: "明夷于南狩，得其大首，不可疾贞。", gloss: "明夷时在南狩得敌之首，不可急图守正。" },
            LineText { classic: "入于左腹，获明夷之心，于出门庭。", gloss: "入于左腹，得明夷之心意，出门庭而退遁。" },
            LineText { classic: "箕子之明夷，利贞。", gloss: "如箕子之明夷韬晦，利守正。" },
            LineText { classic: "不明晦，初登于天，后入于地。", gloss: "昏暗至极，初登于天而后坠入地。明道陨落。" },
        ],
    },
    // 6 地风升
    HexagramText {
        judgment: LineText { classic: "升：元亨，用见大人，勿恤，南征吉。", gloss: "升卦元亨，宜见大人，勿忧，南征则吉。" },
        lines: [
            LineText { classic: "允升，大吉。", gloss: "顺势而升，大吉。" },
            LineText { classic: "孚乃利用禴，无咎。", gloss: "有诚信则宜用简省祭祀，无咎。" },
            LineText { classic: "升虚邑。", gloss: "升入空虚无阻之邑。畅通无碍。" },
            LineText { classic: "王用亨于岐山，吉无咎。", gloss: "王者亨祭于岐山，吉无咎。" },
            LineText { classic: "贞吉，升阶。", gloss: "守正而吉，逐阶而升。" },
            LineText { classic: "冥升，利于不息之贞。", gloss: "昏冥而升，宜不息守正。" },
        ],
    },
    // 7 地天泰
    HexagramText {
        judgment: LineText { classic: "泰：小往大来，吉亨。", gloss: "泰卦：小者去大者来，吉而亨通。" },
        lines: [
            LineText { classic: "拔茅茹，以其汇，征吉。", gloss: "拔茅牵连同类，往而征吉。" },
            LineText { classic: "包荒，用冯河，不遐遗，朋亡，得尚于中行。", gloss: "包容荒秽，徒步涉河，不弃远，亡其朋党，得合中道而行。" },
            LineText { classic: "无平不陂，无往不复，艰贞无咎。勿恤其孚，于食有福。", gloss: "无平不陡，无往不复，艰中守正无咎。勿忧其信，于食有福。" },
            LineText { classic: "翩翩不富，以其邻，不戒以孚。", gloss: "翩然下行而不富，携其邻，不戒而信。" },
            LineText { classic: "帝乙归妹，以祉元吉。", gloss: "帝乙嫁妹，以之受福至吉。" },
            LineText { classic: "城复于隍，勿用师，自邑告命，贞吝。", gloss: "城墙倾复于隍堑，勿用兵，自邑告命，守贞有吝。" },
        ],
    },
    // 8 雷地豫
    HexagramText {
        judgment: LineText { classic: "豫：利建侯行师。", gloss: "豫卦利于封侯建邦与出师。" },
        lines: [
            LineText { classic: "鸣豫，凶。", gloss: "鸣其自豫，凶。" },
            LineText { classic: "介于石，不终日，贞吉。", gloss: "坚介如石，不终日而能悟，守正则吉。" },
            LineText { classic: "盱豫悔，迟有悔。", gloss: "张目企慕于豫则有悔，迟疑亦有悔。" },
            LineText { classic: "由豫，大有得。勿疑，朋盍簪。", gloss: "以宽厚致豫，大有得。勿疑，朋合如簪聚发。" },
            LineText { classic: "贞疾，恒不死。", gloss: "守正而疾，长久不死。" },
            LineText { classic: "冥豫，成有渝，无咎。", gloss: "昏冥而豫，事成而能变，无咎。" },
        ],
    },
    // 9 震为雷
    HexagramText {
        judgment: LineText { classic: "震：亨。震来虩虩，笑言哑哑。震惊百里，不丧匕鬯。", gloss: "震卦亨通。雷震来时瑟瑟惊惧，而后笑语和乐。雷惊百里，不失匕鬯祭祀之礼。" },
        lines: [
            LineText { classic: "震来虩虩，后笑言哑哑，吉。", gloss: "震来惊惧，后笑语和乐，吉。" },
            LineText { classic: "震来厉，亿丧贝，跻于九陵，勿逐，七日得。", gloss: "震来有危厉，巨额丧财，登于九陵，勿逐，七日复得。" },
            LineText { classic: "震苏苏，震行无眚。", gloss: "震而苏醒警惧，震动前行则无眚。" },
            LineText { classic: "震遂泥。", gloss: "震惧陷于泥。坠而失其健。" },
            LineText { classic: "震往来厉，亿无丧，有事。", gloss: "震往来皆有厉，幸无所丧，但有事惧当处。" },
            LineText { classic: "震索索，视矍矍，征凶。震不于其躬，于其邻，无咎。婚媾有言。", gloss: "震而索索，视而矍矍，征则凶。震不及其身而及其邻，无咎。婚姻有闲言。" },
        ],
    },
    // 10 雷水解
    HexagramText {
        judgment: LineText { classic: "解：利西南，无所往，其来复吉。有攸往，夙吉。", gloss: "解卦利西南，无所往则复归而吉；有所往则早行而吉。" },
        lines: [
            LineText { classic: "无咎。", gloss: "无咎。柔居最下，刚柔相济。" },
            LineText { classic: "田获三狐，得黄矢，贞吉。", gloss: "田猎获三狐，得黄矢，守正吉。" },
            LineText { classic: "负且乘，致寇至，贞吝。", gloss: "背负又乘载，招致寇盗，守贞有吝。德不称位。" },
            LineText { classic: "解而拇，朋至斯孚。", gloss: "解开你足大趾之缚，朋来至此而见诚信。" },
            LineText { classic: "君子维有解，吉，有孚于小人。", gloss: "君子若能解难则吉，且信诚感及小人。" },
            LineText { classic: "公用射隼于高墉之上，获之，无不利。", gloss: "公在高墙上射隼而获之，无不利。" },
        ],
    },
    // 11 雷泽归妹
    HexagramText {
        judgment: LineText { classic: "归妹：征凶，无攸利。", gloss: "归妹卦：征则凶，无所利。" },
        lines: [
            LineText { classic: "归妹以归妹以娣，跛能履，征吉。", gloss: "以妹为娣陪嫁，跛足而能履，征吉。" },
            LineText { classic: "眇能视，利幽人之贞。", gloss: "盲而能视，宜幽居守贞之人。" },
            LineText { classic: "归妹以须，反归以娣。", gloss: "以待女而嫁，反以娣身份归嫁。" },
            LineText { classic: "归妹愆期，迟归有时。", gloss: "归妹延期，迟嫁亦有时。" },
            LineText { classic: "帝乙归妹，其君之袂，不如其娣之袂良。月几望，吉。", gloss: "帝乙嫁妹，姊之衣不如娣之衣华美。月近圆，吉。" },
            LineText { classic: "女承筐无实，士刲羊无血，无攸利。", gloss: "女子承筐无实，男子刺羊无血，无所利。" },
        ],
    },
    // 12 雷山小过
    HexagramText {
        judgment: LineText { classic: "小过：亨，利贞，可小事，不可大事。飞鸟遗之音，不宜上宜下，大吉。", gloss: "小过卦亨通，利守正，可小事不可大事。飞鸟遗其音，不宜上宜下，则大吉。" },
        lines: [
            LineText { classic: "飞鸟以凶。", gloss: "飞鸟妄上则凶。" },
            LineText { classic: "过其祖，遇其妣；不及其君，遇其臣，无咎。", gloss: "越过祖父而遇祖母，未及君而遇臣，无咎。" },
            LineText { classic: "弗过防之，从或戕之，凶。", gloss: "不过分防备之，反被戕害，凶。" },
            LineText { classic: "无咎，弗过遇之。往厉必戒，勿用永贞。", gloss: "无咎，不过分而相值。前往有厉须戒，不可固守。" },
            LineText { classic: "密云不雨，自我西郊，公弋取彼在穴。", gloss: "密云不雨，起自我西郊，公射取穴中之鸟。" },
            LineText { classic: "弗遇过之，飞鸟离之，凶，是谓灾眚。", gloss: "不相值而过之，飞鸟离散而凶，是为灾眚。" },
        ],
    },
    // 13 雷火丰
    HexagramText {
        judgment: LineText { classic: "丰：亨，王假之，勿忧，宜日中。", gloss: "丰卦亨通，王者致其盛大，勿忧，宜如日中天。" },
        lines: [
            LineText { classic: "遇其配主，虽旬无咎，往有尚。", gloss: "遇见相配之主，虽十日相当无咎，往则有尚。" },
            LineText { classic: "丰其蔀，日中见斗，往得疑疾，有孚发若，吉。", gloss: "草席遮蔽丰大，日中见斗星，往则生疑疾，有诚信发挥则吉。" },
            LineText { classic: "丰其沛，日中见沫，折其右肱，无咎。", gloss: "丰其幔幕，日中见小星，折其右臂，无咎。" },
            LineText { classic: "丰其蔀，日中见斗，遇其夷主，吉。", gloss: "草席遮蔽丰大，日中见斗星，遇其平主则吉。" },
            LineText { classic: "来章，有庆誉，吉。", gloss: "来致其章美，有庆誉，吉。" },
            LineText { classic: "丰其屋，蔀其家，窥其户，阒其无人，三岁不觌，凶。", gloss: "丰其屋蔀其家，窥其户而寂无人，三年不见，凶。" },
        ],
    },
    // 14 雷风恒
    HexagramText {
        judgment: LineText { classic: "恒：亨，无咎，利贞，利有攸往。", gloss: "恒卦亨通，无咎，利守正，利有所往。" },
        lines: [
            LineText { classic: "浚恒，贞凶，无攸利。", gloss: "深求其恒，守正亦凶，无所利。" },
            LineText { classic: "悔亡。", gloss: "悔恨消亡。" },
            LineText { classic: "不恒其德，或承之羞，贞吝。", gloss: "不能恒守其德，或致蒙羞，守贞有吝。" },
            LineText { classic: "田无禽。", gloss: "田猎无获。" },
            LineText { classic: "恒其德，贞，妇人吉，夫子凶。", gloss: "恒守其德守正，妇人吉而丈夫凶。" },
            LineText { classic: "振恒，上六，凶。", gloss: "振动失恒于上，凶。" },
        ],
    },
    // 15 雷天大壮
    HexagramText {
        judgment: LineText { classic: "大壮：利贞。", gloss: "大壮卦利于守正。" },
        lines: [
            LineText { classic: "壮于趾，征凶，有孚。", gloss: "壮于足趾，征则凶，但存诚信。" },
            LineText { classic: "贞吉。", gloss: "守正则吉。" },
            LineText { classic: "小人用壮，君子用罔，贞厉。羝羊触藩，羸其角。", gloss: "小人逞强，君子用网，守贞有厉。如公羊触藩被羸其角。" },
            LineText { classic: "藩决不羸，壮于大舆之輹。", gloss: "藩篱已决不羸其角，壮如大车之輹。" },
            LineText { classic: "丧羊于易，无悔。", gloss: "丧羊于边邑，无悔。" },
            LineText { classic: "羝羊触藩，不能退，不能遂，无攸利，艰则吉。", gloss: "公羊触藩，进退不得，无所利，处艰则吉。" },
        ],
    },
    // 16 水地比
    HexagramText {
        judgment: LineText { classic: "比：吉。原筮，元永贞，无咎。不宁方来，后夫凶。", gloss: "比卦吉。原审而占长远守正，无咎。不宁之方来附，迟后者凶。" },
        lines: [
            LineText { classic: "有孚比之，无咎。有孚盈缶，终来有它吉。", gloss: "以诚信相比，无咎。诚若盈缶，终致他吉。" },
            LineText { classic: "比之自内，贞吉。", gloss: "发自内心相比，守正吉。" },
            LineText { classic: "比之匪人。", gloss: "比附非其人。" },
            LineText { classic: "外比之，贞吉。", gloss: "外相比之，守正吉。" },
            LineText { classic: "显比，王用三驱，失前禽。邑人不诫，吉。", gloss: "显明相比，王三驱田猎失前禽，邑人不诫，吉。" },
            LineText { classic: "比之无首，凶。", gloss: "比附而无首，凶。" },
        ],
    },
    // 17 水雷屯
    HexagramText {
        judgment: LineText { classic: "屯：元亨利贞，勿用有攸往，利建侯。", gloss: "屯卦元亨利贞，勿有所往，宜立侯以辅。" },
        lines: [
            LineText { classic: "磐桓，利居贞，利建侯。", gloss: "盘桓难进，宜居守正，宜建侯。" },
            LineText { classic: "屯如邅如，乘马班如。匪寇婚媾，女子贞不字，十年乃字。", gloss: "屯难徘徊，乘马班旋。非寇乃婚媾，女子守贞不嫁，十年乃嫁。" },
            LineText { classic: "即鹿无虞，惟入于林中，君子几不如舍，往吝。", gloss: "逐鹿无虞人导引，入林中，君子宜舍不宜往，往则吝。" },
            LineText { classic: "乘马班如，求婚媾，往吉无不利。", gloss: "乘马班旋，求婚媾，往吉无不利。" },
            LineText { classic: "屯其膏，小贞吉，大贞凶。", gloss: "屯其膏泽，小事守正吉，大事守贞凶。" },
            LineText { classic: "乘马班如，泣血涟如。", gloss: "乘马班旋，泣血涟涟。" },
        ],
    },
    // 18 坎为水
    HexagramText {
        judgment: LineText { classic: "习坎：有孚，维心亨，行有尚。", gloss: "重重坎险，有诚信维系心之亨通，行而有尚。" },
        lines: [
            LineText { classic: "习坎，入于坎窞，凶。", gloss: "重坎入于坎陷深处，凶。" },
            LineText { classic: "坎有险，求小得。", gloss: "坎中有险，求得小利。" },
            LineText { classic: "来之坎坎，险且枕，入于坎窞，勿用。", gloss: "坎险接连而来，险且枕危，入于坎陷，勿用。" },
            LineText { classic: "樽酒簋贰，用缶，纳约自牖，终无咎。", gloss: "一樽酒二簋，以瓦缶盛，自牖纳约，终无咎。" },
            LineText { classic: "坎不盈，祗既平，无咎。", gloss: "坎险未盈满，将平则无咎。" },
            LineText { classic: "系用徽纆，寘于丛棘，三岁不得，凶。", gloss: "系以绳索，置于丛棘中，三年不得出，凶。" },
        ],
    },
    // 19 水泽节
    HexagramText {
        judgment: LineText { classic: "节：亨。苦节不可贞。", gloss: "节卦亨通。然苦于过分节制不可守正。" },
        lines: [
            LineText { classic: "不出户庭，无咎。", gloss: "不出户庭，无咎。" },
            LineText { classic: "不出门庭，凶。", gloss: "不出门庭，凶。当出不出。" },
            LineText { classic: "不节若，则嗟若，无咎。", gloss: "不能节制则嗟叹，无咎。" },
            LineText { classic: "安节，亨。", gloss: "安于节制，亨通。" },
            LineText { classic: "甘节，吉，往有尚。", gloss: "甘心节制，吉，往有尚。" },
            LineText { classic: "苦节，贞凶，悔亡。", gloss: "苦于过分节制，守贞凶，悔亡。" },
        ],
    },
    // 20 水山蹇
    HexagramText {
        judgment: LineText { classic: "蹇：利西南，不利东北。利见大人，贞吉。", gloss: "蹇卦利西南，不利东北。利见大人，守正吉。" },
        lines: [
            LineText { classic: "往蹇来誉。", gloss: "往则蹇难，来则有誉。" },
            LineText { classic: "王臣蹇蹇，匪躬之故。", gloss: "王臣蹇蹇济难，并非自身之故。" },
            LineText { classic: "往蹇来反。", gloss: "往则蹇难，来则复反其常。" },
            LineText { classic: "往蹇来连。", gloss: "往则蹇难，来则连合相助。" },
            LineText { classic: "大蹇朋来。", gloss: "大蹇之时朋来相助。" },
            LineText { classic: "往蹇来硕，吉，利见大人。", gloss: "往则蹇难，来则硕大成事，吉，利见大人。" },
        ],
    },
    // 21 水火既济
    HexagramText {
        judgment: LineText { classic: "既济：亨，小利贞，初吉终乱。", gloss: "既济卦亨通，小者利守正，初吉而终乱。" },
        lines: [
            LineText { classic: "曳其轮，濡其尾，无咎。", gloss: "拖曳车轮，濡湿狐尾，无咎。" },
            LineText { classic: "妇丧其茀，勿逐，七日得。", gloss: "妇人丧其首饰，勿逐，七日复得。" },
            LineText { classic: "高宗伐鬼方，三年克之，小人勿用。", gloss: "殷高宗伐鬼方，三年克之，小人勿用。" },
            LineText { classic: "繻有衣袽，终日戒。", gloss: "盛装之中备破衣，终日戒备。" },
            LineText { classic: "东邻杀牛，不如西邻之禴祭，实受其福。", gloss: "东邻杀牛盛祭，不如西邻简祭，实受其福。" },
            LineText { classic: "濡其首，厉。", gloss: "濡湿其首，有危厉。" },
        ],
    },
    // 22 水风井
    HexagramText {
        judgment: LineText { classic: "井：改邑不改井，无丧无得，往来井井。汔至，亦未繘井，羸其瓶，凶。", gloss: "井卦：迁邑不改井，无丧无得，往来井然。汲水将及未出井，毁其瓶，凶。" },
        lines: [
            LineText { classic: "井泥不食，旧井无禽。", gloss: "井有泥不可食，废井无禽。" },
            LineText { classic: "井谷射鲋，瓮敝漏。", gloss: "井谷射小鱼，瓮破漏失。" },
            LineText { classic: "井渫不食，为我心恻，可用汲，王明并受其福。", gloss: "井已疏浚仍不食，使我心恻，可汲用，王明则共受其福。" },
            LineText { classic: "井甃，无咎。", gloss: "井以砖砌，无咎。" },
            LineText { classic: "井冽寒泉食。", gloss: "井泉清冽寒凉可食。" },
            LineText { classic: "井收勿幕，有孚元吉。", gloss: "井收绳不掩盖，有诚信至吉。" },
        ],
    },
    // 23 水天需
    HexagramText {
        judgment: LineText { classic: "需：有孚，光亨，贞吉。利涉大川。", gloss: "需卦有诚信，光耀亨通，守正吉。利涉大川。" },
        lines: [
            LineText { classic: "需于郊，利用恒，无咎。", gloss: "需待于郊野，能恒守则无咎。" },
            LineText { classic: "需于沙，小有言，终吉。", gloss: "需待于沙，小有口舌，终吉。" },
            LineText { classic: "需于泥，致寇至。", gloss: "需待于泥，招致寇盗。" },
            LineText { classic: "需于血，出自穴。", gloss: "需待于血，自穴中而出。" },
            LineText { classic: "需于酒食，贞吉。", gloss: "需待于酒食，守正吉。" },
            LineText { classic: "入于穴，有不速之客三人来，敬之终吉。", gloss: "入于穴，意外来客三人，敬之终吉。" },
        ],
    },
    // 24 泽地萃
    HexagramText {
        judgment: LineText { classic: "萃：亨。王假有庙，利见大人，亨，利贞。用大牲吉，利有攸往。", gloss: "萃卦亨通。王至于宗庙，利见大人。利于守正。用大牲祭吉，利有所往。" },
        lines: [
            LineText { classic: "有孚不终，乃乱乃萃，若号一握为笑，勿恤，往无咎。", gloss: "诚信不终则紊乱而聚，若号呼一握成笑，勿忧，往无咎。" },
            LineText { classic: "引吉无咎，孚乃利用禴。", gloss: "引致而吉无咎，有诚信则宜简祭。" },
            LineText { classic: "萃如嗟如，无攸利，往无咎，小吝。", gloss: "聚而嗟叹，无所利，往无咎，小有吝。" },
            LineText { classic: "大吉，无咎。", gloss: "大吉则无咎。" },
            LineText { classic: "萃有位，无咎，匪孚，元永贞，悔亡。", gloss: "聚而有位无咎，然未得诚信，长久守正则悔亡。" },
            LineText { classic: "赍咨涕洟，无咎。", gloss: "叹息流涕，无咎。" },
        ],
    },
    // 25 泽雷随
    HexagramText {
        judgment: LineText { classic: "随：元亨利贞，无咎。", gloss: "随卦元亨利贞，无咎。" },
        lines: [
            LineText { classic: "官有渝，贞吉。出门交有功。", gloss: "官事有变，守正吉。出门交有功。" },
            LineText { classic: "系小子，失丈夫。", gloss: "系于小子则失丈夫。" },
            LineText { classic: "系丈夫，失小子。随有求得，利居贞。", gloss: "系于丈夫则失小子。随而有求则得，宜居守正。" },
            LineText { classic: "随有获，贞凶，有孚在道，以明，何咎。", gloss: "随而有获，守贞凶，有诚信在道以明，何咎。" },
            LineText { classic: "孚于嘉，吉。", gloss: "诚信于善，吉。" },
            LineText { classic: "拘系之，乃从维之，王用亨于西山。", gloss: "拘系之又从而维系，王亨祭于西山。" },
        ],
    },
    // 26 泽水困
    HexagramText {
        judgment: LineText { classic: "困：亨，贞，大人吉，无咎，有言不信。", gloss: "困卦亨通守正，大人则吉无咎，但有言而无人信。" },
        lines: [
            LineText { classic: "臀困于株木，入于幽谷，三岁不觌。", gloss: "臀困于株木，入于幽谷，三年不见其能展。" },
            LineText { classic: "困于酒食，朱绂方来，利用享祀，征凶，无咎。", gloss: "困于酒食，朱衣方来，宜用享祀，征凶，无咎。" },
            LineText { classic: "困于石，据于蒺藜，入于其宫，不见其妻，凶。", gloss: "困于石，据蒺藜，入宫不见其妻，凶。" },
            LineText { classic: "来徐徐，困于金车，吝，有终。", gloss: "来而徐缓，困于金车，有吝，有终。" },
            LineText { classic: "劓刖，困于赤绂，乃徐有说，利用祭祀。", gloss: "施劓刖之刑，困于赤衣，徐而解脱，宜用祭祀。" },
            LineText { classic: "困于葛藟，于臲卼，曰动悔，有悔，征吉。", gloss: "困于葛藟臲卼之危，动则有悔，悔而后征吉。" },
        ],
    },
    // 27 兑为泽
    HexagramText {
        judgment: LineText { classic: "兑：亨，利贞。", gloss: "兑卦亨通，利守正。" },
        lines: [
            LineText { classic: "和兑，吉。", gloss: "和悦相处，吉。" },
            LineText { classic: "孚兑，吉，悔亡。", gloss: "诚信而悦，吉，悔亡。" },
            LineText { classic: "来兑，凶。", gloss: "求来致悦，凶。" },
            LineText { classic: "商兑未宁，介疾有喜。", gloss: "商筹未宁，介耿有疾而后有喜。" },
            LineText { classic: "孚于剥，有厉。", gloss: "信于剥损小人，有危厉。" },
            LineText { classic: "引兑。", gloss: "引诱致悦。" },
        ],
    },
    // 28 泽山咸
    HexagramText {
        judgment: LineText { classic: "咸：亨，利贞，取女吉。", gloss: "咸卦亨通，利守正，娶女吉。" },
        lines: [
            LineText { classic: "咸其拇。", gloss: "感于足大拇。" },
            LineText { classic: "咸其腓，凶，居吉。", gloss: "感于小腿肚，凶，安居则吉。" },
            LineText { classic: "咸其股，执其随，往吝。", gloss: "感于腿股，执意相随，往则吝。" },
            LineText { classic: "贞吉悔亡，憧憧往来，朋从尔思。", gloss: "守正吉悔亡，往来不绝，朋从尔思。" },
            LineText { classic: "咸其脢，无悔。", gloss: "感于背脊，无悔。" },
            LineText { classic: "咸其辅颊舌。", gloss: "感于口舌辅颊。" },
        ],
    },
    // 29 泽火革
    HexagramText {
        judgment: LineText { classic: "革：己日乃孚，元亨利贞，悔亡。", gloss: "革卦到己日乃能令人信服，元亨利贞，悔亡。" },
        lines: [
            LineText { classic: "巩用黄牛之革。", gloss: "以黄牛之革巩固约束。" },
            LineText { classic: "己日乃革之，征吉，无咎。", gloss: "己日乃变革之，征吉，无咎。" },
            LineText { classic: "征凶，贞厉，革言三就，有孚。", gloss: "征凶贞厉，变革之事三议而后就，有诚信。" },
            LineText { classic: "悔亡，有孚改命，吉。", gloss: "悔亡，有诚信以改天命，吉。" },
            LineText { classic: "大人虎变，未占有孚。", gloss: "大人如虎之变革，未占已有诚信。" },
            LineText { classic: "君子豹变，小人革面，征凶，居贞吉。", gloss: "君子如豹之变，小人改面，征凶，居守正吉。" },
        ],
    },
    // 30 泽风大过
    HexagramText {
        judgment: LineText { classic: "大过：栋桡，利有攸往，亨。", gloss: "大过卦：屋栋桡曲，利有所往，亨通。" },
        lines: [
            LineText { classic: "藉用白茅，无咎。", gloss: "以白茅藉垫祭物，无咎。" },
            LineText { classic: "枯杨生稊，老夫得其女妻，无不利。", gloss: "枯杨生新芽，老夫得少妻，无不利。" },
            LineText { classic: "栋桡，凶。", gloss: "屋栋桡曲将折，凶。" },
            LineText { classic: "栋隆，吉，有它吝。", gloss: "屋栋隆起，吉，但另有它吝。" },
            LineText { classic: "枯杨生华，老妇得其士夫，无咎无誉。", gloss: "枯杨开花，老妇得壮夫，无咎无誉。" },
            LineText { classic: "过涉灭顶，凶，无咎。", gloss: "涉水灭顶，凶，然志在必为无咎。" },
        ],
    },
    // 31 泽天夬
    HexagramText {
        judgment: LineText { classic: "夬：扬于王庭，孚号有厉，告自邑，不利即戎，利有攸往。", gloss: "夬卦：于王庭显扬，诚信号呼有厉，自邑告诫，不利于即戎，利有所往。" },
        lines: [
            LineText { classic: "壮于前趾，往不胜为咎。", gloss: "壮于前趾，往不胜则咎。" },
            LineText { classic: "惕号，莫夜有戎，勿恤。", gloss: "警惧号呼，暮夜有兵戎，勿忧。" },
            LineText { classic: "壮于頄，有凶。君子夬夬，独行遇雨，若濡有愠，无咎。", gloss: "壮于面颊，有凶。君子决然独行遇雨，虽濡有愠，无咎。" },
            LineText { classic: "臀无肤，其行次且，牵羊悔亡，闻言不信。", gloss: "臀无肤，行进迟缓，牵羊悔亡，闻言不信。" },
            LineText { classic: "苋陆夬夬，中行无咎。", gloss: "如苋陆果决去除，中道而行无咎。" },
            LineText { classic: "无号，终有凶。", gloss: "无人号呼，终有凶。" },
        ],
    },
    // 32 山地剥
    HexagramText {
        judgment: LineText { classic: "剥：不利有攸往。", gloss: "剥卦：不利有所往。" },
        lines: [
            LineText { classic: "剥床以足，蔑贞凶。", gloss: "剥床自足始，灭贞则凶。" },
            LineText { classic: "剥床以辨，蔑贞凶。", gloss: "剥床至辨，灭贞则凶。" },
            LineText { classic: "剥之，无咎。", gloss: "剥而无咎。" },
            LineText { classic: "剥床以肤，凶。", gloss: "剥床至于肤，凶。" },
            LineText { classic: "贯鱼，以宫人宠，无不利。", gloss: "贯鱼而入，如宫人受宠，无不利。" },
            LineText { classic: "硕果不食，君子得舆，小人剥庐。", gloss: "硕果不食，君子得车，小人剥其庐。" },
        ],
    },
    // 33 山雷颐
    HexagramText {
        judgment: LineText { classic: "颐：贞吉。观颐，自求口实。", gloss: "颐卦守正吉。观其所养，自求口实。" },
        lines: [
            LineText { classic: "舍尔灵龟，观我朵颐，凶。", gloss: "舍去你的灵龟，只羡我大嚼，凶。" },
            LineText { classic: "颠颐，拂经，于丘颐，征凶。", gloss: "颠倒养道，违背常法，求养于丘，征凶。" },
            LineText { classic: "拂颐，贞凶，十年勿用，无攸利。", gloss: "违养道，守贞凶，十年勿用，无所利。" },
            LineText { classic: "颠颐吉，虎视眈眈，其欲逐逐，无咎。", gloss: "颠颐则吉，虎视眈眈而欲逐逐，无咎。" },
            LineText { classic: "拂经，居贞吉，不可涉大川。", gloss: "违常经，居守正吉，不可涉大川。" },
            LineText { classic: "由颐，厉吉，利涉大川。", gloss: "由以养道，厉而吉，利涉大川。" },
        ],
    },
    // 34 山水蒙
    HexagramText {
        judgment: LineText { classic: "蒙：亨。匪我求童蒙，童蒙求我。初筮告，再三渎，渎则不告。利贞。", gloss: "蒙卦亨通。非我求童蒙，乃童蒙求我。初筮则告，再三则渎，渎则不告。利守正。" },
        lines: [
            LineText { classic: "发蒙，利用刑人，用说桎梏，以往吝。", gloss: "启发蒙昧，宜用刑人以脱桎梏，既往则吝。" },
            LineText { classic: "包蒙吉，纳妇吉，子克家。", gloss: "包容蒙昧吉，纳妇吉，子能克家。" },
            LineText { classic: "勿用取女，见金夫，不有躬，无攸利。", gloss: "勿取此女，见金夫而失身，无所利。" },
            LineText { classic: "困蒙，吝。", gloss: "困于蒙昧，吝。" },
            LineText { classic: "童蒙，吉。", gloss: "童稚蒙昧，吉。" },
            LineText { classic: "击蒙，不利为寇，利御寇。", gloss: "击退蒙昧，不利为寇，利御寇。" },
        ],
    },
    // 35 山泽损
    HexagramText {
        judgment: LineText { classic: "损：有孚，元吉，无咎，可贞，利有攸往。曷之用，二簋可用享。", gloss: "损卦有诚信，元吉无咎，可守贞，利有所往。何用鲜饰，二簋即可用享。" },
        lines: [
            LineText { classic: "已事遄往，无咎，酌损之。", gloss: "讫事速往，无咎，酌情减损。" },
            LineText { classic: "利贞，征凶，弗损益之。", gloss: "利守正，征凶，不可损而当益之。" },
            LineText { classic: "三人行，则损一人；一人行，则得其友。", gloss: "三人行损一人，一人行得其友。" },
            LineText { classic: "损其疾，使遄有喜，无咎。", gloss: "治其病使速有喜，无咎。" },
            LineText { classic: "或益之十朋之龟，弗克违，元吉。", gloss: "或益以十贝之龟，不可拒，元吉。" },
            LineText { classic: "弗损益之，无咎，贞吉，利有攸往，得臣无家。", gloss: "不损反益之，无咎，贞吉，利有所往，得臣无家。" },
        ],
    },
    // 36 艮为山
    HexagramText {
        judgment: LineText { classic: "艮：艮其背，不获其身，行其庭，不见其人，无咎。", gloss: "艮卦止于背，不见其身；行于庭不见其人，无咎。" },
        lines: [
            LineText { classic: "艮其趾，无咎，利永贞。", gloss: "止于足趾，无咎，利于长久守正。" },
            LineText { classic: "艮其腓，不拯其随，其心不快。", gloss: "止于小腿，不能救助相随者，其心不快。" },
            LineText { classic: "艮其限，列其夤，厉熏心。", gloss: "止于腰部，脊肉分裂，厉如薰心。" },
            LineText { classic: "艮其身，无咎。", gloss: "止于其身，无咎。" },
            LineText { classic: "艮其辅，言有序，悔亡。", gloss: "止于口辅，言有序，悔亡。" },
            LineText { classic: "敦艮，吉。", gloss: "敦厚而止，吉。" },
        ],
    },
    // 37 山火贲
    HexagramText {
        judgment: LineText { classic: "贲：亨。小利有攸往。", gloss: "贲卦亨通。小利有所往。" },
        lines: [
            LineText { classic: "贲其趾，舍车而徒。", gloss: "文饰其足趾，舍车徒步。" },
            LineText { classic: "贲其须。", gloss: "文饰其须。" },
            LineText { classic: "贲如濡如，永贞吉。", gloss: "文饰而润泽，长久守正吉。" },
            LineText { classic: "贲如皤如，白马翰如，匪寇婚媾。", gloss: "素白文饰，白马飞驰，非寇乃婚媾。" },
            LineText { classic: "贲于丘园，束帛戋戋，吝，终吉。", gloss: "文饰于丘园，束帛甚少，有吝，终吉。" },
            LineText { classic: "白贲，无咎。", gloss: "素白无饰，无咎。" },
        ],
    },
    // 38 山风蛊
    HexagramText {
        judgment: LineText { classic: "蛊：元亨，利涉大川。先甲三日，后甲三日。", gloss: "蛊卦元亨，利涉大川。先甲三日，后甲三日，慎始善后。" },
        lines: [
            LineText { classic: "干父之蛊，有子，考无咎，厉终吉。", gloss: "整治父之积弊，有子则考无咎，虽厉终吉。" },
            LineText { classic: "干母之蛊，不可贞。", gloss: "整治母之积弊，不可固执守正。" },
            LineText { classic: "干父之蛊，小有悔，无大咎。", gloss: "整治父之积弊，小有悔，无大咎。" },
            LineText { classic: "裕父之蛊，往见吝。", gloss: "宽纵父之积弊，往则有吝。" },
            LineText { classic: "干父之蛊，用誉。", gloss: "整治父之积弊，得美誉。" },
            LineText { classic: "不事王侯，高尚其事。", gloss: "不事王侯，高尚其志。" },
        ],
    },
    // 39 山天大畜
    HexagramText {
        judgment: LineText { classic: "大畜：利贞，不家食吉，利涉大川。", gloss: "大畜卦利守正，不食于家吉，利涉大川。" },
        lines: [
            LineText { classic: "有厉，利已。", gloss: "有危厉，宜止。" },
            LineText { classic: "舆说輹。", gloss: "车脱其輹。" },
            LineText { classic: "良马逐，利艰贞。曰闲舆卫，利有攸往。", gloss: "良马驰逐，利艰中守正。闲习车卫，利有所往。" },
            LineText { classic: "童牛之牿，元吉。", gloss: "童牛加牿，元吉。" },
            LineText { classic: "豮豕之牙，吉。", gloss: "豮豕之牙，吉。" },
            LineText { classic: "何天之衢，亨。", gloss: "荷天之衢，亨通。" },
        ],
    },
    // 40 火地晋
    HexagramText {
        judgment: LineText { classic: "晋：康侯用锡马蕃庶，昼日三接。", gloss: "晋卦：康侯受赐众多马匹，一日三接。" },
        lines: [
            LineText { classic: "晋如，摧如，贞吉。罔孚，裕无咎。", gloss: "晋进而受摧，守正吉。未得信，宽裕无咎。" },
            LineText { classic: "晋如，愁如，贞吉。受兹介福，于其王母。", gloss: "晋进而愁，守正吉。受此大福，于王母。" },
            LineText { classic: "众允，悔亡。", gloss: "众人允许，悔亡。" },
            LineText { classic: "晋如鼫鼠，贞厉。", gloss: "晋进如鼫鼠贪而怯，守贞有厉。" },
            LineText { classic: "悔亡，失得勿恤，往吉无不利。", gloss: "悔亡，得失勿忧，往吉无不利。" },
            LineText { classic: "晋其角，维用伐邑，厉吉无咎，贞吝。", gloss: "晋至于角，唯用伐邑，厉而吉无咎，守贞有吝。" },
        ],
    },
    // 41 火雷噬嗑
    HexagramText {
        judgment: LineText { classic: "噬嗑：亨，利用狱。", gloss: "噬嗑卦亨通，利于用狱断案。" },
        lines: [
            LineText { classic: "屦校灭趾，无咎。", gloss: "戴校灭趾，无咎。小惩大诫。" },
            LineText { classic: "噬肤灭鼻，无咎。", gloss: "噬皮灭鼻，无咎。" },
            LineText { classic: "噬腊肉，遇毒，小吝，无咎。", gloss: "噬腊肉遇毒，小吝，无咎。" },
            LineText { classic: "噬干胏，得金矢，利艰贞，吉。", gloss: "噬带骨干肉得金矢，利艰贞，吉。" },
            LineText { classic: "噬干肉，得黄金，贞厉无咎。", gloss: "噬干肉得黄金，守贞厉无咎。" },
            LineText { classic: "何校灭耳，凶。", gloss: "荷校灭耳，凶。" },
        ],
    },
    // 42 火水未济
    HexagramText {
        judgment: LineText { classic: "未济：亨，小狐汔济，濡其尾，无攸利。", gloss: "未济卦亨通，小狐将渡而濡尾，无所利。" },
        lines: [
            LineText { classic: "濡其尾，吝。", gloss: "濡其尾，吝。" },
            LineText { classic: "曳其轮，贞吉。", gloss: "拖曳车轮，守正吉。" },
            LineText { classic: "未济，征凶，利涉大川。", gloss: "未济之时，征凶，但利涉大川。" },
            LineText { classic: "贞吉，悔亡，震用伐鬼方，三年有赏于大国。", gloss: "守正吉悔亡，用震动之力伐鬼方，三年受赏于大国。" },
            LineText { classic: "贞吉，无悔，君子之光，有孚，吉。", gloss: "守正吉无悔，君子之光有诚信，吉。" },
            LineText { classic: "有孚于饮酒，无咎，濡其首，有孚失是。", gloss: "诚信于饮酒，无咎，濡其首则失其诚信之道。" },
        ],
    },
    // 43 火泽睽
    HexagramText {
        judgment: LineText { classic: "睽：小事吉。", gloss: "睽卦：小事吉。" },
        lines: [
            LineText { classic: "悔亡，丧马勿逐，自复。见恶人无咎。", gloss: "悔亡，丧马勿逐自复。见恶人无咎。" },
            LineText { classic: "遇主于巷，无咎。", gloss: "巷中遇主，无咎。" },
            LineText { classic: "见舆曳，其牛掣，其人天且劓，无初有终。", gloss: "见车被曳，其牛受掣，其人受天劓之刑，无初有终。" },
            LineText { classic: "睽孤，遇元夫，交孚，厉无咎。", gloss: "睽违而孤，遇初见之人相交以诚，有厉无咎。" },
            LineText { classic: "悔亡，厥宗噬肤，往何咎。", gloss: "悔亡，其宗人相噬其肤，往有何咎。" },
            LineText { classic: "睽孤，见豕负涂，载鬼一车，先张之弧，后说之弧，匪寇婚媾，往遇雨则吉。", gloss: "睽孤见豕涂载，鬼车张弧又说弧，非寇乃婚媾，往遇雨则吉。" },
        ],
    },
    // 44 火山旅
    HexagramText {
        judgment: LineText { classic: "旅：小亨，旅贞吉。", gloss: "旅卦小亨，行旅守正吉。" },
        lines: [
            LineText { classic: "旅琐琐，斯其所取灾。", gloss: "行旅琐屑卑屑，自取灾患。" },
            LineText { classic: "旅即次，怀其资，得童仆贞。", gloss: "旅居有次舍，怀其资财，得童仆而贞。" },
            LineText { classic: "旅焚其次，丧其童仆，贞厉。", gloss: "旅次被焚，丧其童仆，守贞有厉。" },
            LineText { classic: "旅于处，得其资斧，我心不快。", gloss: "旅居处得资斧，我心不快。" },
            LineText { classic: "射雉一矢亡，终以誉命。", gloss: "射雉虽失一矢，终以誉命得之。" },
            LineText { classic: "鸟焚其巢，旅人先笑后号咷，丧牛于易，凶。", gloss: "鸟焚其巢，旅人先笑后号咷，丧牛于易，凶。" },
        ],
    },
    // 45 离为火
    HexagramText {
        judgment: LineText { classic: "离：利贞，亨。畜牝牛，吉。", gloss: "离卦利守正，亨通。畜养牝牛吉。" },
        lines: [
            LineText { classic: "履错然，敬之无咎。", gloss: "履步错杂，敬慎则无咎。" },
            LineText { classic: "黄离，元吉。", gloss: "黄色之明丽，元吉。" },
            LineText { classic: "日昃之离，不鼓缶而歌，则大耋之嗟，凶。", gloss: "日昃之离，不鼓缶而歌则老人嗟叹，凶。" },
            LineText { classic: "突如其来如，焚如，死如，弃如。", gloss: "突然而来如焚如死如弃如，凶。不受位之人。" },
            LineText { classic: "出涕沱若，戚嗟若，吉。", gloss: "出涕如沱，悲戚嗟叹，吉。" },
            LineText { classic: "王用出征有嘉，折首获匪其丑，无咎。", gloss: "王用出征而有嘉功，斩首获其丑类，无咎。" },
        ],
    },
    // 46 火风鼎
    HexagramText {
        judgment: LineText { classic: "鼎：元吉，亨。", gloss: "鼎卦元吉，亨通。" },
        lines: [
            LineText { classic: "鼎颠趾，利出否，得妾以其子，无咎。", gloss: "鼎覆足倒利于倾出秽物，得妾以其子，无咎。" },
            LineText { classic: "鼎有实，我仇有疾，不我能即，吉。", gloss: "鼎中有实，我仇有疾不能近我，吉。" },
            LineText { classic: "鼎耳革，其行塞，雉膏不食，方雨亏悔，终吉。", gloss: "鼎耳革变，其行阻塞，雉膏不得食，方雨而悔亏，终吉。" },
            LineText { classic: "鼎折足，覆公餗，其形渥，凶。", gloss: "鼎折足，覆公之餗，其形渥，凶。" },
            LineText { classic: "鼎黄耳金铉，利贞。", gloss: "鼎黄耳金铉，利守正。" },
            LineText { classic: "鼎玉铉，大吉，无不利。", gloss: "鼎玉铉，大吉无不利。" },
        ],
    },
    // 47 火天大有
    HexagramText {
        judgment: LineText { classic: "大有：元亨。", gloss: "大有卦元亨。" },
        lines: [
            LineText { classic: "无交害，匪咎，艰则无咎。", gloss: "无相交之害，本非咎，艰则无咎。" },
            LineText { classic: "大车以载，有攸往，无咎。", gloss: "大车以载，有所往，无咎。" },
            LineText { classic: "公用亨于天子，小人弗克。", gloss: "公亨于天子，小人不能当。" },
            LineText { classic: "匪其彭，无咎。", gloss: "非过盛之彭，无咎。" },
            LineText { classic: "厥孚交如，威如，吉。", gloss: "其孚交而威，吉。" },
            LineText { classic: "自天佑之，吉无不利。", gloss: "自天佑之，吉无不利。" },
        ],
    },
    // 48 风地观
    HexagramText {
        judgment: LineText { classic: "观：盥而不荐，有孚颙若。", gloss: "观卦：盥洗未荐，诚信颙敬。" },
        lines: [
            LineText { classic: "童观，小人无咎，君子吝。", gloss: "童稚之观，小人无咎，君子则有吝。" },
            LineText { classic: "窥观，利女贞。", gloss: "窥牖而观，利女守贞。" },
            LineText { classic: "观我生，进退。", gloss: "观我所生以定进退。" },
            LineText { classic: "观国之光，利用宾于王。", gloss: "观国之光荣，宜为王者宾。" },
            LineText { classic: "观我生，君子无咎。", gloss: "观我之所生，君子无咎。" },
            LineText { classic: "观其生，君子无咎。", gloss: "观其民之所生，君子无咎。" },
        ],
    },
    // 49 风雷益
    HexagramText {
        judgment: LineText { classic: "益：利有攸往，利涉大川。", gloss: "益卦利有所往，利涉大川。" },
        lines: [
            LineText { classic: "利用为大作，元吉无咎。", gloss: "宜为大作，元吉无咎。" },
            LineText { classic: "或益之十朋之龟，弗克违，永贞吉，王用享于帝，吉。", gloss: "或益以十贝之龟，不可拒，永贞吉，王享于帝，吉。" },
            LineText { classic: "益之用凶事，无咎，有孚中行，告公用圭。", gloss: "益用于凶事，无咎，有孚中行，告公用圭。" },
            LineText { classic: "中行告公从，利用为依迁国。", gloss: "中行告公而从，宜依之迁国。" },
            LineText { classic: "有孚惠心，勿问元吉，有孚惠我德。", gloss: "有诚信惠人之心，勿问元吉，人亦以诚信惠我德。" },
            LineText { classic: "莫益之，或击之，立心勿恒，凶。", gloss: "无人益之反或击之，立心不恒，凶。" },
        ],
    },
    // 50 风水涣
    HexagramText {
        judgment: LineText { classic: "涣：亨。王假有庙，利涉大川，利贞。", gloss: "涣卦亨通。王至于庙，利涉大川，利守正。" },
        lines: [
            LineText { classic: "用拯马壮，吉。", gloss: "用拯以壮马，吉。" },
            LineText { classic: "涣奔其机，悔亡。", gloss: "涣散奔就机座，悔亡。" },
            LineText { classic: "涣其躬，无悔。", gloss: "涣散其身，无悔。" },
            LineText { classic: "涣其群，元吉，涣有丘，匪夷所思。", gloss: "涣散其朋党，元吉，涣有丘聚，非常人所能思。" },
            LineText { classic: "涣汗其大号，涣王居，无咎。", gloss: "涣汗其大号，涣王之居，无咎。" },
            LineText { classic: "涣其血，去逖出，无咎。", gloss: "涣散其血，远去而出，无咎。" },
        ],
    },
    // 51 风泽中孚
    HexagramText {
        judgment: LineText { classic: "中孚：豚鱼吉，利涉大川，利贞。", gloss: "中孚卦：豚鱼亦吉，利涉大川，利守正。" },
        lines: [
            LineText { classic: "虞吉，有它不燕。", gloss: "安虞则吉，有他则不安。" },
            LineText { classic: "鸣鹤在阴，其子和之。我有好爵，吾与尔靡之。", gloss: "鸣鹤在阴，其子和之。我有好爵，与你共靡之。" },
            LineText { classic: "得敌，或鼓或罢，或泣或歌。", gloss: "得敌人，或鼓或罢，或泣或歌。" },
            LineText { classic: "月几望，马匹亡，无咎。", gloss: "月近圆，马匹亡，无咎。" },
            LineText { classic: "有孚挛如，无咎。", gloss: "有诚信挛结如一，无咎。" },
            LineText { classic: "翰音登于天，贞凶。", gloss: "翰音登于天，守贞凶。" },
        ],
    },
    // 52 风山渐
    HexagramText {
        judgment: LineText { classic: "渐：女归吉，利贞。", gloss: "渐卦：女子归嫁吉，利守正。" },
        lines: [
            LineText { classic: "鸿渐于干，小子厉，有言无咎。", gloss: "鸿渐于水涯，小子有厉，虽言语之争无咎。" },
            LineText { classic: "鸿渐于磐，饮食衎衎，吉。", gloss: "鸿渐于磐石，饮食和乐，吉。" },
            LineText { classic: "鸿渐于陆，夫征不复，妇孕不育，凶，利御寇。", gloss: "鸿渐于陆，夫征不复，妇孕不育，凶，利御寇。" },
            LineText { classic: "鸿渐于木，或得其桷，无咎。", gloss: "鸿渐于木，或得平柯，无咎。" },
            LineText { classic: "鸿渐于陵，妇三岁不孕，终莫之胜，吉。", gloss: "鸿渐于陵，妇三年不孕，终无人能胜之，吉。" },
            LineText { classic: "鸿渐于阿，其羽可用为仪，吉。", gloss: "鸿渐于阿，其羽可用为仪，吉。" },
        ],
    },
    // 53 风火家人
    HexagramText {
        judgment: LineText { classic: "家人：利女贞。", gloss: "家人卦利女守贞。" },
        lines: [
            LineText { classic: "闲有家，悔亡。", gloss: "防闲其家，悔亡。" },
            LineText { classic: "无攸遂，在中馈，贞吉。", gloss: "无所自营，主中馈之事，守正吉。" },
            LineText { classic: "家人嗃嗃，悔厉吉。妇子嘻嘻，终吝。", gloss: "家人嗃嗃严治，悔厉而吉。妇子嘻嘻失节，终吝。" },
            LineText { classic: "富家，大吉。", gloss: "富其家，大吉。" },
            LineText { classic: "王假有家，勿恤吉。", gloss: "王至于家，勿忧吉。" },
            LineText { classic: "有孚威如，终吉。", gloss: "有诚信威严，终吉。" },
        ],
    },
    // 54 巽为风
    HexagramText {
        judgment: LineText { classic: "巽：小亨，利有攸往，利见大人。", gloss: "巽卦小亨，利有所往，利见大人。" },
        lines: [
            LineText { classic: "进退，利武人之贞。", gloss: "进退犹豫，宜武人之守正。" },
            LineText { classic: "巽在床下，用史巫纷若，吉无咎。", gloss: "巽在床下，用史巫纷若祭之，吉无咎。" },
            LineText { classic: "频巽，吝。", gloss: "频频巽顺，吝。" },
            LineText { classic: "悔亡，田获三品。", gloss: "悔亡，田猎获三品。" },
            LineText { classic: "贞吉悔亡，无不利，无初有终。先庚三日，后庚三日，吉。", gloss: "守贞吉悔亡，无不利，无初有终。先庚三日，后庚三日，吉。" },
            LineText { classic: "巽在床下，丧其资斧，贞凶。", gloss: "巽在床下，丧其资斧，守贞凶。" },
        ],
    },
    // 55 风天小畜
    HexagramText {
        judgment: LineText { classic: "小畜：亨。密云不雨，自我西郊。", gloss: "小畜卦亨通。密云不雨，起自我西郊。" },
        lines: [
            LineText { classic: "复自道，何其咎，吉。", gloss: "复自其道，何咎之有，吉。" },
            LineText { classic: "牵复，吉。", gloss: "牵连而复，吉。" },
            LineText { classic: "舆说辐，夫妻反目。", gloss: "车脱其辐，夫妻反目。" },
            LineText { classic: "有孚，血去惕出，无咎。", gloss: "有诚信，去血出惕，无咎。" },
            LineText { classic: "有孚挛如，富以其邻。", gloss: "有诚信挛结如一，富以其邻。" },
            LineText { classic: "既雨既处，尚德载，妇贞厉，月几望，君子征凶。", gloss: "既雨既止，尚德而载，妇贞厉，月近望，君子征凶。" },
        ],
    },
    // 56 天地否
    HexagramText {
        judgment: LineText { classic: "否：否之匪人，不利君子贞，大往小来。", gloss: "否卦：闭塞之非人，不利君子守贞，大者往上小者来下。" },
        lines: [
            LineText { classic: "拔茅茹，以其汇，贞吉，亨。", gloss: "拔茅连根同类，守贞吉，亨通。" },
            LineText { classic: "包承，小人吉，大人否，亨。", gloss: "包容承受，小人吉，大人处否，亨。" },
            LineText { classic: "包羞。", gloss: "包藏羞辱。" },
            LineText { classic: "有命无咎，畴离祉。", gloss: "有命则无咎，同类共受其祉。" },
            LineText { classic: "休否，大人吉。其亡其亡，系于苞桑。", gloss: "休止其否，大人吉。心忧其亡，系于苞桑。" },
            LineText { classic: "倾否，先否后喜。", gloss: "倾覆其否，先否后喜。" },
        ],
    },
    // 57 天雷无妄
    HexagramText {
        judgment: LineText { classic: "无妄：元亨，利贞。其匪正有眚，不利有攸往。", gloss: "无妄卦元亨，利贞。若行匪正则眚，不利有所往。" },
        lines: [
            LineText { classic: "无妄，往吉。", gloss: "无妄而往，吉。" },
            LineText { classic: "不耕获，不菑畲，则利有攸往。", gloss: "不耕而获不菑而畲，则利有所往。" },
            LineText { classic: "无妄之灾，或系之牛，行人之得，邑人之灾。", gloss: "无妄之灾，或系之牛，行人得之，邑人受其灾。" },
            LineText { classic: "可贞，无咎。", gloss: "可守正，无咎。" },
            LineText { classic: "无妄之疾，勿药有喜。", gloss: "无妄之疾，勿药自喜。" },
            LineText { classic: "无妄行，有眚，无攸利。", gloss: "无妄之时妄行有眚，无所利。" },
        ],
    },
    // 58 天水讼
    HexagramText {
        judgment: LineText { classic: "讼：有孚，窒惕，中吉，终凶。利见大人，不利涉大川。", gloss: "讼卦有诚信，窒惕戒惧，中吉终凶。利见大人，不利涉大川。" },
        lines: [
            LineText { classic: "不永所事，小有言，终吉。", gloss: "不长讼其事，小有口舌，终吉。" },
            LineText { classic: "不克讼，归而逋其邑人三百户，无眚。", gloss: "讼不胜，归而逃匿其邑人三百户，无眚。" },
            LineText { classic: "食旧德，贞厉，终吉。或从王事，无成。", gloss: "食其旧德，守贞厉而终吉。或从王事，无成。" },
            LineText { classic: "不克讼，复即命，渝，安贞吉。", gloss: "讼不胜，复归于命而变渝，安贞则吉。" },
            LineText { classic: "讼元吉。", gloss: "讼而得理，元吉。" },
            LineText { classic: "或锡之鞶带，终朝三褫之。", gloss: "或赐以鞶带，一朝三夺之。" },
        ],
    },
    // 59 天泽履
    HexagramText {
        judgment: LineText { classic: "履：履虎尾，不咥人，亨。", gloss: "履卦：履虎尾，虎不咥人，亨通。" },
        lines: [
            LineText { classic: "素履往无咎。", gloss: "素其履而往，无咎。" },
            LineText { classic: "履道坦坦，幽人贞吉。", gloss: "履道平坦，幽人守贞吉。" },
            LineText { classic: "眇能视，跛能履，履虎尾，咥人凶。武人为于大君。", gloss: "眇而视跛而履，履虎尾咥人凶。武人为大君。" },
            LineText { classic: "履虎尾，愬愬终吉。", gloss: "履虎尾而愬愬戒惧，终吉。" },
            LineText { classic: "夬履，贞厉。", gloss: "果决而履，守贞有厉。" },
            LineText { classic: "视履考祥，其旋元吉。", gloss: "视履考其吉祥，其旋反元吉。" },
        ],
    },
    // 60 天山遁
    HexagramText {
        judgment: LineText { classic: "遁：亨，小利贞。", gloss: "遁卦亨通，小利守贞。" },
        lines: [
            LineText { classic: "遁尾，厉，勿用有攸往。", gloss: "遁居其尾，厉，勿有所往。" },
            LineText { classic: "执之用黄牛之革，莫之胜说。", gloss: "执以黄牛之革固守，莫能脱说。" },
            LineText { classic: "系遁，有疾厉，畜臣妾吉。", gloss: "系居其遁，有疾厉，畜臣妾吉。" },
            LineText { classic: "好遁，君子吉，小人否。", gloss: "好而能遁，君子吉，小人否。" },
            LineText { classic: "嘉遁，贞吉。", gloss: "嘉而能遁，守贞吉。" },
            LineText { classic: "肥遁，无不利。", gloss: "宽裕其遁，无不利。" },
        ],
    },
    // 61 天火同人
    HexagramText {
        judgment: LineText { classic: "同人：同人于野，亨。利涉大川，利君子贞。", gloss: "同人卦：与野之人同心，亨。利涉大川，利君子守贞。" },
        lines: [
            LineText { classic: "同人于门，无咎。", gloss: "同人与门，无咎。" },
            LineText { classic: "同人于宗，吝。", gloss: "同人与宗族，吝。" },
            LineText { classic: "伏戎于莽，升其高陵，三岁不兴。", gloss: "伏兵于莽，登高陵，三年不兴。" },
            LineText { classic: "乘其墉，弗克攻，吉。", gloss: "乘其城墙而不能克攻，吉。" },
            LineText { classic: "同人，先号咷而后笑，大师克相遇。", gloss: "同人，先号咷而后笑，大师克胜相遇。" },
            LineText { classic: "同人于郊，无悔。", gloss: "同人与郊，无悔。" },
        ],
    },
    // 62 天风姤
    HexagramText {
        judgment: LineText { classic: "姤：女壮，勿用取女。", gloss: "姤卦：女壮，勿用娶此女。" },
        lines: [
            LineText { classic: "系于金柅，贞吉，有攸往，见凶，羸豕孚蹢躅。", gloss: "系于金柅，守贞吉，有所往见凶，羸豕欲动蹢躅。" },
            LineText { classic: "包有鱼，无咎，不利宾。", gloss: "包中有鱼，无咎，不利于宾。" },
            LineText { classic: "臀无肤，其行次且，厉，无大咎。", gloss: "臀无肤，行迟缓，有厉，无大咎。" },
            LineText { classic: "包无鱼，起凶。", gloss: "包中无鱼，起凶。" },
            LineText { classic: "以杞包瓜，含章，有陨自天。", gloss: "以杞叶包瓜，含章美，有陨自天。" },
            LineText { classic: "姤其角，吝，无咎。", gloss: "姤遇其角，吝，无咎。" },
        ],
    },
    // 63 乾为天
    HexagramText {
        judgment: LineText { classic: "乾：元，亨，利，贞。", gloss: "乾卦：元始、亨通、和利、守正。" },
        lines: [
            LineText { classic: "潜龙勿用。", gloss: "潜藏之龙，不宜妄动。时机未至。" },
            LineText { classic: "见龙在田，利见大人。", gloss: "见龙在田，利于出现大德之人。" },
            LineText { classic: "君子终日乾乾，夕惕若厉，无咎。", gloss: "君子终日勤勉不息，夜晚警惕，虽厉无咎。" },
            LineText { classic: "或跃在渊，无咎。", gloss: "或跃于深渊，进退审时，无咎。" },
            LineText { classic: "飞龙在天，利见大人。", gloss: "飞龙在天，利于出现大德之人。" },
            LineText { classic: "亢龙有悔。", gloss: "龙飞过高而有悔。盛极必衰。" },
        ],
    },
];

// ── 占断解读 ───────────────────────────────────────────
//
// 传统占法取读规则：
// - 无变爻 → 取本卦卦辞
// - 一至五个变爻 → 取各变爻对应的本卦爻辞（按 `changing_indices` 取本卦六爻文本）
// - 全六爻皆变 → 取之卦卦辞

/// 一条占断解读：标签（如"卦辞"、"初六"）+ 原文 + 释义。
pub struct InterpretationEntry {
    pub label: String,
    pub classic: String,
    pub gloss: String,
}

/// 按传统占法由起卦结果生成解读条目。
pub fn interpretation(result: &HexagramResult) -> Vec<InterpretationEntry> {
    let text = &HEXAGRAM_TEXTS[result.primary.index];

    // 全六爻皆变：读取之卦（变后卦象）的卦辞。
    if result.changing_indices.len() == 6 {
        let relating = result
            .relating
            .expect("all six lines change => relating must exist");
        let relating_text = &HEXAGRAM_TEXTS[relating.index];
        return vec![InterpretationEntry {
            label: "卦辞".to_string(),
            classic: relating_text.judgment.classic.to_string(),
            gloss: relating_text.judgment.gloss.to_string(),
        }];
    }

    // 无变爻：读取本卦卦辞。
    if result.changing_indices.is_empty() {
        return vec![InterpretationEntry {
            label: "卦辞".to_string(),
            classic: text.judgment.classic.to_string(),
            gloss: text.judgment.gloss.to_string(),
        }];
    }

    // 一至五个变爻：逐变爻取本卦对应爻辞。
    result
        .changing_indices
        .iter()
        .zip(result.changing_lines.iter())
        .map(|(&idx, label)| {
            let line_text = &text.lines[idx];
            InterpretationEntry {
                label: label.clone(),
                classic: line_text.classic.to_string(),
                gloss: line_text.gloss.to_string(),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{
        analyze_hexagram, format_changing_line, interpretation, line_label, line_symbol,
        transformed_line_value, HEXAGRAM_NAMES, HEXAGRAM_TEXTS,
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

    #[test]
    fn parameterized_hexagram_mapping() {
        struct Case {
            label: &'static str,
            lines: [u8; 6],
            primary_index: usize,
            primary_name: &'static str,
            changing: Vec<String>,
            relating_name: Option<&'static str>,
            transformed: [u8; 6],
        }

        let cases = [
            Case {
                label: "全少阴为坤",
                lines: [8, 8, 8, 8, 8, 8],
                primary_index: 0,
                primary_name: "坤为地",
                changing: vec![],
                relating_name: None,
                transformed: [8, 8, 8, 8, 8, 8],
            },
            Case {
                label: "下乾上坤为泰",
                lines: [7, 7, 7, 8, 8, 8],
                primary_index: 7,
                primary_name: "地天泰",
                changing: vec![],
                relating_name: None,
                transformed: [7, 7, 7, 8, 8, 8],
            },
            Case {
                label: "下坤上乾为否",
                lines: [8, 8, 8, 7, 7, 7],
                primary_index: 56,
                primary_name: "天地否",
                changing: vec![],
                relating_name: None,
                transformed: [8, 8, 8, 7, 7, 7],
            },
            Case {
                label: "双坎为水",
                lines: [8, 7, 8, 8, 7, 8],
                primary_index: 18,
                primary_name: "坎为水",
                changing: vec![],
                relating_name: None,
                transformed: [8, 7, 8, 8, 7, 8],
            },
            Case {
                label: "全老阳本乾之坤",
                lines: [9, 9, 9, 9, 9, 9],
                primary_index: 63,
                primary_name: "乾为天",
                changing: vec![
                    "初九".to_string(),
                    "九二".to_string(),
                    "九三".to_string(),
                    "九四".to_string(),
                    "九五".to_string(),
                    "上九".to_string(),
                ],
                relating_name: Some("坤为地"),
                transformed: [8, 8, 8, 8, 8, 8],
            },
            Case {
                label: "全老阴本坤之乾",
                lines: [6, 6, 6, 6, 6, 6],
                primary_index: 0,
                primary_name: "坤为地",
                changing: vec![
                    "初六".to_string(),
                    "六二".to_string(),
                    "六三".to_string(),
                    "六四".to_string(),
                    "六五".to_string(),
                    "上六".to_string(),
                ],
                relating_name: Some("乾为天"),
                transformed: [7, 7, 7, 7, 7, 7],
            },
            Case {
                label: "含变爻初九六四",
                lines: [9, 8, 7, 6, 8, 7],
                primary_index: 37,
                primary_name: "山火贲",
                changing: vec!["初九".to_string(), "六四".to_string()],
                relating_name: Some("火山旅"),
                transformed: [8, 8, 7, 7, 8, 7],
            },
        ];

        for case in cases {
            let result = analyze_hexagram(&case.lines);
            assert_eq!(
                result.primary.index, case.primary_index,
                "{}: primary index",
                case.label
            );
            assert_eq!(
                result.primary.name, case.primary_name,
                "{}: primary name",
                case.label
            );
            assert_eq!(
                result.changing_lines, case.changing,
                "{}: changing lines",
                case.label
            );
            assert_eq!(
                result.relating.map(|r| r.name),
                case.relating_name,
                "{}: relating name",
                case.label
            );
            assert_eq!(
                result.transformed_lines, case.transformed,
                "{}: transformed lines",
                case.label
            );
        }
    }

    #[test]
    fn hexagram_texts_complete() {
        // 64 卦 · 每卦 1 卦辞 + 6 爻辞 = 448 条，且均非空。
        assert_eq!(HEXAGRAM_TEXTS.len(), 64);
        assert_eq!(HEXAGRAM_NAMES.len(), 64);
        let mut total = 0usize;
        for (idx, hex) in HEXAGRAM_TEXTS.iter().enumerate() {
            assert!(!hex.judgment.classic.is_empty(), "judgment classic @{}", idx);
            assert!(!hex.judgment.gloss.is_empty(), "judgment gloss @{}", idx);
            total += 1;
            for (l, line) in hex.lines.iter().enumerate() {
                assert!(!line.classic.is_empty(), "line {} classic @{}", l, idx);
                assert!(!line.gloss.is_empty(), "line {} gloss @{}", l, idx);
                total += 1;
            }
        }
        assert_eq!(total, 448);
    }

    #[test]
    fn interpretation_no_changing_returns_judgment() {
        // 无变爻（全少阳）→ 单条卦辞。
        let result = analyze_hexagram(&[7, 7, 7, 7, 7, 7]);
        let entries = interpretation(&result);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].label, "卦辞");
        assert!(!entries[0].classic.is_empty());
        assert!(!entries[0].gloss.is_empty());
    }

    #[test]
    fn interpretation_with_changing_returns_line_texts() {
        // 初六 + 九四 两个变爻 → 两条对应爻辞，label 复用 changing_lines。
        let result = analyze_hexagram(&[6, 7, 8, 9, 8, 7]); // 火水未济
        assert_eq!(result.changing_lines, vec!["初六".to_string(), "九四".to_string()]);
        assert_eq!(result.changing_indices, vec![0, 3]);
        let entries = interpretation(&result);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].label, "初六");
        assert_eq!(entries[1].label, "九四");
        // 校验取的是本卦（未济，index 42）对应爻位的文本。
        let text = &HEXAGRAM_TEXTS[42];
        assert_eq!(entries[0].classic, text.lines[0].classic);
        assert_eq!(entries[1].classic, text.lines[3].classic);
        assert!(!entries[0].gloss.is_empty());
        assert!(!entries[1].gloss.is_empty());
    }

    #[test]
    fn interpretation_all_changing_returns_relating_judgment() {
        // 全六爻皆变（全老阳）→ 之卦（坤为地）卦辞。
        let result = analyze_hexagram(&[9, 9, 9, 9, 9, 9]);
        assert_eq!(result.changing_indices.len(), 6);
        let relating = result.relating.expect("relating exists for all changing");
        assert_eq!(relating.name, "坤为地");
        let entries = interpretation(&result);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].label, "卦辞");
        let kun_text = &HEXAGRAM_TEXTS[relating.index];
        assert_eq!(entries[0].classic, kun_text.judgment.classic);
        assert_eq!(entries[0].gloss, kun_text.judgment.gloss);
    }
}
