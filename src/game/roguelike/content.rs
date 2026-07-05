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
    LosePotions(u32),
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

// Additional story scenes per chapter — the 「缘」 node rolls one scene from
// { STORY_SCENES[chapter] } ∪ STORY_EXTRA[chapter], so repeat runs see
// different beats.

const SCENE_EAVES: StoryScene = StoryScene {
    lines: &[
        "夜雨骤至,你与灵儿躲进一处旧屋檐,火折子的光只照亮彼此的脸。",
        "灵儿数着檐角滴下的雨:「我娘说,雨夜里说的愿望,山神听得最清。」",
        "她偏过头看你:「你许一个,我便也许一个。」",
    ],
    prompt: "你许的愿是——",
    options: &[
        StoryOption {
            label: "「愿身边这个人,岁岁平安。」",
            outcome: Outcome {
                lines: &[
                    "灵儿的耳尖红了,她小声说了自己的愿,却死活不肯讲给你听。",
                    "只是那晚之后,你的伤总好得格外快。",
                    "(情缘 +2,恢复两成五气血)",
                ],
                effect: Effect::HealPct(25),
                daoxin: 0,
                qingyuan: 2,
            },
        },
        StoryOption {
            label: "「愿此剑所指,再无枉死之人。」",
            outcome: Outcome {
                lines: &[
                    "檐外雷声轻轻应了一声。灵儿望着你的侧脸,忽然觉得陌生,又忽然觉得安心。",
                    "(道心 +2,防御 +2)",
                ],
                effect: Effect::GainDef(2),
                daoxin: 2,
                qingyuan: 0,
            },
        },
    ],
};

const SCENE_OPERA: StoryScene = StoryScene {
    lines: &[
        "途经的小村正唱社戏,台上演的是斩妖人斩尽心魔、白日飞升的老戏文。",
        "台下孩童学着挥剑,笑闹声撞进锣鼓里。灵儿买了两串糖葫芦,递给你一串。",
        "戏至高潮,斩妖人却在飞升前回了头——台下有人喝倒彩,有人抹眼泪。",
    ],
    prompt: "你觉得他该不该回头?",
    options: &[
        StoryOption {
            label: "「该。成仙有什么好,人间烟火才是真的。」",
            outcome: Outcome {
                lines: &[
                    "灵儿咬着糖葫芦笑弯了眼:「那你到时候,也记得回头。」",
                    "(情缘 +2,获得一枚药水)",
                ],
                effect: Effect::GainPotions(1),
                daoxin: 0,
                qingyuan: 2,
            },
        },
        StoryOption {
            label: "「不该。既起了誓,回头便是负了所有人。」",
            outcome: Outcome {
                lines: &[
                    "邻座的老者闻言看了你一眼,朝你举了举酒碗:「后生,是个走得远的。」",
                    "(道心 +2,攻击 +2)",
                ],
                effect: Effect::GainAtk(2),
                daoxin: 2,
                qingyuan: 0,
            },
        },
    ],
};

const SCENE_LANTERNS: StoryScene = StoryScene {
    lines: &[
        "疫气稍歇的夜里,江城幸存的人们在渡口放河灯,为逝者引路。",
        "一个失了双亲的小女孩折不好灯纸,急得直掉眼泪。",
        "灵儿蹲下去帮她,回头看你:「再折一盏么?给……将来的我们。」",
    ],
    prompt: "河灯放与不放?",
    options: &[
        StoryOption {
            label: "折一盏双生灯,与她并肩放进江里。",
            outcome: Outcome {
                lines: &[
                    "两盏灯在江心靠在一起,顺流而下,久久不散。",
                    "小女孩破涕为笑:「灯挨着灯,人就不会走散。」",
                    "(情缘 +2,恢复两成气血)",
                ],
                effect: Effect::HealPct(20),
                daoxin: 0,
                qingyuan: 2,
            },
        },
        StoryOption {
            label: "只放一盏无名灯,祭这一城亡魂。",
            outcome: Outcome {
                lines: &[
                    "灯入江心,你在心里把三千户的名字都念了一遍——念不出名字的,就记住灯影。",
                    "(道心 +2,气血上限 +10)",
                ],
                effect: Effect::GainMaxHp(10),
                daoxin: 2,
                qingyuan: 0,
            },
        },
    ],
};

const SCENE_QIN: StoryScene = StoryScene {
    lines: &[
        "废驿站里有位盲眼琴师,守着一张断了两根弦的旧琴,不肯随难民南逃。",
        "「琴是内子的陪嫁。她走在疫里,我走了,谁给她弹《归雁》?」",
        "琴声残缺,却稳得像多年未改的心跳。",
    ],
    prompt: "你——",
    options: &[
        StoryOption {
            label: "以剑代弦,为他补全一曲《归雁》。",
            outcome: Outcome {
                lines: &[
                    "剑鸣清越,续上断弦的空处。曲终,琴师朝虚空一揖:「夫人,有客到。」",
                    "灵儿在门边听完了整曲,眼睛亮亮的没说话。",
                    "(情缘 +2,术法上限 +8)",
                ],
                effect: Effect::GainMaxMp(8),
                daoxin: 0,
                qingyuan: 2,
            },
        },
        StoryOption {
            label: "留下盘缠,劝他随人流南去。",
            outcome: Outcome {
                lines: &[
                    "琴师收了钱,却只买了三炷香。你走出很远,还听见那支残曲跟了一路。",
                    "有些执念劝不动,就像你自己的。",
                    "(道心 +2,损失 30 文)",
                ],
                effect: Effect::LoseGold(30),
                daoxin: 2,
                qingyuan: 0,
            },
        },
    ],
};

const SCENE_PALACE: StoryScene = StoryScene {
    lines: &[
        "京城夜宴,国师隔着满席歌舞举杯敬你:「斩妖人,你剑上妖血未干。」",
        "「我锁妖养煞,是为社稷;你逢妖便斩,又为的什么?说来听听。」",
        "满殿烛火摇了摇。灵儿的手在案下轻轻按住你的手背。",
    ],
    prompt: "你如何作答?",
    options: &[
        StoryOption {
            label: "反手握住灵儿:「为身后之人不必学会握剑。」",
            outcome: Outcome {
                lines: &[
                    "国师盯着你们交握的手看了很久,忽然笑了,笑意却不达眼底。",
                    "「好答案。可惜本座……早已没有身后之人。」",
                    "(情缘 +2,防御 +2)",
                ],
                effect: Effect::GainDef(2),
                daoxin: 0,
                qingyuan: 2,
            },
        },
        StoryOption {
            label: "举杯饮尽:「剑问的是心,不是妖。国师心里有妖。」",
            outcome: Outcome {
                lines: &[
                    "殿内死寂。国师缓缓鼓掌,烛火齐齐矮了三分。",
                    "「今夜之后,京城没有你的容身处了。——但本座敬你这句话。」",
                    "(道心 +2,攻击 +3)",
                ],
                effect: Effect::GainAtk(3),
                daoxin: 2,
                qingyuan: 0,
            },
        },
    ],
};

const SCENE_BONFIRE: StoryScene = StoryScene {
    lines: &[
        "南疆部族的篝火祭上,祭司请远客共舞一曲,说舞给雷神看的人不会说谎。",
        "鼓点如雨。灵儿被姑娘们拉进火光里,笑着朝你伸出手。",
        "祭司低声道:「跳,便把心跳给她看;不跳,便把心留给山看。」",
    ],
    prompt: "鼓点催了三巡,你——",
    options: &[
        StoryOption {
            label: "踏进火光,握住那只手。",
            outcome: Outcome {
                lines: &[
                    "你舞得笨拙,她笑得直不起腰,火星升上夜空像一场倒下的星雨。",
                    "祭司朝雷云举杯:「雷神看见了。」",
                    "(情缘 +3,恢复三成气血)",
                ],
                effect: Effect::HealPct(30),
                daoxin: 0,
                qingyuan: 3,
            },
        },
        StoryOption {
            label: "静坐火外,以剑击节为她伴奏。",
            outcome: Outcome {
                lines: &[
                    "剑鸣与鼓点相和,火光里的身影旋得更轻快了。",
                    "祭司看看你,又看看剑:「也好。山也看见了。」",
                    "(道心 +2,攻击 +2)",
                ],
                effect: Effect::GainAtk(2),
                daoxin: 2,
                qingyuan: 0,
            },
        },
    ],
};

const SCENE_CORRIDOR: StoryScene = StoryScene {
    lines: &[
        "心渊深处有一条回廊,两侧水幕重演着你们的初遇:溪边、木剑、桃木铃。",
        "水幕里的灵儿朝水幕外的你招手,声音隔着一层旧梦:「这次,别走那么快。」",
        "回廊尽头有两扇门:一扇写着「留」,一扇写着「行」。",
    ],
    prompt: "旧梦当前,你推开——",
    options: &[
        StoryOption {
            label: "「行」。梦里多留一刻,现实便多险一分。",
            outcome: Outcome {
                lines: &[
                    "你穿梦而过,不曾回头。身后的水幕轻轻叹了口气,散作满廊星光。",
                    "(道心 +3,攻击 +2)",
                ],
                effect: Effect::GainAtk(2),
                daoxin: 3,
                qingyuan: 0,
            },
        },
        StoryOption {
            label: "「留」。在梦里陪她把那条溪走完。",
            outcome: Outcome {
                lines: &[
                    "梦里的溪很长,长到你想起了每一件差点忘记的小事。",
                    "走出门时,掌心多了一道温热的铃印。",
                    "(情缘 +3,恢复三成五气血)",
                ],
                effect: Effect::HealPct(35),
                daoxin: 0,
                qingyuan: 3,
            },
        },
    ],
};

/// Extra 「缘」 scenes rolled alongside the primary `STORY_SCENES` entry.
pub const STORY_EXTRA: [&[StoryScene]; 4] = [
    &[SCENE_EAVES, SCENE_OPERA],
    &[SCENE_LANTERNS, SCENE_QIN],
    &[SCENE_PALACE, SCENE_BONFIRE],
    &[SCENE_CORRIDOR],
];

/// Number of story scenes available to a chapter's 「缘」 node.
pub fn story_count(chapter: usize) -> usize {
    1 + STORY_EXTRA[chapter.min(3)].len()
}

/// Roll index 0 → the primary scene, 1.. → extras.
pub fn pick_story(chapter: usize, roll: usize) -> &'static StoryScene {
    let chapter = chapter.min(3);
    if roll == 0 {
        &STORY_SCENES[chapter]
    } else {
        &STORY_EXTRA[chapter][(roll - 1).min(STORY_EXTRA[chapter].len() - 1)]
    }
}

// ---------------------------------------------------------------------------
// Random events (「遇」 nodes) — drawn from this pool with the run RNG
// ---------------------------------------------------------------------------

pub const EVENTS: [RandomEvent; 29] = [
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
    RandomEvent {
        title: "走蛟渡河",
        lines: &[
            "渡口无船,只有一头老蛟浮在河心,背上驮着半塌的凉亭。",
            "「渡河十文,」老蛟打了个哈欠,「骑背上抄近路,免钱——抓稳便是。」",
        ],
        options: &[
            EventOption {
                label: "付十文,坐亭中稳稳过河",
                chance: 1.0,
                success: Outcome {
                    lines: &[
                        "河风拂面,老蛟讲了一路水底的旧闻,末了送你一片能安神的老鳞。",
                        "(恢复两成气血)",
                    ],
                    effect: Effect::HealPct(20),
                    daoxin: 0,
                    qingyuan: 0,
                },
                failure: None,
            },
            EventOption {
                label: "免钱!骑蛟背冲过去",
                chance: 0.55,
                success: Outcome {
                    lines: &[
                        "老蛟兴起,浪里三沉三浮,把你甩上对岸时喝彩:「好胆色!赏!」",
                        "(获得 55 文)",
                    ],
                    effect: Effect::GainGold(55),
                    daoxin: 0,
                    qingyuan: 0,
                },
                failure: Some(Outcome {
                    lines: &[
                        "半途一个浪头把你拍进河里,呛得七荤八素,老蛟笑得亭子直晃。",
                        "(损失一成五气血)",
                    ],
                    effect: Effect::DamagePct(15),
                    daoxin: 0,
                    qingyuan: 0,
                }),
            },
        ],
    },
    RandomEvent {
        title: "山神庙求签",
        lines: &[
            "小小山神庙,签筒擦得锃亮。庙祝是个打瞌睡的老猴儿,尾巴卷着蒲扇。",
            "「求签十文,灵不灵看缘分。」它眼也不睁地说。",
        ],
        options: &[
            EventOption {
                label: "诚心求一签(付 10 文)",
                chance: 0.6,
                success: Outcome {
                    lines: &[
                        "上上签:「剑下留情,身后有灯。」老猴儿难得睁眼:「好签,沾点仙气去。」",
                        "(术法上限 +6,恢复一成五气血)",
                    ],
                    effect: Effect::GainMaxMp(6),
                    daoxin: 1,
                    qingyuan: 0,
                },
                failure: Some(Outcome {
                    lines: &[
                        "下下签:「前路见血。」老猴儿把十文钱退给你一半:「签凶,半价。」",
                        "(退回 5 文,无事发生……大概)",
                    ],
                    effect: Effect::GainGold(5),
                    daoxin: 0,
                    qingyuan: 0,
                }),
            },
            EventOption {
                label: "不求签,给山神像掸掸灰",
                chance: 1.0,
                success: Outcome {
                    lines: &[
                        "老猴儿的蒲扇停了停:「比求签的强。」它从供桌下摸出一枚野果丹递给你。",
                        "(获得一枚药水)",
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
        title: "「仙丹」贩子",
        lines: &[
            "路边小贩神神秘秘掀开布:「九转还魂丹!吃一粒多活十年!三十文!」",
            "丹药红得发亮,香气扑鼻——香得有点像糖炒山楂。",
        ],
        options: &[
            EventOption {
                label: "将信将疑买一粒(付 30 文)",
                chance: 0.35,
                success: Outcome {
                    lines: &[
                        "竟真有药力!暖流走遍四肢百骸——虽然多半没有十年,三五天是有的。",
                        "(气血上限 +12)",
                    ],
                    effect: Effect::GainMaxHp(12),
                    daoxin: 0,
                    qingyuan: 0,
                },
                failure: Some(Outcome {
                    lines: &["……就是糖炒山楂。还挺好吃。", "(白花三十文,心情复杂)"],
                    effect: Effect::LoseGold(30),
                    daoxin: 0,
                    qingyuan: 0,
                }),
            },
            EventOption {
                label: "拆穿他:「山楂就说山楂。」",
                chance: 1.0,
                success: Outcome {
                    lines: &[
                        "小贩讪讪一笑,改口吆喝「祖传山楂丸」,生意反而好了,硬塞你两串谢礼。",
                        "(恢复一成气血,道心 +1)",
                    ],
                    effect: Effect::HealPct(10),
                    daoxin: 1,
                    qingyuan: 0,
                },
                failure: None,
            },
        ],
    },
    RandomEvent {
        title: "迷路的书生",
        lines: &[
            "书生抱着一摞书在岔路口打转,已经把同一棵歪脖子树路过了四回。",
            "「在下进京赶考……敢问侠士,哪条路向北?」他的干粮早吃完了。",
        ],
        options: &[
            EventOption {
                label: "分他干粮,送他一程",
                chance: 1.0,
                success: Outcome {
                    lines: &[
                        "书生千恩万谢,临别写了一幅字相赠:「侠之大者」。字真不错。",
                        "(道心 +1,情缘 +1)",
                    ],
                    effect: Effect::None,
                    daoxin: 1,
                    qingyuan: 1,
                },
                failure: None,
            },
            EventOption {
                label: "指个方向,顺便请教兵法韬略",
                chance: 0.7,
                success: Outcome {
                    lines: &[
                        "书生眼睛一亮,就着树影讲了半个时辰的攻守之道,竟颇有见地。",
                        "(防御 +2)",
                    ],
                    effect: Effect::GainDef(2),
                    daoxin: 0,
                    qingyuan: 0,
                },
                failure: Some(Outcome {
                    lines: &[
                        "书生讲得兴起,从孙子讲到棋谱,天黑了才放你走。",
                        "(耽搁半日,无事发生)",
                    ],
                    effect: Effect::None,
                    daoxin: 0,
                    qingyuan: 0,
                }),
            },
        ],
    },
    RandomEvent {
        title: "雨夜狐宅",
        lines: &[
            "暴雨夜,荒山里竟有座亮着灯的宅子。开门的妇人狐眼微挑:「借宿?」",
            "「饭菜管够,只一条规矩——天亮前,莫回头看厨房。」",
        ],
        options: &[
            EventOption {
                label: "守规矩,吃饱睡好",
                chance: 1.0,
                success: Outcome {
                    lines: &[
                        "一夜无梦。晨起宅子已化雾散去,你躺在干爽的草坡上,怀里多了个食盒。",
                        "(恢复三成气血,获得一枚药水)",
                    ],
                    effect: Effect::HealPct(30),
                    daoxin: 0,
                    qingyuan: 0,
                },
                failure: None,
            },
            EventOption {
                label: "半夜偏要回头看一眼",
                chance: 0.5,
                success: Outcome {
                    lines: &[
                        "厨房里,九条雪白的尾巴正在颠勺。狐妇人叹气:「罢了,看都看了,搭把手。」",
                        "你帮忙烧了半夜火,学了一手药膳。(气血上限 +10)",
                    ],
                    effect: Effect::GainMaxHp(10),
                    daoxin: 0,
                    qingyuan: 1,
                },
                failure: Some(Outcome {
                    lines: &[
                        "「说了莫回头。」宅子连人带床把你请了出去,暴雨浇了个透心凉。",
                        "(损失一成五气血)",
                    ],
                    effect: Effect::DamagePct(15),
                    daoxin: 0,
                    qingyuan: 0,
                }),
            },
        ],
    },
    RandomEvent {
        title: "溪心磨剑石",
        lines: &[
            "溪心卧着一块青黑巨石,水流过处铮铮作响,竟是块天生的磨剑石。",
            "石上刻着前人留言:「磨剑者,留下三滴血;取石者,留下一身伤。」",
        ],
        options: &[
            EventOption {
                label: "依言刺指,以血磨剑",
                chance: 1.0,
                success: Outcome {
                    lines: &["三滴血入水,剑锋映出的溪光都锐利了几分。", "(攻击 +2)"],
                    effect: Effect::GainAtk(2),
                    daoxin: 0,
                    qingyuan: 0,
                },
                failure: None,
            },
            EventOption {
                label: "凿一块石芯带走",
                chance: 0.45,
                success: Outcome {
                    lines: &[
                        "石芯温润如玉,贴身藏好,行路时总觉得筋骨沉稳。",
                        "(气血上限 +14)",
                    ],
                    effect: Effect::GainMaxHp(14),
                    daoxin: -1,
                    qingyuan: 0,
                },
                failure: Some(Outcome {
                    lines: &[
                        "凿到一半,溪水陡涨,把你冲出十几丈远,石屑崩了满脸。",
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
        title: "岩蜂酿蜜",
        lines: &[
            "峭壁上垂着一巢金红色的岩蜜,蜂群嗡鸣如雷,蜜香隔着十步都闻得到。",
            "崖下有蜂螫落的野兽白骨——但也有前人搭到一半的采蜜木架。",
        ],
        options: &[
            EventOption {
                label: "借木架攀上去取蜜",
                chance: 0.6,
                success: Outcome {
                    lines: &["得手!岩蜜入口,伤处的钝痛竟一点点化开了。", "(恢复四成气血)"],
                    effect: Effect::HealPct(40),
                    daoxin: 0,
                    qingyuan: 0,
                },
                failure: Some(Outcome {
                    lines: &[
                        "蜂群炸了窝!你抱头滚下木架,肿着半边脸落荒而逃。",
                        "(损失两成气血)",
                    ],
                    effect: Effect::DamagePct(20),
                    daoxin: 0,
                    qingyuan: 0,
                }),
            },
            EventOption {
                label: "在崖下拾些落蜜便走",
                chance: 1.0,
                success: Outcome {
                    lines: &[
                        "石缝里的落蜜混着草屑,味道差些,胜在安稳。",
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
        title: "峭壁灵芝",
        lines: &[
            "断崖石缝里生着一株千年灵芝,紫盖金边,在风里轻轻发亮。",
            "崖高百丈,唯一的落脚处是几蓬看起来不太牢靠的枯藤。",
        ],
        options: &[
            EventOption {
                label: "攀藤采芝",
                chance: 0.5,
                success: Outcome {
                    lines: &[
                        "枯藤晃而未断!灵芝到手,药香沁脾,只嗅一嗅便觉神清气足。",
                        "(气血上限 +16)",
                    ],
                    effect: Effect::GainMaxHp(16),
                    daoxin: 0,
                    qingyuan: 0,
                },
                failure: Some(Outcome {
                    lines: &[
                        "藤断了。多亏半山的松枝接了你一把,灵芝没到手,摔得不轻。",
                        "(损失两成五气血)",
                    ],
                    effect: Effect::DamagePct(25),
                    daoxin: 0,
                    qingyuan: 0,
                }),
            },
            EventOption {
                label: "以剑气隔空削下芝盖",
                chance: 0.4,
                success: Outcome {
                    lines: &[
                        "一道剑气恰到好处,芝盖打着旋儿落进你掌心——这一手连你自己都想喝彩。",
                        "(攻击 +2,恢复两成气血)",
                    ],
                    effect: Effect::HealPct(20),
                    daoxin: 1,
                    qingyuan: 0,
                },
                failure: Some(Outcome {
                    lines: &[
                        "剑气偏了半寸,灵芝连石屑一起崩进深谷,悔得你直跺脚。",
                        "(无事发生,但很心疼)",
                    ],
                    effect: Effect::None,
                    daoxin: 0,
                    qingyuan: 0,
                }),
            },
        ],
    },
    RandomEvent {
        title: "古战场遗骸",
        lines: &[
            "荒草掩着一片古战场,断戟锈甲间,尚有几具无人收敛的骸骨。",
            "一柄断刀插在土里,刀柄上系着褪色的红绳——像是谁的定情之物。",
        ],
        options: &[
            EventOption {
                label: "收敛骸骨,立冢焚香",
                chance: 1.0,
                success: Outcome {
                    lines: &[
                        "三座小冢立起时,荒草间的阴风忽然停了。",
                        "夜里你梦见一队卸甲的兵,朝你抱拳而去。(道心 +2)",
                    ],
                    effect: Effect::GainDef(1),
                    daoxin: 2,
                    qingyuan: 0,
                },
                failure: None,
            },
            EventOption {
                label: "翻检遗物,寻些还能用的",
                chance: 0.65,
                success: Outcome {
                    lines: &[
                        "锈甲下压着一只未曾腐烂的军囊,里头是半袋军饷。",
                        "你把红绳断刀留在了原地。(获得 65 文,道心 -1)",
                    ],
                    effect: Effect::GainGold(65),
                    daoxin: -1,
                    qingyuan: 0,
                },
                failure: Some(Outcome {
                    lines: &[
                        "指尖刚触到军囊,锈甲哗啦一声塌了,惊起满地磷火燎了你一下。",
                        "(损失一成气血,道心 -1)",
                    ],
                    effect: Effect::DamagePct(10),
                    daoxin: -1,
                    qingyuan: 0,
                }),
            },
        ],
    },
    RandomEvent {
        title: "琴音辨路",
        lines: &[
            "雾锁山径,岔路口隐约飘来两缕琴音:左边的曲子欢快,右边的曲子哀婉。",
            "樵夫说过,山里的琴音会引路,也会引人——就看你耳朵信哪边。",
        ],
        options: &[
            EventOption {
                label: "循欢快的琴音走左路",
                chance: 0.6,
                success: Outcome {
                    lines: &[
                        "琴音尽头是一场山民的婚宴,你被拉着喝了三碗喜酒,满袖都是花瓣。",
                        "(恢复两成五气血,情缘 +1)",
                    ],
                    effect: Effect::HealPct(25),
                    daoxin: 0,
                    qingyuan: 1,
                },
                failure: Some(Outcome {
                    lines: &[
                        "琴音是山魈学的!它笑着敲了你一闷棍,抢走了几个铜板。",
                        "(损失 25 文)",
                    ],
                    effect: Effect::LoseGold(25),
                    daoxin: 0,
                    qingyuan: 0,
                }),
            },
            EventOption {
                label: "循哀婉的琴音走右路",
                chance: 0.6,
                success: Outcome {
                    lines: &[
                        "琴音来自一位守墓的老琴师,他见有人肯听完整曲,赠了你一瓶祭余的药酒。",
                        "(获得一枚药水,道心 +1)",
                    ],
                    effect: Effect::GainPotions(1),
                    daoxin: 1,
                    qingyuan: 0,
                },
                failure: Some(Outcome {
                    lines: &[
                        "曲子越听越冷,回过神来已站在断崖边缘,惊出一身冷汗,急急退回岔口。",
                        "(损失一成气血)",
                    ],
                    effect: Effect::DamagePct(10),
                    daoxin: 0,
                    qingyuan: 0,
                }),
            },
        ],
    },
    RandomEvent {
        title: "悬赏皇榜",
        lines: &[
            "驿亭墙上贴着一张皇榜:重金悬赏「妖首」,不问妖之善恶,以头计赏。",
            "榜下围着几个磨刀霍霍的赏金客,正高声谈论昨日斩的「妖」——听着倒像个山民。",
        ],
        options: &[
            EventOption {
                label: "撕了这张榜",
                chance: 0.7,
                success: Outcome {
                    lines: &[
                        "赏金客们一哄而上,又在你按剑的目光里一哄而散。",
                        "亭角躲雨的狸妖朝你深深一揖,留下一小袋谢礼。(获得 40 文,道心 +1)",
                    ],
                    effect: Effect::GainGold(40),
                    daoxin: 1,
                    qingyuan: 0,
                },
                failure: Some(Outcome {
                    lines: &[
                        "赏金客里有硬茬,一场混战虽赶跑了他们,你也挂了彩。",
                        "(损失一成五气血,道心 +1)",
                    ],
                    effect: Effect::DamagePct(15),
                    daoxin: 1,
                    qingyuan: 0,
                }),
            },
            EventOption {
                label: "抄下榜文线索,自去查证真伪",
                chance: 1.0,
                success: Outcome {
                    lines: &[
                        "按图索骥,所谓「妖巢」不过是个山洞药圃。你在榜文背面写下「查无此妖」四字寄回官府。",
                        "(防御 +1,道心 +1)",
                    ],
                    effect: Effect::GainDef(1),
                    daoxin: 1,
                    qingyuan: 0,
                },
                failure: None,
            },
        ],
    },
    RandomEvent {
        title: "虎口樵夫",
        lines: &[
            "林中传来呼救——一只吊睛白虎把樵夫逼上了树,树干已经开始咔咔作响。",
            "怪的是,白虎并不扑咬,只围着树打转,喉咙里呜呜作声,似有冤屈。",
        ],
        options: &[
            EventOption {
                label: "拔剑逼退白虎",
                chance: 0.75,
                success: Outcome {
                    lines: &[
                        "白虎与你对峙片刻,竟衔起树下一只被夹伤的虎崽,一瘸一拐地走了。",
                        "樵夫下树后面红耳赤:「那夹子……是俺下的。」(道心 +1,获得 30 文谢礼)",
                    ],
                    effect: Effect::GainGold(30),
                    daoxin: 1,
                    qingyuan: 0,
                },
                failure: Some(Outcome {
                    lines: &[
                        "白虎虚晃一爪,劲风扫过肩头,火辣辣地疼——但它终究只是护崽,并未下死手。",
                        "(损失一成气血)",
                    ],
                    effect: Effect::DamagePct(10),
                    daoxin: 0,
                    qingyuan: 0,
                }),
            },
            EventOption {
                label: "先看清再动手——顺着虎的目光找",
                chance: 1.0,
                success: Outcome {
                    lines: &[
                        "你在草丛里找到被兽夹夹住的虎崽,撬开铁夹。白虎伏低了头,樵夫在树上大气不敢出。",
                        "母虎衔崽离去前,回头长啸一声,山鸣谷应。(道心 +1,攻击 +1)",
                    ],
                    effect: Effect::GainAtk(1),
                    daoxin: 1,
                    qingyuan: 1,
                },
                failure: None,
            },
        ],
    },
    // --- 链式奇遇(不入随机池,由 RunState::draw_chain_or_event 按进度触发) ---
    // 22 EV_FOX1 · 白狐初遇(第一卷)
    RandomEvent {
        title: "雪尾白狐",
        lines: &[
            "山径旁传来细弱的呜咽——一只雪尾白狐被兽夹咬住了后腿,",
            "血染白毛。它不挣扎,只用琥珀色的眼睛一眨不眨地望着你。",
        ],
        options: &[
            EventOption {
                label: "撬开兽夹,撕衣为它裹伤(用去一瓶药水)",
                chance: 1.0,
                success: Outcome {
                    lines: &[
                        "白狐舔了舔你的手背,一瘸一拐地隐入林中,回头望了你三次。",
                        "(药水 -1,情缘 +1……你总觉得,还会再见到它)",
                    ],
                    effect: Effect::LosePotions(1),
                    daoxin: 0,
                    qingyuan: 1,
                },
                failure: None,
            },
            EventOption {
                label: "山中妖物,多一事不如少一事",
                chance: 1.0,
                success: Outcome {
                    lines: &[
                        "你按剑绕开。身后呜咽渐弱,琥珀色的目光落在你背上,很凉。",
                        "(道心 +1……某个雨夜,或许你会想起这一眼)",
                    ],
                    effect: Effect::None,
                    daoxin: 1,
                    qingyuan: 0,
                },
                failure: None,
            },
        ],
    },
    // 23 EV_FOX2_WARM · 白衣回礼(第二卷,救过)
    RandomEvent {
        title: "白衣回礼",
        lines: &[
            "疫雨夜,一位白衣少女撑伞而来,伞沿一抬,眉眼弯弯:",
            "「恩公,山中兽夹之恩,阿狸记得。」她指尖递来一枚暖玉。",
        ],
        options: &[
            EventOption {
                label: "收下暖玉",
                chance: 1.0,
                success: Outcome {
                    lines: &[
                        "暖玉贴身,寒雨不侵,伤处竟自愈合。(恢复四成气血,得 30 文)",
                        "白衣少女笑着退进雨里:「第三面,再还你一桩大的。」",
                    ],
                    effect: Effect::HealPct(40),
                    daoxin: 0,
                    qingyuan: 1,
                },
                failure: None,
            },
            EventOption {
                label: "「举手之劳,不必挂怀。」",
                chance: 1.0,
                success: Outcome {
                    lines: &[
                        "少女怔了怔,轻声道:「人间竟真有这样的人。」",
                        "(情缘 +2)她把伞留给了你,自己消失在雨里。",
                    ],
                    effect: Effect::None,
                    daoxin: 0,
                    qingyuan: 2,
                },
                failure: None,
            },
        ],
    },
    // 24 EV_FOX2_COLD · 白影避走(第二卷,未救)
    RandomEvent {
        title: "白影避走",
        lines: &[
            "巷口白影一闪——那只瘸腿的白狐。它认出了你,",
            "耳朵倏地压平,警惕地退进雨幕,像在说:人心,凉薄。",
        ],
        options: &[
            EventOption {
                label: "追上去,放下一瓶药水",
                chance: 0.5,
                success: Outcome {
                    lines: &[
                        "白狐迟疑良久,终于叼走了药水。琥珀色的眼睛柔和了一瞬。",
                        "(药水 -1,情缘 +1……或许还来得及)",
                    ],
                    effect: Effect::LosePotions(1),
                    daoxin: 0,
                    qingyuan: 1,
                },
                failure: Some(Outcome {
                    lines: &["白影头也不回地消失了。雨更大了。(无事发生)"],
                    effect: Effect::None,
                    daoxin: 0,
                    qingyuan: 0,
                }),
            },
            EventOption {
                label: "由它去",
                chance: 1.0,
                success: Outcome {
                    lines: &["你收回目光。修行路上,本就不该多牵挂。(道心 +1)"],
                    effect: Effect::None,
                    daoxin: 1,
                    qingyuan: 0,
                },
                failure: None,
            },
        ],
    },
    // 25 EV_FOX3_WARM · 狐仙赠丹(第三卷,善缘)
    RandomEvent {
        title: "狐仙赠丹",
        lines: &[
            "月华如练。白衣少女立于枝头,身后九条雪尾缓缓展开——",
            "「阿狸修行三百年,欠恩不欠情。这枚内丹辉光,赠予恩公。」",
        ],
        options: &[
            EventOption {
                label: "郑重接过",
                chance: 1.0,
                success: Outcome {
                    lines: &[
                        "辉光入体,如月入怀。(获得一件法宝)",
                        "「前路凶险,恩公珍重。」九尾一卷,月下再无狐影。",
                    ],
                    effect: Effect::RandomRelic,
                    daoxin: 0,
                    qingyuan: 2,
                },
                failure: None,
            },
            EventOption {
                label: "「山高水长,后会有期。」",
                chance: 1.0,
                success: Outcome {
                    lines: &[
                        "狐仙深深看了你一眼,化月光渡入你眉心。(气血上限 +12,情缘 +2)",
                        "「那便……以此相护,直到雾散。」",
                    ],
                    effect: Effect::GainMaxHp(12),
                    daoxin: 0,
                    qingyuan: 2,
                },
                failure: None,
            },
        ],
    },
    // 26 EV_FOX3_COLD · 妖狐拦路(第三卷,恶缘)
    RandomEvent {
        title: "妖狐拦路",
        lines: &[
            "阴风骤起。一只瘸腿的九尾妖狐拦在路心,妖瞳猩红:",
            "「当年山径一眼,本座记到今日。」",
        ],
        options: &[
            EventOption {
                label: "拔剑硬闯",
                chance: 0.6,
                success: Outcome {
                    lines: &[
                        "你剑意如虹,妖狐纠缠一阵,恨恨遁去。(道心 +1)",
                        "「修剑先修心——望你早日明白。」风里留下这句。",
                    ],
                    effect: Effect::None,
                    daoxin: 1,
                    qingyuan: 0,
                },
                failure: Some(Outcome {
                    lines: &["妖风撕开护体剑气,你且战且退。(损失一成五气血)"],
                    effect: Effect::DamagePct(15),
                    daoxin: 0,
                    qingyuan: 0,
                }),
            },
            EventOption {
                label: "散财消灾(付 40 文)",
                chance: 1.0,
                success: Outcome {
                    lines: &["妖狐嗤笑一声卷走钱袋:「俗物。」但到底放你过去了。"],
                    effect: Effect::LoseGold(40),
                    daoxin: 0,
                    qingyuan: 0,
                },
                failure: None,
            },
        ],
    },
    // 27 EV_QIN1 · 盲女琴师(第二卷)
    RandomEvent {
        title: "盲女琴师",
        lines: &[
            "疫城渡口,一位盲眼少女怀抱旧琴,指下流出的调子哀而不伤,",
            "竟压住了半条街的疫气。「客人留步——可愿听完这一曲?」",
        ],
        options: &[
            EventOption {
                label: "赠银听曲(付 20 文)",
                chance: 1.0,
                success: Outcome {
                    lines: &[
                        "曲罢,你只觉灵台清明,经脉里多了一缕琴韵。(灵力上限 +4)",
                        "「谢客人。他日再闻此曲,便是重逢。」",
                    ],
                    effect: Effect::GainMaxMp(4),
                    daoxin: 0,
                    qingyuan: 1,
                },
                failure: None,
            },
            EventOption {
                label: "静静听完,不出一声",
                chance: 1.0,
                success: Outcome {
                    lines: &[
                        "一曲终了,少女朝你的方向浅浅一礼:「心静之人,曲子听得最全。」",
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
    // 28 EV_QIN2 · 旧曲重逢(第三卷,听过琴)
    RandomEvent {
        title: "旧曲重逢",
        lines: &[
            "宫墙之外,忽有琴声穿夜而来——正是渡口那支曲子。",
            "盲女琴师循声而立:「我说过,再闻此曲,便是重逢。」",
        ],
        options: &[
            EventOption {
                label: "「一别经年,姑娘别来无恙。」",
                chance: 1.0,
                success: Outcome {
                    lines: &[
                        "她这次奏的是新曲,曲中有山、有雾、有一个执剑北行的人。",
                        "(灵力上限 +6,情缘 +1)「曲名《雾散》。愿它应验。」",
                    ],
                    effect: Effect::GainMaxMp(6),
                    daoxin: 0,
                    qingyuan: 1,
                },
                failure: None,
            },
            EventOption {
                label: "驻足听完,悄然离去",
                chance: 1.0,
                success: Outcome {
                    lines: &["琴声送你到街尾。有些重逢,不必开口。(道心 +1,回三成气血)"],
                    effect: Effect::HealPct(30),
                    daoxin: 1,
                    qingyuan: 0,
                },
                failure: None,
            },
        ],
    },
];

/// Portrait art for an event (indexes into [`EVENTS`]); reuses NPC / creature /
/// prop cutouts from the asset library.
/// 章末决战前的对峙台词(踏入魔门先礼后兵,再入战斗)。
pub fn boss_taunt(boss: super::super::quest::BossKind) -> (&'static str, &'static [&'static str]) {
    use super::super::quest::BossKind;
    match boss {
        BossKind::MoonWraith => (
            "决战 · 月魄妖",
            &[
                "山门内月光凝成实质,一道白影自水月倒影中浮起。",
                "「又是执剑的凡人……你们的魂魄,在月下最是清甜。」",
                "灵儿攥紧了袖角:「逍遥哥哥,它在看我——小心!」",
                "你横剑身前:「今夜之后,水月洞天,再无噬魂之妖。」",
            ],
        ),
        BossKind::RiverDemon => (
            "决战 · 河魇蛟",
            &[
                "江心漩涡轰然炸开,黑鳞巨蛟携着腥雨升出水面。",
                "「三百年了……又有人敢踏进本座的河道?」",
                "「疫气因你而起,灯火因你而熄。」你踏浪而立,剑指蛟目。",
                "「今日,还江城三千户一个太平。」",
            ],
        ),
        BossKind::MiasmaRoot => (
            "决战 · 瘴母根",
            &[
                "祠堂地底,根须如活物般蠕动,紫瘴自根节间汩汩渗出。",
                "「养分……新鲜的养分自己走进来了……」",
                "你想起疫村祠堂里那位抱着孩子的母亲。",
                "「烧了你,雨就停了。」剑锋燃起一线青芒。",
            ],
        ),
        BossKind::MirrorMinister => (
            "决战 · 照影国师",
            &[
                "百面铜镜同时亮起,每一面里都站着一个「你」。",
                "「小侠客,可知镜中人比你更懂你的破绽?」",
                "国师广袖一拂,镜影齐齐拔剑。",
                "「破镜,先破心。」你阖眼再睁,剑心澄明。",
            ],
        ),
        BossKind::ThunderQilin => (
            "决战 · 雷麟",
            &[
                "祭道尽头,雷图腾一座接一座亮起,巨兽踏雷而来。",
                "「南疆的雷,三百年未曾认过主。」",
                "「凡人,接得住第一道雷,再谈资格。」",
                "你反手把剑插进土里,任雷光沿剑身入地——「来。」",
            ],
        ),
        BossKind::DreamEclipse => (
            "决战 · 宿命水影",
            &[
                "灵渊尽头没有妖,只有一泓水——水里映着你来时的每一步。",
                "水影抬头,眉眼竟与你一模一样:「回头吧,雾散不了的。」",
                "「这一路的道心与情缘,都会沉进这里。」",
                "你伸手握住剑柄:「那就连你,一起斩了。」",
            ],
        ),
    }
}

/// 链式奇遇的固定下标(EVENTS 尾部;不进入随机池)。
pub const CHAIN_START: usize = 22;
pub const EV_FOX1: usize = 22;
pub const EV_FOX2_WARM: usize = 23;
pub const EV_FOX2_COLD: usize = 24;
pub const EV_FOX3_WARM: usize = 25;
pub const EV_FOX3_COLD: usize = 26;
pub const EV_QIN1: usize = 27;
pub const EV_QIN2: usize = 28;

pub fn event_portrait(index: usize) -> Option<&'static str> {
    match index {
        0 => Some("npcs/ai_fox_spirit.png"),          // 山间酒肆:老板娘
        1 => Some("npcs/ai_mountain_monk.png"),       // 断剑冢:守冢老人
        2 => Some("npcs/ai_wandering_merchant.png"),  // 落难货郎
        3 => Some("npcs/ai_spirit_guide.png"),        // 古镜残光:白发道人
        5 => Some("props/ai_spirit_lantern.png"),     // 溺水的萤火
        6 => Some("npcs/ai_final_oracle.png"),        // 无名棋局:白衣人
        7 => Some("npcs/ai_shrine_keeper.png"),       // 破庙香火
        8 => Some("props/ai_cave_crystal.png"),       // 灵泉眼
        9 => Some("npcs/forest_ranger.png"),          // 夜行商队:老镖头
        10 => Some("creatures/ai_water_serpent.png"), // 走蛟渡河:老蛟
        11 => Some("npcs/ai_shrine_keeper.png"),      // 山神庙求签
        12 => Some("npcs/ai_wandering_merchant.png"), // 「仙丹」贩子
        13 => Some("npcs/ai_capital_envoy.png"),      // 迷路的书生
        14 => Some("npcs/ai_fox_spirit.png"),         // 雨夜狐宅
        17 => Some("npcs/ai_herb_healer.png"),        // 峭壁灵芝
        19 => Some("npcs/ai_cave_priestess.png"),     // 琴音辨路
        20 => Some("npcs/ai_mansion_spy.png"),        // 悬赏皇榜
        21 => Some("npcs/ai_tribal_chief.png"),       // 虎口樵夫
        _ => None,
    }
}

/// 灵儿的立绘卡(剧情缘节点、歇脚谈心)。
pub const PORTRAIT_LINGER: &str = "npcs/ai_linger.png";
pub const PORTRAIT_MERCHANT: &str = "npcs/ai_wandering_merchant.png";

// ---------------------------------------------------------------------------
// Market node (「市」) — priced picks resolved in code
// ---------------------------------------------------------------------------

pub const MARKET_INTRO: &[&str] = &[
    "山道旁支着几顶货摊,南来北往的行商与妖客在此各摆各的货。",
    "摊主们瞧见你腰间的剑,都热情了三分——斩妖人的钱最好赚。",
];

pub const MARKET_POTION: &str = "买药水(40 文)";
pub const MARKET_TONIC: &str = "买淬体丹(70 文,气血上限 +12)";
pub const MARKET_WHETSTONE: &str = "买砺剑石(60 文,攻击 +2)";
pub const MARKET_LEAVE: &str = "什么都不买,逛逛就走";

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
pub const REST_TALK: &str = "与灵儿谈心(情缘 +1,回复三成气血)";

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

pub const ENDING_BOTH: &[&str] = &[
    "【结局 · 执剑逐月】",
    "水影散尽,心渊里升起一轮从未有人见过的月亮。",
    "你没有留在山上,也没有收起剑——道心与情缘,原来从不必二选其一。",
    "灵儿在渡口等到了你:一手牵她,一手仗剑,便是往后余生。",
    "多年后,南疆到桃溪的山道太平无事。夜里赶路的人说,",
    "总有一对剑侣踏月而行:剑光为灯,铃声为路。",
];

pub const ENDING_DEFEAT: &[&str] = &[
    "【轮回 · 未竟之誓】",
    "剑光熄灭的那一刻,你听见很远的地方有铃响了一声。",
    "黑暗并不冰冷,反而像溪水一样把你托起、送回——",
    "再睁眼,又是桃溪村的清晨,祠堂前的誓言墨迹未干。",
    "妖雾仍在,誓言仍在。这一世,换条路,再走一遍。",
];
