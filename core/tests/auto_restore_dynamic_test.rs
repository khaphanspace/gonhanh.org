//! Dynamic Tests for English Auto-Restore Feature
//!
//! This module generates comprehensive test cases for the auto-restore feature
//! using parameterized tests to ensure complete coverage of all patterns.
//!
//! # Auto-Restore Detection Patterns
//!
//! 1. Invalid initials (F, J, W, Z) - always restore
//! 2. Consonant clusters (bl, cl, fl, etc.) - always restore
//! 3. Modifier + consonant sequences - restore
//! 4. EI/AI vowel pairs with specific patterns - restore
//! 5. Vowel + modifier + vowel patterns - restore based on context
//! 6. W as final (not modifier) - restore
//!
//! # Design Decision
//!
//! Words that form structurally valid Vietnamese (like "test" → "tét") are
//! NOT auto-restored. Users should use ESC or raw mode for these.

mod common;
use common::telex_auto_restore;
use rstest::rstest;

// ============================================================
// PATTERN 1: INVALID INITIAL F
// F is not a valid Vietnamese initial (Vietnamese uses PH for /f/)
// ============================================================

/// Common English words starting with F
const F_WORDS: &[(&str, &str)] = &[
    ("fair ", "fair "),
    ("fall ", "fall "),
    ("false ", "false "),
    ("far ", "far "),
    ("farm ", "farm "),
    ("fast ", "fast "),
    ("fat ", "fat "),
    ("fear ", "fear "),
    ("feed ", "feed "),
    ("feel ", "feel "),
    ("few ", "few "),
    ("file ", "file "),
    ("fill ", "fill "),
    ("film ", "film "),
    ("final ", "final "),
    ("find ", "find "),
    ("fine ", "fine "),
    ("fire ", "fire "),
    ("firm ", "firm "),
    ("first ", "first "),
    ("fish ", "fish "),
    ("fit ", "fit "),
    ("fix ", "fix "),
    ("flag ", "flag "),
    ("flat ", "flat "),
    ("flex ", "flex "),
    ("floor ", "floor "),
    ("flow ", "flow "),
    ("fly ", "fly "),
    ("focus ", "focus "),
    ("fold ", "fold "),
    ("follow ", "follow "),
    ("font ", "font "),
    ("food ", "food "),
    ("foot ", "foot "),
    ("for ", "for "),
    ("force ", "force "),
    ("fork ", "fork "),
    ("form ", "form "),
    ("format ", "format "),
    ("forward ", "forward "),
    ("found ", "found "),
    ("four ", "four "),
    ("frame ", "frame "),
    ("free ", "free "),
    ("fresh ", "fresh "),
    ("from ", "from "),
    ("front ", "front "),
    ("full ", "full "),
    ("fun ", "fun "),
    ("function ", "function "),
    ("future ", "future "),
];

#[test]
fn f_initial_comprehensive() {
    telex_auto_restore(F_WORDS);
}

#[rstest]
#[case("f", "a", "fa")]
#[case("f", "e", "fe")]
#[case("f", "i", "fi")]
#[case("f", "o", "fo")]
#[case("f", "u", "fu")]
#[case("f", "ai", "fai")]
#[case("f", "au", "fau")]
#[case("f", "ea", "fea")]
#[case("f", "ee", "fee")]
#[case("f", "ie", "fie")]
#[case("f", "oo", "foo")]
#[case("f", "ou", "fou")]
fn f_initial_vowel_patterns(#[case] initial: &str, #[case] vowel: &str, #[case] expected: &str) {
    let input = format!("{}{} ", initial, vowel);
    let output = format!("{} ", expected);
    telex_auto_restore(&[(&input, &output)]);
}

// ============================================================
// PATTERN 2: INVALID INITIAL J
// J is not a valid Vietnamese initial
// ============================================================

const J_WORDS: &[(&str, &str)] = &[
    ("job ", "job "),
    ("join ", "join "),
    ("joke ", "joke "),
    ("joy ", "joy "),
    ("judge ", "judge "),
    ("jump ", "jump "),
    ("june ", "june "),
    ("just ", "just "),
    ("java ", "java "),
    ("javascript ", "javascript "),
    ("json ", "json "),
];

#[test]
fn j_initial_comprehensive() {
    telex_auto_restore(J_WORDS);
}

#[rstest]
#[case("j", "a", "ja")]
#[case("j", "e", "je")]
#[case("j", "i", "ji")]
#[case("j", "o", "jo")]
#[case("j", "u", "ju")]
#[case("j", "oi", "joi")]
#[case("j", "ob", "job")]
#[case("j", "oy", "joy")]
fn j_initial_vowel_patterns(#[case] initial: &str, #[case] vowel: &str, #[case] expected: &str) {
    let input = format!("{}{} ", initial, vowel);
    let output = format!("{} ", expected);
    telex_auto_restore(&[(&input, &output)]);
}

// ============================================================
// PATTERN 3: INVALID INITIAL Z
// Z is not a valid Vietnamese initial
// ============================================================

const Z_WORDS: &[(&str, &str)] = &[
    ("zero ", "zero "),
    ("zone ", "zone "),
    ("zoom ", "zoom "),
    ("zip ", "zip "),
    ("zoo ", "zoo "),
];

#[test]
fn z_initial_comprehensive() {
    telex_auto_restore(Z_WORDS);
}

#[rstest]
#[case("z", "a", "za")]
#[case("z", "e", "ze")]
#[case("z", "i", "zi")]
#[case("z", "o", "zo")]
#[case("z", "u", "zu")]
#[case("z", "oo", "zoo")]
#[case("z", "one", "zone")]
fn z_initial_vowel_patterns(#[case] initial: &str, #[case] vowel: &str, #[case] expected: &str) {
    let input = format!("{}{} ", initial, vowel);
    let output = format!("{} ", expected);
    telex_auto_restore(&[(&input, &output)]);
}

// ============================================================
// PATTERN 4: W INITIAL + CONSONANT
// W is not a valid Vietnamese initial
// ============================================================

/// W words that SHOULD restore (have consonants that make them invalid Vietnamese)
const W_WORDS: &[(&str, &str)] = &[
    ("water ", "water "),
    // ("way ", "way "), // "way" → "ưay" - may or may not restore depending on validation
    // ("we ", "we "),   // "we" → "ưe" - valid Vietnamese, NOT restored
    ("week ", "week "),
    ("well ", "well "),
    ("west ", "west "),
    ("what ", "what "),
    ("when ", "when "),
    ("where ", "where "),
    ("which ", "which "),
    ("while ", "while "),
    ("white ", "white "),
    ("who ", "who "),
    ("whole ", "whole "),
    ("why ", "why "),
    ("wide ", "wide "),
    ("wife ", "wife "),
    ("will ", "will "),
    ("win ", "win "),
    ("wind ", "wind "),
    ("window ", "window "),
    ("winter ", "winter "),
    ("wish ", "wish "),
    ("with ", "with "),
    ("within ", "within "),
    ("without ", "without "),
    ("woman ", "woman "),
    ("wonder ", "wonder "),
    ("wood ", "wood "),
    ("word ", "word "),
    ("work ", "work "),
    ("worker ", "worker "),
    ("world ", "world "),
    ("worry ", "worry "),
    ("worth ", "worth "),
    ("would ", "would "),
    ("wow ", "wow "),
    ("wrap ", "wrap "),
    ("wrist ", "wrist "),
    ("write ", "write "),
    ("wrong ", "wrong "),
];

#[test]
fn w_initial_comprehensive() {
    telex_auto_restore(W_WORDS);
}

/// W + single vowel (wa, we, wo) produces valid Vietnamese (ưa, ưe, ươ)
/// W + valid Vietnamese vowel patterns
/// Valid: ưa (W+A), ươ (W+O), ưu (W+U)
/// Invalid: ưe (W+E), ưi (W+I), ưy (W+Y) → auto-restore as English
#[test]
fn w_vowel_produces_valid_vietnamese() {
    telex_auto_restore(&[
        ("wa ", "ưa "), // ưa is valid Vietnamese - keep
        ("we ", "we "), // ưe is NOT valid Vietnamese - restore
        ("wi ", "wi "), // ưi is NOT valid Vietnamese - restore
        ("wo ", "ươ "), // ươ is valid Vietnamese - keep
    ]);
}

/// W + valid Vietnamese finals (ng, n, m, c, t, p) produces valid Vietnamese
/// These are NOT auto-restored because they form valid syllables
#[test]
fn w_final_consonant_produces_valid_vietnamese() {
    telex_auto_restore(&[
        // ư + single final
        ("wng ", "ưng "),  // ưng is valid Vietnamese (w→ư + ng final)
        ("uwng ", "ưng "), // uwng also produces ưng (redundant u)
        ("wn ", "ưn "),    // ưn is valid Vietnamese
        ("wm ", "ưm "),    // ưm is valid Vietnamese
        ("wc ", "ưc "),    // ưc is valid Vietnamese
        ("wt ", "ưt "),    // ưt is valid Vietnamese
        ("wp ", "ưp "),    // ưp is valid Vietnamese
        // ươ + finals (w→ư, o→ơ)
        ("wong ", "ương "), // ương
        ("won ", "ươn "),   // ươn
        ("wom ", "ươm "),   // ươm
        ("woc ", "ươc "),   // ươc
        ("wot ", "ươt "),   // ươt
        ("wop ", "ươp "),   // ươp
    ]);
}

/// W + consonant cluster patterns that should restore
#[rstest]
#[case("wh", "a", "wha")]
#[case("wh", "e", "whe")]
#[case("wh", "i", "whi")]
#[case("wh", "o", "who")]
#[case("wr", "a", "wra")]
#[case("wr", "i", "wri")]
#[case("wr", "o", "wro")]
fn w_consonant_patterns(#[case] initial: &str, #[case] vowel: &str, #[case] expected: &str) {
    let input = format!("{}{} ", initial, vowel);
    let output = format!("{} ", expected);
    telex_auto_restore(&[(&input, &output)]);
}

// ============================================================
// PATTERN 5: CONSONANT CLUSTERS
// Vietnamese does not have consonant clusters (except CH, GH, KH, etc.)
// ============================================================

const CLUSTER_WORDS: &[(&str, &str)] = &[
    // bl- cluster
    ("black ", "black "),
    ("blank ", "blank "),
    ("block ", "block "),
    ("blog ", "blog "),
    ("blood ", "blood "),
    ("blue ", "blue "),
    // br- cluster
    ("brain ", "brain "),
    ("branch ", "branch "),
    ("brand ", "brand "),
    ("break ", "break "),
    ("bridge ", "bridge "),
    ("bring ", "bring "),
    ("broad ", "broad "),
    ("brown ", "brown "),
    // cl- cluster
    ("claim ", "claim "),
    ("class ", "class "),
    ("clean ", "clean "),
    ("clear ", "clear "),
    ("click ", "click "),
    ("client ", "client "),
    ("close ", "close "),
    ("cloud ", "cloud "),
    // cr- cluster
    ("crash ", "crash "),
    ("create ", "create "),
    ("credit ", "credit "),
    ("crew ", "crew "),
    ("cross ", "cross "),
    // dr- cluster
    ("draft ", "draft "),
    ("draw ", "draw "),
    ("dream ", "dream "),
    ("drive ", "drive "),
    ("drop ", "drop "),
    // fl- cluster (also invalid F initial)
    ("flag ", "flag "),
    ("flash ", "flash "),
    ("flat ", "flat "),
    ("flex ", "flex "),
    ("flight ", "flight "),
    ("floor ", "floor "),
    ("flow ", "flow "),
    // fr- cluster (also invalid F initial)
    ("frame ", "frame "),
    ("free ", "free "),
    ("fresh ", "fresh "),
    ("friend ", "friend "),
    ("from ", "from "),
    ("front ", "front "),
    // gl- cluster
    ("glass ", "glass "),
    ("global ", "global "),
    // gr- cluster
    ("grade ", "grade "),
    ("grand ", "grand "),
    ("grant ", "grant "),
    ("graph ", "graph "),
    ("great ", "great "),
    ("green ", "green "),
    ("grid ", "grid "),
    ("ground ", "ground "),
    ("group ", "group "),
    ("grow ", "grow "),
    // pl- cluster
    ("place ", "place "),
    ("plan ", "plan "),
    ("plane ", "plane "),
    ("plant ", "plant "),
    ("platform ", "platform "),
    ("play ", "play "),
    ("please ", "please "),
    ("plus ", "plus "),
    // pr- cluster
    ("practice ", "practice "),
    ("present ", "present "),
    ("press ", "press "),
    ("pretty ", "pretty "),
    ("price ", "price "),
    ("print ", "print "),
    ("private ", "private "),
    ("problem ", "problem "),
    ("process ", "process "),
    ("product ", "product "),
    ("program ", "program "),
    ("project ", "project "),
    // sc- cluster
    ("scale ", "scale "),
    ("scan ", "scan "),
    ("scene ", "scene "),
    ("school ", "school "),
    ("science ", "science "),
    ("scope ", "scope "),
    ("score ", "score "),
    ("screen ", "screen "),
    ("script ", "script "),
    // sk- cluster
    ("skill ", "skill "),
    ("skin ", "skin "),
    ("skip ", "skip "),
    ("sky ", "sky "),
    // sl- cluster
    ("sleep ", "sleep "),
    ("slide ", "slide "),
    ("slow ", "slow "),
    // sm- cluster
    ("small ", "small "),
    ("smart ", "smart "),
    // sn- cluster
    ("snap ", "snap "),
    ("snow ", "snow "),
    // sp- cluster
    ("space ", "space "),
    ("span ", "span "),
    ("speak ", "speak "),
    ("special ", "special "),
    ("speed ", "speed "),
    ("spend ", "spend "),
    ("split ", "split "),
    ("sport ", "sport "),
    ("spot ", "spot "),
    ("spread ", "spread "),
    ("spring ", "spring "),
    // st- cluster
    ("staff ", "staff "),
    ("stage ", "stage "),
    ("stand ", "stand "),
    ("standard ", "standard "),
    ("start ", "start "),
    ("state ", "state "),
    ("static ", "static "),
    ("status ", "status "),
    ("stay ", "stay "),
    ("step ", "step "),
    ("still ", "still "),
    ("stock ", "stock "),
    ("stop ", "stop "),
    ("store ", "store "),
    ("story ", "story "),
    ("strategy ", "strategy "),
    ("stream ", "stream "),
    ("street ", "street "),
    ("string ", "string "),
    ("strong ", "strong "),
    ("structure ", "structure "),
    ("student ", "student "),
    ("study ", "study "),
    ("stuff ", "stuff "),
    ("style ", "style "),
    // Note: sw- cluster is special because W acts as horn modifier
    // "sweet" → "sưêt" (W modifies to ư), not auto-restored
    // These are documented separately below
];

#[test]
fn consonant_clusters_comprehensive() {
    telex_auto_restore(CLUSTER_WORDS);
}

#[rstest]
// bl- cluster
#[case("bl", "a", "bla")]
#[case("bl", "e", "ble")]
#[case("bl", "i", "bli")]
#[case("bl", "o", "blo")]
#[case("bl", "u", "blu")]
// br- cluster
#[case("br", "a", "bra")]
#[case("br", "e", "bre")]
#[case("br", "i", "bri")]
#[case("br", "o", "bro")]
// cl- cluster
#[case("cl", "a", "cla")]
#[case("cl", "e", "cle")]
#[case("cl", "i", "cli")]
#[case("cl", "o", "clo")]
// cr- cluster
#[case("cr", "a", "cra")]
#[case("cr", "e", "cre")]
#[case("cr", "o", "cro")]
// dr- cluster
#[case("dr", "a", "dra")]
#[case("dr", "e", "dre")]
#[case("dr", "i", "dri")]
#[case("dr", "o", "dro")]
// gl- cluster
#[case("gl", "a", "gla")]
#[case("gl", "o", "glo")]
// gr- cluster
#[case("gr", "a", "gra")]
#[case("gr", "e", "gre")]
#[case("gr", "i", "gri")]
#[case("gr", "o", "gro")]
// pl- cluster
#[case("pl", "a", "pla")]
#[case("pl", "e", "ple")]
#[case("pl", "u", "plu")]
// pr- cluster
#[case("pr", "a", "pra")]
#[case("pr", "e", "pre")]
#[case("pr", "i", "pri")]
#[case("pr", "o", "pro")]
// sc- cluster
#[case("sc", "a", "sca")]
#[case("sc", "o", "sco")]
// sk- cluster
#[case("sk", "i", "ski")]
#[case("sk", "y", "sky")]
// sl- cluster
#[case("sl", "e", "sle")]
#[case("sl", "o", "slo")]
// sm- cluster
#[case("sm", "a", "sma")]
// sn- cluster
#[case("sn", "a", "sna")]
#[case("sn", "o", "sno")]
// sp- cluster
#[case("sp", "a", "spa")]
#[case("sp", "e", "spe")]
#[case("sp", "o", "spo")]
// st- cluster
#[case("st", "a", "sta")]
#[case("st", "e", "ste")]
#[case("st", "i", "sti")]
#[case("st", "o", "sto")]
#[case("st", "u", "stu")]
// str- cluster
#[case("str", "a", "stra")]
#[case("str", "e", "stre")]
#[case("str", "i", "stri")]
#[case("str", "o", "stro")]
fn cluster_vowel_patterns(#[case] cluster: &str, #[case] vowel: &str, #[case] expected: &str) {
    let input = format!("{}{} ", cluster, vowel);
    let output = format!("{} ", expected);
    telex_auto_restore(&[(&input, &output)]);
}

// ============================================================
// PATTERN 6: MODIFIER + CONSONANT SEQUENCES
// Example: "text" has x (ngã modifier) followed by t (consonant)
// ============================================================

const MODIFIER_CONSONANT_WORDS: &[(&str, &str)] = &[
    // x + consonant (exp-, ext-, exc-)
    ("expand ", "expand "),
    ("expect ", "expect "),
    ("expense ", "expense "),
    ("experience ", "experience "),
    ("experiment ", "experiment "),
    ("expert ", "expert "),
    ("explain ", "explain "),
    ("explicit ", "explicit "),
    ("explode ", "explode "),
    ("exploit ", "exploit "),
    ("explore ", "explore "),
    ("export ", "export "),
    ("express ", "express "),
    ("extend ", "extend "),
    ("extension ", "extension "),
    ("extent ", "extent "),
    ("extern ", "extern "),
    ("external ", "external "),
    ("extra ", "extra "),
    ("extract ", "extract "),
    ("extreme ", "extreme "),
    ("excel ", "excel "),
    ("except ", "except "),
    ("excess ", "excess "),
    ("exchange ", "exchange "),
    ("excite ", "excite "),
    ("exclude ", "exclude "),
    ("excuse ", "excuse "),
    ("execute ", "execute "),
    // x + t (text, next, etc.)
    ("context ", "context "),
    ("next ", "next "),
    ("text ", "text "),
    ("textbook ", "textbook "),
];

#[test]
fn modifier_consonant_comprehensive() {
    telex_auto_restore(MODIFIER_CONSONANT_WORDS);
}

#[rstest]
// exp- pattern (x followed by p)
#[case("exp", "a", "expa")]
#[case("exp", "e", "expe")]
#[case("exp", "i", "expi")]
#[case("exp", "o", "expo")]
// ext- pattern (x followed by t)
#[case("ext", "e", "exte")]
#[case("ext", "r", "extr")]
// exc- pattern (x followed by c)
#[case("exc", "e", "exce")]
#[case("exc", "i", "exci")]
#[case("exc", "u", "excu")]
// text pattern variations
#[case("text", "", "text")]
#[case("next", "", "next")]
fn modifier_consonant_patterns(#[case] prefix: &str, #[case] suffix: &str, #[case] expected: &str) {
    let input = format!("{}{} ", prefix, suffix);
    let output = format!("{} ", expected);
    telex_auto_restore(&[(&input, &output)]);
}

// ============================================================
// PATTERN 7: EI VOWEL PAIR + MODIFIER
// EI is not a valid Vietnamese diphthong
// ============================================================

const EI_WORDS: &[(&str, &str)] = &[
    ("their ", "their "),
    ("weird ", "weird "),
    ("being ", "being "),
    ("either ", "either "),
    ("neither ", "neither "),
    ("ceiling ", "ceiling "),
    ("receive ", "receive "),
    ("perceive ", "perceive "),
    ("conceive ", "conceive "),
    ("deceive ", "deceive "),
    ("seize ", "seize "),
    ("leisure ", "leisure "),
    ("weight ", "weight "),
    ("height ", "height "),
];

#[test]
fn ei_pattern_comprehensive() {
    telex_auto_restore(EI_WORDS);
}

// ============================================================
// PATTERN 8: VOWEL + R/S/F + E (ORE, ARE, URE, IRE patterns)
// These are common English patterns that should restore
// ============================================================

const ORE_ARE_URE_WORDS: &[(&str, &str)] = &[
    // -ore pattern
    ("are ", "are "),
    ("bare ", "bare "),
    ("bore ", "bore "),
    ("care ", "care "),
    ("core ", "core "),
    ("dare ", "dare "),
    ("fare ", "fare "),
    ("fore ", "fore "),
    ("gore ", "gore "),
    ("hare ", "hare "),
    ("lore ", "lore "),
    ("mare ", "mare "),
    ("more ", "more "),
    ("ore ", "oẻ "), // 3-char diphthong: o+r+e → oẻ (Vietnamese)
    ("pore ", "pore "),
    ("rare ", "rare "),
    ("shore ", "shore "),
    ("snore ", "snore "),
    ("sore ", "sore "),
    ("spare ", "spare "),
    ("store ", "store "),
    ("wore ", "wore "),
    ("score ", "score "),
    // -ure pattern
    ("cure ", "cure "),
    ("lure ", "lure "),
    ("pure ", "pure "),
    ("sure ", "sure "),
    ("azure ", "azure "),
    ("endure ", "endure "),
    ("figure ", "figure "),
    ("future ", "future "),
    ("mature ", "mature "),
    ("nature ", "nature "),
    ("picture ", "picture "),
    ("secure ", "secure "),
    ("structure ", "structure "),
    // -ire pattern
    ("dire ", "dire "),
    ("fire ", "fire "),
    ("hire ", "hire "),
    ("tire ", "tire "),
    ("wire ", "wire "),
    ("admire ", "admire "),
    ("desire ", "desire "),
    ("entire ", "entire "),
    ("expire ", "expire "),
    ("inspire ", "inspire "),
    ("require ", "require "),
    ("retire ", "retire "),
];

#[test]
fn ore_are_ure_ire_comprehensive() {
    telex_auto_restore(ORE_ARE_URE_WORDS);
}

#[rstest]
#[case("", "ore", "oẻ")]
#[case("c", "ore", "core")]
#[case("m", "ore", "more")]
#[case("st", "ore", "store")]
#[case("sc", "ore", "score")]
#[case("c", "are", "care")]
#[case("r", "are", "rare")]
#[case("sp", "are", "spare")]
#[case("c", "ure", "cure")]
#[case("p", "ure", "pure")]
#[case("s", "ure", "sure")]
#[case("f", "ire", "fire")]
#[case("h", "ire", "hire")]
#[case("w", "ire", "wire")]
fn vowel_re_patterns(#[case] prefix: &str, #[case] suffix: &str, #[case] expected: &str) {
    let input = format!("{}{} ", prefix, suffix);
    let output = format!("{} ", expected);
    telex_auto_restore(&[(&input, &output)]);
}

// ============================================================
// PATTERN 9: W AS FINAL (NOT MODIFIER)
// W can't modify A in English (raw, law, saw)
// ============================================================

const W_FINAL_WORDS: &[(&str, &str)] = &[
    ("aw ", "aw "),
    ("caw ", "caw "),
    ("claw ", "claw "),
    ("draw ", "draw "),
    ("flaw ", "flaw "),
    ("gnaw ", "gnaw "),
    ("jaw ", "jaw "),
    ("law ", "law "),
    ("paw ", "paw "),
    ("raw ", "raw "),
    ("saw ", "saw "),
    ("straw ", "straw "),
    ("thaw ", "thaw "),
    // -ew pattern
    ("blew ", "blew "),
    ("brew ", "brew "),
    ("chew ", "chew "),
    ("crew ", "crew "),
    ("dew ", "dew "),
    ("drew ", "drew "),
    ("few ", "few "),
    ("flew ", "flew "),
    ("grew ", "grew "),
    ("knew ", "knew "),
    ("mew ", "mew "),
    ("new ", "new "),
    ("pew ", "pew "),
    ("sew ", "sew "),
    ("slew ", "slew "),
    ("stew ", "stew "),
    ("threw ", "threw "),
    ("view ", "view "),
    ("queue ", "quêu "), // qu + eue = invalid Vietnamese vowel pattern
    // -ow pattern: single valid consonant + ow → cơ (Vietnamese ơ vowel)
    // These form valid Vietnamese syllables (consonant + ơ)
    ("bow ", "bơ "), // bơ = butter
    ("cow ", "cơ "), // cơ = muscle/rice
    ("how ", "hơ "), // hơ = to warm by fire
    ("now ", "nơ "), // nơ = bow/ribbon
    ("row ", "rơ "), // rơ = to fall/drop
    ("sow ", "sơ "), // sơ = initial/raw
    ("vow ", "vơ "), // vơ = to grab/gather
    // -ow pattern: invalid initials → restore to English
    ("brow ", "brow "), // br is invalid initial
    ("plow ", "plow "), // pl is invalid initial
    ("wow ", "wow "),   // w is invalid initial (handled by Pattern 1)
];

#[test]
fn w_final_comprehensive() {
    telex_auto_restore(W_FINAL_WORDS);
}

#[rstest]
#[case("r", "aw", "raw")]
#[case("l", "aw", "law")]
#[case("s", "aw", "saw")]
#[case("j", "aw", "jaw")]
#[case("dr", "aw", "draw")]
#[case("cl", "aw", "claw")]
#[case("n", "ew", "new")]
#[case("f", "ew", "few")]
#[case("d", "ew", "dew")]
#[case("vi", "ew", "view")]
#[case("cr", "ew", "crew")]
fn w_final_patterns(#[case] prefix: &str, #[case] suffix: &str, #[case] expected: &str) {
    let input = format!("{}{} ", prefix, suffix);
    let output = format!("{} ", expected);
    telex_auto_restore(&[(&input, &output)]);
}

// ============================================================
// PATTERN 10: OU PATTERN (NOT VALID VIETNAMESE)
// ============================================================

const OU_WORDS: &[(&str, &str)] = &[
    ("about ", "about "),
    ("account ", "account "),
    ("around ", "around "),
    ("bound ", "bound "),
    ("count ", "count "),
    ("doubt ", "doubt "),
    ("found ", "found "),
    ("ground ", "ground "),
    ("hour ", "hour "),
    ("house ", "house "),
    ("loud ", "loud "),
    ("mount ", "mount "),
    ("mouse ", "mouse "),
    ("out ", "out "),
    ("our ", "our "),
    ("pound ", "pound "),
    ("proud ", "proud "),
    ("round ", "round "),
    ("shout ", "shout "),
    ("sound ", "sound "),
    ("south ", "south "),
    ("thousand ", "thousand "),
    ("trouble ", "trouble "),
    ("without ", "without "),
    ("young ", "young "),
    ("your ", "your "),
    ("youth ", "youth "),
];

#[test]
fn ou_pattern_comprehensive() {
    telex_auto_restore(OU_WORDS);
}

// ============================================================
// TECH & PROGRAMMING TERMS
// ============================================================

const TECH_WORDS: &[(&str, &str)] = &[
    // F initial
    ("facebook ", "facebook "),
    ("figma ", "figma "),
    ("firebase ", "firebase "),
    ("firefox ", "firefox "),
    ("flutter ", "flutter "),
    ("framework ", "framework "),
    ("frontend ", "frontend "),
    ("fullstack ", "fullstack "),
    // W initial
    ("webpack ", "webpack "),
    ("websocket ", "websocket "),
    ("wordpress ", "wordpress "),
    // J initial
    ("java ", "java "),
    ("javascript ", "javascript "),
    ("jest ", "jest "),
    ("json ", "json "),
    ("jsx ", "jsx "),
    ("jupyter ", "jupyter "),
    // Clusters
    ("spring ", "spring "),
    ("stream ", "stream "),
    ("string ", "string "),
    ("struct ", "struct "),
    // Modifier patterns
    ("express ", "express "),
    ("extension ", "extension "),
    // Long words with valid structure (should restore due to other patterns)
    ("localhost ", "localhost "),
    ("request ", "request "),
    ("refresh ", "refresh "),
];

#[test]
fn tech_terms_comprehensive() {
    telex_auto_restore(TECH_WORDS);
}

// ============================================================
// DOCUMENTATION: WORDS THAT PRODUCE VALID VIETNAMESE
// These are NOT auto-restored (by design)
// Users should use ESC or raw mode for these
// ============================================================

/// Vietnamese-first: valid Vietnamese buffer stays Vietnamese.
/// User can type double modifier to escape to English (tesst → test).
#[test]
fn vietnamese_first_valid_buffer() {
    telex_auto_restore(&[
        // Single modifier: Vietnamese-first (buffer is valid Vietnamese)
        ("test ", "tét "), // tét is valid Vietnamese → keep
        ("best ", "bét "), // bét is valid Vietnamese → keep
        ("rest ", "rét "), // rét (cold) is valid Vietnamese → keep
        ("nest ", "nét "), // nét (stroke) is valid Vietnamese → keep
        ("cost ", "cót "), // cót is valid Vietnamese → keep
        ("host ", "hót "), // hót (sing) is valid Vietnamese → keep
        ("lost ", "lót "), // lót (line) is valid Vietnamese → keep
        ("most ", "mót "), // mót is valid Vietnamese → keep
        ("post ", "pót "), // pót is valid Vietnamese → keep
        ("docs ", "dóc "), // dóc is valid Vietnamese → keep
    ]);
}

#[test]
fn double_modifier_restores_english() {
    telex_auto_restore(&[
        // Double modifier: reverts tone → English restore
        ("tesst ", "test "), // double s reverts → test in whitelist → restored
        ("besst ", "best "), // double s reverts → best in whitelist → restored
        ("resst ", "rest "), // double s reverts → rest in whitelist → restored
        ("nesst ", "nest "), // double s reverts → nest in whitelist → restored
        ("cosst ", "cost "), // double s reverts → cost in whitelist → restored
        ("hosst ", "host "), // double s reverts → host in whitelist → restored
        ("losst ", "lost "), // double s reverts → lost in whitelist → restored
        ("mosst ", "most "), // double s reverts → most in whitelist → restored
        ("posst ", "post "), // double s reverts → post in whitelist → restored
    ]);
}

/// SW- cluster: consonant + W is English pattern, should restore
/// In Vietnamese, W (horn/breve modifier) must follow a vowel, not a consonant
#[test]
fn sw_cluster_edge_case() {
    telex_auto_restore(&[
        // "sw" → consonant + W → English pattern → restore
        ("sweet ", "sweet "),   // Restores (consonant+W is English)
        ("swim ", "swim "),     // Restores (consonant+W is English)
        ("swing ", "swing "),   // Restores (consonant+W is English)
        ("switch ", "switch "), // Restores (consonant+W is English)
        ("swift ", "swift "),   // Restores (consonant+W is English)
    ]);
}

/// Test: restore → space → backspace → space → expect restore preserved
/// Scenario: After auto-restore, the restored text should persist through
/// space-backspace-space cycles without re-transforming
#[test]
fn restore_space_backspace_space_cycle() {
    telex_auto_restore(&[
        // "restore" → "rếtore" during typing (s adds sắc to e)
        // space → auto-restore to "restore "
        // backspace → "restore" (delete space, word in buffer)
        // space → "restore " (stays restored, not re-transformed)
        ("restore < ", "restore "),
        // SW words
        ("swim < ", "swim "),
        ("sweet < ", "sweet "),
        ("swing < ", "swing "),
    ]);
}

// ============================================================
// PUNCTUATION TRIGGERS
// ============================================================

#[test]
fn punctuation_triggers_restore() {
    telex_auto_restore(&[
        // Comma trigger
        ("text, ", "text, "),
        ("expect, ", "expect, "),
        ("next, ", "next, "),
        // Period trigger
        ("text. ", "text. "),
        ("expect. ", "expect. "),
        ("next. ", "next. "),
    ]);
}

// ============================================================
// CASE SENSITIVITY
// ============================================================

#[test]
fn case_sensitivity_restore() {
    telex_auto_restore(&[
        // Uppercase F initial
        ("Firefox ", "Firefox "),
        ("Facebook ", "Facebook "),
        ("Flutter ", "Flutter "),
        // Uppercase W initial
        ("Windows ", "Windows "),
        ("Webpack ", "Webpack "),
        ("WebSocket ", "WebSocket "),
        // Uppercase J initial
        ("Java ", "Java "),
        ("JavaScript ", "JavaScript "),
        ("JSON ", "JSON "),
        // Mixed case tech terms
        ("TypeScript ", "TypeScript "),
        ("JavaScript ", "JavaScript "),
    ]);
}

// ============================================================
// DOUBLE MARK KEYS (English words with repeated modifier keys)
// When same mark key is pressed twice, both appear as letters
// This allows typing English words like "issue", "class", "error"
// 4-char words with double modifiers keep reverted result (user intended revert)
// 5+ char words restore to English (user typed English word)
// ============================================================

#[test]
fn double_mark_english_words() {
    telex_auto_restore(&[
        // Words with double 's' (sắc mark key) - 5+ chars restore to English
        ("issue ", "issue "),
        ("class ", "class "),
        ("cross ", "cross "),
        ("dress ", "dress "),
        ("glass ", "glass "),
        ("grass ", "grass "),
        ("gross ", "gross "),
        ("press ", "press "),
        ("stress ", "stress "),
        // Words with double 'f' (huyền mark key) - 5+ chars restore to English
        ("staff ", "staff "),
        ("stuff ", "stuff "),
        ("cliff ", "cliff "),
        ("stiff ", "stiff "),
        // Words with double 'r' (hỏi mark key) - 5+ chars restore to English
        ("error ", "error "),
        ("mirror ", "mirror "),
        ("horror ", "horror "),
        ("terror ", "terror "),
    ]);
}

#[test]
fn double_mark_4char_restores_english() {
    // 4-char words ending with double 'ss' or 'ff': restore to English
    // 's' and 'f' are NOT valid Vietnamese final consonants
    // so "bass", "boss", "buff", "cuff", etc. → restore to English
    telex_auto_restore(&[
        // Double ss at end: restore to English (not Vietnamese)
        ("bass ", "bass "),
        ("boss ", "boss "),
        ("loss ", "loss "),
        ("mass ", "mass "),
        ("mess ", "mess "),
        ("miss ", "miss "),
        ("pass ", "pass "),
        ("less ", "less "),
        // Double ff at end: also restore to English (common English pattern)
        ("buff ", "buff "),
        ("cuff ", "cuff "),
        ("puff ", "puff "),
    ]);
}

/// Double modifier in MIDDLE of word: user reverted, buffer is valid word
/// When buffer ends with common suffix (-er, -or) and is valid, keep it
#[test]
fn double_mark_middle_keeps_valid_word() {
    telex_auto_restore(&[
        // "usser" → first 's' adds sắc to 'u', second 's' reverts
        // Buffer = "user" (valid 4-char word ending in -er), keep it
        ("usser ", "user "),
    ]);
}

// ============================================================
// TEST: Tone override patterns should stay Vietnamese
// ============================================================

#[test]
fn tone_override_stays_vietnamese() {
    telex_auto_restore(&[
        // Tone override: j (nặng) → other tones
        ("chajfo ", "chào "), // j→f: nặng → huyền
        ("chajso ", "cháo "), // j→s: nặng → sắc
        ("chajro ", "chảo "), // j→r: nặng → hỏi
        ("chajxo ", "chão "), // j→x: nặng → ngã
        // Tone override: f (huyền) → other tones
        ("chafso ", "cháo "), // f→s: huyền → sắc
        ("chafro ", "chảo "), // f→r: huyền → hỏi
        ("chafxo ", "chão "), // f→x: huyền → ngã
        ("chafjo ", "chạo "), // f→j: huyền → nặng
        // Tone override: s (sắc) → other tones
        ("chasfo ", "chào "), // s→f: sắc → huyền
        ("chasro ", "chảo "), // s→r: sắc → hỏi
        ("chasxo ", "chão "), // s→x: sắc → ngã
        ("chasjo ", "chạo "), // s→j: sắc → nặng
        // Tone override: r (hỏi) → other tones
        ("charfo ", "chào "), // r→f: hỏi → huyền
        ("charso ", "cháo "), // r→s: hỏi → sắc
        ("charxo ", "chão "), // r→x: hỏi → ngã
        ("charjo ", "chạo "), // r→j: hỏi → nặng
        // Tone override: x (ngã) → other tones
        ("chaxfo ", "chào "), // x→f: ngã → huyền
        ("chaxso ", "cháo "), // x→s: ngã → sắc
        ("chaxro ", "chảo "), // x→r: ngã → hỏi
        ("chaxjo ", "chạo "), // x→j: ngã → nặng
        // Delayed circumflex with tone (from previous fix)
        ("vajan ", "vận "), // a→j→a→n pattern
        ("hajan ", "hận "),
        ("cajan ", "cận "),
    ]);
}
