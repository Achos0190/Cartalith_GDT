//! Creating a note from one of the author's own templates, and inserting one
//! into an open note — `GUI_GAP_REGISTER.md` **VA-02**, and owner **Ruling
//! BF** (`LARGE_ITEM_RULINGS.md`, 2026-09-27).
//!
//! ## The rule: Obsidian's templates, exactly
//!
//! Ruling BF is Ruling AM's rule (`chronos.rs`) applied to templates:
//! **Obsidian's own structure and syntax, with no Cartalith variant**, so a
//! vault set up for Obsidian's core Templates plugin works here unchanged and
//! a note created here reads the same in Obsidian. What that means, and where
//! each part comes from:
//!
//! - **The template folder** is the one Obsidian's core Templates plugin is
//!   set to (`Settings → Core plugins → Templates → Template folder
//!   location`), which Obsidian stores as `folder` in
//!   [`OBSIDIAN_TEMPLATES_JSON`]. Failing that, the Templater community
//!   plugin's `templates_folder` in [`TEMPLATER_DATA_JSON`]. Failing both,
//!   the pre-BF convention (below). [`Config::read`] is the whole resolution,
//!   and [`Config::source`] says which one answered so the window can say so.
//!   A file that is absent, unreadable, not JSON, or holds an empty folder is
//!   "not configured" — never a guessed folder.
//! - **Placeholders** ([`fill`]) are the core plugin's documented set
//!   (<https://obsidian.md/help/plugins/templates>, read 2026-09-27):
//!   `{{title}}` "Title of the active note"; `{{date}}` "Today's date.
//!   Default format: `YYYY-MM-DD`"; `{{time}}` "Current time. Default format:
//!   `HH:mm`"; and `{{date:FORMAT}}`/`{{time:FORMAT}}`, "a colon followed by
//!   a string of Moment.js format tokens". The defaults are overridden by
//!   `templates.json`'s `dateFormat`/`timeFormat`. **Not in that page, and
//!   taken from the plugin's observed behaviour instead:** the variable name
//!   matches case-insensitively and tolerates spaces inside the braces
//!   (`{{ Title }}`); a disclosed reading, not a documented one.
//! - **Moment.js tokens** are [`format_moment`]'s, which follows Moment's own
//!   tokenizer order (see its doc for the supported set and what is left
//!   verbatim).
//! - **Insert template** ([`insert`]): "The content of the template is
//!   inserted at your current cursor position" (the same help page). A
//!   template's properties are **merged** into the note's, per Obsidian's
//!   Properties help ("all the properties from the template will be added to
//!   the note. Obsidian will also merge any properties that exist in your
//!   note with properties in the template"). How the merge resolves a clash
//!   is not in the help; what Obsidian does (forum thread 105716, v1.9.12,
//!   staff-acknowledged as the intended behaviour) is: **list values are
//!   unioned, a scalar is replaced by the template's value.** [`insert`] does
//!   exactly that.
//! - **Templater's `<% … %>`** is JavaScript that only Templater can run. It
//!   is copied verbatim — never executed, never imitated — so opening the
//!   note in Obsidian with Templater still has something to run.
//!
//! ## What the pre-BF behaviour was, and what survives of it
//!
//! Before Ruling BF, [`discover`] took every `.md` whose **path** contained
//! "template" (case-insensitively), because every one of the owner's files
//! says so — `Settlement Template.md`, `Region Template/Regional
//! Overview.md` (`design/vault-templates/`). That rule is kept **only** as
//! [`Source::Fallback`], for a vault with neither setting.
//!
//! Before Ruling BF the filler also deliberately left `{{title}}` and
//! `{{date}}` alone ("a non-name token is not ours to fill"). **That is
//! reversed**: Obsidian fills them, so this does. What it still fills beyond
//! Obsidian's set is the owner's own name convention — any `{{…Name}}`
//! token (`{{Landmark_Name}}`) and the literal `[Name]` — with the entity's
//! name. Every other bracketed prompt (`[If applicable]`, `[Optional]`)
//! survives verbatim: those are instructions to the author.
//!
//! ## Where templates come from
//!
//! **The vault, not this crate.** `design/vault-templates/` holds the owner's
//! real templates as *reference material for this repository*, not content to
//! ship; a registry compiled into the binary would be Cartalith telling an
//! author how to write their notes.

/// Where Obsidian's core Templates plugin keeps its settings, relative to the
/// vault root. (Obsidian lets a user rename its config folder; `.obsidian` is
/// the default and the only one this reads.)
pub const OBSIDIAN_TEMPLATES_JSON: &str = ".obsidian/templates.json";
/// The Templater community plugin's settings file.
pub const TEMPLATER_DATA_JSON: &str = ".obsidian/plugins/templater-obsidian/data.json";
/// Obsidian's `{{date}}` default when `templates.json` sets no `dateFormat`.
pub const DEFAULT_DATE_FORMAT: &str = "YYYY-MM-DD";
/// Obsidian's `{{time}}` default when `templates.json` sets no `timeFormat`.
pub const DEFAULT_TIME_FORMAT: &str = "HH:mm";

/// One candidate template: its vault-relative path and the name to show.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Template {
    pub rel: String,
    pub label: String,
}

/// Which setting named the template folder.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    /// `.obsidian/templates.json`'s `folder`.
    Obsidian,
    /// Templater's `templates_folder`.
    Templater,
    /// Neither: the pre-BF "the path contains *template*" rule.
    Fallback,
}

impl Source {
    /// The lower-case key used in the window/log text (`describe`'s callers
    /// outside this module use this rather than a `Debug` string).
    pub fn as_str(self) -> &'static str {
        match self {
            Source::Obsidian => "obsidian",
            Source::Templater => "templater",
            Source::Fallback => "fallback",
        }
    }
}

/// The vault's template settings, as far as it has any. Absent settings are
/// `None` — the Obsidian defaults are applied where a format is *used*
/// ([`Config::date_format_or_default`]), never stored as if the vault had
/// set them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub source: Source,
    /// The folder, without leading or trailing `/`. `Some` exactly when
    /// `source` is not [`Source::Fallback`].
    pub folder: Option<String>,
    pub date_format: Option<String>,
    pub time_format: Option<String>,
}

impl Config {
    /// No settings at all.
    pub fn fallback() -> Self {
        Config { source: Source::Fallback, folder: None, date_format: None, time_format: None }
    }

    /// Resolves the settings through `read` (a vault-relative path to its
    /// text, `None` when unreadable). Order: Obsidian's core plugin, then
    /// Templater, then the fallback. `dateFormat`/`timeFormat` are
    /// Obsidian's own keys and are read from `templates.json` whichever
    /// source names the folder.
    pub fn read(read: impl Fn(&str) -> Option<String>) -> Self {
        let core = read(OBSIDIAN_TEMPLATES_JSON).and_then(|t| serde_json::from_str::<serde_json::Value>(&t).ok());
        let core_str = |key: &str| core.as_ref().and_then(|v| json_str(v, key));
        let date_format = core_str("dateFormat");
        let time_format = core_str("timeFormat");
        if let Some(folder) = core_str("folder").and_then(|f| folder_of(&f)) {
            return Config { source: Source::Obsidian, folder: Some(folder), date_format, time_format };
        }
        let templater = read(TEMPLATER_DATA_JSON)
            .and_then(|t| serde_json::from_str::<serde_json::Value>(&t).ok())
            .and_then(|v| json_str(&v, "templates_folder"))
            .and_then(|f| folder_of(&f));
        if let Some(folder) = templater {
            return Config { source: Source::Templater, folder: Some(folder), date_format, time_format };
        }
        Config { source: Source::Fallback, folder: None, date_format, time_format }
    }

    /// The `{{date}}` format actually in force: the vault's `dateFormat`, or
    /// Obsidian's own default when it set none.
    pub fn date_format_or_default(&self) -> &str {
        self.date_format.as_deref().unwrap_or(DEFAULT_DATE_FORMAT)
    }

    /// The `{{time}}` format actually in force: the vault's `timeFormat`, or
    /// Obsidian's own default when it set none.
    pub fn time_format_or_default(&self) -> &str {
        self.time_format.as_deref().unwrap_or(DEFAULT_TIME_FORMAT)
    }

    /// One line for the window: where the templates came from.
    pub fn describe(&self) -> String {
        match (self.source, self.folder.as_deref()) {
            (Source::Obsidian, Some(f)) => format!("templates from {f}/ (Obsidian settings)"),
            (Source::Templater, Some(f)) => format!("templates from {f}/ (Templater settings)"),
            _ => "no template folder is set in this vault's Obsidian settings -- showing every note whose path contains \"template\"".to_string(),
        }
    }
}

/// A non-empty string value of `key` in a JSON object.
fn json_str(v: &serde_json::Value, key: &str) -> Option<String> {
    v.as_object()?.get(key)?.as_str().map(str::to_string).filter(|s| !s.trim().is_empty())
}

/// `"/Templates/"` -> `"Templates"`; `None` for a folder that is only
/// slashes and spaces (the vault root is not a template folder Obsidian
/// accepts either).
fn folder_of(s: &str) -> Option<String> {
    let f = s.trim().trim_matches('/').trim();
    if f.is_empty() {
        None
    } else {
        Some(f.to_string())
    }
}

/// The templates among `files` (the bounded `list_markdown` walk, already
/// sorted). With a configured folder: every `.md` under it, labelled by its
/// path inside the folder without `.md` (`Templates/People/NPC.md` ->
/// `People/NPC`). Without one: the pre-BF rule — "template" in the path.
pub fn discover(files: &[String], cfg: &Config) -> Vec<Template> {
    match cfg.folder.as_deref() {
        Some(folder) => {
            let prefix = format!("{folder}/");
            files
                .iter()
                .filter_map(|f| {
                    let inner = f.strip_prefix(&prefix)?;
                    Some(Template { rel: f.clone(), label: strip_md(inner).to_string() })
                })
                .collect()
        }
        None => files
            .iter()
            .filter(|f| f.to_lowercase().contains("template"))
            .map(|f| Template { rel: f.clone(), label: label_for(f) })
            .collect(),
    }
}

/// `s` with a trailing `.md`/`.MD`/… stripped (case-insensitively); `s`
/// itself when it has none.
fn strip_md(s: &str) -> &str {
    if s.len() >= 3 && s[s.len() - 3..].eq_ignore_ascii_case(".md") {
        &s[..s.len() - 3]
    } else {
        s
    }
}

/// A note's title as Obsidian means it: its file's basename, no extension.
pub fn title_of(rel: &str) -> &str {
    strip_md(rel.rsplit('/').next().unwrap_or(rel))
}

/// Fallback labels: `Region Template/Landmarks/Landmark template.md` ->
/// `Landmarks ▸ Landmark template`. The owner's corpus has two byte-identical
/// `Landmark template.md` files, so the parent folder is what tells them
/// apart.
fn label_for(rel: &str) -> String {
    let stem = title_of(rel);
    match rel.rsplit_once('/') {
        Some((dir, _)) => {
            let parent = dir.rsplit('/').next().unwrap_or(dir);
            format!("{parent} \u{25B8} {stem}")
        }
        None => stem.to_string(),
    }
}

/// The vault-relative path a new note for `kind` named `name` goes to —
/// v3's own `Settlements/{name}.md` convention, generalised to every entity
/// kind this port can address.
///
/// The folder is plural and capitalised because that is what the owner's own
/// vault looks like; the file stem is the entity's name with the characters a
/// filesystem refuses replaced by `-` rather than dropped, so two names that
/// differ only in punctuation do not collide.
pub fn suggested_path(kind: crate::EntityKind, name: &str) -> String {
    let folder = match kind {
        crate::EntityKind::Settlement => "Settlements",
        crate::EntityKind::Province => "Provinces",
        crate::EntityKind::Continent => "Continents",
        crate::EntityKind::Faction => "Factions",
        crate::EntityKind::Culture => "Cultures",
        // Owner ruling 13, 2026-09-06: the owner's vault already names a
        // `Landmarks` folder (`Region Template/Landmarks/`).
        crate::EntityKind::Landmark => "Landmarks",
    };
    format!("{folder}/{}.md", sanitise(name))
}

/// A filename stem that Windows, NTFS and POSIX all accept. Leading and
/// trailing dots and spaces go too — Windows silently strips a trailing dot
/// and the file would then not be at the path we recorded.
pub fn sanitise(name: &str) -> String {
    let mut out: String = name
        .chars()
        .map(|c| if matches!(c, '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*') || (c as u32) < 0x20 { '-' } else { c })
        .collect();
    let trimmed = out.trim_matches(|c: char| c == '.' || c.is_whitespace()).to_string();
    out = trimmed;
    if out.is_empty() { "Untitled".to_string() } else { out }
}

// -- The clock ----------------------------------------------------------------

/// A local wall-clock instant, proleptic Gregorian. The caller supplies it —
/// that is the clock seam: `cartalith-godot` passes the system's local time
/// (`Time.get_datetime_dict_from_system()`), tests pass a fixed one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DateTime {
    pub year: i32,
    pub month: u32,
    pub day: u32,
    pub hour: u32,
    pub minute: u32,
    pub second: u32,
}

/// The proleptic Gregorian leap-year rule.
fn is_leap(y: i32) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}

/// Days in month `m` of year `y` (0 for a month outside 1..=12, which
/// [`DateTime::new`] rejects before this is ever called with one).
fn month_len(y: i32, m: u32) -> u32 {
    match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap(y) => 29,
        2 => 28,
        _ => 0,
    }
}

impl DateTime {
    /// `None` for anything that is not a real date and time.
    pub fn new(year: i32, month: u32, day: u32, hour: u32, minute: u32, second: u32) -> Option<Self> {
        let ok = (1..=12).contains(&month) && day >= 1 && day <= month_len(year, month) && hour < 24 && minute < 60 && second < 60;
        ok.then_some(DateTime { year, month, day, hour, minute, second })
    }

    /// Days since 1970-01-01 (Hinnant's `days_from_civil`).
    fn days_from_epoch(&self) -> i64 {
        let y = if self.month <= 2 { self.year as i64 - 1 } else { self.year as i64 };
        let era = if y >= 0 { y } else { y - 399 } / 400;
        let yoe = y - era * 400;
        let m = self.month as i64;
        let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + self.day as i64 - 1;
        let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
        era * 146_097 + doe - 719_468
    }

    /// 0 = Sunday … 6 = Saturday (Moment's `d`).
    pub fn weekday(&self) -> u32 {
        // 1970-01-01 was a Thursday (4).
        (self.days_from_epoch() + 4).rem_euclid(7) as u32
    }

    /// 1-based day of the year.
    pub fn day_of_year(&self) -> u32 {
        (1..self.month).map(|m| month_len(self.year, m)).sum::<u32>() + self.day
    }

    /// ISO 8601 week number (Moment's `W`).
    pub fn iso_week(&self) -> u32 {
        let iso_wd = match self.weekday() {
            0 => 7,
            d => d,
        };
        let week = (self.day_of_year() as i32 - iso_wd as i32 + 10) / 7;
        let weeks_in = |y: i32| {
            let p = |y: i32| (y + y.div_euclid(4) - y.div_euclid(100) + y.div_euclid(400)).rem_euclid(7);
            if p(y) == 4 || p(y - 1) == 3 { 53 } else { 52 }
        };
        if week < 1 {
            weeks_in(self.year - 1) as u32
        } else if week > weeks_in(self.year) {
            1
        } else {
            week as u32
        }
    }
}

// -- Moment.js formatting -------------------------------------------------------

const MONTHS: [&str; 12] =
    ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"];
const DAYS: [&str; 7] = ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"];

/// Moment's English ordinal: 1st 2nd 3rd 4th … 11th 12th 13th … 21st.
fn ordinal(n: i64) -> String {
    let b = n % 10;
    let suffix = if (n % 100) / 10 == 1 {
        "th"
    } else {
        match b {
            1 => "st",
            2 => "nd",
            3 => "rd",
            _ => "th",
        }
    };
    format!("{n}{suffix}")
}

/// Zero-pads `n` to `width` digits, keeping a leading `-` for a negative
/// year outside the padded width (Moment's own zero-fill behaviour).
fn zero_fill(n: i64, width: usize) -> String {
    let sign = if n < 0 { "-" } else { "" };
    format!("{sign}{:0width$}", n.abs(), width = width)
}

/// Moment's token alternation, in Moment's own order (its `formattingTokens`
/// regex) — the order matters because Moment takes the **first** alternative
/// that matches, not the longest: `Do` is tried before `DDDo`, and `Hmm` is
/// one token. Tokens this port does not format (`N`, `w`, `g`, `G`, `S`, `x`,
/// `X`, `z`, `Z`, `y` and their runs) are still recognised here so that they
/// are copied through **whole and verbatim**, rather than half-formatted.
const TOKENS: &[&str] = &[
    "Hmmss", "hmmss", "Hmm", "hmm", "Mo", "MMMM", "MMM", "MM", "M", "Do", "DDDo", "DDDD", "DDD", "DD", "D", "dddd", "ddd",
    "dd", "do", "d", "wo", "ww", "w", "Wo", "WW", "W", "Qo", "Q", "NNNNN", "NNNN", "NNN", "NN", "N", "YYYYYY", "YYYYY",
    "YYYY", "YY", "yyyy", "yyy", "yy", "yo", "y", "ggggg", "gggg", "gg", "GGGGG", "GGGG", "GG", "e", "E", "a", "A", "hh",
    "h", "HH", "H", "kk", "k", "mm", "m", "ss", "s", "SSSSSSSSS", "SSSSSSSS", "SSSSSSS", "SSSSSS", "SSSSS", "SSSS", "SSS",
    "SS", "S", "x", "X", "zz", "z", "ZZ", "Z", "Y",
];

/// One Moment token's value, or `None` for a token this port leaves
/// verbatim (see [`TOKENS`]).
fn token_value(tok: &str, t: &DateTime) -> Option<String> {
    let h12 = match t.hour % 12 {
        0 => 12,
        h => h,
    };
    let y = t.year as i64;
    Some(match tok {
        "Hmm" => format!("{}{:02}", t.hour, t.minute),
        "Hmmss" => format!("{}{:02}{:02}", t.hour, t.minute, t.second),
        "hmm" => format!("{}{:02}", h12, t.minute),
        "hmmss" => format!("{}{:02}{:02}", h12, t.minute, t.second),
        "M" => t.month.to_string(),
        "Mo" => ordinal(t.month as i64),
        "MM" => format!("{:02}", t.month),
        "MMM" => MONTHS[t.month as usize - 1][..3].to_string(),
        "MMMM" => MONTHS[t.month as usize - 1].to_string(),
        "D" => t.day.to_string(),
        "Do" => ordinal(t.day as i64),
        "DD" => format!("{:02}", t.day),
        "DDD" => t.day_of_year().to_string(),
        "DDDo" => ordinal(t.day_of_year() as i64),
        "DDDD" => format!("{:03}", t.day_of_year()),
        "d" | "e" => t.weekday().to_string(),
        "do" => ordinal(t.weekday() as i64),
        "dd" => DAYS[t.weekday() as usize][..2].to_string(),
        "ddd" => DAYS[t.weekday() as usize][..3].to_string(),
        "dddd" => DAYS[t.weekday() as usize].to_string(),
        "E" => match t.weekday() {
            0 => "7".to_string(),
            d => d.to_string(),
        },
        "W" => t.iso_week().to_string(),
        "Wo" => ordinal(t.iso_week() as i64),
        "WW" => format!("{:02}", t.iso_week()),
        "Q" => ((t.month - 1) / 3 + 1).to_string(),
        "Qo" => ordinal(((t.month - 1) / 3 + 1) as i64),
        "Y" => {
            if y <= 9999 {
                zero_fill(y, 4)
            } else {
                format!("+{y}")
            }
        }
        "YY" => zero_fill(y % 100, 2),
        "YYYY" => zero_fill(y, 4),
        "YYYYY" => zero_fill(y, 5),
        "YYYYYY" => format!("{}{}", if y < 0 { "-" } else { "+" }, zero_fill(y.abs(), 6)),
        "A" => (if t.hour < 12 { "AM" } else { "PM" }).to_string(),
        "a" => (if t.hour < 12 { "am" } else { "pm" }).to_string(),
        "H" => t.hour.to_string(),
        "HH" => format!("{:02}", t.hour),
        "h" => h12.to_string(),
        "hh" => format!("{:02}", h12),
        "k" => (if t.hour == 0 { 24 } else { t.hour }).to_string(),
        "kk" => format!("{:02}", if t.hour == 0 { 24 } else { t.hour }),
        "m" => t.minute.to_string(),
        "mm" => format!("{:02}", t.minute),
        "s" => t.second.to_string(),
        "ss" => format!("{:02}", t.second),
        _ => return None,
    })
}

/// Moment's English long-date formats (`locale/en`'s `longDateFormat`, with
/// the lower-case forms derived the way Moment derives them: `MMMM`, `MM`,
/// `DD` and `dddd` shortened by one letter). Order is Moment's
/// `localFormattingTokens`: `LTS|LT|LL?L?L?|l{1,4}`.
const LOCALE_FORMATS: &[(&str, &str)] = &[
    ("LTS", "h:mm:ss A"),
    ("LT", "h:mm A"),
    ("LLLL", "dddd, MMMM D, YYYY h:mm A"),
    ("LLL", "MMMM D, YYYY h:mm A"),
    ("LL", "MMMM D, YYYY"),
    ("L", "MM/DD/YYYY"),
    ("llll", "ddd, MMM D, YYYY h:mm A"),
    ("lll", "MMM D, YYYY h:mm A"),
    ("ll", "MMM D, YYYY"),
    ("l", "M/D/YYYY"),
];

/// Moment's `expandFormat`: locale tokens (`L`, `LT`, `ll`, …) are replaced
/// by their formats before anything else is read -- which is why `l` inside
/// ordinary text expands too, exactly as it does in Obsidian. A `[bracket]`
/// and a backslash-escaped token are skipped. Moment repeats the pass up to
/// six times; one pass suffices here because no English expansion contains a
/// locale token.
fn expand_locale(fmt: &str) -> String {
    let chars: Vec<char> = fmt.chars().collect();
    let mut out = String::with_capacity(fmt.len());
    let mut i = 0;
    let locale_at = |i: usize| {
        LOCALE_FORMATS.iter().find(|(tok, _)| {
            let n = tok.len();
            i + n <= chars.len() && chars[i..i + n].iter().copied().eq(tok.chars())
        })
    };
    while i < chars.len() {
        if chars[i] == '[' {
            if let Some(off) = chars[i + 1..].iter().position(|&x| x == ']' || x == '[') {
                if chars[i + 1 + off] == ']' {
                    out.extend(&chars[i..i + off + 2]);
                    i += off + 2;
                    continue;
                }
            }
        }
        if chars[i] == '\\' {
            if let Some((tok, _)) = locale_at(i + 1) {
                out.push('\\');
                out.push_str(tok);
                i += 1 + tok.len();
                continue;
            }
        }
        if let Some((tok, f)) = locale_at(i) {
            out.push_str(f);
            i += tok.len();
            continue;
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

/// `t` formatted with a Moment.js format string, English locale.
///
/// Supported: `YYYY YY Y YYYYY YYYYYY M Mo MM MMM MMMM D Do DD DDD DDDo DDDD
/// d do dd ddd dddd e E W Wo WW Q Qo H HH h hh k kk m mm s ss A a Hmm Hmmss
/// hmm hmmss`, the English locale formats `LT LTS L LL LLL LLLL l ll lll
/// llll` ([`expand_locale`]), `[literal text]`, and a backslash escaping the
/// token after it.
/// Left verbatim (recognised, not formatted): locale weeks `w`/`g`, ISO week
/// year `G`, era `N`, `y`, fractional seconds `S…` (the clock carries no
/// sub-second), Unix time `x`/`X` and zone `z`/`Z` (the clock carries no
/// zone). Any other character is copied as-is, as Moment does.
pub fn format_moment(fmt: &str, t: &DateTime) -> String {
    let expanded = expand_locale(fmt);
    let chars: Vec<char> = expanded.chars().collect();
    let mut out = String::new();
    let mut i = 0;
    let match_token = |i: usize| -> Option<&'static str> {
        TOKENS.iter().copied().find(|tok| {
            let n = tok.chars().count();
            i + n <= chars.len() && chars[i..i + n].iter().copied().eq(tok.chars())
        })
    };
    while i < chars.len() {
        let c = chars[i];
        if c == '[' {
            // Moment's `\[[^\[]*\]`: up to the next `]`, with no `[` inside.
            if let Some(off) = chars[i + 1..].iter().position(|&x| x == ']' || x == '[') {
                if chars[i + 1 + off] == ']' {
                    out.extend(&chars[i + 1..i + 1 + off]);
                    i += off + 2;
                    continue;
                }
            }
            out.push(c);
            i += 1;
            continue;
        }
        if c == '\\' && i + 1 < chars.len() {
            // Moment strips the backslash and prints what it escapes as text.
            match match_token(i + 1) {
                Some(tok) => {
                    out.push_str(tok);
                    i += 1 + tok.chars().count();
                }
                None => {
                    out.push(chars[i + 1]);
                    i += 2;
                }
            }
            continue;
        }
        if let Some(tok) = match_token(i) {
            match token_value(tok, t) {
                Some(v) => out.push_str(&v),
                None => out.push_str(tok),
            }
            i += tok.chars().count();
            continue;
        }
        out.push(c);
        i += 1;
    }
    out
}

// -- Placeholders -----------------------------------------------------------------

/// Everything [`fill`] substitutes.
#[derive(Debug, Clone, Copy)]
pub struct Fill<'a> {
    /// `{{title}}` — the note's basename ([`title_of`]).
    pub title: &'a str,
    /// `{{…Name}}` and `[Name]` — the entity's name (the owner's convention).
    pub name: &'a str,
    pub now: DateTime,
    pub cfg: &'a Config,
}

/// `template` with its placeholders filled — see this module's doc for the
/// set and its sources. One left-to-right pass, so a substituted value is
/// never itself re-read as a placeholder. `<% … %>` is copied verbatim, and
/// so is every `{{…}}` that is not a placeholder, an unclosed `{{`, and an
/// empty format (`{{date:}}`, which Obsidian's own pattern does not match).
pub fn fill(template: &str, f: &Fill) -> String {
    let mut out = String::with_capacity(template.len() + 32);
    let mut i = 0;
    while i < template.len() {
        let rest = &template[i..];
        if rest.starts_with("<%") {
            if let Some(end) = rest[2..].find("%>") {
                out.push_str(&rest[..2 + end + 2]);
                i += 2 + end + 2;
                continue;
            }
        }
        if rest.starts_with("{{") {
            if let Some(end) = rest[2..].find("}}") {
                let token = &rest[2..2 + end];
                if let Some(v) = placeholder(token, f) {
                    out.push_str(&v);
                    i += 2 + end + 2;
                    continue;
                }
            }
        }
        if rest.starts_with("[Name]") {
            out.push_str(f.name);
            i += 6;
            continue;
        }
        let ch = rest.chars().next().expect("i is a char boundary");
        out.push(ch);
        i += ch.len_utf8();
    }
    out
}

/// One `{{token}}` or `{{token:format}}`'s value, `None` when `token` is not
/// one of the recognised placeholders (so [`fill`] leaves it verbatim).
fn placeholder(token: &str, f: &Fill) -> Option<String> {
    let (key, format) = match token.split_once(':') {
        Some((k, fmt)) => (k.trim(), Some(fmt.trim())),
        None => (token.trim(), None),
    };
    let lower = key.to_lowercase();
    match (lower.as_str(), format) {
        ("title", None) => Some(f.title.to_string()),
        ("date", None) => Some(format_moment(f.cfg.date_format_or_default(), &f.now)),
        ("time", None) => Some(format_moment(f.cfg.time_format_or_default(), &f.now)),
        ("date" | "time", Some(fmt)) if !fmt.is_empty() => Some(format_moment(fmt, &f.now)),
        _ if format.is_none() && lower.ends_with("name") => Some(f.name.to_string()),
        _ => None,
    }
}

// -- Insert template ------------------------------------------------------------------

/// A leading YAML frontmatter block: `(block_len_bytes, inner_lines)`, where
/// the block runs from the opening `---` through the closing `---` line and
/// its newline.
fn frontmatter(text: &str) -> Option<(usize, Vec<&str>)> {
    let first_nl = text.find('\n')?;
    if text[..first_nl].trim_end_matches('\r') != "---" {
        return None;
    }
    let mut pos = first_nl + 1;
    let mut inner = Vec::new();
    while pos <= text.len() {
        let end = text[pos..].find('\n').map(|e| pos + e);
        let line = &text[pos..end.unwrap_or(text.len())];
        if line.trim_end_matches('\r') == "---" {
            return Some((end.map(|e| e + 1).unwrap_or(text.len()), inner));
        }
        inner.push(line);
        match end {
            Some(e) => pos = e + 1,
            None => break,
        }
    }
    None
}

/// One top-level property: its key (`None` for lines before the first key)
/// and its raw lines, the key's own line first.
struct Prop {
    key: Option<String>,
    lines: Vec<String>,
}

/// Groups a frontmatter block's `inner` lines into top-level [`Prop`]s: a
/// non-indented `key: …` line starts a new property, everything else
/// (indented list items, comments, a value that overflows one line) is
/// appended to the property above it.
fn props(lines: &[&str]) -> Vec<Prop> {
    let mut out: Vec<Prop> = Vec::new();
    for l in lines {
        let top = !l.starts_with([' ', '\t', '-', '#']) && !l.trim().is_empty();
        if top {
            if let Some((k, _)) = l.split_once(':') {
                out.push(Prop { key: Some(k.trim().to_string()), lines: vec![l.to_string()] });
                continue;
            }
        }
        match out.last_mut() {
            Some(p) => p.lines.push(l.to_string()),
            None => out.push(Prop { key: None, lines: vec![l.to_string()] }),
        }
    }
    out
}

/// A property's list items, if it is a list: a block list (`key:` then
/// `- a` lines), an inline list (`key: [a, b]`), or an empty value (a list
/// of none, so it unions cleanly). `None` for a scalar.
fn list_items(p: &Prop) -> Option<Vec<String>> {
    let head = &p.lines[0];
    let value = head.split_once(':').map(|(_, v)| v.trim()).unwrap_or("");
    let cont: Vec<&String> = p.lines[1..].iter().filter(|l| !l.trim().is_empty()).collect();
    if value.is_empty() {
        if cont.iter().all(|l| l.trim_start().starts_with("- ") || l.trim() == "-") {
            return Some(cont.iter().map(|l| l.trim_start().trim_start_matches('-').trim().to_string()).collect());
        }
        return None;
    }
    if value.starts_with('[') && value.ends_with(']') && cont.is_empty() {
        let inner = &value[1..value.len() - 1];
        return Some(inner.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect());
    }
    None
}

/// Merges the template's properties into the note's, as Obsidian does:
/// a new key is appended; a key both have is unioned when both values are
/// lists (the note's items first) and otherwise takes the template's value.
fn merge_props(note: Vec<Prop>, tpl: Vec<Prop>) -> Vec<String> {
    let mut merged = note;
    for t in tpl {
        let Some(key) = t.key.clone() else { continue };
        match merged.iter_mut().find(|p| p.key.as_deref() == Some(key.as_str())) {
            None => merged.push(t),
            Some(p) => match (list_items(p), list_items(&t)) {
                (Some(mut a), Some(b)) => {
                    let before = a.len();
                    for item in b {
                        if !a.contains(&item) {
                            a.push(item);
                        }
                    }
                    if a.len() != before {
                        let mut lines = vec![format!("{key}:")];
                        lines.extend(a.iter().map(|i| format!("  - {i}")));
                        p.lines = lines;
                    }
                }
                _ => p.lines = t.lines,
            },
        }
    }
    merged.into_iter().flat_map(|p| p.lines).collect()
}

/// Obsidian's **Insert template**: `filled` (a template already through
/// [`fill`]) goes into `note` at the caret, a character offset. The
/// template's frontmatter is merged into the note's rather than pasted into
/// its body ([`merge_props`]); a note without frontmatter gains the
/// template's. A caret inside the note's own frontmatter inserts just after
/// it. Returns the new text and the caret after the inserted body.
pub fn insert(note: &str, caret_chars: usize, filled: &str) -> (String, usize) {
    let (tpl_fm, tpl_body) = match frontmatter(filled) {
        Some((len, lines)) => (Some(lines), &filled[len..]),
        None => (None, filled),
    };
    let (note_fm_len, note_fm) = match frontmatter(note) {
        Some((len, lines)) => (len, Some(lines)),
        None => (0, None),
    };
    let caret_byte = note.char_indices().nth(caret_chars).map(|(b, _)| b).unwrap_or(note.len()).max(note_fm_len);
    let body_before = &note[note_fm_len..caret_byte];
    let body_after = &note[caret_byte..];

    let head = match (note_fm, tpl_fm) {
        (None, None) => String::new(),
        (Some(_), None) => note[..note_fm_len].to_string(),
        (n, Some(t)) => {
            let merged = merge_props(props(&n.unwrap_or_default()), props(&t));
            let mut s = String::from("---\n");
            for l in merged {
                s.push_str(&l);
                s.push('\n');
            }
            s.push_str("---\n");
            s
        }
    };
    let text = format!("{head}{body_before}{tpl_body}{body_after}");
    let caret = head.chars().count() + body_before.chars().count() + tpl_body.chars().count();
    (text, caret)
}

/// Tests for template discovery, the Obsidian-settings resolution order, the
/// Moment.js formatter, placeholder filling and Insert-template's
/// frontmatter merge -- Ruling BF's exactness bar, checked against Obsidian's
/// help page and documented forum behaviour rather than this module's own
/// output.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::EntityKind;

    /// `&str` fixtures to the `Vec<String>` [`discover`] and `list_markdown`
    /// actually deal in.
    fn files(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    /// 2010-02-14 15:25:50 -- the instant Moment's own documentation uses for
    /// its `format()` examples, so the expected strings below are Moment's
    /// published outputs, not values computed by this module.
    fn moment_doc_instant() -> DateTime {
        DateTime::new(2010, 2, 14, 15, 25, 50).unwrap()
    }

    // -- the template folder --------------------------------------------------------

    /// A `Config::read` closure backed by an in-memory map, standing in for
    /// the vault's real file reads.
    fn reader(map: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> {
        let owned: Vec<(String, String)> = map.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect();
        move |p: &str| owned.iter().find(|(k, _)| k == p).map(|(_, v)| v.clone())
    }

    #[test]
    fn templates_json_names_the_folder_and_the_formats() {
        // Protects: templates.json's folder and format keys are read and win
        // over Templater's, and discover() returns only that folder's notes.
        let cfg = Config::read(reader(&[
            (".obsidian/templates.json", r#"{"folder":"Templates","dateFormat":"DD.MM.YYYY","timeFormat":"HH:mm:ss"}"#),
            (".obsidian/plugins/templater-obsidian/data.json", r#"{"templates_folder":"Other"}"#),
        ]));
        assert_eq!(cfg.source, Source::Obsidian, "the core plugin's setting wins over Templater's");
        assert_eq!(cfg.folder.as_deref(), Some("Templates"));
        assert_eq!(cfg.date_format.as_deref(), Some("DD.MM.YYYY"));
        assert_eq!(cfg.time_format.as_deref(), Some("HH:mm:ss"));
        assert_eq!(cfg.describe(), "templates from Templates/ (Obsidian settings)");
        let got = discover(
            &files(&["Index.md", "Settlement Template.md", "Templates/NPC.md", "Templates/People/Ruler.md", "TemplatesX/No.md"]),
            &cfg,
        );
        let pairs: Vec<(&str, &str)> = got.iter().map(|t| (t.rel.as_str(), t.label.as_str())).collect();
        assert_eq!(
            pairs,
            [("Templates/NPC.md", "NPC"), ("Templates/People/Ruler.md", "People/Ruler")],
            "only the folder's files -- not a lookalike folder, not a file named *template* elsewhere"
        );
    }

    #[test]
    fn a_folder_written_with_slashes_is_the_same_folder() {
        // Protects: folder_of() trims slashes and whitespace, an unset
        // format stays None (not the default), and a blank saved format
        // ("" or " ") is treated as unset too.
        let cfg = Config::read(reader(&[(".obsidian/templates.json", r#"{"folder":"/Meta/Templates/"}"#)]));
        assert_eq!(cfg.folder.as_deref(), Some("Meta/Templates"));
        assert_eq!(cfg.date_format, None, "an unset format is absent, not the default");
        // A format field left blank in Obsidian's settings is saved as "" and
        // means the default, not an empty format.
        let blank = Config::read(reader(&[(".obsidian/templates.json", r#"{"folder":"T","dateFormat":"","timeFormat":" "}"#)]));
        assert_eq!((blank.date_format.as_deref(), blank.time_format.as_deref()), (None, None));
        assert_eq!(blank.date_format_or_default(), "YYYY-MM-DD");
        assert_eq!(cfg.date_format_or_default(), "YYYY-MM-DD");
        assert_eq!(cfg.time_format_or_default(), "HH:mm");
    }

    #[test]
    fn templater_alone_names_the_folder() {
        // Protects: Templater's setting names the folder when the core
        // plugin's does not, and the core plugin's date/time formats still
        // apply even when it names no folder.
        let cfg = Config::read(reader(&[(".obsidian/plugins/templater-obsidian/data.json", r#"{"templates_folder":"_tpl","trigger_on_file_creation":false}"#)]));
        assert_eq!(cfg.source, Source::Templater);
        assert_eq!(cfg.folder.as_deref(), Some("_tpl"));
        assert_eq!(cfg.describe(), "templates from _tpl/ (Templater settings)");
        // Core settings present but with no folder: still Templater's.
        let cfg2 = Config::read(reader(&[
            (".obsidian/templates.json", r#"{"dateFormat":"YYYY"}"#),
            (".obsidian/plugins/templater-obsidian/data.json", r#"{"templates_folder":"_tpl"}"#),
        ]));
        assert_eq!(cfg2.source, Source::Templater);
        assert_eq!(cfg2.date_format.as_deref(), Some("YYYY"), "the core formats still apply");
    }

    #[test]
    fn neither_setting_falls_back_to_the_path_rule() {
        // Protects: with no settings at all, discover() falls back to the
        // pre-BF "template" in the path rule and labels by parent folder.
        let cfg = Config::read(reader(&[]));
        assert_eq!(cfg, Config::fallback());
        let got = discover(
            &files(&[
                "Index.md",
                "Landmark template.md",
                "Locations/Nareth.md",
                "Region Template/Landmarks/Landmark template.md",
                "Region Template/Regional Overview.md",
                "Settlement Template.md",
            ]),
            &cfg,
        );
        let rels: Vec<&str> = got.iter().map(|t| t.rel.as_str()).collect();
        assert_eq!(
            rels,
            [
                "Landmark template.md",
                "Region Template/Landmarks/Landmark template.md",
                "Region Template/Regional Overview.md",
                "Settlement Template.md",
            ]
        );
        assert_eq!(got[0].label, "Landmark template");
        assert_eq!(got[1].label, "Landmarks \u{25B8} Landmark template");
    }

    #[test]
    fn malformed_or_empty_settings_are_not_configured() {
        // Protects: unparseable, empty, or non-string/non-folder JSON never
        // names a folder -- it falls back rather than guessing.
        for bad in ["{not json", "[]", r#"{"folder":""}"#, r#"{"folder":"  /  "}"#, r#"{"folder":7}"#, ""] {
            let cfg = Config::read(reader(&[(".obsidian/templates.json", bad)]));
            assert_eq!(cfg.source, Source::Fallback, "{bad:?} must not name a folder");
            assert_eq!(cfg.folder, None);
        }
        let cfg = Config::read(reader(&[(".obsidian/plugins/templater-obsidian/data.json", "{\"templates_folder\":")]));
        assert_eq!(cfg.source, Source::Fallback);
    }

    // -- the clock and Moment ---------------------------------------------------------

    #[test]
    fn datetime_refuses_what_is_not_a_date() {
        // Protects: DateTime::new rejects a non-leap Feb 29, month 13 and
        // hour 24, and accepts a real leap-year Feb 29.
        assert!(DateTime::new(2026, 2, 29, 0, 0, 0).is_none());
        assert!(DateTime::new(2024, 2, 29, 0, 0, 0).is_some());
        assert!(DateTime::new(2026, 13, 1, 0, 0, 0).is_none());
        assert!(DateTime::new(2026, 1, 1, 24, 0, 0).is_none());
    }

    #[test]
    fn moment_documentations_own_examples() {
        // Protects: format_moment reproduces Moment.js's own published
        // examples verbatim, including the English locale long-date formats.
        // https://momentjs.com/docs/#/displaying/format/ -- the examples on
        // the home page, for Sunday 2010-02-14 15:25:50.
        let t = moment_doc_instant();
        assert_eq!(format_moment("dddd, MMMM Do YYYY, h:mm:ss a", &t), "Sunday, February 14th 2010, 3:25:50 pm");
        assert_eq!(format_moment("ddd, hA", &t), "Sun, 3PM");
        assert_eq!(format_moment("[Today is] dddd", &t), "Today is Sunday");
        assert_eq!(format_moment("YYYY [escaped] YYYY", &t), "2010 escaped 2010");
        // The English locale formats (`locale/en` longDateFormat).
        assert_eq!(format_moment("LT", &t), "3:25 PM");
        assert_eq!(format_moment("LTS", &t), "3:25:50 PM");
        assert_eq!(format_moment("L", &t), "02/14/2010");
        assert_eq!(format_moment("l", &t), "2/14/2010");
        assert_eq!(format_moment("LL", &t), "February 14, 2010");
        assert_eq!(format_moment("ll", &t), "Feb 14, 2010");
        assert_eq!(format_moment("LLL", &t), "February 14, 2010 3:25 PM");
        assert_eq!(format_moment("LLLL", &t), "Sunday, February 14, 2010 3:25 PM");
        assert_eq!(format_moment("llll", &t), "Sun, Feb 14, 2010 3:25 PM");
        assert_eq!(format_moment(r"[LL] \LT", &t), "LL LT", "bracketed and escaped locale tokens are text");
    }

    #[test]
    fn every_supported_token_against_a_fixed_instant() {
        // Protects: every token format_moment formats (as opposed to leaving
        // verbatim) against one fixed instant, including an unclosed `[`.
        // Sunday 2026-09-27 09:05:03: day 270 of the year, ISO week 39.
        let t = DateTime::new(2026, 9, 27, 9, 5, 3).unwrap();
        let cases = [
            ("YYYY", "2026"), ("YY", "26"), ("Y", "2026"), ("M", "9"), ("MM", "09"), ("Mo", "9th"),
            ("MMM", "Sep"), ("MMMM", "September"), ("D", "27"), ("DD", "27"), ("Do", "27th"),
            ("DDD", "270"), ("DDDD", "270"), ("d", "0"), ("dd", "Su"), ("ddd", "Sun"), ("dddd", "Sunday"),
            ("E", "7"), ("W", "39"), ("WW", "39"), ("Q", "3"), ("H", "9"), ("HH", "09"), ("h", "9"),
            ("hh", "09"), ("k", "9"), ("m", "5"), ("mm", "05"), ("s", "3"), ("ss", "03"), ("A", "AM"),
            ("a", "am"), ("Hmm", "905"), ("YYYY-MM-DD", "2026-09-27"), ("HH:mm", "09:05"),
            ("[at] HH\\h", "at 09h"), ("\\YYYY", "YYYY"), ("x SSS Z", "x SSS Z"),
            // Moment reads an unclosed `[` as text and everything after it as
            // tokens: `l` expands to `M/D/YYYY`, `s` is seconds, `e` and `d` weekday.
            ("[unclosed", "[unc9/27/2026o300"),
        ];
        for (f, want) in cases {
            assert_eq!(format_moment(f, &t), want, "format {f:?}");
        }
    }

    #[test]
    fn midnight_noon_and_ordinals() {
        // Protects: 12-hour clock at midnight/noon (h/hh/A/k/kk), the
        // ordinal suffix table, and an ISO week crossing a year boundary
        // either way.
        let midnight = DateTime::new(2026, 1, 1, 0, 0, 0).unwrap();
        assert_eq!(format_moment("h hh A k kk dddd W", &midnight), "12 12 AM 24 24 Thursday 1");
        let noon = DateTime::new(2026, 1, 2, 12, 0, 0).unwrap();
        assert_eq!(format_moment("h A", &noon), "12 PM");
        for (d, want) in [(1, "1st"), (2, "2nd"), (3, "3rd"), (4, "4th"), (11, "11th"), (12, "12th"), (13, "13th"), (21, "21st"), (22, "22nd"), (23, "23rd"), (31, "31st")] {
            let t = DateTime::new(2026, 1, d, 0, 0, 0).unwrap();
            assert_eq!(format_moment("Do", &t), want);
        }
        // ISO week that belongs to the previous year: Fri 2021-01-01 is week 53.
        assert_eq!(format_moment("W", &DateTime::new(2021, 1, 1, 0, 0, 0).unwrap()), "53");
        // and one that belongs to the next: Mon 2024-12-30 is week 1.
        assert_eq!(format_moment("W", &DateTime::new(2024, 12, 30, 0, 0, 0).unwrap()), "1");
    }

    // -- placeholders ------------------------------------------------------------------

    #[test]
    fn obsidians_placeholders_are_filled_with_the_vaults_formats() {
        // Protects: title/date/time (with and without an explicit format)
        // fill with the vault's own formats, matching case-insensitively and
        // tolerating spaces inside the braces.
        let cfg = Config { source: Source::Obsidian, folder: Some("Templates".into()), date_format: Some("DD.MM.YYYY".into()), time_format: None };
        let f = Fill { title: "Kel Var", name: "Kel/Var", now: moment_doc_instant(), cfg: &cfg };
        let t = "# {{title}}\ncreated {{date}} at {{time}}\n{{date:dddd}} {{time:YYYY}} {{ Title }} {{DATE}}\n";
        assert_eq!(fill(t, &f), "# Kel Var\ncreated 14.02.2010 at 15:25\nSunday 2010 Kel Var 14.02.2010\n");
    }

    #[test]
    fn the_owners_name_tokens_still_fill_and_everything_else_survives() {
        // Protects: `[Name]` and any `{{...Name}}` token fill with the
        // entity's name, while `[If applicable]`, an unknown token, an empty
        // format and an unclosed token all survive verbatim.
        let cfg = Config::fallback();
        let f = Fill { title: "Kel-Var", name: "Kel/Var", now: moment_doc_instant(), cfg: &cfg };
        let t = "## Settlement Profile: [Name]\n**Former Names:** [If applicable]\n# {{Landmark_Name}} {{Region_Name}}\n\
                 {{date}} {{unknown}} {{date:}} {{title\n";
        assert_eq!(
            fill(t, &f),
            "## Settlement Profile: Kel/Var\n**Former Names:** [If applicable]\n# Kel/Var Kel/Var\n\
             2010-02-14 {{unknown}} {{date:}} {{title\n"
        );
    }

    #[test]
    fn templater_commands_are_left_verbatim() {
        // Protects: `<% ... %>` and `<%* ... %>` blocks are copied verbatim
        // (never executed or imitated), even one containing a `{{date}}`
        // that would otherwise be a placeholder.
        let cfg = Config::fallback();
        let f = Fill { title: "T", name: "N", now: moment_doc_instant(), cfg: &cfg };
        let t = "<% tp.date.now(\"{{date}}\") %> {{date}} <%* tR += tp.file.title %> <% unclosed {{title}}";
        assert_eq!(fill(t, &f), "<% tp.date.now(\"{{date}}\") %> 2010-02-14 <%* tR += tp.file.title %> <% unclosed T");
    }

    #[test]
    fn fill_handles_multibyte_and_unclosed_tokens() {
        // Protects: fill() advances by char, not byte, around multibyte
        // text, and an unclosed `{{` and an empty template are left/returned
        // as-is rather than panicking.
        let cfg = Config::fallback();
        let f = Fill { title: "T", name: "X", now: moment_doc_instant(), cfg: &cfg };
        assert_eq!(fill("Ré{{Name}}ém", &f), "RéXém");
        assert_eq!(fill("{{Name", &f), "{{Name");
        assert_eq!(fill("", &f), "");
    }

    #[test]
    fn title_is_the_basename() {
        // Protects: title_of strips any extension case-insensitively and
        // returns the whole string when there is no path or extension.
        assert_eq!(title_of("Settlements/Kel Var.md"), "Kel Var");
        assert_eq!(title_of("Root.MD"), "Root");
        assert_eq!(title_of("NoExt"), "NoExt");
    }

    // -- insert --------------------------------------------------------------------------

    #[test]
    fn insert_puts_the_body_at_the_caret() {
        // Protects: insert() splices the filled template in at the caret and
        // returns the caret position after it, counting characters (not
        // bytes) across a multibyte note.
        let (t, c) = insert("ab\ncd", 3, "X\nY\n");
        assert_eq!(t, "ab\nX\nY\ncd");
        assert_eq!(c, 7);
        let (t, c) = insert("é", 1, "z");
        assert_eq!((t.as_str(), c), ("éz", 2), "the caret counts characters, not bytes");
    }

    #[test]
    fn a_template_with_properties_merges_them_like_obsidian() {
        // Protects: merge_props unions list properties (note's items first),
        // a scalar takes the template's value, and a key only the template
        // has is appended -- Obsidian's forum-documented merge behaviour.
        let note = "---\ntags:\n  - foo\n  - bar\ndate: 2020-12-12\nkeep: me\n---\nBody\n";
        let tpl = "---\ntags: [journal, foo]\ndate: 2010-02-14\nnew: yes\n---\n## Log\n";
        let caret = note.chars().count(); // end of the note
        let (t, c) = insert(note, caret, tpl);
        assert_eq!(
            t,
            "---\ntags:\n  - foo\n  - bar\n  - journal\ndate: 2010-02-14\nkeep: me\nnew: yes\n---\nBody\n## Log\n",
            "lists union (note's items first), a scalar takes the template's value, a new key is appended"
        );
        assert_eq!(c, t.chars().count());
    }

    #[test]
    fn a_note_without_properties_gains_the_templates() {
        // Protects: a note with no frontmatter gains the template's whole,
        // a caret inside the note's own frontmatter still inserts just below
        // it, and an already-matching list keeps its own spelling rather
        // than being rewritten.
        let (t, c) = insert("Body", 4, "---\na: 1\n---\n!");
        assert_eq!(t, "---\na: 1\n---\nBody!");
        assert_eq!(c, 18);
        // A caret inside the note's own properties inserts just below them.
        let (t2, _) = insert("---\nx: 1\n---\nB", 5, "I");
        assert_eq!(t2, "---\nx: 1\n---\nIB");
        // An unchanged list keeps its own spelling.
        let (t3, _) = insert("---\ntags: [a]\n---\n", 0, "---\ntags: [a]\n---\n");
        assert_eq!(t3, "---\ntags: [a]\n---\n");
    }

    // -- unchanged helpers -------------------------------------------------------------------

    #[test]
    fn suggested_paths_follow_v3s_convention_for_every_kind() {
        // Protects: suggested_path names the right plural folder for every
        // EntityKind this port can address, v3's `{Folder}/{name}.md`.
        assert_eq!(suggested_path(EntityKind::Settlement, "Nareth"), "Settlements/Nareth.md");
        assert_eq!(suggested_path(EntityKind::Province, "Lower Vale"), "Provinces/Lower Vale.md");
        assert_eq!(suggested_path(EntityKind::Continent, "Vantharis"), "Continents/Vantharis.md");
        assert_eq!(suggested_path(EntityKind::Faction, "Draumr League"), "Factions/Draumr League.md");
        assert_eq!(suggested_path(EntityKind::Culture, "Riverlands"), "Cultures/Riverlands.md");
        assert_eq!(suggested_path(EntityKind::Landmark, "Waterfall (120, 64)"), "Landmarks/Waterfall (120, 64).md");
    }

    #[test]
    fn a_name_that_is_not_a_filename_still_becomes_one() {
        // Protects: sanitise() replaces every filesystem-reserved character
        // with `-`, trims leading/trailing dots and whitespace, falls back
        // to "Untitled" for an empty result, and keeps distinct names
        // distinct after sanitising.
        assert_eq!(sanitise("Kel/Var: the Deep"), "Kel-Var- the Deep");
        assert_eq!(sanitise("  ..  "), "Untitled");
        assert_eq!(sanitise("Trailing."), "Trailing");
        assert_eq!(sanitise(""), "Untitled");
        assert_ne!(sanitise("A:B"), sanitise("A?"));
    }
}
