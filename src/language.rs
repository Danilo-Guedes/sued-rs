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
    /// The opening line on a fresh ask screen.
    pub welcome_line: &'static str,
    pub intro: IntroTexts,
}

#[derive(Debug, Copy, Clone)]
pub struct IntroTexts {
    pub subtitle: &'static str,
    pub attention: &'static str,
    pub welcome: &'static str,
    pub disclaimer: &'static str,
    pub continue_btn: &'static str,
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
                welcome_line: "Pergunte-me o que deseja saber, humano...",
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
                welcome_line: "Ask me what you wish to know, human...",
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
                welcome_line: "Pregúntame lo que deseas saber, humano...",
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
                !lang.translation().welcome_line.is_empty(),
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
                    a.translation().welcome_line,
                    b.translation().welcome_line,
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
}
