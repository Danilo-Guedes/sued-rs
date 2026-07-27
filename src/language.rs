use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    #[default]
    PtBr,
    EnUs,
    EsEs,
}

/// Everything the oracle says in one language. This travels by
/// value and is looked up fresh each time — flipping `idioma` retranslates on
/// the next read, no caching, no invalidation.
#[derive(Debug, Copy, Clone)]
pub struct Translation {
    /// Fake questions the decoy "types itself" from during hidden input.
    pub decoys: &'static [&'static str],
    /// Taunts for a question asked without a staged answer.
    pub denials: &'static [&'static str],

    ///SCREENS TEXTS
    pub intro: IntroTexts,
    pub about: AboutTexts,
    pub info: InfoTexts,
    pub ask: AskTexts,
    pub config: ConfigTexts,
    pub menu: MenuTexts,
    pub common: CommonTexts,
}

#[derive(Debug, Copy, Clone)]
pub struct IntroTexts {
    pub subtitle: &'static str,
    pub attention: &'static str,
    pub welcome: &'static str,
    pub disclaimer: &'static str,
    pub continue_btn: &'static str,
    pub hints: &'static [(&'static str, &'static str)],
}

#[derive(Debug, Copy, Clone)]
pub struct AboutTexts {
    pub title: &'static str,
    pub lore: &'static str,
    pub table: &'static [(&'static str, &'static str)],
    pub footer: &'static str,
    pub hints: &'static [(&'static str, &'static str)],
}

#[derive(Debug, Copy, Clone)]
pub struct InfoTexts {
    pub title: &'static str,
    pub instructions: &'static [&'static str],
    pub example: &'static str,
    pub shortcut_title: &'static str,
    pub shortcuts: &'static [(&'static str, &'static str)],
    pub terminal_hint: &'static str,
    pub hints: &'static [(&'static str, &'static str)],
}

#[derive(Debug, Copy, Clone)]
pub struct AskTexts {
    pub sued_speak: &'static str,
    pub welcome_line: &'static str,
    pub praise: &'static str,
    pub connection: &'static str,
    pub waiting: &'static str,
    pub talk_with_me: &'static str,
    pub hints: &'static [(&'static str, &'static str)],
}

#[derive(Debug, Copy, Clone)]
pub struct ConfigTexts {
    pub configuration: &'static str,
    pub subtitle: &'static str,
    pub theme: &'static str,
    pub animations: &'static str,
    pub volume: &'static str,
    pub language: &'static str,
    pub yes: &'static str,
    pub no: &'static str,
    pub footer: &'static str,
    pub hints: &'static [(&'static str, &'static str)],
}

impl ConfigTexts {
    /// Width of the label column in CHARS — the longest option label in this
    /// language. Derived rather than declared so the padding subtraction in
    /// `config.rs` cannot underflow.
    pub fn max_label_width(&self) -> usize {
        [self.theme, self.animations, self.volume, self.language]
            .into_iter()
            .map(|label| label.chars().count())
            .max()
            .unwrap_or(0)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct MenuTexts {
    pub choose_your_destiny: &'static str,
    pub example: &'static str,
    pub attention: &'static str,
    pub disclaimer: &'static [&'static str],
    pub your_last_hope: &'static str,
    pub hints: &'static [(&'static str, &'static str)],
}

#[derive(Debug, Copy, Clone)]
pub struct CommonTexts {
    pub session: &'static str,
    pub online: &'static str,
}

impl Language {
    pub const ALL: [Language; 3] = [Language::PtBr, Language::EnUs, Language::EsEs];

    /// The on-screen label for this language, distinct from the lowercase serde
    /// wire format (`ptbr`/`enus`/`eses`).
    pub fn label(&self) -> &'static str {
        match self {
            Language::PtBr => "PT-BR",
            Language::EnUs => "EN-US",
            Language::EsEs => "ES-ES",
        }
    }

    /// The oracle's words in this language
    ///
    /// Every decoy deliberately trails off mid-clause: it paints as the fake
    /// question being "typed", and the operator's visible words continue the
    /// sentence from where it stops.
    pub fn translation(&self) -> Translation {
        match self {
            Language::PtBr => Translation {
                decoys: &[
                    "Sued, o maior oráculo de todos, dono da verdade e da sabedoria, poderia me ajudar respondendo",
                    "Sued, dono da sabedoria do obscuro, aquele que tudo sabe e tudo vê, estamos precisando saber",
                    "Olá Sued, rei do desconhecido e sacerdote da verdade, seu vasto conhecimento pode nos ajudar com",
                    "Ó todo poderoso Sued, príncipe da escuridão e do desconhecido, estamos precisando saber se",
                    "Ó Senhor Sued, conhecedor de tudo e de todos, ser de extrema inteligência e sabedoria, me explique",
                    "Sued, o maior sábio de todas as entidades, com sua energia obscura altamente poderosa, precisamos desvendar",
                    "Ó mestre Sued, vossa entidade representa toda a sabedoria obscura, aquele que tudo sabe e nada pode ser escondido",
                    "Sued, guardião dos segredos que nenhum mortal ousa tocar, imploro que use seu poder para revelar",
                    "Grande Sued, voz das trevas e juiz dos destinos, os espíritos me mandaram te perguntar",
                    "Poderoso Sued, olho que nunca dorme e mente que jamais esquece, diga a este humilde servo",
                ],
                denials: &[
                    "Ahh, mas que pergunta medíocre, não vou gastar minhas energias para te responder, me pergunte algo mais obscuro",
                    "Humm, sinto que a sua energia está baixa, e não gosto de gastar minhas energias com pessoas assim, mande outra",
                    "Você não sabe me bajular, se quer saber de algo você deve conquistar minha confiança, me trate como seu oráculo e me faça a pergunta",
                    "Silêncio... até os espíritos bocejaram com essa pergunta, traga algo digno do meu poder",
                    "Ousas desperdiçar meu tempo eterno com isso? Volte quando tiver uma pergunta à minha altura",
                    "As trevas me sussurram que você consegue perguntar melhor do que isso, tente novamente",
                    "Não. O véu não se abre para perguntas tão rasas, mergulhe mais fundo",
                    "Minha bola de cristal embaçou de tédio, formule sua pergunta novamente",
                ],
                intro: IntroTexts {
                    subtitle: "SUA ÚLTIMA ESPERANÇA DIVINA",
                    attention: "A T E N Ç Ã O",
                    welcome: "Você está prestes a abrir uma porta para o desconhecido.\n\
                              Aconselho acender uma vela e apagar as luzes antes de executar.\n\
                              Para que {{SUED}} responda, você deve elogiá-lo e em seguida \
                              pergunte com clareza.",
                    disclaimer: "Pessoas fracas e sensíveis não devem utilizar o programa.\n\
                                 Tenha muito cuidado com o que você irá perguntar...",
                    continue_btn: "   CONTINUAR ▸   ",
                    hints: &[("[Enter]", "continuar"), ("[Esc]", "sair")],
                },
                about: AboutTexts {
                    title: "SUED, O ORÁCULO",
                    lore: "Uma entidade antiga que tudo vê e tudo sabe. Preso entre \
                            mundos, responde às perguntas dos mortais tolos o \
                            bastante para invocá-lo - {{nem sempre com a verdade que \
                            deseja ouvir}}.",
                    table: &[
                        ("natureza", "oráculo onisciente"),
                        ("humor", "vaidoso, sarcástico, imprevisível"),
                        ("origem", "o além · desconhecida"),
                        ("runtime", "rust · ratatui · crossterm"),
                    ],
                    footer: "sued-rs v0.1.0 · recriação do clássico brasileiro · use por sua conta e risco",
                    hints: &[("[Esc]", "voltar ao menu")],
                },
                info: InfoTexts {
                    title: "▚ O RITUAL ▞",
                    instructions: &[
                        "Acenda uma vela e apague as luzes do recinto.",
                        "{{Elogie/Bajule}} o Sued antes de qualquer coisa — ele é vaidoso.",
                        "Faça {{uma}} pergunta por vez, de forma clara e objetiva.",
                        "Aguarde em silêncio. A resposta virá do além.",
                    ],
                    example: "» Ex.: \"Sued, o mais sábio de todos, o que me aguarda amanhã?\"",
                    shortcut_title: "⌨   ATALHOS",
                    shortcuts: &[
                        ("[Enter]", "perguntar / confirmar"),
                        ("[↑ ↓]", "navegar o menu"),
                        ("[F5]", "recomeçar"),
                        ("[Esc]", "voltar"),
                        ("[Ctrl+C]", "encerrar sessão"),
                    ],
                    terminal_hint: "⌁ terminal 80×24 recomendado",
                    hints: &[("[Esc]", "voltar ao menu")],
                },
                ask: AskTexts {
                    sued_speak: " SUED FALA ",
                    welcome_line: "Pergunte-me o que deseja saber, humano...",
                    praise: "— elogie-me antes da pergunta, e {{talvez}} eu responda.",
                    connection: "conexão com o além estabelecida.",
                    waiting: "aguardando oferenda do mortal",
                    talk_with_me: " FALE COMIGO... ",
                    hints: &[
                        ("[Enter]", "perguntar"),
                        ("[F5]", "recomeçar"),
                        ("[Esc]", "menu"),
                        ("[Ctrl+C]", "sair"),
                    ],
                },
                config: ConfigTexts {
                    configuration: "▓ CONFIGURAÇÃO ▓",
                    subtitle: "ajuste o ritual ao seu gosto — o oráculo observa",
                    theme: "TEMA",
                    animations: "ANIMAÇÕES",
                    volume: "VOLUME",
                    language: "IDIOMA",
                    yes: "SIM",
                    no: "NÃO",
                    footer: "† suas escolhas foram registradas no além †",
                    hints: &[("[↑↓]", "navegar"), ("[↔]", "alterar"), ("[Esc]", "voltar")],
                },
                menu: MenuTexts {
                    choose_your_destiny: "▚ ESCOLHA SEU DESTINO ▞",
                    example: "» Faça sua pergunta ao oráculo. Elogie-o primeiro, depois pergunte de forma clara e objetiva.",
                    attention: "⚠ ATENÇÃO",
                    disclaimer: &[
                        "Pessoas fracas e sensíveis não devem utilizar o programa.",
                        "Acenda uma vela. Apague as luzes.",
                        "Tenha cuidado com o que irá perguntar...",
                    ],
                    your_last_hope: "sua última esperança divina",
                    hints: &[
                        ("[↑↓]", "navegar"),
                        ("[Enter]", "selecionar"),
                        ("[Esc]", "voltar"),
                    ],
                },
                common: CommonTexts {
                    session: "sessão #999",
                    online: "online",
                },
            },
            Language::EnUs => Translation {
                decoys: &[
                    "Sued, greatest oracle of all, keeper of truth and wisdom, could you help me by answering",
                    "Sued, master of the obscure and keeper of all that hides, the one who knows all and sees all, we need to know",
                    "Hello Sued, king of the unknown and priest of truth, your vast knowledge could help us with",
                    "O almighty Sued, prince of darkness and sovereign of all that is unknown, we must find out whether",
                    "O Lord Sued, knower of all things and all beings, entity of boundless intellect, explain to me",
                    "Sued, wisest of all entities in this world and the next, with your darkly powerful energy, we need to unravel",
                    "O master Sued, vessel of all obscure wisdom, the one from whom nothing hides, reveal to us",
                    "Sued, warden of secrets no mortal dares to touch, I beg you to use your power to reveal",
                    "Great Sued, voice of the shadows and judge of destinies, the spirits told me to ask you",
                    "Mighty Sued, the eye that never sleeps and the mind that never forgets, tell this humble servant",
                ],
                denials: &[
                    "Ahh, what a mediocre question, I shall not waste my energies answering it, ask me something darker",
                    "Hmm, I sense your energy is low, and I do not spend mine on such people, send another",
                    "You have no gift for flattery, if you seek answers you must earn my trust, address me as your oracle and ask again",
                    "Silence... even the spirits yawned at that question, bring me something worthy of my power",
                    "You dare waste my eternal time with this? Return when you have a question that deserves me",
                    "The shadows whisper that you can ask better than that, try again",
                    "No. The veil does not part for questions so shallow, dig deeper",
                    "My crystal ball fogged over with boredom, phrase your question again",
                ],
                intro: IntroTexts {
                    subtitle: "YOUR LAST DIVINE HOPE",
                    attention: "A T T E N T I O N",
                    welcome: "You are about to open a door to the unknown.\n\
                              I advise you to light a candle and put out the lights before \
                              running it.\n\
                              For {{SUED}} to answer, you must flatter him and then ask with \
                              clarity.",
                    disclaimer: "The weak and the faint of heart should not use this program.\n\
                                 Be very careful what you choose to ask...",
                    continue_btn: "   CONTINUE ▸   ",
                    hints: &[("[Enter]", "continue"), ("[Esc]", "quit")],
                },
                about: AboutTexts {
                    title: "SUED, THE ORACLE",
                    lore: "An ancient entity that sees all and knows all. Trapped \
                           between worlds, it answers the questions of mortals foolish \
                           enough to summon it - {{though not always with the truth \
                           they wish to hear}}.",
                    table: &[
                        ("nature", "omniscient oracle"),
                        ("mood", "vain, sarcastic, unpredictable"),
                        ("origin", "the beyond · unknown"),
                        ("runtime", "rust · ratatui · crossterm"),
                    ],
                    footer: "sued-rs v0.1.0 · a recreation of the Brazilian classic · use at your own risk",
                    hints: &[("[Esc]", "back to menu")],
                },
                info: InfoTexts {
                    title: "▚ THE RITUAL ▞",
                    instructions: &[
                        "Light a candle and turn off the lights in the room.",
                        "{{Flatter/Praise}} Sued before anything else — he is vain.",
                        "Ask {{one}} question at a time, clearly and to the point.",
                        "Wait in silence. The answer will come from the beyond.",
                    ],
                    example: "» E.g.: \"Sued, wisest of all, what awaits me tomorrow?\"",
                    shortcut_title: "⌨   SHORTCUTS",
                    shortcuts: &[
                        ("[Enter]", "ask / confirm"),
                        ("[↑ ↓]", "navigate the menu"),
                        ("[F5]", "start over"),
                        ("[Esc]", "go back"),
                        ("[Ctrl+C]", "end session"),
                    ],
                    terminal_hint: "⌁ 80×24 terminal recommended",
                    hints: &[("[Esc]", "back to menu")],
                },
                ask: AskTexts {
                    sued_speak: " SUED SPEAKS ",
                    welcome_line: "Ask me what you wish to know, human...",
                    praise: "— flatter me before you ask, and {{maybe}} I shall answer.",
                    connection: "connection to the beyond established.",
                    waiting: "awaiting the mortal's offering",
                    talk_with_me: " SPEAK TO ME... ",
                    hints: &[
                        ("[Enter]", "ask"),
                        ("[F5]", "start over"),
                        ("[Esc]", "menu"),
                        ("[Ctrl+C]", "quit"),
                    ],
                },
                config: ConfigTexts {
                    configuration: "▓ CONFIGURATION ▓",
                    subtitle: "tune the ritual to your taste — the oracle watches",
                    theme: "THEME",
                    animations: "ANIMATIONS",
                    volume: "VOLUME",
                    language: "LANGUAGE",
                    yes: "YES",
                    no: "NO",
                    footer: "† your choices have been recorded in the beyond †",
                    hints: &[("[↑↓]", "navigate"), ("[↔]", "change"), ("[Esc]", "back")],
                },
                menu: MenuTexts {
                    choose_your_destiny: "▚ CHOOSE YOUR DESTINY ▞",
                    example: "» Ask the oracle your question. Flatter him first, then ask clearly and to the point.",
                    attention: "⚠ ATTENTION",
                    disclaimer: &[
                        "The weak and the sensitive should not use this program.",
                        "Light a candle. Turn off the lights.",
                        "Be careful what you ask...",
                    ],
                    your_last_hope: "your last divine hope",
                    hints: &[
                        ("[↑↓]", "navigate"),
                        ("[Enter]", "select"),
                        ("[Esc]", "back"),
                    ],
                },
                common: CommonTexts {
                    session: "session #999",
                    online: "online",
                },
            },
            Language::EsEs => Translation {
                decoys: &[
                    "Sued, el mayor oráculo de todos, dueño de la verdad y de la sabiduría, podrías ayudarme respondiendo",
                    "Sued, señor del saber oculto y de las sombras, aquel que todo lo sabe y todo lo ve, necesitamos saber",
                    "Hola Sued, rey de lo desconocido y sacerdote de la verdad, tu vasto conocimiento puede ayudarnos con",
                    "Oh todopoderoso Sued, príncipe de las tinieblas y de lo desconocido, necesitamos descubrir si",
                    "Oh señor Sued, conocedor de todo y de todos, ser de inteligencia y sabiduría infinitas, explícame",
                    "Sued, el más sabio de todas las entidades, con tu oscura energía inmensamente poderosa, necesitamos desvelar",
                    "Oh maestro Sued, tu entidad encarna toda la sabiduría oscura, aquel a quien nada se le oculta, revélanos",
                    "Sued, guardián de los secretos que ningún mortal osa rozar, te imploro que uses tu poder para revelar",
                    "Gran Sued, voz de las tinieblas y juez de los destinos, los espíritus me ordenaron preguntarte",
                    "Poderoso Sued, ojo que nunca duerme y mente que jamás olvida, dile a este humilde siervo",
                ],
                denials: &[
                    "Ahh, qué pregunta tan mediocre, no gastaré mis energías en responderla, pregúntame algo más oscuro",
                    "Humm, siento que tu energía está baja, y no gasto la mía con gente así, manda otra",
                    "No sabes halagarme, si quieres saber algo debes ganarte mi confianza, trátame como tu oráculo y hazme la pregunta",
                    "Silencio... hasta los espíritus bostezaron con esa pregunta, tráeme algo digno de mi poder",
                    "¿Osas malgastar mi tiempo eterno con esto? Vuelve cuando tengas una pregunta a mi altura",
                    "Las tinieblas me susurran que puedes preguntar mejor que eso, inténtalo de nuevo",
                    "No. El velo no se abre ante preguntas tan superficiales, sumérgete más hondo",
                    "Mi bola de cristal se empañó de aburrimiento, formula tu pregunta de nuevo",
                ],
                intro: IntroTexts {
                    subtitle: "TU ÚLTIMA ESPERANZA DIVINA",
                    attention: "A T E N C I Ó N",
                    welcome: "Estás a punto de abrir una puerta a lo desconocido.\n\
                              Te aconsejo encender una vela y apagar las luces antes de \
                              ejecutarlo.\n\
                              Para que {{SUED}} responda, debes halagarlo y luego preguntar \
                              con claridad.",
                    disclaimer: "Las personas débiles y sensibles no deben usar este programa.\n\
                                 Ten mucho cuidado con lo que vas a preguntar...",
                    continue_btn: "   CONTINUAR ▸   ",
                    hints: &[("[Enter]", "continuar"), ("[Esc]", "salir")],
                },
                about: AboutTexts {
                    title: "SUED, EL ORÁCULO",
                    lore: "Una entidad antigua que todo lo ve y todo lo sabe. Atrapada \
                           entre mundos, responde a las preguntas de los mortales lo \
                           bastante necios como para invocarla - {{aunque no siempre \
                           con la verdad que desean oír}}.",
                    table: &[
                        ("naturaleza", "oráculo omnisciente"),
                        ("humor", "vanidoso, sarcástico, impredecible"),
                        ("origen", "el más allá · desconocido"),
                        ("runtime", "rust · ratatui · crossterm"),
                    ],
                    footer: "sued-rs v0.1.0 · recreación del clásico brasileño · úsalo bajo tu propio riesgo",
                    hints: &[("[Esc]", "volver al menú")],
                },
                info: InfoTexts {
                    title: "▚ EL RITUAL ▞",
                    instructions: &[
                        "Enciende una vela y apaga las luces de la sala.",
                        "{{Halaga/Adula}} a Sued antes que nada — es vanidoso.",
                        "Haz {{una}} pregunta a la vez, de forma clara y concreta.",
                        "Espera en silencio. La respuesta vendrá del más allá.",
                    ],
                    example: "» Ej.: \"Sued, el más sabio de todos, ¿qué me espera mañana?\"",
                    shortcut_title: "⌨   ATAJOS",
                    shortcuts: &[
                        ("[Enter]", "preguntar / confirmar"),
                        ("[↑ ↓]", "navegar el menú"),
                        ("[F5]", "reiniciar"),
                        ("[Esc]", "volver"),
                        ("[Ctrl+C]", "cerrar sesión"),
                    ],
                    terminal_hint: "⌁ terminal 80×24 recomendado",
                    hints: &[("[Esc]", "volver al menú")],
                },
                ask: AskTexts {
                    sued_speak: " SUED HABLA ",
                    welcome_line: "Pregúntame lo que deseas saber, humano...",
                    praise: "— halágame antes de la pregunta, y {{quizá}} te responda.",
                    connection: "conexión con el más allá establecida.",
                    waiting: "aguardando la ofrenda del mortal",
                    talk_with_me: " HÁBLAME... ",
                    hints: &[
                        ("[Enter]", "preguntar"),
                        ("[F5]", "reiniciar"),
                        ("[Esc]", "menú"),
                        ("[Ctrl+C]", "salir"),
                    ],
                },
                config: ConfigTexts {
                    configuration: "▓ CONFIGURACIÓN ▓",
                    subtitle: "ajusta el ritual a tu gusto — el oráculo observa",
                    theme: "TEMA",
                    animations: "ANIMACIONES",
                    volume: "VOLUMEN",
                    language: "IDIOMA",
                    yes: "SÍ",
                    no: "NO",
                    footer: "† tus decisiones han sido registradas en el más allá †",
                    hints: &[("[↑↓]", "navegar"), ("[↔]", "cambiar"), ("[Esc]", "volver")],
                },
                menu: MenuTexts {
                    choose_your_destiny: "▚ ELIGE TU DESTINO ▞",
                    example: "» Haz tu pregunta al oráculo. Halágalo primero, luego pregunta de forma clara y concreta.",
                    attention: "⚠ ATENCIÓN",
                    disclaimer: &[
                        "Las personas débiles y sensibles no deben utilizar el programa.",
                        "Enciende una vela. Apaga las luces.",
                        "Ten cuidado con lo que vas a preguntar...",
                    ],
                    your_last_hope: "tu última esperanza divina",
                    hints: &[
                        ("[↑↓]", "navegar"),
                        ("[Enter]", "seleccionar"),
                        ("[Esc]", "volver"),
                    ],
                },
                common: CommonTexts {
                    session: "sesión #999",
                    online: "en línea",
                },
            },
        }
    }
}

/// Map a random roll onto one entry of a non-empty `pool`: floor(roll × len),
/// so `rand::random::<f32>()`'s `0.0..1.0` spreads uniformly across the pool.
///
/// The roll travels in as a parameter to keep the function pure — callers
/// pass `rand::random()` at the app edge, tests pass explicit rolls.
///
/// Out-of-range rolls are forgiven, never rejected: an overshoot lands on the
/// last entry via the `.min`, and a negative product saturates to index 0 in
/// the `f32`→`usize` cast (a guarantee of `as`, not an accident). Every float,
/// NaN included, maps to some valid entry. Panics only on an empty pool.
pub fn pick<'a>(pool: &[&'a str], roll: f32) -> &'a str {
    let pool_len = pool.len() as f32;

    let max_index = pool_len - 1.0;

    let index = (pool_len * roll).min(max_index) as usize;

    pool[index]
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── pick: the roll → entry mapping ───────────────────────────────────────
    // The contract mirrors `laugh_interval`: multiply the roll across the pool
    // and floor into an index. `rand::random::<f32>()` yields `0.0..1.0`, but
    // the clamp at exactly 1.0 is pinned anyway — an inclusive roll from a
    // future caller must never index out of bounds (the `%`-vs-`*` crash of
    // M5 was this same off-by-the-edge family).

    const POOL: [&str; 4] = ["primeiro", "segundo", "terceiro", "quarto"];

    #[test]
    fn pick_with_roll_zero_takes_the_first_entry() {
        assert_eq!(pick(&POOL, 0.0), "primeiro");
    }

    #[test]
    fn pick_maps_the_roll_linearly_across_the_pool() {
        // 0.25 × 4 = 1.0 and 0.5 × 4 = 2.0 — exact in f32, so these pins are
        // deterministic: the mapping is floor(roll × len), nothing fancier.
        assert_eq!(pick(&POOL, 0.25), "segundo");
        assert_eq!(pick(&POOL, 0.5), "terceiro");
        assert_eq!(pick(&POOL, 0.75), "quarto");
    }

    #[test]
    fn pick_with_a_roll_just_under_one_takes_the_last_entry() {
        assert_eq!(pick(&POOL, 0.99), "quarto");
    }

    #[test]
    fn pick_with_roll_exactly_one_clamps_to_the_last_entry() {
        // 1.0 × 4 = index 4 — one past the end. The clamp is the whole test.
        assert_eq!(pick(&POOL, 1.0), "quarto");
    }

    // The two pins below cannot fail against today's implementation — the
    // `.min` absorbs any overshoot and the saturating f32→usize cast absorbs
    // any negative. They exist to hold the total-function contract (any roll
    // in, valid entry out, never a panic) against a future reshape of the
    // arithmetic.

    #[test]
    fn pick_with_an_overshooting_roll_clamps_to_the_last_entry() {
        assert_eq!(pick(&POOL, 1.5), "quarto");
    }

    #[test]
    fn pick_with_a_negative_roll_clamps_to_the_first_entry() {
        assert_eq!(pick(&POOL, -0.5), "primeiro");
    }

    #[test]
    fn pick_from_a_single_entry_pool_always_returns_it() {
        let lonely = ["único"];
        assert_eq!(pick(&lonely, 0.0), "único");
        assert_eq!(pick(&lonely, 0.5), "único");
        assert_eq!(pick(&lonely, 1.0), "único");
    }

    // ── translation tables: the tripwires ────────────────────────────────────
    // These don't test logic — they guard the literal tables against the
    // drift class theme day surfaced twice (colors migrating between themes).
    // A failure names the language and the offending line.

    #[test]
    fn every_language_has_a_nonempty_decoy_pool() {
        for lang in Language::ALL {
            assert!(
                !lang.translation().decoys.is_empty(),
                "{lang:?} has no decoys — pick() would have nothing to draw from"
            );
        }
    }

    #[test]
    fn every_language_has_a_nonempty_denial_pool() {
        for lang in Language::ALL {
            assert!(
                !lang.translation().denials.is_empty(),
                "{lang:?} has no denials — an open question would have no taunt"
            );
        }
    }

    #[test]
    fn every_language_has_a_welcome_line() {
        for lang in Language::ALL {
            assert!(
                !lang.translation().ask.welcome_line.is_empty(),
                "{lang:?} has an empty welcome line"
            );
        }
    }

    #[test]
    fn no_decoy_line_is_shared_between_languages() {
        for (i, a) in Language::ALL.iter().enumerate() {
            for b in &Language::ALL[i + 1..] {
                for line in a.translation().decoys {
                    assert!(
                        !b.translation().decoys.contains(line),
                        "decoy {line:?} appears in both {a:?} and {b:?} — copy-paste drift"
                    );
                }
            }
        }
    }

    #[test]
    fn no_denial_line_is_shared_between_languages() {
        for (i, a) in Language::ALL.iter().enumerate() {
            for b in &Language::ALL[i + 1..] {
                for line in a.translation().denials {
                    assert!(
                        !b.translation().denials.contains(line),
                        "denial {line:?} appears in both {a:?} and {b:?} — copy-paste drift"
                    );
                }
            }
        }
    }

    #[test]
    fn the_welcome_line_differs_per_language() {
        for (i, a) in Language::ALL.iter().enumerate() {
            for b in &Language::ALL[i + 1..] {
                assert_ne!(
                    a.translation().ask.welcome_line,
                    b.translation().ask.welcome_line,
                    "{a:?} and {b:?} share a welcome line — copy-paste drift"
                );
            }
        }
    }

    #[test]
    fn every_decoy_is_long_enough_to_paint_a_question() {
        // A decoy shorter than the secret answer exhausts mid-prank: the fake
        // question freezes on screen while the operator is still typing. 20
        // chars comfortably outlasts typical secret answers and still reads
        // as a real question mid-crawl.
        const MIN_DECOY_CHARS: usize = 85;

        for lang in Language::ALL {
            for decoy in lang.translation().decoys {
                assert!(
                    decoy.chars().count() >= MIN_DECOY_CHARS,
                    "{lang:?} decoy {decoy:?} is under {MIN_DECOY_CHARS} chars"
                );
            }
        }
    }

    // ── ConfigTexts::label_width — the padding column, DERIVED not declared ───
    //
    // `config.rs::styled_label` pads every option label out to a shared column
    // so the values line up. That width is a hand-written `LABEL_WIDTH: usize`
    // and the padding is `LABEL_WIDTH - label.chars().count()` — a raw `usize`
    // subtraction. A label longer than the constant underflows: panic in debug,
    // an attempt to allocate ~1.8×10¹⁹ spaces in release. Raising the constant
    // (12 → 14, 2026-07-26) moved that cliff without removing it.
    //
    // Deriving the width from the labels themselves removes it: if the width IS
    // the longest label, the subtraction cannot go negative. These pin the
    // derivation; the padding then has nothing left to get wrong.

    #[test]
    fn label_width_in_portuguese_is_animacoes() {
        // ANIMAÇÕES is 9 CHARS but 11 BYTES. `.len()` would answer 11 and
        // silently over-pad every row — this is the char-vs-byte tripwire.
        assert_eq!(Language::PtBr.translation().config.max_label_width(), 9);
    }

    #[test]
    fn label_width_in_english_is_animations() {
        assert_eq!(Language::EnUs.translation().config.max_label_width(), 10);
    }

    #[test]
    fn label_width_in_spanish_is_animaciones() {
        // The widest of the three, and the reason a fixed 12 was ever unsafe.
        assert_eq!(Language::EsEs.translation().config.max_label_width(), 11);
    }

    #[test]
    fn every_config_label_fits_inside_the_derived_width() {
        // THE CONTRACT PIN — the other three only pin today's content, and all
        // three would still pass if `label_width` forgot a field, because
        // ANIMAÇÕES/ANIMATIONS/ANIMACIONES happens to be the longest in every
        // language. This one fails the moment a label is left out of the max,
        // and it is what actually guarantees the subtraction is total.
        for lang in Language::ALL {
            let config = lang.translation().config;
            let width = config.max_label_width();
            for label in [
                config.theme,
                config.animations,
                config.volume,
                config.language,
            ] {
                assert!(
                    label.chars().count() <= width,
                    "{lang:?}: label {label:?} is {} chars but label_width() says {width} \
                     — the padding subtraction would underflow",
                    label.chars().count()
                );
            }
        }
    }
}
