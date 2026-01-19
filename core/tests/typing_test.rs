//! Typing Tests - Real-world typing scenarios, sentences, behaviors

mod common;
use common::{telex, telex_auto_restore, telex_traditional, vni, vni_traditional};

// ============================================================
// BACKSPACE & CORRECTIONS
// ============================================================

const TELEX_BACKSPACE: &[(&str, &str)] = &[
    ("vieet<s", "viế"),
    ("chaof<o", "chào"),
    ("toi<as", "toá"),
    ("a<b", "b"),
    ("ab<<cd", "cd"),
    ("abcd<<<", "a"),
    ("vieets<<<ng", "vng"),
    // Backspace and retype with compound vowels
    ("dduowfng<<sng", "đướng"), // đường with ng deleted, then sắc + ng added back
];

const VNI_BACKSPACE: &[(&str, &str)] = &[("a1<a2", "à"), ("o6<o7", "ơ")];

// ============================================================
// TYPOS & WRONG ORDER
// ============================================================

const TELEX_TYPOS: &[(&str, &str)] = &[
    // Wrong order - mark before vowel
    ("sa", "sa"),
    ("as", "á"),
    // ============================================================
    // OUT-OF-ORDER TONE PLACEMENT - COMPREHENSIVE TEST SUITE
    // Covers ALL vowel patterns from Vietnamese phonology rules
    // Engine should AUTO-CORRECT tone position based on rules
    // ============================================================
    //
    // ============================================================
    // GROUP 1: TONE MOVES TO 2ND VOWEL (Medial + Main patterns)
    // When typing tone on 1st vowel then adding 2nd → tone moves to 2nd
    // ============================================================
    //
    // --- Pattern: oa (medial o + main a) ---
    ("osa", "oá"),   // o + s + a → oá
    ("ofa", "oà"),   // o + f + a → oà
    ("ora", "oả"),   // o + r + a → oả
    ("oxa", "oã"),   // o + x + a → oã
    ("oja", "oạ"),   // o + j + a → oạ
    ("hosa", "hoá"), // with initial: h + o + s + a → hoá
    ("hofa", "hoà"), // with initial: h + o + f + a → hoà
    ("tosa", "toá"), // with initial: t + o + s + a → toá
    ("losa", "loá"), // with initial: l + o + s + a → loá
    //
    // --- Pattern: oă (medial o + main ă) ---
    ("osaw", "oắ"),   // o + s + a + w → oắ (aw = ă)
    ("hosaw", "hoắ"), // h + o + s + a + w → hoắ
    ("xosaw", "xoắ"), // x + o + s + a + w → xoắ
    //
    // --- Pattern: oe (medial o + main e) ---
    ("ose", "oé"),     // o + s + e → oé
    ("ofe", "oè"),     // o + f + e → oè
    ("khose", "khoé"), // kh + o + s + e → khoé
    ("xose", "xoé"),   // x + o + s + e → xoé
    //
    // --- Pattern: uê (medial u + main ê) ---
    ("usee", "uế"),   // u + s + ee → uế
    ("husee", "huế"), // h + u + s + ee → huế
    ("tusee", "tuế"), // t + u + s + ee → tuế
    //
    // --- Pattern: uy (medial u + main y) ---
    ("usy", "uý"),     // u + s + y → uý
    ("qusy", "quý"),   // qu + s + y → quý
    ("husy", "huý"),   // h + u + s + y → huý
    ("tusy", "tuý"),   // t + u + s + y → tuý
    ("thusy", "thuý"), // th + u + s + y → thuý
    //
    // --- Pattern: ua (after q) → qua ---
    ("qusa", "quá"), // q + u + s + a → quá
    ("qufa", "quà"), // q + u + f + a → quà
    ("qura", "quả"), // q + u + r + a → quả
    ("quxa", "quã"), // q + u + x + a → quã
    ("quja", "quạ"), // q + u + j + a → quạ
    //
    // --- Pattern: iê (compound vowel) ---
    ("isee", "iế"),   // i + s + ee → iế
    ("tisee", "tiế"), // t + i + s + ee → tiế
    ("kisee", "kiế"), // k + i + s + ee → kiế
    ("lisee", "liế"), // l + i + s + ee → liế
    //
    // --- Pattern: uô (compound vowel) ---
    ("usoo", "uố"),   // u + s + oo → uố
    ("musoo", "muố"), // m + u + s + oo → muố
    ("cusoo", "cuố"), // c + u + s + oo → cuố
    ("lusoo", "luố"), // l + u + s + oo → luố
    //
    // --- Pattern: ươ (compound vowel) ---
    ("uwsow", "ướ"),   // uw + s + ow → ướ
    ("muwsow", "mướ"), // m + uw + s + ow → mướ
    ("luwsow", "lướ"), // l + uw + s + ow → lướ
    ("suwsow", "sướ"), // s + uw + s + ow → sướ
    //
    // ============================================================
    // GROUP 2: TONE STAYS ON 1ST VOWEL (Main + Glide patterns)
    // When typing tone on 1st vowel then adding glide → tone stays
    // ============================================================
    //
    // --- Pattern: ai (a + glide i) ---
    ("asi", "ái"),   // a + s + i → ái
    ("afi", "ài"),   // a + f + i → ài
    ("ari", "ải"),   // a + r + i → ải
    ("axi", "ãi"),   // a + x + i → ãi
    ("aji", "ại"),   // a + j + i → ại
    ("masi", "mái"), // m + a + s + i → mái
    ("basi", "bái"), // b + a + s + i → bái
    ("tasi", "tái"), // t + a + s + i → tái
    //
    // --- Pattern: ao (a + glide o) ---
    ("aso", "áo"),   // a + s + o → áo
    ("afo", "ào"),   // a + f + o → ào
    ("aro", "ảo"),   // a + r + o → ảo
    ("axo", "ão"),   // a + x + o → ão
    ("ajo", "ạo"),   // a + j + o → ạo
    ("caso", "cáo"), // c + a + s + o → cáo
    ("baso", "báo"), // b + a + s + o → báo
    ("saso", "sáo"), // s + a + s + o → sáo
    //
    // --- Pattern: au (a + glide u) ---
    ("asu", "áu"),   // a + s + u → áu
    ("afu", "àu"),   // a + f + u → àu
    ("aru", "ảu"),   // a + r + u → ảu
    ("axu", "ãu"),   // a + x + u → ãu
    ("aju", "ạu"),   // a + j + u → ạu
    ("sasu", "sáu"), // s + a + s + u → sáu
    ("masu", "máu"), // m + a + s + u → máu
    //
    // --- Pattern: ay (a + glide y) ---
    ("asy", "áy"),   // a + s + y → áy
    ("afy", "ày"),   // a + f + y → ày
    ("ary", "ảy"),   // a + r + y → ảy
    ("axy", "ãy"),   // a + x + y → ãy
    ("ajy", "ạy"),   // a + j + y → ạy
    ("masy", "máy"), // m + a + s + y → máy
    ("hasy", "háy"), // h + a + s + y → háy
    //
    // --- Pattern: âu (â + glide u) ---
    ("aasu", "ấu"),   // aa + s + u → ấu (aa = â)
    ("aafu", "ầu"),   // aa + f + u → ầu
    ("aaru", "ẩu"),   // aa + r + u → ẩu
    ("aaxu", "ẫu"),   // aa + x + u → ẫu
    ("aaju", "ậu"),   // aa + j + u → ậu
    ("daasu", "dấu"), // d + aa + s + u → dấu
    ("caasu", "cấu"), // c + aa + s + u → cấu
    //
    // --- Pattern: ây (â + glide y) ---
    ("aasy", "ấy"),   // aa + s + y → ấy (aa = â)
    ("aafy", "ầy"),   // aa + f + y → ầy
    ("aary", "ẩy"),   // aa + r + y → ẩy
    ("aaxy", "ẫy"),   // aa + x + y → ẫy
    ("aajy", "ậy"),   // aa + j + y → ậy
    ("daasy", "dấy"), // d + aa + s + y → dấy
    ("maasy", "mấy"), // m + aa + s + y → mấy
    //
    // --- Pattern: eo (e + glide o) ---
    ("eso", "éo"),     // e + s + o → éo
    ("efo", "èo"),     // e + f + o → èo
    ("ero", "ẻo"),     // e + r + o → ẻo
    ("exo", "ẽo"),     // e + x + o → ẽo
    ("ejo", "ẹo"),     // e + j + o → ẹo
    ("keso", "kéo"),   // k + e + s + o → kéo
    ("treso", "tréo"), // tr + e + s + o → tréo
    // Issue #48: dd + eso pattern (đ + e + s + o → đéo)
    ("ddeso", "đéo"), // dd + e + s + o → đéo
    ("ddefo", "đèo"), // dd + e + f + o → đèo (đèo = mountain pass)
    ("ddero", "đẻo"), // dd + e + r + o → đẻo
    ("ddexo", "đẽo"), // dd + e + x + o → đẽo
    ("ddejo", "đẹo"), // dd + e + j + o → đẹo
    //
    // --- Pattern: êu (ê + glide u) ---
    ("eesu", "ếu"),   // ee + s + u → ếu (ee = ê)
    ("eefu", "ều"),   // ee + f + u → ều
    ("eeru", "ểu"),   // ee + r + u → ểu
    ("eexu", "ễu"),   // ee + x + u → ễu
    ("eeju", "ệu"),   // ee + j + u → ệu
    ("keesu", "kếu"), // k + ee + s + u → kếu
    ("reesu", "rếu"), // r + ee + s + u → rếu
    //
    // --- Pattern: ia (i + glide a) - descending diphthong ---
    ("isa", "ía"),   // i + s + a → ía
    ("ifa", "ìa"),   // i + f + a → ìa
    ("ira", "ỉa"),   // i + r + a → ỉa
    ("ixa", "ĩa"),   // i + x + a → ĩa
    ("ija", "ịa"),   // i + j + a → ịa
    ("kisa", "kía"), // k + i + s + a → kía
    ("misa", "mía"), // m + i + s + a → mía
    ("tisa", "tía"), // t + i + s + a → tía
    //
    // --- Pattern: iu (i + glide u) ---
    ("isu", "íu"),   // i + s + u → íu
    ("ifu", "ìu"),   // i + f + u → ìu
    ("iru", "ỉu"),   // i + r + u → ỉu
    ("ixu", "ĩu"),   // i + x + u → ĩu
    ("iju", "ịu"),   // i + j + u → ịu
    ("disu", "díu"), // d + i + s + u → díu
    ("kisu", "kíu"), // k + i + s + u → kíu
    //
    // --- Pattern: oi (o + glide i) ---
    ("osi", "ói"),   // o + s + i → ói
    ("ofi", "òi"),   // o + f + i → òi
    ("ori", "ỏi"),   // o + r + i → ỏi
    ("oxi", "õi"),   // o + x + i → õi
    ("oji", "ọi"),   // o + j + i → ọi
    ("dosi", "dói"), // d + o + s + i → dói
    ("nosi", "nói"), // n + o + s + i → nói
    //
    // --- Pattern: ôi (ô + glide i) ---
    ("oosi", "ối"),   // oo + s + i → ối (oo = ô)
    ("oofi", "ồi"),   // oo + f + i → ồi
    ("oori", "ổi"),   // oo + r + i → ổi
    ("ooxi", "ỗi"),   // oo + x + i → ỗi
    ("ooji", "ội"),   // oo + j + i → ội
    ("doosi", "dối"), // d + oo + s + i → dối
    ("toosi", "tối"), // t + oo + s + i → tối
    //
    // --- Pattern: ơi (ơ + glide i) ---
    ("owsi", "ới"),   // ow + s + i → ới (ow = ơ)
    ("owfi", "ời"),   // ow + f + i → ời
    ("owri", "ởi"),   // ow + r + i → ởi
    ("owxi", "ỡi"),   // ow + x + i → ỡi
    ("owji", "ợi"),   // ow + j + i → ợi
    ("bowsi", "bới"), // b + ow + s + i → bới
    ("dowsi", "dới"), // d + ow + s + i → dới
    //
    // --- Pattern: ui (u + glide i) ---
    ("usi", "úi"),   // u + s + i → úi
    ("ufi", "ùi"),   // u + f + i → ùi
    ("uri", "ủi"),   // u + r + i → ủi
    ("uxi", "ũi"),   // u + x + i → ũi
    ("uji", "ụi"),   // u + j + i → ụi
    ("tusi", "túi"), // t + u + s + i → túi
    ("musi", "múi"), // m + u + s + i → múi
    //
    // --- Pattern: ưi (ư + glide i) ---
    ("uwsi", "ứi"),   // uw + s + i → ứi (uw = ư)
    ("uwfi", "ừi"),   // uw + f + i → ừi
    ("uwri", "ửi"),   // uw + r + i → ửi
    ("uwxi", "ữi"),   // uw + x + i → ữi
    ("uwji", "ựi"),   // uw + j + i → ựi
    ("guwsi", "gứi"), // g + uw + s + i → gứi
    //
    // --- Pattern: ưu (ư + glide u) ---
    ("uwsu", "ứu"),   // uw + s + u → ứu (uw = ư)
    ("uwfu", "ừu"),   // uw + f + u → ừu
    ("uwru", "ửu"),   // uw + r + u → ửu
    ("uwxu", "ữu"),   // uw + x + u → ữu
    ("uwju", "ựu"),   // uw + j + u → ựu
    ("luwsu", "lứu"), // l + uw + s + u → lứu
    ("huwsu", "hứu"), // h + uw + s + u → hứu
    //
    // --- Pattern: ua (NOT after q) → u is main ---
    ("musa", "múa"), // m + u + s + a → múa
    ("cusa", "cúa"), // c + u + s + a → cúa
    ("mufa", "mùa"), // m + u + f + a → mùa
    ("mura", "mủa"), // m + u + r + a → mủa
    ("muxa", "mũa"), // m + u + x + a → mũa
    ("muja", "mụa"), // m + u + j + a → mụa
    //
    // --- Pattern: ưa (ư is main, a is glide) ---
    ("uwsa", "ứa"),   // uw + s + a → ứa
    ("uwfa", "ừa"),   // uw + f + a → ừa
    ("uwra", "ửa"),   // uw + r + a → ửa
    ("uwxa", "ữa"),   // uw + x + a → ữa
    ("uwja", "ựa"),   // uw + j + a → ựa
    ("muwsa", "mứa"), // m + uw + s + a → mứa
    ("cuwsa", "cứa"), // c + uw + s + a → cứa
    //
    // ============================================================
    // GROUP 3: TRIPLE VOWELS - TONE ON MIDDLE VOWEL
    // ============================================================
    //
    // --- Pattern: oai (o + a + i) → tone on a ---
    ("osai", "oái"),      // o + s + a + i → oái
    ("hosai", "hoái"),    // h + o + s + a + i → hoái
    ("ngoafif", "ngoài"), // correct existing test
    //
    // --- Pattern: oay (o + a + y) → tone on a ---
    ("osay", "oáy"),   // o + s + a + y → oáy
    ("xosay", "xoáy"), // x + o + s + a + y → xoáy
    //
    // --- Pattern: uôi (u + ô + i) → tone on ô ---
    ("uoosi", "uối"),   // u + oo + s + i → uối
    ("cuoosi", "cuối"), // c + u + oo + s + i → cuối
    ("tuoosi", "tuối"), // t + u + oo + s + i → tuối
    //
    // --- Pattern: ươi (ư + ơ + i) → tone on ơ ---
    ("uwowsi", "ưới"),   // uw + ow + s + i → ưới
    ("muwowsi", "mưới"), // m + uw + ow + s + i → mưới
    ("tuwowsi", "tưới"), // t + uw + ow + s + i → tưới
    //
    // --- Pattern: ươu (ư + ơ + u) → tone on ơ ---
    ("uwowsu", "ướu"),   // uw + ow + s + u → ướu
    ("ruwowsu", "rướu"), // r + uw + ow + s + u → rướu
    ("huwowsu", "hướu"), // h + uw + ow + s + u → hướu
    //
    // --- Pattern: iêu (i + ê + u) → tone on ê ---
    ("ieesu", "iếu"),   // i + ee + s + u → iếu
    ("tieesu", "tiếu"), // t + i + ee + s + u → tiếu
    ("kieesu", "kiếu"), // k + i + ee + s + u → kiếu
    //
    // --- Pattern: yêu (y + ê + u) → tone on ê ---
    ("yeesu", "yếu"), // y + ee + s + u → yếu
    ("yeefu", "yều"), // y + ee + f + u → yều
    //
    // --- Pattern: uây (u + â + y) → tone on â ---
    ("uaasy", "uấy"),     // u + aa + s + y → uấy
    ("khuaasy", "khuấy"), // kh + u + aa + s + y → khuấy
    //
    // Note: oeo triple vowel (khoèo, ngoẹo) has a fundamental conflict with
    // Telex's oo→ô transformation. When typing o-e-o, the second 'o' triggers
    // circumflex on the first 'o'. This is a rare pattern and a known Telex limitation.
    // For oeo words, users typically type the tone AFTER all vowels: "oeos" → but
    // this also conflicts with oo→ô. VNI handles this better with numeric tones.
    //
    // --- Pattern: uyê (u + y + ê) → tone on ê (LAST vowel) ---
    ("uysee", "uyế"),     // u + y + s + ee → uyế
    ("quysee", "quyế"),   // qu + y + s + ee → quyế
    ("khuysee", "khuyế"), // kh + u + y + s + ee → khuyế
    //
    // ============================================================
    // GROUP 4: VV + C (Two vowels + final consonant)
    // Tone always on 2nd vowel when closed syllable
    // ============================================================
    //
    // --- oa + C → tone on a ---
    ("toasn", "toán"),   // t + o + a + s + n → toán
    ("hoasm", "hoám"),   // h + o + a + s + m → hoám
    ("loangs", "loáng"), // l + o + a + n + g + s → loáng? (complex)
    //
    // --- ua + C (with q) → tone on a ---
    ("quasn", "quán"),   // qu + a + s + n → quán
    ("quasng", "quáng"), // qu + a + s + ng → quáng
    //
    // --- ua + C (without q) → tone on a (medial+main pattern) ---
    // With final consonant, 'u' becomes medial, 'a' becomes main vowel
    // Same as "chuẩn" - ch + u + â + n → tone on â
    ("muasn", "muán"), // m + u + a + s + n → muán (main vowel 'a' gets tone)
    //
    // --- iê + C → tone on ê ---
    ("tieesn", "tiến"), // t + i + ee + s + n → tiến
    ("bieesp", "biếp"), // b + i + ee + s + p → biếp? (invalid?)
    //
    // --- uô + C → tone on ô ---
    ("muoosn", "muốn"), // m + u + oo + s + n → muốn
    ("cuoosc", "cuốc"), // c + u + oo + s + c → cuốc
    //
    // --- ươ + C → tone on ơ ---
    ("muwowsn", "mướn"),   // m + uw + ow + s + n → mướn
    ("luwowsng", "lướng"), // l + uw + ow + s + ng → lướng
    //
    // ============================================================
    // GROUP 5: gi- INITIAL (special handling)
    // ============================================================
    //
    ("gisa", "giá"),      // gi + s + a → giá (gi is initial, a is vowel)
    ("giaf", "già"),      // gi + a + f → già
    ("gioosng", "giống"), // gi + oo + s + ng → giống
    //
    // ============================================================
    // EXISTING TEST CASES (preserved)
    // ============================================================
    //
    // Double mark → revert (only reverting key appears - standard IME behavior)
    ("ass", "as"),
    ("aff", "af"),
    ("arr", "ar"),
    // Double tone → revert
    ("aaa", "aa"),
    ("ooo", "oo"),
    ("aww", "aw"),
    // Change mark mid-word
    ("asf", "à"),
    ("afs", "á"),
    // Tone then mark
    ("aas", "ấ"),
    ("ees", "ế"),
    ("oos", "ố"),
    // Mark then tone
    ("asa", "ấ"),
    ("oso", "ố"),
];

// ============================================================
// RAPID TYPING PATTERNS
// ============================================================

const TELEX_RAPID: &[(&str, &str)] = &[
    ("ngoafif", "ngoài"),
    ("nguwowif", "người"),
    // Common words typed fast
    ("truwowngf", "trường"),
    ("dduwowcj", "được"),
    ("suwowngs", "sướng"),
    ("buwowms", "bướm"),
    ("vieetj", "việt"),
    ("tieengs", "tiếng"),
    ("muoons", "muốn"),
    ("cuoocj", "cuộc"),
    ("thuoocj", "thuộc"),
    // ============================================================
    // "ưu" vowel cluster - horn on FIRST u (DELAYED modifier pattern)
    // ============================================================
    // 2-char
    ("uuw", "ưu"), // advantage prefix
    // 3-char (consonant + ưu)
    ("luuw", "lưu"), // save/store
    ("huuw", "hưu"), // retire
    ("suuw", "sưu"), // collect (sưu tầm)
    // 3-char with mark (consonant + ứu/ừu/ửu/ữu/ựu)
    ("cuuws", "cứu"), // rescue
    ("luuws", "lứu"), // (test pattern)
    ("luuwf", "lừu"), // (test pattern)
    ("luuwr", "lửu"), // (test pattern)
    ("luuwx", "lữu"), // (test pattern)
    ("luuwj", "lựu"), // pomegranate
    // 4-char (2-consonant cluster + ưu)
    ("nguuw", "ngưu"), // ox/cow (牛)
    ("khuuw", "khưu"), // (test pattern)
    ("truuw", "trưu"), // (test pattern)
    ("thuuw", "thưu"), // (test pattern)
    // 4-char with mark
    ("nguuws", "ngứu"), // ngứu: delayed pattern
    ("khuuwf", "khừu"), // khừu: delayed pattern
    // 5-char "nghiêu" pattern (ngh + iê + u)
    ("nghieeu", "nghiêu"), // nghiêu: valid ngh + front vowels
    // Alternative typing (inline modifier)
    ("luwu", "lưu"),   // inline
    ("cuwus", "cứu"),  // inline with mark
    ("nguwu", "ngưu"), // inline 4-char
];

const VNI_RAPID: &[(&str, &str)] = &[
    ("ngu7o72i2", "người"),
    ("to6i1", "tối"),
    // "ưu" vowel cluster - horn on FIRST u (not second)
    ("luu7", "lưu"),  // save/store - DELAYED modifier
    ("lu7u", "lưu"),  // inline modifier
    ("uu7", "ưu"),    // advantage prefix - DELAYED
    ("luu71", "lứu"), // with mark - DELAYED
    ("huu7", "hưu"),  // retire - DELAYED
    ("cuu71", "cứu"), // save/rescue - DELAYED
];

// ============================================================
// CAPITALIZATION
// ============================================================

const TELEX_CAPS: &[(&str, &str)] = &[
    ("viEets", "viẾt"),
    ("VIEETJ", "VIỆT"),
    ("VIEETS", "VIẾT"),
    ("DDUWOWNGF", "ĐƯỜNG"),
    ("DDUWOWCJ", "ĐƯỢC"),
    ("TRUWOWNGF", "TRƯỜNG"),
    ("NGUWOWIF", "NGƯỜI"),
];

const VNI_CAPS: &[(&str, &str)] = &[
    ("VIE65T", "VIỆT"),
    ("D9U7O7NG2", "ĐƯỜNG"),
    ("D9U7O7C5", "ĐƯỢC"),
    ("TRU7O7NG2", "TRƯỜNG"),
];

// ============================================================
// GREETINGS
// ============================================================

const TELEX_GREETINGS: &[(&str, &str)] = &[
    ("xin chaof", "xin chào"),
    ("tamj bieetj", "tạm biệt"),
    ("camr own", "cảm ơn"),
    ("xin looxix", "xin lỗi"),
];

const VNI_GREETINGS: &[(&str, &str)] = &[
    ("xin cha2o", "xin chào"),
    ("ta5m bie65t", "tạm biệt"),
    ("ca3m o7n", "cảm ơn"),
];

// ============================================================
// PROVERBS (TỤC NGỮ)
// ============================================================

const TELEX_PROVERBS: &[(&str, &str)] = &[
    ("hocj mootj bieets muwowif", "học một biết mười"),
    (
        "ddi mootj ngayf ddangf hocj mootj sangf khoon",
        "đi một ngày đàng học một sàng khôn",
    ),
    ("toots goox hown ddepj nguwowif", "tốt gỗ hơn đẹp người"),
    ("uoongs nuwowcs nhows nguoonf", "uống nước nhớ nguồn"),
    ("nuwowcs chayr ddas monf", "nước chảy đá mòn"),
];

const VNI_PROVERBS: &[(&str, &str)] = &[
    ("ho5c mo65t bie61t mu7o7i2", "học một biết mười"),
    ("uo61ng nu7o71c nho71 nguo62n", "uống nước nhớ nguồn"),
    ("to61t go64 ho7n d9e5p ngu7o7i2", "tốt gỗ hơn đẹp người"),
    ("nu7o71c cha3y d9a1 mo2n", "nước chảy đá mòn"),
];

// ============================================================
// IDIOMS (THÀNH NGỮ)
// ============================================================

const TELEX_IDIOMS: &[(&str, &str)] = &[
    ("an cuw lacj nghieepj", "an cư lạc nghiệp"),
    ("ddoongf taam hieepj luwcj", "đồng tâm hiệp lực"),
    ("thowif gian laf tieenf bacj", "thời gian là tiền bạc"),
];

// ============================================================
// DAILY CONVERSATIONS
// ============================================================

const TELEX_DAILY: &[(&str, &str)] = &[
    (
        "hoom nay thowif tieets thees naof",
        "hôm nay thời tiết thế nào",
    ),
    ("banj ddi ddaau vaayj", "bạn đi đâu vậy"),
    ("tooi ddang ddi lafm", "tôi đang đi làm"),
    ("mootj ly caf phee nhes", "một ly cà phê nhé"),
    ("bao nhieeu tieenf", "bao nhiêu tiền"),
];

const VNI_DAILY: &[(&str, &str)] = &[
    (
        "ho6m nay tho7i2 tie61t the61 na2o",
        "hôm nay thời tiết thế nào",
    ),
    ("ba5n d9i d9a6u va65y", "bạn đi đâu vậy"),
    ("bao nhie6u tie62n", "bao nhiêu tiền"),
];

// ============================================================
// FOOD
// ============================================================

const TELEX_FOOD: &[(&str, &str)] = &[
    ("cho tooi xem thuwcj ddown", "cho tôi xem thực đơn"),
    (
        "tooi muoons goij mootj phaanf phowr",
        "tôi muốn gọi một phần phở",
    ),
    ("ddoof awn raats ngon", "đồ ăn rất ngon"),
    ("tinhs tieenf nhes", "tính tiền nhé"),
];

// ============================================================
// EXPRESSIONS
// ============================================================

const TELEX_EXPRESSIONS: &[(&str, &str)] = &[
    ("khoong sao", "không sao"),
    ("dduwowcj roofif", "được rồi"),
    ("binhf thuwowngf", "bình thường"),
    ("sao cungx dduwowcj", "sao cũng được"),
    ("tuyeetj vowif", "tuyệt vời"),
    ("ddepj quas", "đẹp quá"),
];

// ============================================================
// POETRY (TRUYỆN KIỀU)
// ============================================================

const TELEX_POETRY: &[(&str, &str)] = &[
    (
        "trawm nawm trong coix nguwowif ta",
        "trăm năm trong cõi người ta",
    ),
    (
        "chuwx taif chuwx meenhj kheos laf ghets nhau",
        "chữ tài chữ mệnh khéo là ghét nhau",
    ),
];

// ============================================================
// LONG SENTENCES
// ============================================================

const TELEX_LONG: &[(&str, &str)] = &[
    (
        "vieetj nam laf mootj quoocs gia nawmf owr ddoong nam as",
        "việt nam là một quốc gia nằm ở đông nam á",
    ),
    (
        "thur ddoo cura vieetj nam laf thanhf phoos haf nooij",
        "thủ đô của việt nam là thành phố hà nội",
    ),
];

const VNI_LONG: &[(&str, &str)] = &[
    (
        "vie65t nam la2 mo65t quo61c gia na82m o73 d9o6ng nam a1",
        "việt nam là một quốc gia nằm ở đông nam á",
    ),
    (
        "thu3 d9o6 cu3a vie65t nam la2 tha2nh pho61 ha2 no65i",
        "thủ đô của việt nam là thành phố hà nội",
    ),
];

// ============================================================
// MIXED CASE SENTENCES
// ============================================================

const TELEX_MIXED_CASE: &[(&str, &str)] = &[
    ("Xin chaof", "Xin chào"),
    ("Vieetj Nam", "Việt Nam"),
    ("VIEETJ NAM", "VIỆT NAM"),
    ("Thanhf phoos Hoof Chis Minh", "Thành phố Hồ Chí Minh"),
];

const VNI_MIXED_CASE: &[(&str, &str)] = &[
    ("Xin cha2o", "Xin chào"),
    ("Vie65t Nam", "Việt Nam"),
    ("Tha2nh pho61 Ho62 Chi1 Minh", "Thành phố Hồ Chí Minh"),
];

// ============================================================
// COMMON ISSUES - Real bugs found in production
// ============================================================

const TELEX_COMMON_ISSUES: &[(&str, &str)] = &[
    // Issue 2.1: Dính chữ (aa -> aâ instead of â)
    ("aa", "â"),
    ("ee", "ê"),
    ("oo", "ô"),
    ("dd", "đ"),
    ("DD", "Đ"),
    // Issue 2.4: Lặp chữ (được -> đđược)
    ("dduwowcj", "được"),
    // Issue #14: Alternative typing with wo → ươ compound
    ("ddwocj", "được"),
    // Issue #29: uw + o should form ươ compound (u with horn + o → ươ)
    // Pattern: ư + o + consonant
    ("dduwocj", "được"),
    ("nuwocs", "nước"),
    ("suwongs", "sướng"),
    ("truwongf", "trường"),
    // Pattern: ư + o + i (triphthong ươi)
    ("nguwoif", "người"),
    ("muwoif", "mười"),
    ("tuwoir", "tưởi"),
    // Pattern: w alone creates ư, then o forms ươ
    ("nwocj", "nược"),
    ("swongs", "sướng"),
    ("bwomf", "bườm"),
    ("twoir", "tưởi"),
    // Edge case: ươ without final consonant (open syllable)
    ("ruwouj", "rượu"),
    ("ddif", "đì"),
    ("ddi", "đi"),
    ("ddang", "đang"),
    ("ddaauf", "đầu"),
    // Issue 2.4: Mất dấu (trường -> trương)
    ("truwowngf", "trường"),
    ("dduwowngf", "đường"),
    ("nguwowif", "người"),
    ("muwowif", "mười"),
    // Letter vs modifier ambiguity
    ("sa", "sa"),
    ("as", "á"),
    ("sas", "sá"),
    ("sass", "sas"), // first s modifier for sắc, second s reverts + outputs one s
    ("fa", "fa"),
    ("af", "à"),
    // Long compound words
    ("nghieeng", "nghiêng"),
    ("khuyeens", "khuyến"),
    ("nguoongf", "nguồng"),
    // ươu triphthong (hươu = deer)
    ("huouw", "hươu"),
    ("ruwowuj", "rượu"),
];

const VNI_COMMON_ISSUES: &[(&str, &str)] = &[
    // Not sticky
    ("a6", "â"),
    ("e6", "ê"),
    ("o6", "ô"),
    ("d9", "đ"),
    ("D9", "Đ"),
    // No double đ
    ("d9u7o7c5", "được"),
    ("d9i", "đi"),
    ("d9ang", "đang"),
    // Preserve tone mark
    ("tru7o7ng2", "trường"),
    ("d9u7o7ng2", "đường"),
    ("ngu7o7i2", "người"),
    // Real words with ươ
    ("nu7o7c1", "nước"),
    ("bu7o7m1", "bướm"),
    ("su7o7ng1", "sướng"),
    ("lu7o7ng2", "lường"),
    ("thu7o7ng2", "thường"),
    ("hu7o7ng1", "hướng"),
    ("vu7o7n2", "vườn"),
    // Real words with ua vs qua
    ("mua2", "mùa"),
    ("chua1", "chúa"),
    ("rua2", "rùa"),
    ("lua1", "lúa"),
    ("su7a4", "sữa"),
    ("qua1", "quá"),
    ("qua3", "quả"),
    ("qua2", "quà"),
    // Real words with iê
    ("vie65t", "việt"),
    ("tie61ng", "tiếng"),
    ("bie63n", "biển"),
    ("mie61ng", "miếng"),
    ("die64n", "diễn"),
    ("kie63m", "kiểm"),
    ("tie62n", "tiền"),
    ("hie63u", "hiểu"),
    // Mixed common words
    ("co1", "có"),
    ("kho6ng", "không"),
    ("la2", "là"),
    ("d9i", "đi"),
    ("ve62", "về"),
    ("a8n", "ăn"),
    ("o6ng1", "ống"),
    ("ba2n", "bàn"),
    ("nha2", "nhà"),
    ("hoc5", "học"),
    // ươu triphthong (hươu = deer)
    ("huou7", "hươu"),
    ("ruo7u5", "rượu"),
];

// ============================================================
// MARK REPOSITIONING - Complex diacritic interactions
// ============================================================

const VNI_MARK_REPOSITION: &[(&str, &str)] = &[
    // ua patterns
    ("ua27", "ừa"),
    ("ua2", "ùa"),
    ("ua7", "ưa"),
    // oa patterns
    ("oa26", "oầ"),
    ("o6a2", "ồa"),
    ("oa2", "oà"),
    // uo compound with marks - Issue #133: only 'o' gets horn when no final
    ("uo71", "uớ"), // Issue #133: only 'o' gets horn
    ("uo72", "uờ"),
    ("uo73", "uở"),
    ("uo74", "uỡ"),
    ("uo75", "uợ"),
    ("uo17", "uớ"),
    ("uo27", "uờ"),
    // When 'u' already has horn (u7...), 'o' gets horn too
    ("u7o71", "ướ"),
    ("u7o72", "ườ"),
    // ua vs qua
    ("ua1", "úa"),
    ("ua2", "ùa"),
    ("qua1", "quá"),
    ("qua2", "quà"),
    ("u7a1", "ứa"),
    ("u7a2", "ừa"),
    ("ua17", "ứa"),
    ("ua27", "ừa"),
];

const TELEX_MARK_REPOSITION: &[(&str, &str)] = &[
    // ua pattern: when U has mark, horn goes to U (not breve on A)
    ("uafw", "ừa"), // uaf → ùa, then w → ừa (horn on U)
    ("uwaf", "ừa"),
    ("oafw", "oằ"),
    // ươ compound - when 'u' already has horn (uwow...), 'o' gets horn too
    ("uwows", "ướ"),
    ("uwowf", "ườ"),
    ("uwowr", "ưở"),
    ("uwowx", "ưỡ"),
    ("uwowj", "ượ"),
    // Issue #133: when 'u' doesn't have horn yet (uow...), only 'o' gets horn
    ("uows", "uớ"),
    ("uowf", "uờ"),
    // Real words
    ("nuwowcs", "nước"),
    ("buwowms", "bướm"),
    ("suwowngs", "sướng"),
    ("luwowngf", "lường"),
    ("dduwowngf", "đường"),
    ("truwowngf", "trường"),
    ("thuwowngf", "thường"),
    ("huwowngs", "hướng"),
    ("vuwownf", "vườn"),
    // ua vs qua
    ("muaf", "mùa"),
    ("chuas", "chúa"),
    ("chuwa", "chưa"), // horn on u, not breve on a
    ("thuwa", "thưa"), // horn on u
    ("muwa", "mưa"),   // rain - horn on u
    ("ruaf", "rùa"),
    ("luas", "lúa"),
    ("suwax", "sữa"),
    ("quas", "quá"),
    ("quar", "quả"),
    ("quaf", "quà"),
    // iê words
    ("vieetj", "việt"),
    ("tieengs", "tiếng"),
    ("bieenr", "biển"),
    ("mieengs", "miếng"),
    ("dieenx", "diễn"),
    ("kieemr", "kiểm"),
    ("tieenf", "tiền"),
    ("hieeur", "hiểu"),
];

// ============================================================
// DELAYED INPUT PATTERNS
// ============================================================

const TELEX_DELAYED_PATTERNS: &[(&str, &str)] = &[
    ("tungw", "tưng"),
    ("tongw", "tơng"),
    ("tangw", "tăng"),
    ("tuow", "tuơ"),     // Issue #133: only 'o' gets horn when no final
    ("nguoiw", "ngươi"), // Has 'i' after 'o', so both get horn
    // ua + w -> ưa (horn on u, not breve on a)
    ("chuaw", "chưa"),
    ("thuaw", "thưa"),
    ("muaw", "mưa"),
];

const VNI_DELAYED_PATTERNS: &[(&str, &str)] = &[
    ("tung7", "tưng"),
    ("tong7", "tơng"),
    ("tang8", "tăng"),
    // VNI allows delayed stroke - '9' is always intentional, not a letter
    // All positions of '9' should work
    ("d9ung", "đung"),
    ("du9ng", "đung"),
    ("dung9", "đung"),
    ("D9ung", "Đung"),
    ("Du9ng", "Đung"),
    ("Dung9", "Đung"),
];

// ============================================================
// DELAYED TONE MARKS - Typing tone after completing the word
// ============================================================

const TELEX_DELAYED_TONE: &[(&str, &str)] = &[
    // Single vowel words - tone at end
    ("bas", "bá"),
    ("caf", "cà"),
    ("mar", "mả"),
    ("lax", "lã"),
    ("taj", "tạ"),
    // au diphthong with tone between vowels
    ("mafu", "màu"), // m + a + f + u → màu (tone before second vowel)
    // oa/oe patterns - tone on second vowel
    ("hoaf", "hoà"),
    ("loas", "loá"),
    ("hoej", "hoẹ"),
    // ai/ao/au patterns - tone on first vowel
    ("mais", "mái"),
    ("laof", "lào"),
    ("daur", "dảu"),
    ("tais", "tái"),
    ("caof", "cào"),
    ("baur", "bảu"),
    // Common single vowel words with delayed tone
    ("lams", "lám"),
    ("lamf", "làm"),
    ("cons", "cón"),
    ("bonf", "bòn"),
    // Words with final consonant - tone on vowel before final
    ("bangs", "báng"),
    ("dangf", "dàng"),
    ("mangr", "mảng"),
    ("tangx", "tãng"),
    ("sangj", "sạng"),
    // Words with circumflex (ô/ê/â) - need double vowel first
    ("khoongf", "khồng"),
    ("hoongf", "hồng"),
    ("coongs", "cống"),
    ("toongs", "tống"),
    // Words with horn (ư/ơ) - uw/ow
    ("tuwf", "từ"),
    ("cuwf", "cừ"),
    ("tows", "tớ"),
    ("howf", "hờ"),
];

const VNI_DELAYED_TONE: &[(&str, &str)] = &[
    // Single vowel words - tone at end
    ("ba1", "bá"),
    ("ca2", "cà"),
    ("ma3", "mả"),
    ("la4", "lã"),
    ("ta5", "tạ"),
    // oa/oe patterns - tone on second vowel
    ("hoa2", "hoà"),
    ("loa1", "loá"),
    ("hoe5", "hoẹ"),
    // ai/ao/au patterns - tone on first vowel
    ("mai1", "mái"),
    ("lao2", "lào"),
    ("dau3", "dảu"),
    ("tai1", "tái"),
    ("cao2", "cào"),
    ("bau3", "bảu"),
    // Common single vowel words with delayed tone
    ("lam1", "lám"),
    ("lam2", "làm"),
    ("con1", "cón"),
    ("bon2", "bòn"),
    // Words with final consonant - tone on vowel before final
    ("bang1", "báng"),
    ("dang2", "dàng"),
    ("mang3", "mảng"),
    ("tang4", "tãng"),
    ("sang5", "sạng"),
    // Words with circumflex (ô/ê/â) - need mark first (6)
    ("khong62", "khồng"),
    ("hong62", "hồng"),
    ("cong61", "cống"),
    ("tong61", "tống"),
    // Words with horn (ư/ơ) - need mark (7)
    ("tu72", "từ"),
    ("cu72", "cừ"),
    ("to71", "tớ"),
    ("ho72", "hờ"),
    // VNI delayed stroke - '9' is always intentional stroke command
    // All positions of '9' should work for đường
    ("d9uong72", "đường"),  // d9 + uong + 7 (horn) + 2 (huyền)
    ("du9ong72", "đường"),  // d + u9 + ong + 7 + 2
    ("duo9ng72", "đường"),  // d + uo + 9 + ng + 7 + 2
    ("duon9g72", "đường"),  // d + uon + 9 + g + 7 + 2
    ("duong972", "đường"),  // d + uong + 9 + 7 + 2
    ("truong72", "trường"), // no đ, just ươ compound
    ("nuoc71", "nước"),     // no đ
    ("nguoi72", "người"),   // no đ
    // đ with adjacent d9
    ("d9i", "đi"),
    ("d9o1", "đó"),
    ("d9ang2", "đàng"),
];

// ============================================================
// SWITCHING DIACRITICS - circumflex ↔ horn
// ============================================================

const VNI_SWITCH_DIACRITICS: &[(&str, &str)] = &[
    // Single vowel: switch circumflex ↔ horn (last modifier wins)
    ("o67", "ơ"),  // ô + 7 → ơ (switch to horn)
    ("o76", "ô"),  // ơ + 6 → ô (switch to circumflex)
    ("o676", "ô"), // multiple switches, last wins
    ("o767", "ơ"), // multiple switches, last wins
    // Compound vowel: switch between ươ and uô
    ("uo76", "uô"), // ươ + 6 → uô (switch to circumflex)
    ("uo67", "ươ"), // uô + 7 → ươ (switch to horn)
    // Real words: last modifier wins
    ("buong76", "buông"),  // buơng + 6 → buông (last is circumflex)
    ("buong67", "buơng"),  // buông + 7 → buơng (last is horn)
    ("buong767", "buơng"), // can switch multiple times
    ("buong676", "buông"), // can switch multiple times
];

const TELEX_SWITCH_DIACRITICS: &[(&str, &str)] = &[
    // Single vowel: switch circumflex ↔ horn (last modifier wins)
    ("oow", "ơ"),  // ô + w → ơ (switch to horn)
    ("owo", "ô"),  // ơ + o → ô (switch to circumflex)
    ("oowo", "ô"), // multiple switches, last wins
    ("owow", "ơ"), // multiple switches, last wins
    // Compound vowel: switch between ươ and uô
    ("uowo", "uô"), // ươ + o → uô (switch to circumflex)
    ("uoow", "ươ"), // uô + w → ươ (switch to horn)
];

// ============================================================
// TEST FUNCTIONS
// ============================================================

#[test]
fn telex_backspace() {
    telex(TELEX_BACKSPACE);
}

#[test]
fn vni_backspace() {
    vni(VNI_BACKSPACE);
}

#[test]
fn telex_typos() {
    telex(TELEX_TYPOS);
}

#[test]
fn telex_rapid_typing() {
    telex(TELEX_RAPID);
}

#[test]
fn vni_rapid_typing() {
    vni(VNI_RAPID);
}

#[test]
fn telex_capitalization() {
    telex(TELEX_CAPS);
}

#[test]
fn vni_capitalization() {
    vni(VNI_CAPS);
}

#[test]
fn telex_greetings() {
    telex(TELEX_GREETINGS);
}

#[test]
fn vni_greetings() {
    vni(VNI_GREETINGS);
}

#[test]
fn telex_proverbs() {
    telex(TELEX_PROVERBS);
}

#[test]
fn vni_proverbs() {
    vni(VNI_PROVERBS);
}

#[test]
fn telex_idioms() {
    telex(TELEX_IDIOMS);
}

#[test]
fn telex_daily_conversations() {
    telex(TELEX_DAILY);
}

#[test]
fn vni_daily_conversations() {
    vni(VNI_DAILY);
}

#[test]
fn telex_food() {
    telex(TELEX_FOOD);
}

#[test]
fn telex_expressions() {
    telex(TELEX_EXPRESSIONS);
}

#[test]
fn telex_poetry() {
    telex(TELEX_POETRY);
}

#[test]
fn telex_long_sentences() {
    telex(TELEX_LONG);
}

#[test]
fn vni_long_sentences() {
    vni(VNI_LONG);
}

#[test]
fn telex_mixed_case() {
    telex(TELEX_MIXED_CASE);
}

#[test]
fn vni_mixed_case() {
    vni(VNI_MIXED_CASE);
}

#[test]
fn telex_common_issues() {
    telex(TELEX_COMMON_ISSUES);
}

#[test]
fn vni_common_issues() {
    vni(VNI_COMMON_ISSUES);
}

#[test]
fn vni_mark_repositioning() {
    vni(VNI_MARK_REPOSITION);
}

#[test]
fn telex_mark_repositioning() {
    telex(TELEX_MARK_REPOSITION);
}

#[test]
fn telex_delayed_patterns() {
    telex(TELEX_DELAYED_PATTERNS);
}

#[test]
fn vni_delayed_patterns() {
    vni(VNI_DELAYED_PATTERNS);
}

#[test]
fn telex_delayed_tone() {
    telex(TELEX_DELAYED_TONE);
}

#[test]
fn vni_delayed_tone() {
    vni(VNI_DELAYED_TONE);
}

#[test]
fn telex_switch_diacritics() {
    telex(TELEX_SWITCH_DIACRITICS);
}

#[test]
fn vni_switch_diacritics() {
    vni(VNI_SWITCH_DIACRITICS);
}

// ============================================================
// NON-ADJACENT STROKE - Issue #51
// ============================================================
//
// Telex stroke behavior:
// - dd → đ (adjacent stroke, always works)
// - d + vowel + d → deferred (open syllable, waits for mark key)
// - d + vowel + consonant + d → đ + vowel + consonant (has final, immediate stroke)
//
// Delayed stroke is DEFERRED for open syllables to prevent "dede" → "đê"
// Only when a mark key (s,f,r,x,j) is typed does the stroke + mark apply together.
// This enables "dods" → "đó" while preventing "dedicated" → "đeicated".

const TELEX_NON_ADJACENT_STROKE: &[(&str, &str)] = &[
    // English words with invalid Vietnamese vowel patterns stay unchanged
    // (ea, io, etc. are NOT valid Vietnamese diphthongs)
    ("deadline", "deadline"),
    ("dedicated", "dedicated"),
    ("decided", "decided"),
    // Open syllables with tone keys (a, e, o, w) - tone applies, enabling fast typing
    // This allows "dodo" → "đô" when user types d-o-d-o quickly
    ("dede", "đê"), // Short stroke + circumflex
    ("dada", "đâ"), // Short stroke + circumflex
    ("dodo", "đô"), // Short stroke + circumflex
    // Mixed: adjacent dd at start
    ("ddead", "đead"),           // dd at start is adjacent → đ, then "ead"
    ("ddedicated", "đedicated"), // dd at start
    // Note: "deadd" → "deadd" because "dead" is invalid (d not a valid final),
    // so even though 5th d is adjacent to 4th d, validation fails
    ("deadd", "deadd"),
];

const VNI_NON_ADJACENT_STROKE: &[(&str, &str)] = &[
    // In VNI, d9 → đ only when '9' immediately follows 'd'
    // "deadline" has no '9', so it stays unchanged
    ("deadline", "deadline"),
    ("dedicated", "dedicated"),
    ("d9eadline", "đeadline"), // d9 at start → đ
];

#[test]
fn telex_non_adjacent_stroke() {
    telex(TELEX_NON_ADJACENT_STROKE);
}

#[test]
fn vni_non_adjacent_stroke() {
    vni(VNI_NON_ADJACENT_STROKE);
}

// ============================================================
// INVALID BREVE PATTERNS - Issue #44
// ============================================================
//
// Vietnamese phonology: 'ă' (breve) requires final consonant
// Valid: trăm, năm, răng, xăng, bắt, căn, ...
// Invalid: ră, să, lă, dă (no final consonant)
//
// These patterns should NOT be transformed because they result
// in invalid Vietnamese syllables.

const TELEX_INVALID_BREVE_OPEN: &[(&str, &str)] = &[
    // Breve always applies immediately for "aw" pattern (valid Vietnamese spelling)
    ("raw", "ră"),
    ("saw", "să"),
    ("law", "lă"),
    ("daw", "dă"),
    ("taw", "tă"),
    ("naw", "nă"),
    ("maw", "mă"),
    ("caw", "că"),
    ("baw", "bă"),
    ("haw", "hă"),
    ("gaw", "gă"),
    ("vaw", "vă"),
    ("xaw", "xă"),
    // Two consonant initials + aw
    ("thaw", "thă"),
    ("chaw", "chă"),
    ("nhaw", "nhă"),
    ("khaw", "khă"),
    ("phaw", "phă"),
    ("traw", "tră"),
    ("ngaw", "ngă"),
    // Invalid Vietnamese spelling - breve blocked by validation
    ("kaw", "kaw"), // k before a is invalid
    // Standalone aw
    ("aw", "ă"),
];

const VNI_INVALID_BREVE_OPEN: &[(&str, &str)] = &[
    // Breve always applies immediately for "a8" pattern
    // English auto-restore handles English words separately when space is typed
    ("ra8", "ră"),
    ("sa8", "să"),
    ("la8", "lă"),
    ("ta8", "tă"),
    ("na8", "nă"),
    ("ma8", "mă"),
    ("ca8", "că"),
    ("ba8", "bă"),
    ("da8", "dă"),
    // Two consonant initials
    ("tha8", "thă"),
    ("tra8", "tră"),
    ("nga8", "ngă"),
    // Standalone a8
    ("a8", "ă"),
];

// Valid breve patterns - with final consonant (should transform)
const TELEX_VALID_BREVE: &[(&str, &str)] = &[
    // These ARE valid because they have final consonants
    ("trawm", "trăm"),    // trăm - hundred (no tone)
    ("nawm", "năm"),      // năm - year/five (no tone)
    ("rawng", "răng"),    // răng - tooth (no tone)
    ("xawng", "xăng"),    // xăng - gasoline (no tone)
    ("bawts", "bắt"),     // bắt - catch (s = sắc tone)
    ("cawn", "căn"),      // căn - room (no tone)
    ("dawngr", "dẳng"),   // dẳng - (r = hỏi tone)
    ("hawng", "hăng"),    // hăng - eager (no tone)
    ("lawng", "lăng"),    // lăng - mausoleum (no tone)
    ("sawcs", "sắc"),     // sắc - sharp (s = sắc tone)
    ("tawngs", "tắng"),   // tắng - (s = sắc tone)
    ("nawngs", "nắng"),   // nắng - sunny (s = sắc tone)
    ("vawngs", "vắng"),   // vắng - absent (s = sắc tone)
    ("mawts", "mắt"),     // mắt - eye (s = sắc tone)
    ("thawngs", "thắng"), // thắng - win (s = sắc tone)
    ("khawcs", "khắc"),   // khắc - to carve (s = sắc tone)
    // Multi-syllable words
    ("trawm nawm", "trăm năm"),    // trăm năm (no tones)
    ("sawngx sangf", "sẵng sàng"), // sẵng sàng (sawngx = sẵng, sangf = sàng)
    // Ethnic minority language place names (issue #134)
    // Vietnamese province names use breve patterns: Đắk Lắk, Đắk Nông
    ("ddawks", "đắk"),            // đắk - lowercase
    ("Ddawks", "Đắk"),            // Đắk - capitalized (first D caps)
    ("DDawks", "Đắk"),            // Đắk - DD pattern
    ("Lawks", "Lắk"),             // Lắk - capitalized
    ("lawks", "lắk"),             // lắk - lowercase
    ("Ddawks Lawks", "Đắk Lắk"),  // Đắk Lắk - full province name
    ("Ddawks Noong", "Đắk Nông"), // Đắk Nông province (oo = ô)
    // Kr initial for ethnic minority words (Krông Búk district)
    ("Kroong", "Krông"),          // Krông - Kr initial + ô
    ("Busk", "Búk"),              // Búk - B + ú + k
    ("Kroong Busk", "Krông Búk"), // Krông Búk - full district name
];

const VNI_VALID_BREVE: &[(&str, &str)] = &[
    // These ARE valid because they have final consonants
    ("tra8m", "trăm"),  // trăm - hundred
    ("na8m", "năm"),    // năm - year/five
    ("ra8ng", "răng"),  // răng - tooth
    ("xa8ng", "xăng"),  // xăng - gasoline
    ("ba81t", "bắt"),   // bắt - catch
    ("ca8n", "căn"),    // căn - room
    ("na81ng", "nắng"), // nắng - sunny
    ("ma81t", "mắt"),   // mắt - eye
    // Ethnic minority language place names (issue #134)
    ("d9a81k", "đắk"),            // đắk - lowercase (d9=đ, a8=ă, 1=sắc)
    ("D9a81k", "Đắk"),            // Đắk - capitalized
    ("La81k", "Lắk"),             // Lắk - capitalized
    ("la81k", "lắk"),             // lắk - lowercase
    ("D9a81k La81k", "Đắk Lắk"),  // Đắk Lắk - full province name
    ("D9a81k No6ng", "Đắk Nông"), // Đắk Nông province (o6 = ô)
];

// ============================================================
// INVALID BREVE + VOWEL PATTERNS (ăi, ăo, ău, ăy)
// ============================================================
//
// In Vietnamese, breve 'ă' CANNOT be followed by another vowel.
// Valid: ăn, ăm, ăng, ăp, ăt, ăc (consonant endings)
// Invalid: ăi, ăo, ău, ăy (vowel endings)

const TELEX_INVALID_BREVE_DIPHTHONG: &[(&str, &str)] = &[
    // Breve always applies immediately for "aw" pattern
    ("awi", "ăi"),
    ("awo", "ăo"),
    ("awu", "ău"),
    ("awy", "ăy"),
    // With consonant before 'a' → breve still applies
    ("tawi", "tăi"),
    ("tawo", "tăo"),
    ("tawu", "tău"),
    ("tawy", "tăy"),
    ("mawi", "măi"),
    ("mawo", "măo"),
    ("lawi", "lăi"),
    ("lawo", "lăo"),
    // With tone marks
    ("tawis", "tắi"),
    ("tawof", "tằo"),
    // When vowel is between 'a' and 'w' → no breve (w not adjacent to a)
    ("taiw", "taiw"), // t + ai + w → 'ai' diphthong, 'w' not applied
];

const VNI_INVALID_BREVE_DIPHTHONG: &[(&str, &str)] = &[
    // Breve always applies immediately for "a8" pattern
    ("a8i", "ăi"),
    ("a8o", "ăo"),
    ("a8u", "ău"),
    ("a8y", "ăy"),
    // With consonant before 'a' → breve still applies
    ("ta8i", "tăi"),
    ("ta8o", "tăo"),
    ("ma8i", "măi"),
    ("la8i", "lăi"),
];

// ============================================================
// ENGLISH WORDS WITH AW PATTERN (should NOT transform)
// ============================================================
//
// Common English words containing "aw" should stay as-is
// because they don't form valid Vietnamese syllables.

const TELEX_ENGLISH_AW_WORDS: &[(&str, &str)] = &[
    // Common English words with "aw" - space triggers auto-restore
    ("raw ", "raw "), // raw data
    ("saw ", "saw "), // I saw
    ("law ", "law "), // law firm
    // "w" alone converts to "ư", auto-restore reverts on space
    ("we ", "we "),             // we → ưe → we (auto-restore)
    ("draw ", "draw "),         // draw a picture
    ("straw ", "straw "),       // drinking straw
    ("claw ", "claw "),         // cat's claw
    ("flaw ", "flaw "),         // design flaw
    ("jaw ", "jaw "),           // jaw bone
    ("paw ", "paw "),           // dog's paw
    ("craw ", "craw "),         // in my craw
    ("gnaw ", "gnaw "),         // gnaw at
    ("thaw ", "thaw "),         // thaw frozen
    ("outlaw ", "outlaw "),     // outlaw
    ("jigsaw ", "jigsaw "),     // jigsaw puzzle
    ("seesaw ", "seesaw "),     // seesaw
    ("coleslaw ", "coleslaw "), // coleslaw
    // Capital letters
    ("Raw ", "Raw "),
    ("LAW ", "LAW "),
    ("Draw ", "Draw "),
    ("DRAW ", "DRAW "),
    // Mixed with Vietnamese - space separates words
    // Note: "data" gets circumflex from vowel trigger (d-a-t-a → dât)
    // "dât" (circumflex + stop consonant, no mark) is NOT real Vietnamese → restore
    ("raw data ", "raw data "),
    ("raw vieetj", "raw việt"), // "raw" stays, "việt" transforms
    // Vietnamese words with similar patterns - should NOT be restored
    ("loxoi ", "lỗi "), // lỗi - Vietnamese, NOT restored
    ("soso ", "số "),   // số - Vietnamese, NOT restored
    ("waf ", "ừa "),    // ừa - Vietnamese (ư + huyền + a)
    ("was ", "ứa "),    // ứa - Vietnamese (ư + sắc + a)
    ("waj ", "ựa "),    // ựa - Vietnamese (ư + nặng + a)
    // W shortcut (w→ư) with marks - should NOT be restored
    ("wf ", "ừ "),   // w→ư, f→huyền: ừ (valid Vietnamese interjection)
    ("Wf ", "Ừ "),   // W→Ư, f→huyền: Ừ
    ("ws ", "ứ "),   // w→ư, s→sắc: ứ
    ("wmf ", "ừm "), // w→ư, m, f→huyền on ư: ừm (valid Vietnamese)
    ("Wmf ", "Ừm "), // W→Ư, m, f→huyền on Ư: Ừm
    ("wms ", "ứm "), // w→ư, m, s→sắc on ư: ứm
    // nghiệp/hiệp patterns - valid Vietnamese, NOT restored
    ("nghieepj ", "nghiệp "), // nghiệp - occupation/karma
    ("hieepj ", "hiệp "),     // hiệp - united/round
    // xếp pattern (ê) - all 3 typing orders
    ("xeeps ", "xếp "), // standard: ee → p → s
    ("xepes ", "xếp "), // delayed circumflex: e → p → e → s
    ("xepse ", "xếp "), // post-tone: e → p → s → e
    // xếp with different tones (post-tone circumflex)
    // Note: 'r' doesn't work after consonant (treated as consonant, not tone modifier)
    ("xepfe ", "xềp "), // huyền tone
    ("xepxe ", "xễp "), // ngã tone
    ("xepje ", "xệp "), // nặng tone
    // tấm pattern (â) - all 3 typing orders
    ("taams ", "tấm "), // standard: aa → m → s
    ("tamas ", "tấm "), // delayed circumflex: a → m → a → s
    ("tamsa ", "tấm "), // post-tone: a → m → s → a
    // tốt pattern (ô) - all 3 typing orders
    ("toots ", "tốt "), // standard: oo → t → s
    ("totos ", "tốt "), // delayed circumflex: o → t → o → s
    ("totso ", "tốt "), // post-tone: o → t → s → o
    // Combined: sắp xếp
    ("sapws xeeps ", "sắp xếp "), // standard
    ("sapws xepes ", "sắp xếp "), // delayed
    ("sapws xepse ", "sắp xếp "), // post-tone
    // au diphthong with delayed tone
    ("mauf ", "màu "), // m + a + u + f → màu (delayed huyền tone on 'a')
    ("mafu ", "màu "), // m + a + f + u → màu (tone between vowels)
    // Delayed stroke + delayed circumflex + mark
    ("datdas ", "đất "), // đất - delayed stroke (d) + delayed circumflex (a) + sắc (s)
    // đường pattern
    ("dduowfng ", "đường "), // đường - dd + ươ + huyền + ng (NOT restored)
    // Backspace and retype pattern (with auto-restore enabled)
    ("dduowfng<<sng", "đướng"), // delete ng, retype ng → back to đường
    // qu + ao + huyền - mark position flexibility
    ("quaof ", "quào "), // quào - mark typed after vowels
    ("quafo ", "quào "), // quào - mark typed between vowels (same result)
    // oai triphthong - tone always on 'a' (middle vowel)
    ("khoais ", "khoái "), // khoái - tone on a (comfortable, satisfied)
    ("ngoais ", "ngoái "), // ngoái - tone on a (look back)
    // Stroke revert: "didd" → stroke triggered then reverted → "did" (not "didd")
    ("didd you", "did you"), // English "did you" with typo
];

// ============================================================
// ISSUE #133: HORN PLACEMENT - "huơ" vs "hươ"
// ============================================================
// In words like "huơ" and "khuơ", only 'o' gets the horn mark.
// The 'u' remains unchanged because "uơ" is a valid vowel cluster
// where horn applies only to 'o'.
// GitHub: https://github.com/tuyenvm/OpenKey/issues/133

const TELEX_HORN_PLACEMENT: &[(&str, &str)] = &[
    // "uơ" pattern - only 'o' gets horn, 'u' stays unchanged
    ("huow", "huơ"),   // huơ - to wave hands (NOT hươ)
    ("khuow", "khuơ"), // khuơ - to stir (dialectal) (NOT khươ)
    // Contrast with "ươ" pattern - both vowels get horn when there's a final
    ("duowc", "dươc"),  // both u and o get horn (ươ pattern) when final exists
    ("duowcj", "dược"), // dược - medicine (horn + nặng mark)
    ("muowif", "mười"), // mười - ten (horn + huyền mark on ờ)
    ("luowst", "lướt"), // lướt - to slide (final 't' triggers horn on 'u')
    // "Qu-" initial pattern - 'u' is part of initial consonant, should NOT get horn
    ("Quoiws", "Quới"), // Quới - name (NOT Qưới, 'u' is part of "Qu" initial)
    ("quoocs", "quốc"), // quốc - country (NOT qướcc, 'u' is part of "Qu" initial)
];

// ============================================================
// ISSUE #312: SAME VOWEL AFTER MARKED VOWEL
// ============================================================
// When a vowel already has a diacritic mark, typing the same vowel
// again should append the raw vowel, not apply circumflex.
// Example: "chưa" + "a" → "chưaa" (NOT "chưâ")
// GitHub: https://github.com/tuyenvm/OpenKey/issues/312

const TELEX_SAME_VOWEL_AFTER_MARK: &[(&str, &str)] = &[
    // Issue #312: After a vowel already has a mark (horn/circumflex/breve),
    // typing the same vowel again should append raw, NOT apply circumflex.
    //
    // "chuwa" → "chưa" (horn applied via uw pattern)
    // "chưa" + "a" → should be "chưaa" (NOT "chưâ")
    ("chuwaa", "chưaa"), // chưa + a → chưaa (NOT chưâ)
    // More examples with horn on 'u'
    ("tuwaa", "tưaa"), // tưa + a → tưaa (NOT tưâ)
    ("muwaa", "mưaa"), // mưa + a → mưaa (NOT mưâ)
                       // Note: "aaa", "eee", "ooo" use standard revert behavior (third same key reverts)
                       // "aaa" → "aa" (circumflex applied, then reverted - standard Vietnamese IME)
                       // This is CORRECT and NOT a bug - it's the standard revert pattern.
];

// ============================================================
// EDGE CASES: PARTIAL WORDS / INTERMEDIATE STATES
// ============================================================
//
// When typing incrementally, intermediate states should behave correctly.

const TELEX_BREVE_EDGE_CASES: &[(&str, &str)] = &[
    // Breve always applies immediately for "aw" pattern
    ("traw", "tră"),   // breve applied immediately
    ("trawm", "trăm"), // with final consonant
    ("naw", "nă"),     // breve applied immediately
    ("nawm", "năm"),   // with final consonant
    ("raw", "ră"),     // breve applied immediately
    ("rawng", "răng"), // with final consonant
    // OA patterns (for contrast - these should transform)
    ("hoa", "hoa"),  // valid open syllable
    ("hoaf", "hoà"), // valid with tone
    // Standalone aw
    ("awng", "ăng"), // ăng is valid (final consonant)
];

// ============================================================
// TEST FUNCTIONS
// ============================================================

#[test]
fn telex_invalid_breve_open_syllable() {
    telex(TELEX_INVALID_BREVE_OPEN);
}

#[test]
fn vni_invalid_breve_open_syllable() {
    vni(VNI_INVALID_BREVE_OPEN);
}

#[test]
fn telex_valid_breve_with_final() {
    telex(TELEX_VALID_BREVE);
}

#[test]
fn vni_valid_breve_with_final() {
    vni(VNI_VALID_BREVE);
}

#[test]
fn telex_invalid_breve_diphthong() {
    telex(TELEX_INVALID_BREVE_DIPHTHONG);
}

#[test]
fn vni_invalid_breve_diphthong() {
    vni(VNI_INVALID_BREVE_DIPHTHONG);
}

// NOTE: Requires english_auto_restore to be enabled (experimental feature).
#[test]
fn telex_english_aw_words() {
    telex_auto_restore(TELEX_ENGLISH_AW_WORDS);
}

#[test]
fn telex_breve_edge_cases() {
    telex(TELEX_BREVE_EDGE_CASES);
}

// Issue #133: Horn placement - "uơ" vs "ươ" patterns
#[test]
fn telex_horn_placement() {
    telex(TELEX_HORN_PLACEMENT);
}

// Issue #312: Same vowel after marked vowel
#[test]
fn telex_same_vowel_after_mark() {
    telex(TELEX_SAME_VOWEL_AFTER_MARK);
}

// ============================================================
// TRADITIONAL TONE PLACEMENT - Issue #64
// ============================================================
//
// When "modern tone" setting is OFF (traditional style):
// - hòa, thúy (tone on 1st vowel) instead of hoà, thuý (tone on 2nd)
//
// Bug: Even with setting OFF, out-of-order typing (e.g., "xosa", "tufy")
// still produces modern-style results due to hardcoded `true` in
// `reposition_tone_if_needed()` function.

const TELEX_TRADITIONAL_TONE: &[(&str, &str)] = &[
    // ============================================================
    // Issue #64: Out-of-order typing with traditional tone setting
    // When typing tone BEFORE the second vowel, then adding second vowel,
    // tone should stay on FIRST vowel in traditional mode
    // ============================================================
    //
    // --- Pattern: oa (traditional: tone on 'o') ---
    ("osa", "óa"),   // o + s + a → óa (NOT oá)
    ("ofa", "òa"),   // o + f + a → òa (NOT oà)
    ("ora", "ỏa"),   // o + r + a → ỏa (NOT oả)
    ("oxa", "õa"),   // o + x + a → õa (NOT oã)
    ("oja", "ọa"),   // o + j + a → ọa (NOT oạ)
    ("hosa", "hóa"), // h + o + s + a → hóa (NOT hoá)
    ("hofa", "hòa"), // h + o + f + a → hòa (NOT hoà)
    ("xosa", "xóa"), // x + o + s + a → xóa (NOT xoá) - Issue #64 case
    ("losa", "lóa"), // l + o + s + a → lóa (NOT loá)
    ("tosa", "tóa"), // t + o + s + a → tóa (NOT toá)
    //
    // --- Pattern: oe (traditional: tone on 'o') ---
    ("ose", "óe"),     // o + s + e → óe (NOT oé)
    ("ofe", "òe"),     // o + f + e → òe (NOT oè)
    ("khose", "khóe"), // kh + o + s + e → khóe (NOT khoé)
    ("xose", "xóe"),   // x + o + s + e → xóe (NOT xoé)
    //
    // --- Pattern: uy (traditional: tone on 'u') ---
    ("usy", "úy"),     // u + s + y → úy (NOT uý)
    ("ufy", "ùy"),     // u + f + y → ùy (NOT uỳ)
    ("ury", "ủy"),     // u + r + y → ủy (NOT uỷ)
    ("uxy", "ũy"),     // u + x + y → ũy (NOT uỹ)
    ("ujy", "ụy"),     // u + j + y → ụy (NOT uỵ)
    ("tusy", "túy"),   // t + u + s + y → túy (NOT tuý)
    ("tufy", "tùy"),   // t + u + f + y → tùy (NOT tuỳ) - Issue #64 case
    ("husy", "húy"),   // h + u + s + y → húy (NOT huý)
    ("thusy", "thúy"), // th + u + s + y → thúy (NOT thuý)
    //
    // --- qu- special case (always tone on 2nd vowel regardless) ---
    // qu is treated as initial, so 'u' is not the vowel
    ("qusy", "quý"), // qu + s + y → quý (same in both modes)
    //
    // ============================================================
    // Delayed tone (typing tone AFTER all vowels) - should also respect setting
    // ============================================================
    //
    // --- oa + delayed tone ---
    ("hoas", "hóa"), // h + o + a + s → hóa (traditional)
    ("hoaf", "hòa"), // h + o + a + f → hòa (traditional)
    ("xoas", "xóa"), // x + o + a + s → xóa (traditional)
    //
    // --- oe + delayed tone ---
    ("khoes", "khóe"), // kh + o + e + s → khóe (traditional)
    //
    // --- uy + delayed tone ---
    ("tuys", "túy"),   // t + u + y + s → túy (traditional)
    ("tuyf", "tùy"),   // t + u + y + f → tùy (traditional)
    ("thuys", "thúy"), // th + u + y + s → thúy (traditional)
    //
    // ============================================================
    // Normal order (tone typed correctly) - for comparison
    // ============================================================
    ("hosa", "hóa"),   // same as above
    ("thusa", "thúa"), // th + u + s + a → thúa (u is main vowel, a is glide)
];

const VNI_TRADITIONAL_TONE: &[(&str, &str)] = &[
    // ============================================================
    // Issue #64: VNI with traditional tone setting
    // ============================================================
    //
    // --- Pattern: oa (traditional: tone on 'o') ---
    ("o1a", "óa"),   // o + 1 + a → óa (NOT oá)
    ("o2a", "òa"),   // o + 2 + a → òa (NOT oà)
    ("ho1a", "hóa"), // h + o + 1 + a → hóa (NOT hoá)
    ("ho2a", "hòa"), // h + o + 2 + a → hòa (NOT hoà)
    ("xo1a", "xóa"), // x + o + 1 + a → xóa (NOT xoá) - Issue #64 case
    //
    // --- Pattern: oe (traditional: tone on 'o') ---
    ("o1e", "óe"),     // o + 1 + e → óe (NOT oé)
    ("kho1e", "khóe"), // kh + o + 1 + e → khóe (NOT khoé)
    //
    // --- Pattern: uy (traditional: tone on 'u') ---
    ("u1y", "úy"),   // u + 1 + y → úy (NOT uý)
    ("u2y", "ùy"),   // u + 2 + y → ùy (NOT uỳ)
    ("tu1y", "túy"), // t + u + 1 + y → túy (NOT tuý)
    ("tu2y", "tùy"), // t + u + 2 + y → tùy (NOT tuỳ) - Issue #64 case
    //
    // --- Delayed tone ---
    ("hoa1", "hóa"), // h + o + a + 1 → hóa (traditional)
    ("hoa2", "hòa"), // h + o + a + 2 → hòa (traditional)
    ("tuy1", "túy"), // t + u + y + 1 → túy (traditional)
    ("tuy2", "tùy"), // t + u + y + 2 → tùy (traditional)
];

#[test]
fn telex_traditional_tone_placement() {
    telex_traditional(TELEX_TRADITIONAL_TONE);
}

#[test]
fn vni_traditional_tone_placement() {
    vni_traditional(VNI_TRADITIONAL_TONE);
}
