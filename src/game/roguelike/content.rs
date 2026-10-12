//! Narrative & event content for the roguelike run: chapter cards, story
//! scenes, the random-event pool, rest options, and endings.
//!
//! All text is original xianxia-flavoured writing for this project. Story
//! choices push the 道心 / 情缘 counters that pick the final ending.

use super::narrative::Cost;
use super::{Relic, skill::SkillId};

// ---------------------------------------------------------------------------
// Effects an option can apply
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Effect {
    None,
    /// Heal percentage of max hp.
    HealPct(i32),
    /// Lose percentage of max hp, never lethal.
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
    Relic(Relic),
    LearnSkill(SkillId),
    RestoreMpPct(i32),
    /// Authored rewards can include resources and a build-defining gift.
    Bundle(&'static [Effect]),
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
    pub cost: Cost,
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

pub const CHAPTER_CARDS: [&[&str]; 7] = [
    &[
        "【第一卷 · 余杭夜雨】",
        "余杭客栈外妖雾骤起,破庙赤火连着十里坡一路烧来。",
        "村人受伤,渡路被妖雾封住。李逍遥带剑出门,先查客栈外的妖讯。",
        "灵儿在渡口等他:她能辨认妖雾中的灵息,也想查清村人遇袭的缘由。",
    ],
    &[
        "【第二卷 · 水月洞天】",
        "赤鬼倒下后,残余黑气仍指向水月洞天。只除追兵,并未封住妖雾的源头。",
        "灵儿的铃声在泉边响起,像在提醒你誓言已经落地。",
        "洞天门后,月魄妖正等着把同行之人拆成倒影。",
    ],
    &[
        "【第三卷 · 苏州江灯】",
        "封住洞天裂口后,你们循阵上残留的黑鳞来到苏州;这里的河灯正逆流入雾。",
        "摆渡人说:「蛟影不除,这一江的船都会载着亡魂回来。」",
        "灵儿低声道:「水路也会记得谁被它带走。」",
    ],
    &[
        "【第四卷 · 白河疫雨】",
        "河魇退去,获救船工却染着黑水疫。林月衡带来白河求医的消息,与你们同行。",
        "老医跪求:「疫有母根,不斩根,村镇三千户皆成孤坟。」",
        "这一路,怕是要见许多不忍见之事。",
    ],
    &[
        "【第五卷 · 京华镜影】",
        "白河母根已除,井底却留下国师府的镇水符。为查清疫水来处,你们赶赴京城。",
        "皇榜悬赏斩妖人,暗巷里却有人递来一句:「妖非祸源,人心才是。」",
        "腕上的桃木铃,第一次自己响了一声。",
    ],
    &[
        "【第六卷 · 南疆雷誓】",
        "镜阵破后,密图指向南疆引雷祭道。守鼓人南瑶接过密图,答应带你们查明锁麟旧债。",
        "南瑶望着雷纹沉默很久:「那是族里的旧誓,也是旧债。」",
        "若接不住这道雷,终局水门便不会为外人打开。",
    ],
    &[
        "【终卷 · 心渊照影】",
        "雷麟的封锁解除,雷光照出流入心渊的黑水源头;四人循水而下,要终止妖雾与疫水。",
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
    pub cost: Cost,
    pub outcome: Outcome,
}

#[derive(Clone, Copy)]
pub struct StoryScene {
    pub title: &'static str,
    pub lines: &'static [&'static str],
    pub prompt: &'static str,
    pub options: &'static [StoryOption],
}

pub const STORY_SCENES: [StoryScene; 7] = [
    StoryScene {
        title: "渡口同行",
        lines: &[
            "客栈后院的旧渡口,灵儿带着伤药来找你,发梢还沾着夜露。",
            "灵儿:「妖雾里的灵息我认得。把村人安顿好后,我陪你走,别一个人冒险。」",
            "她把伤药塞进你怀里,又系上一枚桃木铃:「雾里若走散,循铃声找我。」",
        ],
        prompt: "你如何作答?",
        options: &[
            StoryOption {
                label: "握住她的手:「等雾散了,我带你看真正的月亮。」",
                cost: Cost::Free,
                outcome: Outcome {
                    lines: &[
                        "灵儿愣了愣,忽然笑出声,把半块桂花糕也塞给你。",
                        "「那说好了。骗人的话,铃会记得。」",
                    ],
                    effect: Effect::Bundle(&[Effect::GainMaxHp(8), Effect::GainPotions(1)]),
                    daoxin: 0,
                    qingyuan: 2,
                },
            },
            StoryOption {
                label: "接过伤药:「你先安顿村人,我在坡口等你。」",
                cost: Cost::Free,
                outcome: Outcome {
                    lines: &[
                        "她没再说话,只把铃又系紧了一圈。",
                        "山风灌进衣袖,你的木剑忽然稳了几分。",
                    ],
                    effect: Effect::Bundle(&[
                        Effect::GainAtk(1),
                        Effect::GainMaxMp(4),
                        Effect::GainPotions(1),
                    ]),
                    daoxin: 2,
                    qingyuan: 0,
                },
            },
        ],
    },
    StoryScene {
        title: "泉心辨影",
        lines: &[
            "水月洞天的泉心映出两个影子:一个是你,一个却站在灵儿身后。",
            "灵儿抬手按住桃木铃:「它在学我的声音,想让你回头。」",
            "泉面一圈圈泛开,像在问你要先护人,还是先破阵。",
        ],
        prompt: "月影逼近,你先——",
        options: &[
            StoryOption {
                label: "挡在灵儿身前,先稳住她的灵息。",
                cost: Cost::Free,
                outcome: Outcome {
                    lines: &[
                        "灵儿的指尖终于不再发冷,铃声也稳成一线。",
                        "灵儿把稳住灵息的月轮诀教给你。月影慢了一息,足够你们看清真正的阵眼。",
                    ],
                    effect: Effect::Bundle(&[
                        Effect::LearnSkill(SkillId::MoonHalo),
                        Effect::GainMaxMp(4),
                    ]),
                    daoxin: 0,
                    qingyuan: 2,
                },
            },
            StoryOption {
                label: "闭眼听水声,一剑斩向假影。",
                cost: Cost::Free,
                outcome: Outcome {
                    lines: &[
                        "剑光落处,假影碎成一地冷月。灵阵忽然安静下来。",
                        "灵儿指向碎影中的寒纹:「将灵力凝在这里,就能以霜枪截住下一道假影。」",
                    ],
                    effect: Effect::Bundle(&[
                        Effect::LearnSkill(SkillId::FrostLance),
                        Effect::GainMaxMp(4),
                    ]),
                    daoxin: 2,
                    qingyuan: 0,
                },
            },
        ],
    },
    StoryScene {
        title: "船夫守灯",
        lines: &[
            "苏州渡口的夜里,一个老船夫把破灯推到你面前。",
            "「我儿子追河魇去了,灯若还亮,说明他还在水下等人拉一把。」",
            "灵儿望着逆流的灯,眼里映着一整条发冷的江。",
        ],
        prompt: "你如何处置这盏灯?",
        options: &[
            StoryOption {
                label: "替船夫守灯,等水下的人回应。",
                cost: Cost::Free,
                outcome: Outcome {
                    lines: &[
                        "灯芯忽然跳了一下,水面浮起一枚湿透的渡符。",
                        "老船夫听见灯中的回应,把预备的船钱交给你们:「带着这个,再去把他找回来。」",
                    ],
                    effect: Effect::Bundle(&[Effect::HealPct(25), Effect::GainGold(30)]),
                    daoxin: 0,
                    qingyuan: 2,
                },
            },
            StoryOption {
                label: "带灯入雾,用它标出河魇巢口。",
                cost: Cost::Free,
                outcome: Outcome {
                    lines: &[
                        "灯火在雾里拉成一条直线,黑鳞影子终于露出尾端。",
                        "灵儿辨出灯火划过的水纹,教你以沧浪之势催动剑气。你记下巢口,准备继续救人。",
                    ],
                    effect: Effect::LearnSkill(SkillId::TidalDragon),
                    daoxin: 2,
                    qingyuan: 0,
                },
            },
        ],
    },
    StoryScene {
        title: "祠堂药引",
        lines: &[
            "疫村祠堂,一位母亲抱着昏睡的孩子跪在药炉前。",
            "老医叹道:「炉中只剩一副药引。给了孩子,你身上的旧伤便压不住了。」",
            "灵儿看着你,没有说话——她知道你听得见孩子的呼吸声。",
        ],
        prompt: "药引给谁?",
        options: &[
            StoryOption {
                label: "给孩子。旧伤而已,扛得住。",
                cost: Cost::Free,
                outcome: Outcome {
                    lines: &[
                        "孩子的热度退了。那位母亲朝你磕了三个头,你没敢受全。",
                        "灵儿替你裹紧旧伤。孩子的母亲分出两瓶普通伤药:「治不了疫,至少能缓你的外伤。」",
                    ],
                    effect: Effect::Bundle(&[
                        Effect::DamagePct(15),
                        Effect::GainPotions(2),
                        Effect::LearnSkill(SkillId::MoonHalo),
                    ]),
                    daoxin: 0,
                    qingyuan: 2,
                },
            },
            StoryOption {
                label: "自己服下。斩不了瘴母,救谁都是空话。",
                cost: Cost::Free,
                outcome: Outcome {
                    lines: &[
                        "药力化开,经脉如洗。你在祠堂梁上刻下一行字:",
                        "「先破母根,再回村救人。」老医另画出缚根的藤纹,教你用它截住瘴气。",
                    ],
                    effect: Effect::Bundle(&[
                        Effect::HealPct(30),
                        Effect::LearnSkill(SkillId::ThornCage),
                    ]),
                    daoxin: 2,
                    qingyuan: 0,
                },
            },
        ],
    },
    SCENE_PALACE,
    StoryScene {
        title: "雷泽问愿",
        lines: &[
            "南疆雷泽,古麟锁于青铜柱间,鳞下压着一枚褪色的许愿笺。",
            "笺上写着「愿阿爹平安归家」。南瑶认出背面的国师旧印,却无法确定是谁写下的愿。",
            "灵儿轻声:「原来锁妖的人,也曾被谁这样盼着回家。」",
        ],
        prompt: "面对锁链,你——",
        options: &[
            StoryOption {
                label: "护住古麟:「先止住锁链抽取的雷,再破阵放它出来。」",
                cost: Cost::Free,
                outcome: Outcome {
                    lines: &[
                        "灵儿以灵息护住鳞下伤口,古麟不再挣扎。外阵仍在,须到祭道尽头击破它。",
                        "灵儿在雨里朝你伸出手,你们在雷声里笑得像两个孩子。",
                    ],
                    effect: Effect::Bundle(&[
                        Effect::GainMaxMp(4),
                        Effect::LearnSkill(SkillId::MountainAegis),
                    ]),
                    daoxin: 0,
                    qingyuan: 2,
                },
            },
            StoryOption {
                label: "借散雷淬剑:「不再抽它的灵息,用溢出的雷破锁阵。」",
                cost: Cost::Free,
                outcome: Outcome {
                    lines: &[
                        "你以剑引雷,剑身多了一道细密的雷纹。",
                        "南瑶稳住雷路,让散雷落在剑上。你们记下锁阵方位,继续寻找阵眼。",
                    ],
                    effect: Effect::Bundle(&[
                        Effect::GainMaxMp(4),
                        Effect::LearnSkill(SkillId::StormChain),
                    ]),
                    daoxin: 2,
                    qingyuan: 0,
                },
            },
        ],
    },
    StoryScene {
        title: "心渊拒梦",
        lines: &[
            "心渊入口,水面浮出你一路走来的倒影:渡口、祠堂、雷泽。",
            "水影低语:「放下剑,便还你一个人人无恙的梦。」",
            "腕上的桃木铃轻轻晃着,像是在等你的答案。",
        ],
        prompt: "梦与剑,你选——",
        options: &[
            StoryOption {
                label: "摇响铃铛:「梦是假的,身边同伴的声音才是真的。」",
                cost: Cost::Free,
                outcome: Outcome {
                    lines: &[
                        "铃声穿透水面,倒影尽碎。灵儿就在你身旁握住了你的手,其余两人也走过来。",
                        "你们沿着真实的脚步声越过入口，没有接受水影许下的假归途。",
                    ],
                    effect: Effect::Bundle(&[Effect::HealPct(30), Effect::RestoreMpPct(30)]),
                    daoxin: 0,
                    qingyuan: 3,
                },
            },
            StoryOption {
                label: "举剑入渊:「我斩的不是妖,是不敢面对的自己。」",
                cost: Cost::Free,
                outcome: Outcome {
                    lines: &[
                        "水面平息,剑光澄澈如初雪。道心至此,再无杂音。",
                        "你回身确认同伴都在，才继续向黑水深处走去。",
                    ],
                    effect: Effect::Bundle(&[Effect::GainAtk(2), Effect::RestoreMpPct(30)]),
                    daoxin: 3,
                    qingyuan: 0,
                },
            },
        ],
    },
];

// Additional story scenes follow each chapter opening in authored order.
// Randomness belongs to encounters, never to causal main-story sequencing.

const SCENE_EAVES: StoryScene = StoryScene {
    title: "檐下许愿",
    lines: &[
        "夜雨骤至,你与灵儿躲进一处旧屋檐,火折子的光只照亮彼此的脸。",
        "灵儿数着檐角滴下的雨:「我娘说,雨夜里说的愿望,山神听得最清。」",
        "她偏过头看你:「你许一个,我便也许一个。」",
    ],
    prompt: "你许的愿是——",
    options: &[
        StoryOption {
            label: "「愿身边这个人,岁岁平安。」",
            cost: Cost::Free,
            outcome: Outcome {
                lines: &[
                    "灵儿的耳尖红了,她小声说了自己的愿,却死活不肯讲给你听。",
                    "她将余下的伤药递给你,替你重新包扎了伤口。",
                ],
                effect: Effect::HealPct(25),
                daoxin: 0,
                qingyuan: 2,
            },
        },
        StoryOption {
            label: "「愿此剑所指,再无枉死之人。」",
            cost: Cost::Free,
            outcome: Outcome {
                lines: &[
                    "檐外雷声轻轻应了一声。灵儿望着你的侧脸,忽然觉得陌生,又忽然觉得安心。",
                    "你以门边的旧木拆过几招，直到剑势不再随着雷声摇摆。",
                ],
                effect: Effect::GainDef(2),
                daoxin: 2,
                qingyuan: 0,
            },
        },
    ],
};

const SCENE_OPERA: StoryScene = StoryScene {
    title: "避难棚旧戏",
    lines: &[
        "路边避难棚里,老艺人借一面小锣唱起旧戏:斩妖人斩尽心魔,将要白日飞升。",
        "台下孩童学着挥剑,笑闹声撞进锣鼓里。灵儿买了两串糖葫芦,递给你一串。",
        "戏至高潮,斩妖人却在飞升前回了头。暂歇的行人听着,有人笑,有人抹眼泪。",
    ],
    prompt: "你觉得他该不该回头?",
    options: &[
        StoryOption {
            label: "「该。成仙有什么好,人间烟火才是真的。」",
            cost: Cost::Free,
            outcome: Outcome {
                lines: &[
                    "灵儿咬着糖葫芦笑弯了眼:「那你到时候,也记得回头。」",
                    "老艺人把备用伤药分给赶路的人，盼你们守住棚外这一段路。",
                ],
                effect: Effect::GainPotions(1),
                daoxin: 0,
                qingyuan: 2,
            },
        },
        StoryOption {
            label: "「不该。既起了誓,回头便是负了所有人。」",
            cost: Cost::Free,
            outcome: Outcome {
                lines: &[
                    "邻座的老者闻言看了你一眼,朝你举了举酒碗:「后生,是个走得远的。」",
                    "你向他请教持剑守路的招式，起身时又将那一式练了一遍。",
                ],
                effect: Effect::GainAtk(2),
                daoxin: 2,
                qingyuan: 0,
            },
        },
    ],
};

const SCENE_LANTERNS: StoryScene = StoryScene {
    title: "渡岸祈灯",
    lines: &[
        "循着船夫的灯探过河雾后,你们在安全的渡岸歇脚。岸边人们为失踪的船工点起祈愿灯。",
        "一个失了双亲的小女孩折不好灯纸,急得直掉眼泪。",
        "灵儿蹲下去帮她,回头看你:「再折一盏么?给……将来的我们。」",
    ],
    prompt: "河灯放与不放?",
    options: &[
        StoryOption {
            label: "折一盏双生灯,与她并肩放进江里。",
            cost: Cost::Free,
            outcome: Outcome {
                lines: &[
                    "两盏灯在江心靠在一起,顺流而下,久久不散。",
                    "小女孩破涕为笑:「灯挨着灯,人就不会走散。」",
                ],
                effect: Effect::HealPct(20),
                daoxin: 0,
                qingyuan: 2,
            },
        },
        StoryOption {
            label: "只放一盏无名灯,祭这一城亡魂。",
            cost: Cost::Free,
            outcome: Outcome {
                lines: &[
                    "灯入江心,你在心里把三千户的名字都念了一遍——念不出名字的,就记住灯影。",
                    "河风将灯送出渡岸，你们回到行人歇脚的棚中，替他们守住这一夜。",
                ],
                effect: Effect::GainMaxHp(10),
                daoxin: 2,
                qingyuan: 0,
            },
        },
    ],
};

const SCENE_QIN: StoryScene = StoryScene {
    title: "断弦归雁",
    lines: &[
        "废驿站里有位老琴师许伯,守着一张断了两根弦的旧琴,不肯随难民南逃。",
        "「琴是内子的陪嫁。她走在疫里,我走了,谁给她弹《归雁》?」",
        "琴声残缺,却稳得像多年未改的心跳。",
    ],
    prompt: "你——",
    options: &[
        StoryOption {
            label: "以剑代弦,为他补全一曲《归雁》。",
            cost: Cost::Free,
            outcome: Outcome {
                lines: &[
                    "剑鸣清越,续上断弦的空处。曲终,琴师朝虚空一揖:「夫人,有客到。」",
                    "灵儿在门边听完了整曲,眼睛亮亮的没说话。",
                ],
                effect: Effect::GainMaxMp(8),
                daoxin: 0,
                qingyuan: 2,
            },
        },
        StoryOption {
            label: "留下盘缠,劝他随人流南去。",
            cost: Cost::Gold(30),
            outcome: Outcome {
                lines: &[
                    "琴师收下盘缠,把亡妻留下的引魂灯递给你:「有人还记得她的曲子,这盏灯便不必守在旧屋里。」",
                    "他答应随下一批行人离开。你们没有替他忘记,只是给了他继续走的路。",
                ],
                effect: Effect::Relic(Relic::SoulLantern),
                daoxin: 2,
                qingyuan: 0,
            },
        },
    ],
};

const SCENE_PALACE: StoryScene = StoryScene {
    title: "夜宴问剑",
    lines: &[
        "京城夜宴,国师隔着满席歌舞举杯敬你:「斩妖人,你剑上妖血未干。」",
        "「我锁妖养煞,是为社稷;你逢妖便斩,又为的什么?说来听听。」",
        "满殿烛火摇了摇。灵儿的手在案下轻轻按住你的手背。",
    ],
    prompt: "你如何作答?",
    options: &[
        StoryOption {
            label: "反手握住灵儿:「为身后之人不必学会握剑。」",
            cost: Cost::Free,
            outcome: Outcome {
                lines: &[
                    "国师笑而不答。散宴后,线人递来一面阴阳镜:「留着看清他藏起的另一张脸。」",
                    "「好答案。可惜本座……早已没有身后之人。」",
                ],
                effect: Effect::Relic(Relic::YinYangMirror),
                daoxin: 0,
                qingyuan: 2,
            },
        },
        StoryOption {
            label: "举杯饮尽:「剑问的是心,不是妖。国师心里有妖。」",
            cost: Cost::Free,
            outcome: Outcome {
                lines: &[
                    "殿内死寂。国师缓缓鼓掌,烛火齐齐矮了三分。",
                    "国师下了逐客令。林月衡在暗巷接住你,交来从库房带出的御剑心诀。",
                ],
                effect: Effect::Relic(Relic::SwordSutra),
                daoxin: 2,
                qingyuan: 0,
            },
        },
    ],
};

const SCENE_BONFIRE: StoryScene = StoryScene {
    title: "篝火共舞",
    lines: &[
        "南疆部族的篝火祭上,祭司请远客共舞一曲,说舞给雷神看的人不会说谎。",
        "鼓点如雨。灵儿被姑娘们拉进火光里,笑着朝你伸出手。",
        "祭司低声道:「跳,便把心跳给她看;不跳,便把心留给山看。」",
    ],
    prompt: "鼓点催了三巡,你——",
    options: &[
        StoryOption {
            label: "踏进火光,握住那只手。",
            cost: Cost::Free,
            outcome: Outcome {
                lines: &[
                    "你舞得笨拙,她笑得直不起腰,火星升上夜空像一场倒下的星雨。",
                    "祭司朝雷云举杯:「雷神看见了。」",
                ],
                effect: Effect::HealPct(30),
                daoxin: 0,
                qingyuan: 3,
            },
        },
        StoryOption {
            label: "静坐火外,以剑击节为她伴奏。",
            cost: Cost::Free,
            outcome: Outcome {
                lines: &[
                    "剑鸣与鼓点相和,火光里的身影旋得更轻快了。",
                    "祭司看看你,又看看剑:「也好。山也看见了。」",
                ],
                effect: Effect::GainAtk(2),
                daoxin: 2,
                qingyuan: 0,
            },
        },
    ],
};

const SCENE_DRUM_OATH: StoryScene = StoryScene {
    title: "守鼓旧债",
    lines: &[
        "雷鼓祭台背后刻着一列守鼓人的名字,最末一行被刀刮得只剩一个「瑶」字。",
        "南瑶用指腹慢慢描过缺口:「我爹守过这面鼓,也替族里背走了不该背的罪。」",
        "远处雷声压下来。她没有回头:「旧债落到我手里,你说该认,还是该断?」",
    ],
    prompt: "你把手按在旧鼓上——",
    options: &[
        StoryOption {
            label: "「债可以认,但往后由我们一起还。」",
            cost: Cost::Free,
            outcome: Outcome {
                lines: &[
                    "南瑶终于转身,把一截褪色鼓绳系在你的剑穗旁。",
                    "第一声鼓响时,雷没有劈向祭台,反而沿四人的影子铺出一条亮路。",
                ],
                effect: Effect::GainMaxMp(8),
                daoxin: 0,
                qingyuan: 3,
            },
        },
        StoryOption {
            label: "「该认的记住,不该认的从这一刀断掉。」",
            cost: Cost::Free,
            outcome: Outcome {
                lines: &[
                    "你沿着旧刀痕补下一剑,石上的残名没有消失,却不再像一道枷锁。",
                    "南瑶收起鼓槌:「好。名字留下,罪不再传。」",
                ],
                effect: Effect::GainDef(2),
                daoxin: 3,
                qingyuan: 0,
            },
        },
    ],
};

const SCENE_CORRIDOR: StoryScene = StoryScene {
    title: "旧梦回廊",
    lines: &[
        "心渊深处有一条回廊,两侧水幕重演着你们的初遇:溪边、木剑、桃木铃。",
        "水幕里的灵儿朝水幕外的你招手,声音隔着一层旧梦:「这次,别走那么快。」",
        "回廊尽头有两扇门:一扇写着「留」,一扇写着「行」。",
    ],
    prompt: "旧梦当前,你推开——",
    options: &[
        StoryOption {
            label: "「行」。梦里多留一刻,现实便多险一分。",
            cost: Cost::Free,
            outcome: Outcome {
                lines: &[
                    "你穿梦而过,不曾回头。身后的水幕轻轻叹了口气,散作满廊星光。",
                    "走过回廊后，你看清了梦影的破绽，将这一剑教给身后的同伴。",
                ],
                effect: Effect::GainAtk(2),
                daoxin: 3,
                qingyuan: 0,
            },
        },
        StoryOption {
            label: "「留」。在梦里陪她把那条溪走完。",
            cost: Cost::Free,
            outcome: Outcome {
                lines: &[
                    "梦里的溪很长,长到你想起了每一件差点忘记的小事。",
                    "走出门时,掌心多了一道温热的铃印。",
                ],
                effect: Effect::HealPct(35),
                daoxin: 0,
                qingyuan: 3,
            },
        },
    ],
};

const SCENE_DREAM_INN: StoryScene = StoryScene {
    title: "客栈热面",
    lines: &[
        "回梦客栈里灶火正旺,婶婶背对门口切着姜丝,仿佛你只出门打了一壶酒。",
        "桌上摆着一碗还冒热气的面。她骂道:「臭小子,走那么久,还知道回来?」",
        "门外水声越来越近,这顿饭若坐下,终门便会迟开一刻。",
    ],
    prompt: "你站在熟悉的门槛上——",
    options: &[
        StoryOption {
            label: "坐下吃完这碗面,再好好说一声告别。",
            cost: Cost::Free,
            outcome: Outcome {
                lines: &[
                    "面还是记忆里的味道。婶婶嘴上不停数落,却悄悄多卧了一个蛋。",
                    "碗底见空时,客栈随水光散去,你终于把那句「我会回来」说完整。",
                ],
                effect: Effect::HealPct(30),
                daoxin: 0,
                qingyuan: 3,
            },
        },
        StoryOption {
            label: "把剑穗留在桌边:「等路走完,我亲自回来吃。」",
            cost: Cost::Free,
            outcome: Outcome {
                lines: &[
                    "婶婶没有回头,只把那碗面扣在锅里保温。",
                    "你跨出门时不再回望。归途不是梦里的一顿饭,是门后还要争来的明天。",
                ],
                effect: Effect::GainMaxHp(10),
                daoxin: 3,
                qingyuan: 0,
            },
        },
    ],
};

const SCENE_WISH_LAMPS: StoryScene = StoryScene {
    title: "万灯记人",
    lines: &[
        "心渊回廊忽然亮起许多小灯:渡口船夫、白河病童、苗岭鼓使的声音一盏盏传来。",
        "水影借他们的面容许诺回报。灵儿提醒你:「这些只是记忆,不能替活着的人许愿。」",
        "水影冷笑:「借众人的愿走到最后,还算是你自己的剑吗?」",
    ],
    prompt: "众愿聚在掌前,你——",
    options: &[
        StoryOption {
            label: "记住灯中的人:「他们还在外面,我们要把路走完。」",
            cost: Cost::Free,
            outcome: Outcome {
                lines: &[
                    "你们没有把幻灯当作援军,而是彼此照应,在黑水上寻到落脚处。",
                    "灵儿念起沿途相遇的名字,提醒大家为何走到了这里。",
                ],
                effect: Effect::GainPotions(2),
                daoxin: 0,
                qingyuan: 3,
            },
        },
        StoryOption {
            label: "把灯送回人间:「他们的愿该照自己的家。」",
            cost: Cost::Free,
            outcome: Outcome {
                lines: &[
                    "万点微光逆流而上,回到每一扇仍有人等待的窗。",
                    "你只留下一盏无名灯。没有借来的光,脚下那一步反而更稳。",
                ],
                effect: Effect::GainDef(2),
                daoxin: 3,
                qingyuan: 0,
            },
        },
    ],
};

const SCENE_SPIRIT_ALTAR: StoryScene = StoryScene {
    title: "灵台共守",
    lines: &[
        "女娲灵台浮出一道古老祭文:若以灵血续阵,可让终门安静百年。",
        "灵儿读完后伸出手腕,语气平静得让人害怕:「我的血能做到。你不必替我决定。」",
        "林月衡按住剑柄,南瑶则望向你。灵台在等一个不能收回的回答。",
    ],
    prompt: "祭文亮到最后一行,你——",
    options: &[
        StoryOption {
            label: "握住她的手:「要守便一起守,不拿你一人换百年。」",
            cost: Cost::Free,
            outcome: Outcome {
                lines: &[
                    "四道灵息同时落入阵心,古祭文被改写成一个从未有过的「同」字。",
                    "灵台震了很久,最终没有索取任何人的血。",
                ],
                effect: Effect::GainMaxMp(10),
                daoxin: 0,
                qingyuan: 3,
            },
        },
        StoryOption {
            label: "收剑退开:「你的命由你选,我负责让选择有退路。」",
            cost: Cost::Free,
            outcome: Outcome {
                lines: &[
                    "灵儿看了你很久,最终没有割腕,而是以指尖灵光重排祭文。",
                    "你没有替她拒绝,也没有任她独担。阵心因此认下了两个人的意志。",
                ],
                effect: Effect::GainAtk(2),
                daoxin: 3,
                qingyuan: 0,
            },
        },
    ],
};

const SCENE_SWORD_MIRROR: StoryScene = StoryScene {
    title: "断镜并肩",
    lines: &[
        "水魔鳞潮间立着一面断镜,镜中只有林月衡一人守在崩塌的石门后。",
        "她看见自己的倒影,淡淡道:「镜子说总得留一个人断后。它倒很懂我。」",
        "石门已经开始下沉,镜中结局正等着谁先相信。",
    ],
    prompt: "林月衡抬剑走向石门,你——",
    options: &[
        StoryOption {
            label: "与她并肩斩镜:「断后不是把同伴关在门外。」",
            cost: Cost::Free,
            outcome: Outcome {
                lines: &[
                    "两道剑光在镜心交错,预言碎成无数条谁也不必独留的路。",
                    "林月衡收剑时笑了一声:「这次算你抢了我的差事。」",
                ],
                effect: Effect::HealPct(35),
                daoxin: 0,
                qingyuan: 3,
            },
        },
        StoryOption {
            label: "替她撑住石门:「你带他们走,这一回换我守。」",
            cost: Cost::Free,
            outcome: Outcome {
                lines: &[
                    "林月衡没有争辩,只在穿门前以剑鞘重重敲了你的肩。",
                    "你等所有人越过才一剑断柱。镜中的孤影慢了一步,被碎石永远埋住。",
                ],
                effect: Effect::GainDef(3),
                daoxin: 3,
                qingyuan: 0,
            },
        },
    ],
};

const SCENE_LAST_BELL: StoryScene = StoryScene {
    title: "终门铃誓",
    lines: &[
        "天蛇祭阶尽头,桃木铃终于响了第三声。终门后的水影用每个人的声音说着告别。",
        "南瑶把鼓绳缠紧,林月衡横剑在前,灵儿只问了一句:「进去以后,还一起回来吗?」",
        "谁都知道这不是一句轻易能许的承诺。",
    ],
    prompt: "终门开启前,你的回答是——",
    options: &[
        StoryOption {
            label: "伸出手:「一个都不少。回来再听婶婶骂我。」",
            cost: Cost::Free,
            outcome: Outcome {
                lines: &[
                    "四只手在铃声里叠到一起,终门第一次没有映出孤身一人的影子。",
                    "哪怕承诺未必都能兑现,此刻也没有谁被留在告别之外。",
                ],
                effect: Effect::HealPct(40),
                daoxin: 0,
                qingyuan: 3,
            },
        },
        StoryOption {
            label: "拔剑答道:「不许空话,只许尽力活着走出来。」",
            cost: Cost::Free,
            outcome: Outcome {
                lines: &[
                    "林月衡第一个点头,南瑶敲响半声战鼓,灵儿把铃重新系紧。",
                    "你没有向命运许诺结局,只把能做的每一步都握在手里。",
                ],
                effect: Effect::GainAtk(3),
                daoxin: 3,
                qingyuan: 0,
            },
        },
    ],
};

/// Extra 「缘」 scenes in chronological order after the primary scene.
pub const STORY_EXTRA: [&[StoryScene]; 7] = [
    &[SCENE_EAVES, SCENE_OPERA],
    &[SCENE_EAVES],
    &[SCENE_LANTERNS, SCENE_QIN],
    &[SCENE_QIN],
    &[SCENE_OPERA],
    &[SCENE_DRUM_OATH, SCENE_BONFIRE],
    &[
        SCENE_CORRIDOR,
        SCENE_DREAM_INN,
        SCENE_WISH_LAMPS,
        SCENE_SPIRIT_ALTAR,
        SCENE_SWORD_MIRROR,
        SCENE_LAST_BELL,
    ],
];

/// Number of story scenes available to a chapter's 「缘」 node.
pub fn story_count(chapter: usize) -> usize {
    let chapter = chapter.min(STORY_EXTRA.len() - 1);
    1 + STORY_EXTRA[chapter].len()
}

/// Roll index 0 → the primary scene, 1.. → extras.
pub fn pick_story(chapter: usize, roll: usize) -> &'static StoryScene {
    let chapter = chapter.min(STORY_EXTRA.len() - 1);
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
                label: "豪饮三碗",
                cost: Cost::Gold(30),
                chance: 0.65,
                success: Outcome {
                    lines: &["酒入愁肠,百脉俱畅!老板娘还多送了你一坛。"],
                    effect: Effect::HealPct(40),
                    daoxin: 0,
                    qingyuan: 0,
                },
                failure: Some(Outcome {
                    lines: &["酒里有蒙汗药!你挣扎着逃出,酒肆化作一缕青烟。"],
                    effect: Effect::DamagePct(15),
                    daoxin: 0,
                    qingyuan: 0,
                }),
            },
            EventOption {
                label: "只讨碗水,道谢离开",
                cost: Cost::Free,
                chance: 1.0,
                success: Outcome {
                    lines: &["老板娘眼中闪过一丝赞许:「小郎君心定,送你一句:前路莫贪。」"],
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
                cost: Cost::Free,
                chance: 0.6,
                success: Outcome {
                    lines: &["断剑化光入体,前辈剑意在你经脉中流转!"],
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
                cost: Cost::Free,
                chance: 1.0,
                success: Outcome {
                    lines: &["碑成之刻,风止剑息。一位守冢老人默默递来一瓶伤药。"],
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
                cost: Cost::Free,
                chance: 1.0,
                success: Outcome {
                    lines: &["货郎千恩万谢,硬塞给你一把碎银。"],
                    effect: Effect::GainGold(60),
                    daoxin: 0,
                    qingyuan: 1,
                },
                failure: None,
            },
            EventOption {
                label: "买下他担子里剩下的伤药",
                cost: Cost::Gold(25),
                chance: 1.0,
                success: Outcome {
                    lines: &["「侠士厚道!」他把仅剩的伤药全给了你,只收半价。"],
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
                cost: Cost::Free,
                chance: 0.55,
                success: Outcome {
                    lines: &["掌风过处,你只觉筋骨齐鸣,竟被喂拳喂出了几分火候!"],
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
                cost: Cost::Free,
                chance: 1.0,
                success: Outcome {
                    lines: &["道人抚须而笑,镜面荡开涟漪,一缕清气没入你眉心。"],
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
                label: "掏钱赌一把",
                cost: Cost::Gold(50),
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
                    lines: &["布下只有一块画了符的砖头。狸猫早已卷摊跑路。"],
                    effect: Effect::None,
                    daoxin: 0,
                    qingyuan: 0,
                }),
            },
            EventOption {
                label: "验看摊货,指出鱼目石再离开",
                cost: Cost::Free,
                chance: 1.0,
                success: Outcome {
                    lines: &[
                        "你指出摊上混入的鱼目石。狸猫撤下假货,分你一点验货酬钱:「省得坏了招牌。」",
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
                cost: Cost::Free,
                chance: 0.75,
                success: Outcome {
                    lines: &["小妖抖干翅膀,绕着你转了三圈,把一粒暖融融的光种埋进你掌心。"],
                    effect: Effect::GainMaxHp(10),
                    daoxin: 0,
                    qingyuan: 1,
                },
                failure: Some(Outcome {
                    lines: &["潭底暗流凶猛,你呛了好几口水才把它托上岸。小妖愧疚地贴了贴你的脸。"],
                    effect: Effect::DamagePct(10),
                    daoxin: 0,
                    qingyuan: 1,
                }),
            },
            EventOption {
                label: "折根芦苇递过去",
                cost: Cost::Free,
                chance: 1.0,
                success: Outcome {
                    lines: &["小妖攀着芦苇爬上岸,朝你眨了眨眼,点亮了前方的路。"],
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
            "「试棋先付四十文。能破此局,我传你一手剑招;破不了,也没有退钱的规矩。」",
        ],
        options: &[
            EventOption {
                label: "落子",
                cost: Cost::Gold(40),
                chance: 0.5,
                success: Outcome {
                    lines: &["你一子破局!白衣人抚掌:「妙。」指尖点上你的剑脊。"],
                    effect: Effect::GainAtk(2),
                    daoxin: 1,
                    qingyuan: 0,
                },
                failure: Some(Outcome {
                    lines: &["三手之后满盘皆输。白衣人笑着收走了你的钱袋子一角。"],
                    effect: Effect::None,
                    daoxin: 0,
                    qingyuan: 0,
                }),
            },
            EventOption {
                label: "掀不动这浑水,绕亭而过",
                cost: Cost::Free,
                chance: 1.0,
                success: Outcome {
                    lines: &["身后传来一声轻笑:「知进退,也算一种棋力。」"],
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
                label: "上香,再把神像扶正",
                cost: Cost::Gold(20),
                chance: 1.0,
                success: Outcome {
                    lines: &["香烟笔直升起。夜里赶路,你总觉得脚下的石子都被人悄悄捡走了。"],
                    effect: Effect::GainDef(2),
                    daoxin: 1,
                    qingyuan: 0,
                },
                failure: None,
            },
            EventOption {
                label: "搜刮功德箱",
                cost: Cost::Free,
                chance: 0.4,
                success: Outcome {
                    lines: &["箱底果然有前人留下的香火钱。你揣进怀里,总觉得神像在看你。"],
                    effect: Effect::GainGold(70),
                    daoxin: -1,
                    qingyuan: 0,
                },
                failure: Some(Outcome {
                    lines: &["功德箱「啪」地夹住你的手!挣脱时撞翻了半个供桌。"],
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
                cost: Cost::Free,
                chance: 1.0,
                success: Outcome {
                    lines: &["清泉入腹,伤处发痒——那是愈合的征兆。"],
                    effect: Effect::HealPct(35),
                    daoxin: 0,
                    qingyuan: 0,
                },
                failure: None,
            },
            EventOption {
                label: "灌满所有水囊",
                cost: Cost::Free,
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
                    lines: &["泉水在囊中化作黑雾反噬!你丢下水囊仓皇而退。"],
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
                cost: Cost::Free,
                chance: 0.55,
                success: Outcome {
                    lines: &["邪路无事,反倒在路边捡到前人失落的包袱。"],
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
                cost: Cost::Free,
                chance: 1.0,
                success: Outcome {
                    lines: &["一夜太平。老镖头临别塞给你一枚护心木牌:「江湖再见。」"],
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
                cost: Cost::Gold(10),
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
                cost: Cost::Free,
                chance: 0.55,
                success: Outcome {
                    lines: &["老蛟兴起,浪里三沉三浮,把你甩上对岸时喝彩:「好胆色!赏!」"],
                    effect: Effect::GainGold(55),
                    daoxin: 0,
                    qingyuan: 0,
                },
                failure: Some(Outcome {
                    lines: &["半途一个浪头把你拍进河里,呛得七荤八素,老蛟笑得亭子直晃。"],
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
                label: "诚心求一签",
                cost: Cost::Gold(10),
                chance: 0.6,
                success: Outcome {
                    lines: &["上上签:「剑下留情,身后有灯。」老猴儿难得睁眼:「好签,沾点仙气去。」"],
                    effect: Effect::GainMaxMp(6),
                    daoxin: 1,
                    qingyuan: 0,
                },
                failure: Some(Outcome {
                    lines: &["下下签:「前路见血。」老猴儿把十文钱退给你一半:「签凶,半价。」"],
                    effect: Effect::GainGold(5),
                    daoxin: 0,
                    qingyuan: 0,
                }),
            },
            EventOption {
                label: "不求签,给山神像掸掸灰",
                cost: Cost::Free,
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
                label: "将信将疑买一粒",
                cost: Cost::Gold(30),
                chance: 0.35,
                success: Outcome {
                    lines: &["竟真有药力!暖流走遍四肢百骸——虽然多半没有十年,三五天是有的。"],
                    effect: Effect::GainMaxHp(12),
                    daoxin: 0,
                    qingyuan: 0,
                },
                failure: Some(Outcome {
                    lines: &["……就是糖炒山楂。还挺好吃。"],
                    effect: Effect::None,
                    daoxin: 0,
                    qingyuan: 0,
                }),
            },
            EventOption {
                label: "拆穿他:「山楂就说山楂。」",
                cost: Cost::Free,
                chance: 1.0,
                success: Outcome {
                    lines: &["小贩讪讪一笑,改口吆喝「祖传山楂丸」,生意反而好了,硬塞你两串谢礼。"],
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
                cost: Cost::Free,
                chance: 1.0,
                success: Outcome {
                    lines: &["书生千恩万谢,临别写了一幅字相赠:「侠之大者」。字真不错。"],
                    effect: Effect::None,
                    daoxin: 1,
                    qingyuan: 1,
                },
                failure: None,
            },
            EventOption {
                label: "指个方向,顺便请教兵法韬略",
                cost: Cost::Free,
                chance: 0.7,
                success: Outcome {
                    lines: &["书生眼睛一亮,就着树影讲了半个时辰的攻守之道,竟颇有见地。"],
                    effect: Effect::GainDef(2),
                    daoxin: 0,
                    qingyuan: 0,
                },
                failure: Some(Outcome {
                    lines: &["书生讲得兴起,从孙子讲到棋谱,天黑了才放你走。"],
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
                cost: Cost::Free,
                chance: 1.0,
                success: Outcome {
                    lines: &["一夜无梦。晨起宅子已化雾散去,你躺在干爽的草坡上,怀里多了个食盒。"],
                    effect: Effect::HealPct(30),
                    daoxin: 0,
                    qingyuan: 0,
                },
                failure: None,
            },
            EventOption {
                label: "半夜偏要回头看一眼",
                cost: Cost::Free,
                chance: 0.5,
                success: Outcome {
                    lines: &[
                        "厨房里,九条雪白的尾巴正在颠勺。狐妇人叹气:「罢了,看都看了,搭把手。」",
                        "你帮忙烧了半夜火,学了一手药膳。",
                    ],
                    effect: Effect::GainMaxHp(10),
                    daoxin: 0,
                    qingyuan: 1,
                },
                failure: Some(Outcome {
                    lines: &["「说了莫回头。」宅子连人带床把你请了出去,暴雨浇了个透心凉。"],
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
                cost: Cost::HealthPct(12),
                chance: 1.0,
                success: Outcome {
                    lines: &["三滴血入水,剑锋映出的溪光都锐利了几分。"],
                    effect: Effect::GainAtk(2),
                    daoxin: 0,
                    qingyuan: 0,
                },
                failure: None,
            },
            EventOption {
                label: "凿一块石芯带走",
                cost: Cost::Free,
                chance: 0.45,
                success: Outcome {
                    lines: &["石芯温润如玉,贴身藏好,行路时总觉得筋骨沉稳。"],
                    effect: Effect::GainMaxHp(14),
                    daoxin: -1,
                    qingyuan: 0,
                },
                failure: Some(Outcome {
                    lines: &["凿到一半,溪水陡涨,把你冲出十几丈远,石屑崩了满脸。"],
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
                cost: Cost::Free,
                chance: 0.6,
                success: Outcome {
                    lines: &["得手!岩蜜入口,伤处的钝痛竟一点点化开了。"],
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
                cost: Cost::Free,
                chance: 1.0,
                success: Outcome {
                    lines: &["石缝里的落蜜混着草屑,味道差些,胜在安稳。"],
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
                cost: Cost::Free,
                chance: 0.5,
                success: Outcome {
                    lines: &["枯藤晃而未断!灵芝到手,药香沁脾,只嗅一嗅便觉神清气足。"],
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
                cost: Cost::Free,
                chance: 0.4,
                success: Outcome {
                    lines: &["一道剑气恰到好处,芝盖打着旋儿落进你掌心——这一手连你自己都想喝彩。"],
                    effect: Effect::HealPct(20),
                    daoxin: 1,
                    qingyuan: 0,
                },
                failure: Some(Outcome {
                    lines: &["剑气偏了半寸,灵芝连石屑一起崩进深谷,悔得你直跺脚。"],
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
                cost: Cost::Free,
                chance: 1.0,
                success: Outcome {
                    lines: &[
                        "三座小冢立起时,荒草间的阴风忽然停了。",
                        "夜里你梦见一队卸甲的兵,朝你抱拳而去。",
                    ],
                    effect: Effect::GainDef(1),
                    daoxin: 2,
                    qingyuan: 0,
                },
                failure: None,
            },
            EventOption {
                label: "翻检遗物,寻些还能用的",
                cost: Cost::Free,
                chance: 0.65,
                success: Outcome {
                    lines: &[
                        "锈甲下压着一只未曾腐烂的军囊,里头是半袋军饷。",
                        "你把红绳断刀留在了原地。",
                    ],
                    effect: Effect::GainGold(65),
                    daoxin: -1,
                    qingyuan: 0,
                },
                failure: Some(Outcome {
                    lines: &["指尖刚触到军囊,锈甲哗啦一声塌了,惊起满地磷火燎了你一下。"],
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
                cost: Cost::Free,
                chance: 0.6,
                success: Outcome {
                    lines: &["琴音尽头是一场山民的婚宴,你被拉着喝了三碗喜酒,满袖都是花瓣。"],
                    effect: Effect::HealPct(25),
                    daoxin: 0,
                    qingyuan: 1,
                },
                failure: Some(Outcome {
                    lines: &["琴音是山魈学的!它笑着敲了你一闷棍,抢走了几个铜板。"],
                    effect: Effect::LoseGold(25),
                    daoxin: 0,
                    qingyuan: 0,
                }),
            },
            EventOption {
                label: "循哀婉的琴音走右路",
                cost: Cost::Free,
                chance: 0.6,
                success: Outcome {
                    lines: &["琴音来自一位守墓的老琴师,他见有人肯听完整曲,赠了你一瓶祭余的药酒。"],
                    effect: Effect::GainPotions(1),
                    daoxin: 1,
                    qingyuan: 0,
                },
                failure: Some(Outcome {
                    lines: &["曲子越听越冷,回过神来已站在断崖边缘,惊出一身冷汗,急急退回岔口。"],
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
                cost: Cost::Free,
                chance: 0.7,
                success: Outcome {
                    lines: &[
                        "赏金客们一哄而上,又在你按剑的目光里一哄而散。",
                        "亭角躲雨的狸妖朝你深深一揖,留下一小袋谢礼。",
                    ],
                    effect: Effect::GainGold(40),
                    daoxin: 1,
                    qingyuan: 0,
                },
                failure: Some(Outcome {
                    lines: &["赏金客里有硬茬,一场混战虽赶跑了他们,你也挂了彩。"],
                    effect: Effect::DamagePct(15),
                    daoxin: 1,
                    qingyuan: 0,
                }),
            },
            EventOption {
                label: "抄下榜文线索,自去查证真伪",
                cost: Cost::Free,
                chance: 1.0,
                success: Outcome {
                    lines: &[
                        "按图索骥,所谓「妖巢」不过是个山洞药圃。你在榜文背面写下「查无此妖」四字寄回官府。",
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
                cost: Cost::Free,
                chance: 0.75,
                success: Outcome {
                    lines: &[
                        "白虎与你对峙片刻,竟衔起树下一只被夹伤的虎崽,一瘸一拐地走了。",
                        "樵夫下树后面红耳赤:「那夹子……是俺下的。」",
                    ],
                    effect: Effect::GainGold(30),
                    daoxin: 1,
                    qingyuan: 0,
                },
                failure: Some(Outcome {
                    lines: &["白虎虚晃一爪,劲风扫过肩头,火辣辣地疼——但它终究只是护崽,并未下死手。"],
                    effect: Effect::DamagePct(10),
                    daoxin: 0,
                    qingyuan: 0,
                }),
            },
            EventOption {
                label: "先看清再动手——顺着虎的目光找",
                cost: Cost::Free,
                chance: 1.0,
                success: Outcome {
                    lines: &[
                        "你在草丛里找到被兽夹夹住的虎崽,撬开铁夹。白虎伏低了头,樵夫在树上大气不敢出。",
                        "母虎衔崽离去前,回头长啸一声,山鸣谷应。",
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
                cost: Cost::Potion(1),
                chance: 1.0,
                success: Outcome {
                    lines: &["白狐舔了舔你的手背,一瘸一拐地隐入林中,回头望了你三次。"],
                    effect: Effect::None,
                    daoxin: 0,
                    qingyuan: 1,
                },
                failure: None,
            },
            EventOption {
                label: "山中妖物,多一事不如少一事",
                cost: Cost::Free,
                chance: 1.0,
                success: Outcome {
                    lines: &["你按剑绕开。身后呜咽渐弱,琥珀色的目光落在你背上,很凉。"],
                    effect: Effect::None,
                    daoxin: 1,
                    qingyuan: 0,
                },
                failure: None,
            },
        ],
    },
    // 23 EV_FOX2_WARM · 白衣回礼(苏州起,救过)
    RandomEvent {
        title: "白衣回礼",
        lines: &[
            "雨夜,一位白衣少女撑伞而来,伞沿一抬,眉眼弯弯:",
            "「恩公,山中兽夹之恩,阿狸记得。」她指尖递来一枚暖玉。",
        ],
        options: &[
            EventOption {
                label: "收下暖玉",
                cost: Cost::Free,
                chance: 1.0,
                success: Outcome {
                    lines: &[
                        "暖玉贴身,寒雨不侵,伤处竟自愈合。",
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
                cost: Cost::Free,
                chance: 1.0,
                success: Outcome {
                    lines: &[
                        "少女怔了怔,轻声道:「人间竟真有这样的人。」",
                        "她把伞留给了你,自己消失在雨里。",
                    ],
                    effect: Effect::None,
                    daoxin: 0,
                    qingyuan: 2,
                },
                failure: None,
            },
        ],
    },
    // 24 EV_FOX2_COLD · 白影避走(苏州起,未救)
    RandomEvent {
        title: "白影避走",
        lines: &[
            "巷口白影一闪——那只瘸腿的白狐。它认出了你,",
            "耳朵倏地压平,警惕地退进雨幕,像在说:人心,凉薄。",
        ],
        options: &[
            EventOption {
                label: "追上去,放下一瓶药水",
                cost: Cost::Potion(1),
                chance: 1.0,
                success: Outcome {
                    lines: &["白狐迟疑良久,终于叼走了药水。琥珀色的眼睛柔和了一瞬。"],
                    effect: Effect::None,
                    daoxin: 0,
                    qingyuan: 1,
                },
                failure: None,
            },
            EventOption {
                label: "由它去",
                cost: Cost::Free,
                chance: 1.0,
                success: Outcome {
                    lines: &["你收回目光。修行路上,本就不该多牵挂。"],
                    effect: Effect::None,
                    daoxin: 1,
                    qingyuan: 0,
                },
                failure: None,
            },
        ],
    },
    // 25 EV_FOX3_WARM · 狐仙赠丹(南疆起,善缘)
    RandomEvent {
        title: "狐仙护礼",
        lines: &[
            "月华如练。白衣少女立于枝头,身后九条雪尾缓缓展开——",
            "「那瓶药救回的,不只是一条腿。这袋息壤药材能护住你的伤,我还带来一枚虎符。」",
        ],
        options: &[
            EventOption {
                label: "收下息壤袋,让药香护住往后的战斗",
                cost: Cost::Free,
                chance: 1.0,
                success: Outcome {
                    lines: &[
                        "药袋贴身,温暖的药香护住伤处。",
                        "「前路凶险,恩公珍重。」九尾一卷,月下再无狐影。",
                    ],
                    effect: Effect::Relic(Relic::BreathSoil),
                    daoxin: 0,
                    qingyuan: 2,
                },
                failure: None,
            },
            EventOption {
                label: "请她把药材留给伤者,收下护身虎符",
                cost: Cost::Free,
                chance: 1.0,
                success: Outcome {
                    lines: &[
                        "狐仙把息壤袋收回药匣,将护身虎符系在你的剑柄上。",
                        "「那便……以此相护,直到雾散。」",
                    ],
                    effect: Effect::Relic(Relic::TigerTalisman),
                    daoxin: 0,
                    qingyuan: 2,
                },
                failure: None,
            },
        ],
    },
    // 26 EV_FOX3_COLD · 妖狐拦路(南疆起,未和解)
    RandomEvent {
        title: "妖狐拦路",
        lines: &[
            "阴风骤起。一只瘸腿的九尾妖狐拦在路心,妖瞳猩红:",
            "「那天山径一眼,我记到今日。你走过许多地方,仍不肯回头么?」",
        ],
        options: &[
            EventOption {
                label: "拔剑硬闯",
                cost: Cost::Free,
                chance: 0.6,
                success: Outcome {
                    lines: &[
                        "你剑意如虹,妖狐纠缠一阵,恨恨遁去。",
                        "「修剑先修心——望你早日明白。」风里留下这句。",
                    ],
                    effect: Effect::None,
                    daoxin: 1,
                    qingyuan: 0,
                },
                failure: Some(Outcome {
                    lines: &["妖风撕开护体剑气,你且战且退。"],
                    effect: Effect::DamagePct(15),
                    daoxin: 0,
                    qingyuan: 0,
                }),
            },
            EventOption {
                label: "散财消灾",
                cost: Cost::Gold(40),
                chance: 1.0,
                success: Outcome {
                    lines: &["妖狐嗤笑一声卷走钱袋:「俗物。」但到底放你过去了。"],
                    effect: Effect::None,
                    daoxin: 0,
                    qingyuan: 0,
                },
                failure: None,
            },
        ],
    },
    // 27 EV_QIN1 · 盲女琴师(苏州至京城)
    RandomEvent {
        title: "盲女琴师",
        lines: &[
            "城外渡口,盲女琴师苏弦怀抱旧琴,指下流出的调子哀而不伤,",
            "竟压住了岸边游荡的阴气。「客人留步——可愿听完这一曲?」",
        ],
        options: &[
            EventOption {
                label: "赠银听曲",
                cost: Cost::Gold(20),
                chance: 1.0,
                success: Outcome {
                    lines: &[
                        "曲罢,你只觉灵台清明,经脉里多了一缕琴韵。",
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
                cost: Cost::Free,
                chance: 1.0,
                success: Outcome {
                    lines: &["一曲终了,少女朝你的方向浅浅一礼:「心静之人,曲子听得最全。」"],
                    effect: Effect::None,
                    daoxin: 1,
                    qingyuan: 0,
                },
                failure: None,
            },
        ],
    },
    // 28 EV_QIN2 · 旧曲重逢(心渊外,已听过琴)
    RandomEvent {
        title: "旧曲重逢",
        lines: &[
            "心渊外的避雨棚中,忽有琴声穿夜而来——正是渡口那支曲子。",
            "琴师跟着难民一路南行,如今为守门人奏曲:「再闻此曲,便是重逢。」",
            "她接上旧曲的下半阕:「这首叫《雾散》。无论你是否停留,愿你们走到它应验的那一天。」",
        ],
        options: &[
            EventOption {
                label: "「一路风雨,姑娘别来无恙。」",
                cost: Cost::Free,
                chance: 1.0,
                success: Outcome {
                    lines: &[
                        "她这次奏的是新曲,曲中有山、有雾、有一个执剑北行的人。",
                        "「曲名《雾散》。愿它应验。」",
                    ],
                    effect: Effect::GainMaxMp(6),
                    daoxin: 0,
                    qingyuan: 1,
                },
                failure: None,
            },
            EventOption {
                label: "驻足听完,悄然离去",
                cost: Cost::Free,
                chance: 1.0,
                success: Outcome {
                    lines: &["琴声送你到街尾。有些重逢,不必开口。"],
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
pub fn boss_taunt(
    boss: super::super::quest::BossKind,
    heart_over_resolve: bool,
) -> (&'static str, &'static [&'static str]) {
    use super::super::quest::BossKind;
    match boss {
        BossKind::MountainFiend => (
            "决战 · 赤鬼山妖",
            &[
                "破庙石阶尽头,赤色妖影拖着断裂铜环一步步踏出火光。",
                "「小村来的剑童,也敢追到本王山门前?」",
                "灵儿握紧桃木铃:「它身上的妖火,就是夜里惊动村口的源头。」",
                "你提剑挡在众人前:「那就从这里开始,把归路烧干净。」",
            ],
        ),
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
        BossKind::DreamEclipse if heart_over_resolve => (
            "决战 · 宿命水影",
            &[
                "灵渊尽头没有妖,只有一泓水——水里映着你来时的每一步。",
                "水面轻晃,浮出的竟是灵儿的眉眼:「逍遥哥哥,留下来陪我,好不好?」",
                "腕上的桃木铃轻轻响了一声——真正的她,还在雾外等你。",
                "「你不是她。」你握紧剑柄,「让开。」",
            ],
        ),
        BossKind::DreamEclipse => (
            "决战 · 宿命水影",
            &[
                "灵渊尽头没有妖,只有一泓水——水里映着你来时的每一步。",
                "水影抬头,眉眼竟与你一模一样,手中之剑也分毫不差。",
                "「你我同源,你的道,不过是我做过的一场梦。」",
                "你伸手握住剑柄:「那就连这场梦,一起斩了。」",
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
        23 | 25 => Some("npcs/ai_fox_spirit.png"),
        27 | 28 => Some("npcs/ai_cave_priestess.png"),
        21 => Some("npcs/ai_tribal_chief.png"), // 虎口樵夫
        _ => None,
    }
}

/// 灵儿的立绘卡(剧情缘节点、歇脚谈心)。
pub const PORTRAIT_LINGER: &str = "npcs/ai_linger.png";
pub const PORTRAIT_SWORD_SISTER: &str = "npcs/ai_sword_sister.png";
pub const PORTRAIT_SPIRIT_WITCH: &str = "npcs/ai_spirit_guide.png";
pub const PORTRAIT_MERCHANT: &str = "npcs/ai_wandering_merchant.png";

pub fn story_portrait(chapter: usize, roll: usize) -> &'static str {
    match (chapter, roll) {
        (5, 1) => PORTRAIT_SPIRIT_WITCH,
        (6, 5) => PORTRAIT_SWORD_SISTER,
        _ => PORTRAIT_LINGER,
    }
}

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
    "背风的岩窝里升起一小堆篝火,你把干粮与守夜的行囊摆在火边。",
    "难得的喘息。歇脚之后,前路还长。",
];

// ---------------------------------------------------------------------------
// Endings
// ---------------------------------------------------------------------------

pub const ENDING_LOVE: &[&str] = &[
    "【结局 · 铃归渡口】",
    "宿命水影溃散时,你没有去看它沉没,而是转身向山下跑。",
    "渡口的灯还亮着。你与灵儿并肩走到灯下,腕上的桃木铃应着江风轻响——",
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
    "你与灵儿并肩走回渡口:一手牵她,一手仗剑,再难的路也不必独走。",
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
