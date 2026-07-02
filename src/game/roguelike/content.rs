//! Narrative & event content for the roguelike run: chapter cards, story
//! scenes, the random-event pool, rest options, and endings.
//!
//! All text is original xianxia-flavoured writing for this project. Story
//! choices push the 道心 / 情缘 counters that pick the final ending.

// ---------------------------------------------------------------------------
// Effects an option can apply
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Effect {
    None,
    /// Heal percentage of max hp.
    HealPct(i32),
    /// Lose percentage of max hp (never lethal — clamps to 1).
    DamagePct(i32),
    GainPotions(u32),
    GainGold(u32),
    LoseGold(u32),
    GainAtk(i32),
    GainDef(i32),
    GainMaxHp(i32),
    GainMaxMp(i32),
    /// Grant a random relic the player does not own yet.
    RandomRelic,
}

#[derive(Clone, Copy)]
pub struct Outcome {
    pub lines: &'static [&'static str],
    pub effect: Effect,
    pub daoxin: i32,
    pub qingyuan: i32,
}

#[derive(Clone, Copy)]
pub struct EventOption {
    pub label: &'static str,
    /// Probability the option succeeds; `failure` is used otherwise.
    pub chance: f32,
    pub success: Outcome,
    pub failure: Option<Outcome>,
}

#[derive(Clone, Copy)]
pub struct RandomEvent {
    pub title: &'static str,
    pub lines: &'static [&'static str],
    pub options: &'static [EventOption],
}

// ---------------------------------------------------------------------------
// Chapter cards (shown once when a chapter map opens)
// ---------------------------------------------------------------------------

pub const CHAPTER_CARDS: [&[&str]; 4] = [
    &[
        "【第一卷 · 桃溪村誓】",
        "桃溪村外妖雾骤起,月魄妖踞于水月洞天,吞人魂魄。",
        "李逍遥背起木剑,在祠堂前立誓:雾散之前,绝不回头。",
        "灵儿把一枚桃木铃系在他腕上:「铃响三声,记得回家。」",
    ],
    &[
        "【第二卷 · 江雾疫火】",
        "顺流而下,江城灯火尽熄,疫气伏行水脉之间。",
        "药铺老医跪求:「疫有母根,不斩根,城中三千户皆成孤坟。」",
        "灵儿低声道:「这一路,怕是要见许多不忍见之事。」",
    ],
    &[
        "【第三卷 · 京华南疆】",
        "京城朱门之内,有人借妖行法;南疆雷泽之上,古麟被锁为刀。",
        "皇榜悬赏斩妖人,暗巷里却有人递来一句:「妖非祸源,人心才是。」",
        "腕上的桃木铃,第一次自己响了一声。",
    ],
    &[
        "【终卷 · 心渊照影】",
        "一切线索沉入心渊:水影之下,盘着以宿命为食的旧魔。",
        "它开口,用的竟是李逍遥自己的声音:「你斩的每一妖,都是你放不下的执念。」",
        "铃响第二声。再响一声,便是永别,或是归途。",
    ],
];

// ---------------------------------------------------------------------------
// Story scenes — the fixed 「缘」 node in each chapter
// ---------------------------------------------------------------------------

#[derive(Clone, Copy)]
pub struct StoryOption {
    pub label: &'static str,
    pub outcome: Outcome,
}

#[derive(Clone, Copy)]
pub struct StoryScene {
    pub lines: &'static [&'static str],
    pub prompt: &'static str,
    pub options: &'static [StoryOption],
}

pub const STORY_SCENES: [StoryScene; 4] = [
    StoryScene {
        lines: &[
            "溪边旧渡口,灵儿追了上来,发梢还沾着夜露。",
            "灵儿:「我知道拦不住你。可你要是不回来,这铃我就日日摇给山听。」",
            "她把伤药塞进你怀里,眼睛亮得像水月洞天里那汪泉。",
        ],
        prompt: "你如何作答?",
        options: &[
            StoryOption {
                label: "握住她的手:「等雾散了,我带你看真正的月亮。」",
                outcome: Outcome {
                    lines: &[
                        "灵儿愣了愣,忽然笑出声,把半块桂花糕也塞给你。",
                        "「那说好了。骗人的话,铃会记得。」",
                        "(情缘 +2,气血上限 +10)",
                    ],
                    effect: Effect::GainMaxHp(10),
                    daoxin: 0,
                    qingyuan: 2,
                },
            },
            StoryOption {
                label: "转身向山:「斩妖之事,不容儿女情长。」",
                outcome: Outcome {
                    lines: &[
                        "她没再说话,只把铃又系紧了一圈。",
                        "山风灌进衣袖,你的木剑忽然稳了几分。",
                        "(道心 +2,攻击 +2)",
                    ],
                    effect: Effect::GainAtk(2),
                    daoxin: 2,
                    qingyuan: 0,
                },
            },
        ],
    },
    StoryScene {
        lines: &[
            "疫村祠堂,一位母亲抱着昏睡的孩子跪在药炉前。",
            "老医叹道:「炉中只剩一副药引。给了孩子,你身上的旧伤便压不住了。」",
            "灵儿看着你,没有说话——她知道你听得见孩子的呼吸声。",
        ],
        prompt: "药引给谁?",
        options: &[
            StoryOption {
                label: "给孩子。旧伤而已,扛得住。",
                outcome: Outcome {
                    lines: &[
                        "孩子的热度退了。那位母亲朝你磕了三个头,你没敢受全。",
                        "灵儿悄悄把自己的披风撕了一半,替你裹紧旧伤。",
                        "(情缘 +2,损失两成气血,获得两枚药水)",
                    ],
                    effect: Effect::DamagePct(20),
                    daoxin: 0,
                    qingyuan: 2,
                },
            },
            StoryOption {
                label: "自己服下。斩不了瘴母,救谁都是空话。",
                outcome: Outcome {
                    lines: &[
                        "药力化开,经脉如洗。你在祠堂梁上刻下一行字:",
                        "「今日欠一命,他日以瘴母之首偿。」",
                        "(道心 +2,恢复三成气血,攻击 +2)",
                    ],
                    effect: Effect::HealPct(30),
                    daoxin: 2,
                    qingyuan: 0,
                },
            },
        ],
    },
    StoryScene {
        lines: &[
            "南疆雷泽,古麟锁于青铜柱间,鳞下压着一枚褪色的许愿笺。",
            "笺上是稚嫩笔迹:「愿阿爹平安归家。」——是国师幼年所书。",
            "灵儿轻声:「原来锁妖的人,也曾被谁这样盼着回家。」",
        ],
        prompt: "面对锁链,你——",
        options: &[
            StoryOption {
                label: "断链放麟:「被锁的从来不该是它。」",
                outcome: Outcome {
                    lines: &[
                        "古麟长鸣,雷光洗过山泽,替你劈开了前路的瘴气。",
                        "灵儿在雨里朝你伸出手,你们在雷声里笑得像两个孩子。",
                        "(情缘 +2,获得一件法宝)",
                    ],
                    effect: Effect::RandomRelic,
                    daoxin: 0,
                    qingyuan: 2,
                },
            },
            StoryOption {
                label: "借雷淬剑:「大义当前,不得不为。」",
                outcome: Outcome {
                    lines: &[
                        "你以剑引雷,剑身多了一道细密的雷纹。",
                        "古麟静静看着你,眼里没有恨,只有一声几不可闻的叹息。",
                        "(道心 +2,攻击 +3)",
                    ],
                    effect: Effect::GainAtk(3),
                    daoxin: 2,
                    qingyuan: 0,
                },
            },
        ],
    },
    StoryScene {
        lines: &[
            "心渊入口,水面浮出你一路走来的倒影:渡口、祠堂、雷泽。",
            "水影低语:「放下剑,便还你一个人人无恙的梦。」",
            "腕上的桃木铃轻轻晃着,像是在等你的答案。",
        ],
        prompt: "梦与剑,你选——",
        options: &[
            StoryOption {
                label: "摇响铃铛:「梦是假的,她在等我回家是真的。」",
                outcome: Outcome {
                    lines: &[
                        "铃声穿透水面,倒影尽碎。你听见很远的地方,有人也摇响了铃。",
                        "(情缘 +3,恢复四成气血)",
                    ],
                    effect: Effect::HealPct(40),
                    daoxin: 0,
                    qingyuan: 3,
                },
            },
            StoryOption {
                label: "举剑入渊:「我斩的不是妖,是不敢面对的自己。」",
                outcome: Outcome {
                    lines: &[
                        "水面平息,剑光澄澈如初雪。道心至此,再无杂音。",
                        "(道心 +3,攻击 +3)",
                    ],
                    effect: Effect::GainAtk(3),
                    daoxin: 3,
                    qingyuan: 0,
                },
            },
        ],
    },
];

// ---------------------------------------------------------------------------
// Random events (「遇」 nodes) — drawn from this pool with the run RNG
// ---------------------------------------------------------------------------

pub const EVENTS: [RandomEvent; 10] = [
    RandomEvent {
        title: "山间酒肆",
        lines: &[
            "半山腰竟有一间酒肆,老板娘笑吟吟地招呼:「客官,来碗醉仙酿?」",
            "酒香醇厚,可这荒山野岭的酒肆,总透着几分古怪。",
        ],
        options: &[
            EventOption {
                label: "豪饮三碗(付 30 文)",
                chance: 0.65,
                success: Outcome {
                    lines: &["酒入愁肠,百脉俱畅!老板娘还多送了你一坛。", "(恢复四成气血)"],
                    effect: Effect::HealPct(40),
                    daoxin: 0,
                    qingyuan: 0,
                },
                failure: Some(Outcome {
                    lines: &[
                        "酒里有蒙汗药!你挣扎着逃出,酒肆化作一缕青烟。",
                        "(损失一成五气血)",
                    ],
                    effect: Effect::DamagePct(15),
                    daoxin: 0,
                    qingyuan: 0,
                }),
            },
            EventOption {
                label: "只讨碗水,道谢离开",
                chance: 1.0,
                success: Outcome {
                    lines: &[
                        "老板娘眼中闪过一丝赞许:「小郎君心定,送你一句:前路莫贪。」",
                        "(道心 +1)",
                    ],
                    effect: Effect::None,
                    daoxin: 1,
                    qingyuan: 0,
                },
                failure: None,
            },
        ],
    },
    RandomEvent {
        title: "断剑冢",
        lines: &[
            "乱石堆里插满断剑,皆是历代斩妖人留下的。",
            "最深处一柄剑仍在轻鸣,似有剑意未散。",
        ],
        options: &[
            EventOption {
                label: "拔剑,承其剑意",
                chance: 0.6,
                success: Outcome {
                    lines: &["断剑化光入体,前辈剑意在你经脉中流转!", "(攻击 +3)"],
                    effect: Effect::GainAtk(3),
                    daoxin: 0,
                    qingyuan: 0,
                },
                failure: Some(Outcome {
                    lines: &["剑意反噬,你虎口迸裂,踉跄后退。", "(损失两成气血)"],
                    effect: Effect::DamagePct(20),
                    daoxin: 0,
                    qingyuan: 0,
                }),
            },
            EventOption {
                label: "俯身合十,为亡者立碑",
                chance: 1.0,
                success: Outcome {
                    lines: &[
                        "碑成之刻,风止剑息。一位守冢老人默默递来一瓶伤药。",
                        "(获得一枚药水,道心 +1)",
                    ],
                    effect: Effect::GainPotions(1),
                    daoxin: 1,
                    qingyuan: 0,
                },
                failure: None,
            },
        ],
    },
    RandomEvent {
        title: "落难货郎",
        lines: &[
            "货郎的担子翻在路边,妖兽的爪印还新鲜。",
            "他抱着最后一只木箱瑟瑟发抖:「侠士救命!货没了不打紧,箱里是给闺女的嫁妆……」",
        ],
        options: &[
            EventOption {
                label: "护送他到前面的官道",
                chance: 1.0,
                success: Outcome {
                    lines: &["货郎千恩万谢,硬塞给你一把碎银。", "(获得 60 文)"],
                    effect: Effect::GainGold(60),
                    daoxin: 0,
                    qingyuan: 1,
                },
                failure: None,
            },
            EventOption {
                label: "买下他担子里剩下的伤药",
                chance: 1.0,
                success: Outcome {
                    lines: &[
                        "「侠士厚道!」他把仅剩的伤药全给了你,只收半价。",
                        "(花 25 文,获得两枚药水)",
                    ],
                    effect: Effect::GainPotions(2),
                    daoxin: 0,
                    qingyuan: 0,
                },
                failure: None,
            },
        ],
    },
    RandomEvent {
        title: "古镜残光",
        lines: &[
            "废弃道观里,一面古镜蒙尘。拂去尘埃,镜中映出的却不是你,",
            "而是一位白发道人,朝你缓缓伸出手掌。",
        ],
        options: &[
            EventOption {
                label: "以掌相抵,接他一式",
                chance: 0.55,
                success: Outcome {
                    lines: &[
                        "掌风过处,你只觉筋骨齐鸣,竟被喂拳喂出了几分火候!",
                        "(气血上限 +14)",
                    ],
                    effect: Effect::GainMaxHp(14),
                    daoxin: 0,
                    qingyuan: 0,
                },
                failure: Some(Outcome {
                    lines: &[
                        "一掌之威远超所料,你倒飞出去,撞塌了半面泥墙。",
                        "(损失两成五气血)",
                    ],
                    effect: Effect::DamagePct(25),
                    daoxin: 0,
                    qingyuan: 0,
                }),
            },
            EventOption {
                label: "整衣,朝镜中人一拜",
                chance: 1.0,
                success: Outcome {
                    lines: &[
                        "道人抚须而笑,镜面荡开涟漪,一缕清气没入你眉心。",
                        "(术法上限 +8)",
                    ],
                    effect: Effect::GainMaxMp(8),
                    daoxin: 1,
                    qingyuan: 0,
                },
                failure: None,
            },
        ],
    },
    RandomEvent {
        title: "妖市黑摊",
        lines: &[
            "雾里亮着一盏绿灯笼:妖怪们摆的夜市。摊主是只戴斗笠的狸猫,",
            "爪边摆着一件蒙布的「宝贝」:「不问来路,五十文,拿走。」",
        ],
        options: &[
            EventOption {
                label: "掏钱赌一把(付 50 文)",
                chance: 0.5,
                success: Outcome {
                    lines: &[
                        "揭开布——竟真是件法宝!狸猫嘟囔:「亏本了亏本了……」",
                        "(获得一件法宝)",
                    ],
                    effect: Effect::RandomRelic,
                    daoxin: 0,
                    qingyuan: 0,
                },
                failure: Some(Outcome {
                    lines: &[
                        "布下只有一块画了符的砖头。狸猫早已卷摊跑路。",
                        "(白花五十文)",
                    ],
                    effect: Effect::LoseGold(50),
                    daoxin: 0,
                    qingyuan: 0,
                }),
            },
            EventOption {
                label: "拱手不买,借道而行",
                chance: 1.0,
                success: Outcome {
                    lines: &[
                        "狸猫斗笠一抬:「有眼力,不贪的人走夜路稳当。」它送你一小袋铜钱开路。",
                        "(获得 30 文)",
                    ],
                    effect: Effect::GainGold(30),
                    daoxin: 1,
                    qingyuan: 0,
                },
                failure: None,
            },
        ],
    },
    RandomEvent {
        title: "溺水的萤火",
        lines: &[
            "夜潭里一团萤光正在下沉——是只落水的萤灯小妖,",
            "翅膀湿透,眼看就要沉进潭底的黑暗里。",
        ],
        options: &[
            EventOption {
                label: "下水捞它",
                chance: 0.75,
                success: Outcome {
                    lines: &[
                        "小妖抖干翅膀,绕着你转了三圈,把一粒暖融融的光种埋进你掌心。",
                        "(气血上限 +10,情缘 +1)",
                    ],
                    effect: Effect::GainMaxHp(10),
                    daoxin: 0,
                    qingyuan: 1,
                },
                failure: Some(Outcome {
                    lines: &[
                        "潭底暗流凶猛,你呛了好几口水才把它托上岸。小妖愧疚地贴了贴你的脸。",
                        "(损失一成气血,情缘 +1)",
                    ],
                    effect: Effect::DamagePct(10),
                    daoxin: 0,
                    qingyuan: 1,
                }),
            },
            EventOption {
                label: "折根芦苇递过去",
                chance: 1.0,
                success: Outcome {
                    lines: &[
                        "小妖攀着芦苇爬上岸,朝你眨了眨眼,点亮了前方的路。",
                        "(恢复一成五气血)",
                    ],
                    effect: Effect::HealPct(15),
                    daoxin: 0,
                    qingyuan: 0,
                },
                failure: None,
            },
        ],
    },
    RandomEvent {
        title: "无名棋局",
        lines: &[
            "山亭石桌上摆着一盘残棋,对面坐着个看不清面目的白衣人。",
            "「一子定输赢。赢了,我教你一手;输了,留下点买路财。」",
        ],
        options: &[
            EventOption {
                label: "落子",
                chance: 0.5,
                success: Outcome {
                    lines: &[
                        "你一子破局!白衣人抚掌:「妙。」指尖点上你的剑脊。",
                        "(攻击 +2,防御 +1)",
                    ],
                    effect: Effect::GainAtk(2),
                    daoxin: 1,
                    qingyuan: 0,
                },
                failure: Some(Outcome {
                    lines: &[
                        "三手之后满盘皆输。白衣人笑着收走了你的钱袋子一角。",
                        "(损失 40 文)",
                    ],
                    effect: Effect::LoseGold(40),
                    daoxin: 0,
                    qingyuan: 0,
                }),
            },
            EventOption {
                label: "掀不动这浑水,绕亭而过",
                chance: 1.0,
                success: Outcome {
                    lines: &["身后传来一声轻笑:「知进退,也算一种棋力。」", "(防御 +1)"],
                    effect: Effect::GainDef(1),
                    daoxin: 0,
                    qingyuan: 0,
                },
                failure: None,
            },
        ],
    },
    RandomEvent {
        title: "破庙香火",
        lines: &[
            "破庙里的土地神像缺了半边肩膀,香炉却干干净净。",
            "案上留着字条:「添香一炷,护你一程。」",
        ],
        options: &[
            EventOption {
                label: "上香,再把神像扶正(付 20 文修缮)",
                chance: 1.0,
                success: Outcome {
                    lines: &[
                        "香烟笔直升起。夜里赶路,你总觉得脚下的石子都被人悄悄捡走了。",
                        "(防御 +2)",
                    ],
                    effect: Effect::GainDef(2),
                    daoxin: 1,
                    qingyuan: 0,
                },
                failure: None,
            },
            EventOption {
                label: "搜刮功德箱",
                chance: 0.4,
                success: Outcome {
                    lines: &[
                        "箱底果然有前人留下的香火钱。你揣进怀里,总觉得神像在看你。",
                        "(获得 70 文,道心 -1)",
                    ],
                    effect: Effect::GainGold(70),
                    daoxin: -1,
                    qingyuan: 0,
                },
                failure: Some(Outcome {
                    lines: &[
                        "功德箱「啪」地夹住你的手!挣脱时撞翻了半个供桌。",
                        "(损失一成五气血,道心 -1)",
                    ],
                    effect: Effect::DamagePct(15),
                    daoxin: -1,
                    qingyuan: 0,
                }),
            },
        ],
    },
    RandomEvent {
        title: "灵泉眼",
        lines: &[
            "岩缝间一汪灵泉,水汽凝而不散,隐隐有药香。",
            "泉边石刻:「一人一瓢,多取者噬。」",
        ],
        options: &[
            EventOption {
                label: "依言取一瓢饮",
                chance: 1.0,
                success: Outcome {
                    lines: &["清泉入腹,伤处发痒——那是愈合的征兆。", "(恢复三成五气血)"],
                    effect: Effect::HealPct(35),
                    daoxin: 0,
                    qingyuan: 0,
                },
                failure: None,
            },
            EventOption {
                label: "灌满所有水囊",
                chance: 0.45,
                success: Outcome {
                    lines: &[
                        "泉眼晃了晃,竟没有发作。你带走了满满两囊灵泉。",
                        "(获得两枚药水)",
                    ],
                    effect: Effect::GainPotions(2),
                    daoxin: -1,
                    qingyuan: 0,
                },
                failure: Some(Outcome {
                    lines: &[
                        "泉水在囊中化作黑雾反噬!你丢下水囊仓皇而退。",
                        "(损失两成气血,道心 -1)",
                    ],
                    effect: Effect::DamagePct(20),
                    daoxin: -1,
                    qingyuan: 0,
                }),
            },
        ],
    },
    RandomEvent {
        title: "夜行商队的传闻",
        lines: &[
            "篝火边,商队老镖头请你喝了碗热汤,压低声音:",
            "「前头的路,一条近但邪性,一条远但太平。信我一句,还是信你的剑?」",
        ],
        options: &[
            EventOption {
                label: "「信剑。」抄近路",
                chance: 0.55,
                success: Outcome {
                    lines: &["邪路无事,反倒在路边捡到前人失落的包袱。", "(获得 80 文)"],
                    effect: Effect::GainGold(80),
                    daoxin: 0,
                    qingyuan: 0,
                },
                failure: Some(Outcome {
                    lines: &["近路果然有伏!你且战且退,肩头挨了一爪。", "(损失两成气血)"],
                    effect: Effect::DamagePct(20),
                    daoxin: 0,
                    qingyuan: 0,
                }),
            },
            EventOption {
                label: "「信你。」随商队绕行",
                chance: 1.0,
                success: Outcome {
                    lines: &[
                        "一夜太平。老镖头临别塞给你一枚护心木牌:「江湖再见。」",
                        "(防御 +1,恢复一成五气血)",
                    ],
                    effect: Effect::HealPct(15),
                    daoxin: 0,
                    qingyuan: 1,
                },
                failure: None,
            },
        ],
    },
];

// ---------------------------------------------------------------------------
// Rest node (「歇」) options
// ---------------------------------------------------------------------------

pub const REST_INTRO: &[&str] = &[
    "背风的岩窝里升起一小堆篝火,灵儿把干粮分成两半。",
    "难得的喘息。歇脚之后,前路还长。",
];

// Rest is resolved in code (heal amount depends on 醉仙酿), so only labels here.
pub const REST_MEDITATE: &str = "打坐调息(回复五成气血)";
pub const REST_SPAR: &str = "温酒论剑(攻击 +2,回复一成气血)";

// ---------------------------------------------------------------------------
// Endings
// ---------------------------------------------------------------------------

pub const ENDING_LOVE: &[&str] = &[
    "【结局 · 铃归渡口】",
    "宿命水影溃散时,你没有去看它沉没,而是转身向山下跑。",
    "渡口的灯还亮着。灵儿站在灯下,腕间的铃和你怀里的那枚同时响了——",
    "第三声铃响,不是永别。",
    "「回来啦?」「回来了。这次,带你看真正的月亮。」",
    "后来桃溪村的孩子都说,雾散那晚,溪上有两个影子并肩踏月而行。",
];

pub const ENDING_RESOLVE: &[&str] = &[
    "【结局 · 剑照心渊】",
    "水影沉没,心渊澄明。你把木剑插在渊口,像立了一块无字碑。",
    "千百执念自剑身流过,再无一丝能撼动你的剑心。",
    "下山时,你把桃木铃留在了祠堂——有些牵挂,供起来比带走更妥帖。",
    "此后江湖传闻:有位无名剑客,斩妖不留名,过村不饮酒,",
    "只在每年雾起之夜,回桃溪村替人守一晚渡口。",
];

pub const ENDING_DEFEAT: &[&str] = &[
    "【轮回 · 未竟之誓】",
    "剑光熄灭的那一刻,你听见很远的地方有铃响了一声。",
    "黑暗并不冰冷,反而像溪水一样把你托起、送回——",
    "再睁眼,又是桃溪村的清晨,祠堂前的誓言墨迹未干。",
    "妖雾仍在,誓言仍在。这一世,换条路,再走一遍。",
];
