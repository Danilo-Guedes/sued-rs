use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    #[default]
    EnUs,
    PtBr,
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
    /// The refusal reserved for a question too SHORT to be a question (G17) —
    /// picked instead of a random `denial`, so it is a sibling of that pool and
    /// lives beside it rather than in `AskTexts`.
    ///
    /// ⚠ **`{question}` is real substitution, not `{{markup}}`.** `template.rs`
    /// only marks segments for the accent colour, so `{{question}}` would print
    /// the literal word. And this string must stay **markup-free entirely**:
    /// replies render through `typewriter_reveal`, which reveals a *prefix* —
    /// a prefix of `"foo {{bar}}"` is `"foo {{ba"`, broken braces on screen for
    /// every frame of the crawl. Markup and the typewriter do not compose.
    ///
    /// ⚠ The placeholder lives INSIDE the translated string, never concatenated
    /// in Rust: word order is not universal, and which word goes where belongs
    /// to the language (`template.rs`'s own argument).
    pub rebuke: &'static str,

    /// The OPERATOR's manual, printed by `--how-it-works` and never drawn on a
    /// screen — see `cli::how_it_works_text` for why that split is the design
    /// and not a convenience.
    ///
    /// ⚠ `{repo}` is real substitution, not `{{markup}}`: this string goes to
    /// stdout as plain text and never passes through `template.rs`.
    ///
    /// ⏳ **PROVISIONAL COPY.** PLAN §G16 schedules the real prose for Phase 6,
    /// written in one pass with the README and the story popover — three
    /// outputs, one job, and writing them apart is how they drift.
    pub how_it_works: &'static str,

    ///SCREENS TEXTS
    pub intro: IntroTexts,
    pub about: AboutTexts,
    pub info: InfoTexts,
    pub ask: AskTexts,
    pub history: HistoryTexts,
    pub confirm: ConfirmTexts,
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
    pub story: StoryTexts,
}

/// The `[?]` popover on About — the one place this app speaks out of character.
///
/// ⚠ **What is NOT here, on purpose:** the `;` toggle, hidden mode, and how the
/// decoy paints itself. Those are the *operator's* manual and they live outside
/// the app in `--how-it-works`, because anything drawn on screen can be read
/// over the operator's shoulder mid-prank. This struct is the *story* only —
/// who wrote it, why, and where the source is. Do not let the two drift back
/// together (PLAN §G16).
///
/// 📌 `body` **does** take `{{accent}}` markup, like `AboutTexts::lore`.
/// ⚠ An earlier draft of this comment claimed it did not, on the theory that
/// resolving markup would fight the `line_count` measurement. It does not — as
/// long as `styled_line` runs FIRST, per source line: the braces are gone before
/// anything is measured, so `line_count` sees the rendered width. Feed the raw
/// string to a `Paragraph` and you get the opposite of both halves — braces on
/// screen, and four phantom columns per marker in the measurement.
///
/// The links and the command are NOT fields: they carry no language. See
/// `constants::{AUTHOR_GITHUB, AUTHOR_LINKEDIN, HOW_IT_WORKS_COMMAND}`.
#[derive(Debug, Copy, Clone)]
pub struct StoryTexts {
    pub title: &'static str,
    /// The long one — paragraphs separated by a blank line. Scrolls.
    pub body: &'static str,
    /// Byline, pinned below the prose so it is never below the fold.
    pub signature: &'static str,
    /// The question that hands the reader off to the operator's manual.
    pub bridge: &'static str,
    /// "rode:" / "run:" — the verb in front of the command.
    pub run_prefix: &'static str,
    /// ⚠ Two hints, not one slice, and the split is load-bearing. The strip is
    /// composed at render time because the scroll keys only exist *conditionally*
    /// — on a terminal tall enough to hold the whole story there is nothing to
    /// scroll, and advertising the keys anyway reads as broken scrolling. Only
    /// the render knows which case it is (see `story::render`'s return), so the
    /// pieces travel separately and it assembles them.
    pub scroll_hint: (&'static str, &'static str),
    pub close_hint: (&'static str, &'static str),
}

#[derive(Debug, Copy, Clone)]
pub struct InfoTexts {
    pub title: &'static str,
    pub instructions: &'static [&'static str],
    pub example: &'static str,
    /// ⚠ `shortcut_title` + `shortcuts` were CUT by G20, and not for staleness.
    /// This screen instructs the *mark*; a key table listing `[F5]` — the
    /// operator's panic button, which burns the staged answer — was printing the
    /// one thing the victim must never learn. The operator's table lives in
    /// `--how-it-works` now, where only the operator can read it.
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
    pub spells: &'static [&'static str],
}

#[derive(Debug, Copy, Clone)]
pub struct HistoryTexts {
    pub title: &'static str,
    pub you: &'static str,
    pub hints: &'static [(&'static str, &'static str)],
}

#[derive(Debug, Copy, Clone)]
pub struct ConfirmTexts {
    pub title: &'static str,
    pub lore_text: &'static str,
    pub abandon_question: &'static str,
    pub leave: &'static str,
    pub stay: &'static str,
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
    pub const ALL: [Language; 3] = [Language::EnUs, Language::PtBr, Language::EsEs];

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
                rebuke: "{question} ??? Você não entendeu o que eu disse? Me bajule primeiro, mortal, e só então pergunte, por quê humanos dificultam tanto?",
                how_it_works: "\
SueD é uma pegadinha. Não existe oráculo nenhum.

Quem responde é VOCÊ. O truque é digitar a resposta em segredo enquanto
finge estar digitando a pergunta.

Teste você mesmo:

  1. Abra \"Perguntar\".
  2. Aperte  ;  — nada muda na tela. Você está em modo OCULTO.
  3. Digite a RESPOSTA que o Sued deve dar. A tela mostra uma pergunta
     falsa se escrevendo sozinha, um caractere por tecla apertada.
  4. Aperte  ;  de novo para voltar ao normal, termine o elogio de onde
     parou e finalize a sua pergunta.
  5. Enter. O Sued pondera e então \"revela\" o que você preparou.

Aperte Enter sem nada preparado e ele se recusa a responder — uma
provocação, ou uma bronca se a pergunta foi curta demais.

Se a pergunta falsa estiver acabando — ou seja, você está digitando uma
resposta longa — você ouvirá o estrondo de um raio, sinal de que precisa
terminar sua resposta escondida logo. Isso acontece quando ainda faltam
{THUNDER_AT_CHARS_REMAINING} caracteres da pergunta falsa.

Algumas dicas que deixam a brincadeira mais interessante:

  1. Se possível, construa uma história antes de apresentar o SUED, algo
     como \"consegui um software secreto\" ou \"achei este software sombrio
     que faz coisas estranhas\".
  2. Evite perguntas e respostas muito diretas, como \"quem foi?\". Quanto
     mais elaboradas as perguntas e as respostas, mais impressionante o
     truque fica. O SUED também se recusa a responder perguntas com
     menos de {SHORT_QUESTION_CHARS} caracteres.
  3. Como em qualquer peça ou apresentação, fica muito melhor quando
     você ensaia antes — assim já terá pegado o jeito de conduzir a
     pegadinha.
  4. Separe um tempo para escolher boas perguntas sobre a vítima, e
     prefira assuntos que não são conhecidos por todos: isso faz a
     experiência dela ser bem mais aterrorizante.
  5. Seja um condutor da brincadeira. Para evitar que percebam que você
     digita uma coisa enquanto eles leem outra, vá falando em voz alta
     o elogio que o SUED exige antes da pergunta.
  6. Às vezes se permita fazer perguntas sem resposta escondida, e deixe
     o SUED recusar. Assim não fica óbvio que só funciona quando é você
     quem conduz.
  7. O truque funciona melhor com pessoas não muito ligadas em
     tecnologia, sejam crianças ou adultos — aproveite para testar com
     seu sobrinho, ou com seu pai e sua mãe.

Principais comandos:

  ;         liga/desliga o modo oculto — o truque inteiro
  F5        botão de pânico: queima a resposta preparada e recomeça
  F1        histórico da sessão
  Esc       volta / fecha o que estiver por cima
  Ctrl+C    sai na hora

Não deixe isto na tela. Todo o resto do app é escrito para a vítima;
esta é a única página escrita para você.

feito por: Danilo Guedes
fonte: {repo}",
                intro: IntroTexts {
                    subtitle: "SUA ÚLTIMA ESPERANÇA DIVINA",
                    attention: "A T E N Ç Ã O",
                    welcome: "Você está prestes a abrir uma porta para o desconhecido.\n\
                              Aconselho acender uma vela e apagar as luzes antes de executar.\n\
                              Para que {{SUED}} responda, você deve elogiá-lo e em seguida \
                              pergunte com clareza.",
                    disclaimer: "Pessoas fracas e sensíveis não devem utilizar o programa.\n\
                                 Tenha muito cuidado com o que você irá perguntar...",
                    continue_btn: "CONTINUAR",
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
                        ("runtime", "rust · ratatui · crossterm · kira"),
                    ],
                    footer: concat!(
                        "sued-rs v",
                        env!("CARGO_PKG_VERSION"),
                        " · recriação do clássico brasileiro · use por sua conta e risco"
                    ),
                    hints: &[("[Esc]", "voltar ao menu"), ("[?]", "por trás do véu")],
                    story: StoryTexts {
                        title: "POR TRÁS DO VÉU",
                        // ⚠ EVERY line ends in `\` and every paragraph break is an
                        // explicit `\n\n`. A source line that just ENDS keeps its
                        // newline *and* the indentation of the next line — which is
                        // how the first draft got a paragraph break followed by 31
                        // literal spaces on screen. The `\` is what eats both.
                        body: "Eu tinha uns dez anos quando vi o Sued pela primeira \
                               vez, a internet ainda era algo novo no Brasil, tudo era \
                               novidade. Um amigo me chamou para perto do computador, e \
                               falou que havia baixado um programa {{sinistro}} que \
                               sabia de coisas profundas e obscuras. O programa era \
                               realmente com um tema de terror, todo em vermelho e \
                               preto, com imagens do tal {{SUED}}.\n\n\
                               Foi então que iniciamos uma sessão de perguntas, e logo \
                               no início me lembro de {{me arrepiar}} quando vi SUED \
                               digitando algo íntimo sobre mim e minha família. Este \
                               amigo continuou pregando a peça em mim, e me lembro de \
                               ficar de queixo caído, sem entender como {{SUED}} sabia \
                               de tudo aquilo, só poderia ser algo do {{além}}. Foi \
                               então que, após algum tempo, meu amigo revelou o segredo \
                               e caímos na gargalhada.\n\n\
                               {{SUED}} provavelmente nasceu como brincadeira de porão \
                               no Brasil dos anos 2000 e correu o país de disquete em \
                               disquete, de lan house em lan house.\n\n\
                               Esta versão é uma recriação em Rust 🦀 em forma de CLI — \
                               o primeiro projeto que escrevi na linguagem.",
                        signature: "Danilo Guedes · Desenvolvedor de Software que ama \
                                    aprender e resolver problemas com tecnologia",
                        bridge: "curioso pra saber como o oráculo funciona?",
                        run_prefix: "rode:",
                        scroll_hint: ("[↑↓ PgUp PgDn]", "rolar"),
                        close_hint: ("[Esc] [?]", "fechar"),
                    },
                },
                info: InfoTexts {
                    title: "O RITUAL",
                    instructions: &[
                        "Acenda uma vela e apague as luzes do recinto.",
                        "{{Elogie/Bajule}} o Sued antes de qualquer coisa — ele é vaidoso.",
                        "Faça {{uma}} pergunta por vez, de forma clara e objetiva.",
                        "Aguarde em silêncio. A resposta virá do além.",
                    ],
                    example: "Ex.: \"Sued, o mais sábio de todos, o que me aguarda amanhã?\"",
                    terminal_hint: "terminal {size} recomendado",
                    hints: &[("[Esc]", "voltar ao menu")],
                },
                ask: AskTexts {
                    sued_speak: "SUED FALA",
                    welcome_line: "Pergunte-me o que deseja saber, humano...",
                    praise: "— elogie-me antes da pergunta, e {{talvez}} eu responda.",
                    connection: "conexão com o além estabelecida.",
                    waiting: "aguardando oferenda do mortal",
                    talk_with_me: "FALE COMIGO...",
                    hints: &[
                        ("[Enter]", "perguntar"),
                        ("[F1]", "histórico"),
                        ("[F5]", "recomeçar"),
                        ("[Esc]", "menu"),
                        ("[Ctrl+C]", "sair"),
                    ],
                    spells: &[
                        "folheando os livros proibidos das trevas",
                        "cobrando favores antigos do outro lado",
                        "consultando o desconhecido nas sombras",
                        "invocando ecos distantes do abismo",
                        "despertando os que dormem sob a terra",
                        "acendendo as velas pretas do ritual",
                        "abrindo as portas trancadas do submundo",
                        "negociando com as sombras do porão",
                    ],
                },
                history: HistoryTexts {
                    title: "HISTÓRICO DA SESSÃO",
                    you: "VOCÊ",
                    hints: &[
                        ("[↑↓]", "rolar"),
                        ("[PgUp PgDn]", "rolar página"),
                        ("[Esc]", "fechar"),
                        ("[Ctrl+C]", "sair"),
                    ],
                },
                confirm: ConfirmTexts {
                    title: "O VÉU VAI SE FECHAR",
                    lore_text: "Ao partir, tudo o que foi dito aqui retorna ao silêncio. O oráculo esquecerá vossa voz, e o que vos foi revelado jamais será revelado outra vez",
                    abandon_question: "Deseja mesmo abandonar a sessão?",
                    leave: "QUE ASSIM SEJA",
                    stay: "PERMANECER",
                    hints: &[
                        ("[← →]", "escolher"),
                        ("[Enter]", "confirmar"),
                        ("[Esc]", "cancelar"),
                        ("[Ctrl+C]", "sair"),
                    ],
                },
                config: ConfigTexts {
                    configuration: "CONFIGURAÇÃO",
                    subtitle: "ajuste o ritual ao seu gosto — o oráculo observa",
                    theme: "TEMA",
                    animations: "ANIMAÇÕES",
                    volume: "VOLUME",
                    language: "IDIOMA",
                    yes: "SIM",
                    no: "NÃO",
                    footer: "suas escolhas foram registradas no além",
                    hints: &[("[↑↓]", "navegar"), ("[↔]", "alterar"), ("[Esc]", "voltar")],
                },
                menu: MenuTexts {
                    choose_your_destiny: "ESCOLHA SEU DESTINO",
                    example: "Faça sua pergunta ao oráculo. Elogie-o primeiro, depois pergunte de forma clara e objetiva.",
                    attention: "ATENÇÃO",
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
                rebuke: "{question} ??? Did you not understand what I said? Flatter me first, mortal, and only then ask, why do humans make it so difficult?",
                how_it_works: "\
SueD is a prank. There is no oracle.

YOU are the one answering. The trick is typing the answer in secret while
you appear to be typing the question.

Try it yourself:

  1. Open \"Ask\".
  2. Press  ;  — nothing on screen changes. You are now in HIDDEN mode.
  3. Type the ANSWER you want SueD to give. The screen shows a fake
     question writing itself, one character per key you press.
  4. Press  ;  again to go back to normal, finish the flattery where you
     left it, and round off your question.
  5. Enter. SueD ponders, then \"reveals\" what you staged.

Press Enter with nothing staged and he refuses instead — a taunt, or a
rebuke if the question was too short.

If the fake question is running out — that is, you are typing a long
answer — you will hear a thunderclap, your cue to finish the hidden
answer soon. It fires while there are still
{THUNDER_AT_CHARS_REMAINING} characters of fake question left.

A few things that make the game far better:

  1. If you can, build a story before introducing SueD: \"I got hold of
     some secret software\", or \"I found this grim program that does
     strange things\".
  2. Avoid blunt questions and answers like \"who was it?\". The more
     elaborate both are, the more impressive the trick. SueD also
     refuses questions shorter than {SHORT_QUESTION_CHARS} characters.
  3. Like any act, it plays much better rehearsed — a run-through is how
     you get the feel for steering the whole thing.
  4. Spend some time picking good questions about your mark, and prefer
     subjects not everyone in the room knows: that is what makes it
     genuinely unsettling for them.
  5. Be the host. To stop anyone noticing you type one thing while they
     read another, say the flattery SueD demands out loud as you go.
  6. Now and then, ask with nothing staged and let SueD refuse. It stops
     being obvious that this only works while you are driving.
  7. It lands best on people who are not especially technical, children
     and adults alike — try it on your niece, or on your parents.

Main keys:

  ;         toggle hidden mode — the whole trick
  F5        panic button: burns the staged answer and starts over
  F1        transcript of the séance
  Esc       back / close whatever is on top
  Ctrl+C    quit immediately

Do not leave this on screen. Everything else in the app is written for
the mark; this is the only page written for you.

made by: Danilo Guedes
source: {repo}",
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
                    continue_btn: "CONTINUE",
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
                        ("runtime", "rust · ratatui · crossterm · kira"),
                    ],
                    footer: concat!(
                        "sued-rs v",
                        env!("CARGO_PKG_VERSION"),
                        " · a recreation of the Brazilian classic · use at your own risk"
                    ),
                    hints: &[("[Esc]", "back to menu"), ("[?]", "behind the veil")],
                    story: StoryTexts {
                        title: "BEHIND THE VEIL",
                        // ⚠ Tracks the PT, which is the ORIGINAL — this is his
                        // memory and the PT is the one with the voice. Same
                        // paragraph count and roughly the same length on purpose:
                        // the box wraps at 64 columns, so a translation that drifts
                        // long silently changes how far the reader has to scroll.
                        body: "I was about ten when I first saw Sued. The internet \
                               was still new in Brazil and everything was a novelty. \
                               A friend called me over to his computer and told me he \
                               had downloaded a {{sinister}} program that knew deep \
                               and obscure things. It really did have a horror theme, \
                               all in red and black, with pictures of this \
                               {{SUED}}.\n\n\
                               That was when we started a round of questions, and \
                               right at the start I remember {{the shiver}} when I saw \
                               SUED typing something intimate about me and my family. \
                               My friend kept the joke going, and I remember standing \
                               there jaw-dropped, unable to work out how {{SUED}} knew \
                               all of it — it could only be something from {{beyond}}. \
                               Then, after a while, my friend gave up the secret and \
                               we fell about laughing.\n\n\
                               {{SUED}} was probably born as a basement prank in 2000s \
                               Brazil, and travelled the country floppy by floppy, LAN \
                               house by LAN house.\n\n\
                               This version is a recreation in Rust 🦀 as a CLI — the \
                               first project I ever wrote in the language.",
                        signature: "Danilo Guedes · Software developer who loves \
                                    learning and solving problems with technology",
                        bridge: "curious how the oracle really works?",
                        run_prefix: "run:",
                        scroll_hint: ("[↑↓ PgUp PgDn]", "scroll"),
                        close_hint: ("[Esc] [?]", "close"),
                    },
                },
                info: InfoTexts {
                    title: "THE RITUAL",
                    instructions: &[
                        "Light a candle and turn off the lights in the room.",
                        "{{Flatter/Praise}} Sued before anything else — he is vain.",
                        "Ask {{one}} question at a time, clearly and to the point.",
                        "Wait in silence. The answer will come from the beyond.",
                    ],
                    example: "E.g.: \"Sued, wisest of all, what awaits me tomorrow?\"",
                    terminal_hint: "{size} terminal recommended",
                    hints: &[("[Esc]", "back to menu")],
                },
                ask: AskTexts {
                    sued_speak: "SUED SPEAKS",
                    welcome_line: "Ask me what you wish to know, human...",
                    praise: "— flatter me before you ask, and {{maybe}} I shall answer.",
                    connection: "connection to the beyond established.",
                    waiting: "awaiting the mortal's offering",
                    talk_with_me: "SPEAK TO ME...",
                    hints: &[
                        ("[Enter]", "ask"),
                        ("[F1]", "history"),
                        ("[F5]", "start over"),
                        ("[Esc]", "menu"),
                        ("[Ctrl+C]", "quit"),
                    ],
                    spells: &[
                        "leafing through the forbidden dark books",
                        "calling in old favors from the other side",
                        "consulting the unknown in the shadows",
                        "invoking distant echoes from the abyss",
                        "waking those who sleep beneath the earth",
                        "lighting the black candles of the rite",
                        "unlocking the doors of the underworld",
                        "bargaining with the shadows in the cellar",
                    ],
                },
                history: HistoryTexts {
                    title: "SESSION HISTORY",
                    you: "YOU",
                    hints: &[
                        ("[↑↓]", "scroll"),
                        ("[PgUp PgDn]", "scroll page"),
                        ("[Esc]", "close"),
                        ("[Ctrl+C]", "quit"),
                    ],
                },
                confirm: ConfirmTexts {
                    title: "THE VEIL IS ABOUT TO CLOSE",
                    lore_text: "When you depart, all that was spoken here returns to silence. \
                                The oracle will forget your voice, and what was revealed to \
                                you shall never be revealed again",
                    abandon_question: "Do you truly wish to abandon the session?",
                    leave: "SO BE IT",
                    stay: "REMAIN",
                    hints: &[
                        ("[← →]", "choose"),
                        ("[Enter]", "confirm"),
                        ("[Esc]", "cancel"),
                        ("[Ctrl+C]", "quit"),
                    ],
                },
                config: ConfigTexts {
                    configuration: "CONFIGURATION",
                    subtitle: "tune the ritual to your taste — the oracle watches",
                    theme: "THEME",
                    animations: "ANIMATIONS",
                    volume: "VOLUME",
                    language: "LANGUAGE",
                    yes: "YES",
                    no: "NO",
                    footer: "your choices have been recorded in the beyond",
                    hints: &[("[↑↓]", "navigate"), ("[↔]", "change"), ("[Esc]", "back")],
                },
                menu: MenuTexts {
                    choose_your_destiny: "CHOOSE YOUR DESTINY",
                    example: "Ask the oracle your question. Flatter him first, then ask clearly and to the point.",
                    attention: "ATTENTION",
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
                rebuke: "{question} ??? ¿No entendiste lo que dije? Halágame primero, mortal, y sólo entonces pregunta, ¿por qué los humanos lo complican tanto?",
                how_it_works: "\
SueD es una broma. No hay ningún oráculo.

Quien responde eres TÚ. El truco es escribir la respuesta en secreto
mientras aparentas estar escribiendo la pregunta.

Pruébalo tú mismo:

  1. Abre \"Preguntar\".
  2. Pulsa  ;  — nada cambia en pantalla. Estás en modo OCULTO.
  3. Escribe la RESPUESTA que quieres que dé SueD. La pantalla muestra
     una pregunta falsa escribiéndose sola, un carácter por tecla.
  4. Pulsa  ;  otra vez para volver a lo normal, termina el halago donde
     lo dejaste y remata tu pregunta.
  5. Enter. SueD medita y entonces \"revela\" lo que preparaste.

Pulsa Enter sin nada preparado y se niega a responder — una burla, o un
reproche si la pregunta fue demasiado corta.

Si la pregunta falsa se está acabando — es decir, estás escribiendo una
respuesta larga — oirás el estruendo de un rayo, la señal para terminar
pronto la respuesta oculta. Suena cuando todavía quedan
{THUNDER_AT_CHARS_REMAINING} caracteres de pregunta falsa.

Algunos consejos que hacen el juego mucho mejor:

  1. Si puedes, construye una historia antes de presentar a SueD: \"he
     conseguido un software secreto\", o \"encontré este programa sombrío
     que hace cosas raras\".
  2. Evita preguntas y respuestas demasiado directas, como \"¿quién
     fue?\". Cuanto más elaboradas sean, más impresionante queda el
     truco. SueD también rechaza preguntas de menos de
     {SHORT_QUESTION_CHARS} caracteres.
  3. Como en cualquier número, sale mucho mejor si lo ensayas antes: así
     ya le habrás cogido el punto a conducir la broma.
  4. Dedica un rato a elegir buenas preguntas sobre tu víctima, y
     prefiere temas que no conozca todo el mundo: eso es lo que la hace
     de verdad inquietante.
  5. Sé el conductor del juego. Para que nadie note que escribes una
     cosa mientras leen otra, ve diciendo en voz alta el halago que SueD
     exige antes de la pregunta.
  6. De vez en cuando pregunta sin nada preparado y deja que SueD se
     niegue. Así no queda obvio que sólo funciona cuando lo llevas tú.
  7. Funciona mejor con gente poco metida en tecnología, tanto niños
     como adultos — pruébalo con tu sobrino, o con tus padres.

Teclas principales:

  ;         activa/desactiva el modo oculto — el truco entero
  F5        botón de pánico: quema la respuesta preparada y reinicia
  F1        historial de la sesión
  Esc       volver / cerrar lo que esté encima
  Ctrl+C    salir de inmediato

No dejes esto en pantalla. Todo lo demás en la app está escrito para la
víctima; esta es la única página escrita para ti.

hecho por: Danilo Guedes
fuente: {repo}",
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
                    continue_btn: "CONTINUAR",
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
                        ("runtime", "rust · ratatui · crossterm · kira"),
                    ],
                    footer: concat!(
                        "sued-rs v",
                        env!("CARGO_PKG_VERSION"),
                        " · recreación del clásico brasileño · úsalo bajo tu propio riesgo"
                    ),
                    hints: &[("[Esc]", "volver al menú"), ("[?]", "tras el velo")],
                    story: StoryTexts {
                        title: "TRAS EL VELO",
                        // ⚠ Same note as the EN — the PT is the original.
                        body: "Tenía unos diez años cuando vi a Sued por primera vez. \
                               Internet todavía era algo nuevo en Brasil, todo era una \
                               novedad. Un amigo me llamó cerca del computador y me \
                               dijo que había bajado un programa {{siniestro}} que \
                               sabía de cosas profundas y oscuras. El programa \
                               realmente tenía un tema de terror, todo en rojo y \
                               negro, con imágenes del tal {{SUED}}.\n\n\
                               Fue entonces que empezamos una sesión de preguntas, y \
                               enseguida recuerdo {{el escalofrío}} cuando vi a SUED \
                               escribiendo algo íntimo sobre mí y mi familia. Ese \
                               amigo siguió gastándome la broma, y recuerdo quedarme \
                               boquiabierto, sin entender cómo {{SUED}} sabía todo \
                               aquello; solo podía ser algo del {{más allá}}. Al cabo \
                               de un rato mi amigo reveló el secreto y nos morimos de \
                               risa.\n\n\
                               {{SUED}} probablemente nació como una broma de sótano \
                               en el Brasil de los años 2000 y recorrió el país de \
                               disquete en disquete, de ciber en ciber.\n\n\
                               Esta versión es una recreación en Rust 🦀 en forma de \
                               CLI — el primer proyecto que escribí en el lenguaje.",
                        signature: "Danilo Guedes · Desarrollador de software al que \
                                    le encanta aprender y resolver problemas con \
                                    tecnología",
                        bridge: "¿con curiosidad por cómo funciona el oráculo?",
                        run_prefix: "ejecuta:",
                        scroll_hint: ("[↑↓ PgUp PgDn]", "desplazar"),
                        close_hint: ("[Esc] [?]", "cerrar"),
                    },
                },
                info: InfoTexts {
                    title: "EL RITUAL",
                    instructions: &[
                        "Enciende una vela y apaga las luces de la sala.",
                        "{{Halaga/Adula}} a Sued antes que nada — es vanidoso.",
                        "Haz {{una}} pregunta a la vez, de forma clara y concreta.",
                        "Espera en silencio. La respuesta vendrá del más allá.",
                    ],
                    example: "Ej.: \"Sued, el más sabio de todos, ¿qué me espera mañana?\"",
                    terminal_hint: "terminal {size} recomendado",
                    hints: &[("[Esc]", "volver al menú")],
                },
                ask: AskTexts {
                    sued_speak: "SUED HABLA",
                    welcome_line: "Pregúntame lo que deseas saber, humano...",
                    praise: "— halágame antes de la pregunta, y {{quizá}} te responda.",
                    connection: "conexión con el más allá establecida.",
                    waiting: "aguardando la ofrenda del mortal",
                    talk_with_me: "HÁBLAME...",
                    hints: &[
                        ("[Enter]", "preguntar"),
                        ("[F1]", "historial"),
                        ("[F5]", "reiniciar"),
                        ("[Esc]", "menú"),
                        ("[Ctrl+C]", "salir"),
                    ],
                    spells: &[
                        "hojeando los libros prohibidos y malditos",
                        "cobrando viejos favores del otro lado",
                        "consultando lo desconocido en las sombras",
                        "invocando ecos lejanos del abismo",
                        "despertando a los que duermen bajo tierra",
                        "encendiendo las velas negras del ritual",
                        "forzando las puertas del inframundo",
                        "negociando con las sombras del sótano",
                    ],
                },
                history: HistoryTexts {
                    title: "HISTORIAL DE LA SESIÓN",
                    you: "TÚ",
                    hints: &[
                        ("[↑↓]", "desplazar"),
                        ("[PgUp PgDn]", "desplazar página"),
                        ("[Esc]", "cerrar"),
                        ("[Ctrl+C]", "salir"),
                    ],
                },
                confirm: ConfirmTexts {
                    title: "EL VELO VA A CERRARSE",
                    lore_text: "Al partir, todo lo que aquí se dijo vuelve al silencio. El \
                                oráculo olvidará vuestra voz, y lo que os fue revelado jamás \
                                será revelado otra vez",
                    abandon_question: "¿Deseas de verdad abandonar la sesión?",
                    leave: "QUE ASÍ SEA",
                    stay: "PERMANECER",
                    hints: &[
                        ("[← →]", "elegir"),
                        ("[Enter]", "confirmar"),
                        ("[Esc]", "cancelar"),
                        ("[Ctrl+C]", "salir"),
                    ],
                },
                config: ConfigTexts {
                    configuration: "CONFIGURACIÓN",
                    subtitle: "ajusta el ritual a tu gusto — el oráculo observa",
                    theme: "TEMA",
                    animations: "ANIMACIONES",
                    volume: "VOLUMEN",
                    language: "IDIOMA",
                    yes: "SÍ",
                    no: "NO",
                    footer: "tus decisiones han sido registradas en el más allá",
                    hints: &[("[↑↓]", "navegar"), ("[↔]", "cambiar"), ("[Esc]", "volver")],
                },
                menu: MenuTexts {
                    choose_your_destiny: "ELIGE TU DESTINO",
                    example: "Haz tu pregunta al oráculo. Halágalo primero, luego pregunta de forma clara y concreta.",
                    attention: "ATENCIÓN",
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

    // ── which language SueD ships in ─────────────────────────────────────────
    // A product decision, not an implementation detail: SueD boots in English
    // because crates.io is an English storefront and `cargo install` traffic is
    // global. `Configuration::default()` delegates here, so config.rs can only
    // pin THAT delegation — it cannot pin the choice itself without asserting
    // `default() == default()`. These two are that choice's only home: flipping
    // `#[default]` or reordering `ALL` has to break a test, deliberately.

    #[test]
    fn english_is_the_language_sued_ships_in() {
        assert_eq!(Language::default(), Language::EnUs);
    }

    #[test]
    fn the_language_cycle_starts_at_english() {
        // `ALL` is what [←→] walks on the config screen and the order the chips
        // render in — a separate fact from `#[default]`, and separately flippable.
        assert_eq!(Language::ALL[0], Language::EnUs);
    }

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

    // ── The version in the UI must BE the crate version, not a copy of it ────
    //
    // `AboutTexts.footer` carried a hand-typed "sued-rs v0.1.0" in all three
    // languages. Bumping `Cargo.toml` for the v1.0.0 tag would have left the
    // About screen — the one screen whose job is saying what this program is —
    // confidently announcing v0.1.0, three times over.
    //
    // ⚠ THIS IS A TRIPWIRE, NOT A SPEC: it passes today whether the version is
    // hardcoded or derived, because the crate really is at the hardcoded value.
    // It goes red at exactly the moment the bug would ship — the version bump —
    // which is the whole point. Same class as the decoy-length and
    // cross-language-distinct pins: dormant until content drifts.

    #[test]
    fn every_about_footer_carries_the_real_crate_version() {
        let version = env!("CARGO_PKG_VERSION");

        for lang in Language::ALL {
            let footer = lang.translation().about.footer;
            assert!(
                footer.contains(version),
                "{lang:?} about footer says {footer:?} but the crate is v{version} \
                 — read the version from Cargo.toml with env!(\"CARGO_PKG_VERSION\")"
            );
        }
    }
}
