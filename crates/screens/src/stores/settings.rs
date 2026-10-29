//! **The settings store** — the viewer's preferences the Settings pages show and change.
//!
//! A preference is a rung on a ladder ([`Pref`]): the subtitle size, the subtitle position and
//! Skip intros (Off, Show button, Automatic — the player's Skip button, [`SKIP_OFF`]…).
//! The ladders' rungs are the media layer's (`sn_media::prefs::SubtitleSize::LADDER`,
//! `SubtitlePosition::LADDER`) — this store keeps the index, the application maps it to the
//! media type, publishes it to the player and persists it, and its tests hold the two ladders the
//! same length with the same default. The audio, subtitle and stream-sorting languages are not
//! ladders but ordered lists the viewer edits ([`LangList`], [`SettingsStore::edit`]). The search
//! addon filter and the recent searches are settings too, held by the search store beside the rows
//! they shape.

/// A preference the Settings pages offer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Pref {
    SubtitleSize = 0,
    SubtitlePosition = 1,
    SkipIntros = 2,
    /// Dolby Vision Profile 7: HDR10 (the base layer) or converted to Profile 8.1 (`sn_media::prefs::DvProfile7`).
    DvProfile7 = 3,
    /// Whether subtitles show: [`SUBS_OFF`], [`SUBS_ON`], [`SUBS_FORCED`], [`SUBS_AUTO`].
    SubtitleMode = 4,
    /// What an episode's end (or Up Next) plays: [`NEXT_SAME`] or [`NEXT_CHOOSE`].
    NextEpisode = 5,
    /// Off (rung 0) or On: Forced only also accepts a forced track in a subtitle language when the
    /// audio's language has none (`TrackPrefs::forced_fallback`).
    ForcedFallback = 6,
    /// The text caption's colour, box, outline and opacity (`sn_media::prefs::SubtitleColor`,
    /// `SubtitleBackground`, `SubtitleOutline`, `SubtitleOpacity`; stremio's `subtitlesTextColor`,
    /// `subtitlesBackgroundColor`, `subtitlesOutlineColor`, `subtitlesOpacity`).
    SubtitleColor = 7,
    SubtitleBackground = 8,
    SubtitleOutline = 9,
    SubtitleOpacity = 10,
    /// Style ASS subtitles: Off (rung 0, an ASS track is plain dialogue in the caption style) or
    /// On (it keeps its own styling, the default). Stremio's `assSubtitlesStyling`.
    AssStyling = 11,
    /// Auto-play next episode: Off (rung 0, an episode's end returns to where it was played from;
    /// Up Next is still offered) or On (the default). Stremio's `bingeWatching`.
    AutoPlay = 12,
    /// When Up Next appears: [`UP_NEXT_CREDITS`] (at the credits marker when the episode has one,
    /// else [`UP_NEXT_FALLBACK_S`] before the end — the default), or a fixed lead
    /// ([`UP_NEXT_LEADS_S`]). Stremio's `nextVideoNotificationDuration`.
    UpNextAt = 13,
    /// The interface language: rung `r` is `sn_platform::i18n::Preference::ALL[r]` (System
    /// first, the default). Read once at boot, so a change takes effect at the next launch.
    Language = 14,
    /// What a track of more than two channels the app decodes itself becomes
    /// (`sn_media::prefs::MultichannelAudio`): [`MULTI_AUTO`] (the default), [`MULTI_DOLBY_DIGITAL`],
    /// [`MULTI_DOLBY_DIGITAL_PLUS`], [`MULTI_PCM`], [`MULTI_STEREO`], [`MULTI_DPLII`].
    MultichannelAudio = 15,
    /// The programme guide Live TV shows now / next from: Off (rung 0) or a region's public XMLTV
    /// guide, rungs 1… in [`TV_GUIDES`] order.
    TvGuide = 16,
    /// How each addon's streams are listed: [`ORDER_ADDON`] (as the addon sent them, the
    /// default), [`ORDER_LANGUAGE`] (cached first, then the Stream sorting languages) or
    /// [`ORDER_QUALITY`] (cached first, then the highest resolution).
    StreamOrder = 17,
}

/// [`Pref::StreamOrder`]'s rungs.
pub const ORDER_ADDON: u8 = 0;
pub const ORDER_LANGUAGE: u8 = 1;
pub const ORDER_QUALITY: u8 = 2;

/// [`Pref::TvGuide`]'s regions, rungs 1… in order: the settings file's name for each and its
/// public XMLTV guide (epgshare01, rebuilt daily from the broadcasters' own listings).
pub const TV_GUIDES: [(&str, &str); 7] = [
    ("de", "https://epgshare01.online/epgshare01/epg_ripper_DE1.xml.gz"),
    ("at", "https://epgshare01.online/epgshare01/epg_ripper_AT1.xml.gz"),
    ("ch", "https://epgshare01.online/epgshare01/epg_ripper_CH1.xml.gz"),
    ("uk", "https://epgshare01.online/epgshare01/epg_ripper_UK1.xml.gz"),
    ("fr", "https://epgshare01.online/epgshare01/epg_ripper_FR1.xml.gz"),
    ("it", "https://epgshare01.online/epgshare01/epg_ripper_IT1.xml.gz"),
    ("es", "https://epgshare01.online/epgshare01/epg_ripper_ES1.xml.gz"),
];

/// The guide [`Pref::TvGuide`]'s `rung` loads; `None` for Off.
pub fn tv_guide_url(rung: u8) -> Option<&'static str> {
    TV_GUIDES.get(usize::from(rung).checked_sub(1)?).map(|g| g.1)
}

/// The settings file's name for [`Pref::TvGuide`]'s `rung` (empty for Off), and back.
pub fn tv_guide_name(rung: u8) -> &'static str {
    usize::from(rung).checked_sub(1).and_then(|i| TV_GUIDES.get(i)).map_or("", |g| g.0)
}
pub fn tv_guide_rung(name: &str) -> u8 {
    TV_GUIDES.iter().position(|g| g.0 == name).map_or(0, |i| i as u8 + 1)
}

/// [`Pref::MultichannelAudio`]'s rungs (`sn_media::prefs::MultichannelAudio::LADDER` order).
pub const MULTI_AUTO: u8 = 0;
pub const MULTI_DOLBY_DIGITAL: u8 = 1;
pub const MULTI_DOLBY_DIGITAL_PLUS: u8 = 2;
pub const MULTI_PCM: u8 = 3;
pub const MULTI_STEREO: u8 = 4;
pub const MULTI_DPLII: u8 = 5;

/// [`Pref::UpNextAt`]'s rung 0, and the lead it falls back to without a credits marker
/// (stremio-core's default `nextVideoNotificationDuration`, 35 s).
pub const UP_NEXT_CREDITS: u8 = 0;
pub const UP_NEXT_FALLBACK_S: u64 = 35;
/// [`Pref::UpNextAt`]'s fixed leads, rungs 1… in order (seconds before the end; a credits marker
/// is not consulted).
pub const UP_NEXT_LEADS_S: [u64; 4] = [15, 25, 35, 45];

impl Pref {
    /// When Up Next appears at rung `r`: the lead before the end in ms, and whether a credits
    /// marker takes over when the episode has one.
    pub fn up_next_lead(r: u8) -> (u64, bool) {
        match r.checked_sub(1).and_then(|i| UP_NEXT_LEADS_S.get(usize::from(i))) {
            Some(s) => (s * 1000, false),
            None => (UP_NEXT_FALLBACK_S * 1000, true),
        }
    }
}

/// [`Pref::NextEpisode`]'s rungs: the next episode's stream of the same release when its addon
/// has one (else the stream list), or always the stream list.
pub const NEXT_SAME: u8 = 0;
pub const NEXT_CHOOSE: u8 = 1;

/// [`Pref::SubtitleMode`]'s rungs: none; always (the preferred language's); only a forced track
/// (signs and foreign dialogue); and on only when the audio is not in the first preferred audio
/// language (forced only when it is).
pub const SUBS_OFF: u8 = 0;
pub const SUBS_ON: u8 = 1;
pub const SUBS_FORCED: u8 = 2;
pub const SUBS_AUTO: u8 = 3;

/// **The three ordered language lists** the viewer edits (Settings > Playback), first preferred
/// first, as `sn_base::tracks::language_code` codes (`"DE"`):
///
/// - [`LangList::Audio`]: the audio track a file starts on; empty plays the file's default track.
/// - [`LangList::Subtitles`]: the subtitles shown; empty is the app's language, then English.
/// - [`LangList::Streams`]: the streams a stream list puts first; it follows the audio list until
///   the viewer gives it its own (and an empty audio list sorts by the app's language, then English).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LangList {
    Audio = 0,
    Subtitles = 1,
    Streams = 2,
}

/// An edit of a [`LangList`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LangOp {
    /// Add a language (any case, ISO 639-1 or -2) at the end (nothing when it is listed, or not a
    /// language the app names).
    Add(&'static str),
    /// Remove the `n`-th.
    Remove(usize),
    /// Move the `n`-th one place earlier / later.
    Up(usize),
    Down(usize),
    /// [`LangList::Streams`] only: follow the audio list (`true`), or keep a copy of it as its own.
    SameAsAudio(bool),
}

/// The language orders the settings file's old rungs named (`stream_languages`, `audio_language`,
/// `subtitle_language`, rung `r` past 0 being entry `r - 1`): what [`migrate_rungs`] turns them into.
const OLD_LANGUAGE_RUNGS: [&[&str]; 6] = [&["EN"], &["DE", "EN"], &["FR", "EN"], &["ES", "EN"], &["IT", "EN"], &["RU", "EN"]];

/// The app's language, then English (Belarusian reads Russian, the language its dubs are in,
/// before English).
fn app_languages() -> &'static [&'static str] {
    match sn_platform::i18n::current().language().tag() {
        "es" => &["ES", "EN"],
        "fr" => &["FR", "EN"],
        "it" => &["IT", "EN"],
        "de" => &["DE", "EN"],
        "be" => &["RU", "EN"],
        _ => &["EN"],
    }
}

fn own(l: &[&str]) -> Vec<String> {
    l.iter().map(|c| (*c).to_string()).collect()
}

/// **An earlier build's language rungs as the lists that mean the same** (`audio`, `subtitles`,
/// `streams`): an old rung past 0 is its order; audio rung 0 (the file's default) and subtitle rung 0
/// (the app's language, then English) are the empty lists that still mean exactly that; stream rung
/// 0 (the app's language) follows the audio list when that is also at rung 0 (which then sorts by the
/// app's language, the same answer) and is the app's languages as its own list otherwise. An
/// out-of-range rung is rung 0.
pub fn migrate_rungs(stream: u8, audio: u8, subtitle: u8) -> (Vec<String>, Vec<String>, Option<Vec<String>>) {
    let order = |r: u8| r.checked_sub(1).and_then(|r| OLD_LANGUAGE_RUNGS.get(usize::from(r))).map(|l| own(l));
    let audio_list = order(audio).unwrap_or_default();
    let streams = match order(stream) {
        Some(l) => Some(l),
        None if audio_list.is_empty() => None,
        None => Some(own(app_languages())),
    };
    (audio_list, order(subtitle).unwrap_or_default(), streams)
}

/// `codes` as a list the store keeps: each a language the app names, as its code, once.
fn normalized(codes: &[String]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for c in codes {
        let code = language_code(c);
        if sn_base::tracks::language_index(&code).is_some() && !out.contains(&code) {
            out.push(code);
        }
    }
    out
}

pub use sn_base::tracks::{language_code, Track, TrackPrefs};

/// The tracks [`SettingsStore::pick_tracks`] chose, as indices into the lists it was given.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Picked {
    pub audio: Option<usize>,
    /// `None`: subtitles off.
    pub subtitle: Option<usize>,
}

/// [`Pref::SkipIntros`]' rungs.
pub const SKIP_OFF: u8 = 0;
pub const SKIP_BUTTON: u8 = 1;
pub const SKIP_AUTO: u8 = 2;

impl Pref {
    pub const ALL: [Pref; PREFS] = [
        Pref::SubtitleSize,
        Pref::SubtitlePosition,
        Pref::SkipIntros,
        Pref::DvProfile7,
        Pref::SubtitleMode,
        Pref::NextEpisode,
        Pref::ForcedFallback,
        Pref::SubtitleColor,
        Pref::SubtitleBackground,
        Pref::SubtitleOutline,
        Pref::SubtitleOpacity,
        Pref::AssStyling,
        Pref::AutoPlay,
        Pref::UpNextAt,
        Pref::Language,
        Pref::MultichannelAudio,
        Pref::TvGuide,
        Pref::StreamOrder,
    ];

    /// How many rungs the ladder has (`sn_media::prefs`'s `LADDER.len()`).
    pub const fn rungs(self) -> u8 {
        match self {
            Pref::SubtitleSize => 5,
            Pref::SubtitlePosition => 3,
            Pref::SkipIntros => 3,
            Pref::DvProfile7 => 2,
            Pref::SubtitleMode => 4,
            Pref::NextEpisode => 2,
            Pref::ForcedFallback => 2,
            Pref::SubtitleColor => 5,
            Pref::SubtitleBackground => 4,
            Pref::SubtitleOutline => 3,
            Pref::SubtitleOpacity => 3,
            Pref::AssStyling => 2,
            Pref::AutoPlay => 2,
            Pref::UpNextAt => 1 + UP_NEXT_LEADS_S.len() as u8,
            Pref::Language => sn_platform::i18n::Preference::ALL.len() as u8,
            Pref::MultichannelAudio => 6,
            Pref::TvGuide => 1 + TV_GUIDES.len() as u8,
            Pref::StreamOrder => 3,
        }
    }

    /// The preference's name (the Settings row, the picker's title, the player's `…` menu row).
    pub fn label(self) -> &'static str {
        use sn_platform::i18n::msg;
        match self {
            Pref::SubtitleSize => msg::app_settings_subtitle_size(),
            Pref::SubtitlePosition => msg::app_settings_subtitle_position(),
            Pref::SkipIntros => msg::app_settings_skip_intros(),
            Pref::DvProfile7 => msg::app_settings_dv_profile7(),
            Pref::SubtitleMode => msg::app_settings_subtitle_mode(),
            Pref::NextEpisode => msg::app_settings_next_episode(),
            Pref::ForcedFallback => msg::app_settings_forced_fallback(),
            Pref::SubtitleColor => msg::app_settings_subtitle_color(),
            Pref::SubtitleBackground => msg::app_settings_subtitle_background(),
            Pref::SubtitleOutline => msg::app_settings_subtitle_outline(),
            Pref::SubtitleOpacity => msg::app_settings_subtitle_opacity(),
            Pref::AssStyling => msg::app_settings_ass_styling(),
            Pref::AutoPlay => msg::app_settings_auto_play(),
            Pref::UpNextAt => msg::app_settings_up_next_at(),
            Pref::Language => msg::app_settings_language(),
            Pref::MultichannelAudio => msg::app_settings_multichannel_audio(),
            Pref::TvGuide => msg::app_settings_tv_guide(),
            Pref::StreamOrder => msg::app_settings_stream_order(),
        }
    }

    /// The label of rung `r`.
    pub fn rung_label(self, r: u8) -> &'static str {
        use sn_platform::i18n::msg;
        match (self, r) {
            (Pref::SubtitleSize, 0) => msg::app_settings_size_small(),
            (Pref::SubtitleSize, 1) => msg::app_settings_size_medium(),
            (Pref::SubtitleSize, 2) => msg::app_settings_size_large(),
            (Pref::SubtitleSize, 3) => msg::app_settings_size_extra_large(),
            (Pref::SubtitleSize, _) => msg::app_settings_size_huge(),
            (Pref::SubtitlePosition, 0) => msg::app_settings_position_low(),
            (Pref::SubtitlePosition, 1) => msg::app_settings_position_middle(),
            (Pref::SubtitlePosition, _) => msg::app_settings_position_high(),
            (Pref::SkipIntros, SKIP_OFF) => msg::app_settings_skip_off(),
            (Pref::SkipIntros, SKIP_BUTTON) => msg::app_settings_skip_button(),
            (Pref::SkipIntros, _) => msg::app_settings_skip_auto(),
            (Pref::DvProfile7, 0) => msg::app_settings_dv_hdr10(),
            (Pref::DvProfile7, _) => msg::app_settings_dv_try(),
            (Pref::SubtitleMode, SUBS_OFF) => msg::app_settings_subs_off(),
            (Pref::SubtitleMode, SUBS_ON) => msg::app_settings_subs_on(),
            (Pref::SubtitleMode, SUBS_FORCED) => msg::app_settings_subs_forced(),
            (Pref::SubtitleMode, _) => msg::app_settings_subs_auto(),
            (Pref::NextEpisode, NEXT_SAME) => msg::app_settings_next_same(),
            (Pref::NextEpisode, _) => msg::app_settings_next_choose(),
            (Pref::ForcedFallback, 0) => msg::app_settings_forced_fallback_off(),
            (Pref::ForcedFallback, _) => msg::app_settings_forced_fallback_on(),
            (Pref::SubtitleColor, 0) => msg::app_settings_color_white(),
            (Pref::SubtitleColor, 1) => msg::app_settings_color_yellow(),
            (Pref::SubtitleColor, 2) => msg::app_settings_color_cyan(),
            (Pref::SubtitleColor, 3) => msg::app_settings_color_green(),
            (Pref::SubtitleColor, _) => msg::app_settings_color_grey(),
            (Pref::SubtitleBackground, 0) => msg::app_settings_background_none(),
            (Pref::SubtitleBackground, 1) => msg::app_settings_background_black_50(),
            (Pref::SubtitleBackground, 2) => msg::app_settings_background_black_75(),
            (Pref::SubtitleBackground, _) => msg::app_settings_background_black(),
            (Pref::SubtitleOutline, 0) => msg::app_settings_color_black(),
            (Pref::SubtitleOutline, 1) => msg::app_settings_color_grey(),
            (Pref::SubtitleOutline, _) => msg::app_settings_color_white(),
            (Pref::SubtitleOpacity, 0) => msg::app_settings_opacity_100(),
            (Pref::SubtitleOpacity, 1) => msg::app_settings_opacity_75(),
            (Pref::SubtitleOpacity, _) => msg::app_settings_opacity_50(),
            (Pref::AssStyling | Pref::AutoPlay, 0) => msg::app_settings_switch_off(),
            (Pref::AssStyling | Pref::AutoPlay, _) => msg::app_settings_switch_on(),
            (Pref::UpNextAt, UP_NEXT_CREDITS) => msg::app_settings_up_next_credits(),
            (Pref::UpNextAt, 1) => msg::app_settings_up_next_15(),
            (Pref::UpNextAt, 2) => msg::app_settings_up_next_25(),
            (Pref::UpNextAt, 3) => msg::app_settings_up_next_35(),
            (Pref::UpNextAt, _) => msg::app_settings_up_next_45(),
            (Pref::MultichannelAudio, MULTI_AUTO) => msg::app_settings_multichannel_auto(),
            (Pref::MultichannelAudio, MULTI_DOLBY_DIGITAL) => msg::app_settings_multichannel_dolby_digital(),
            (Pref::MultichannelAudio, MULTI_DOLBY_DIGITAL_PLUS) => msg::app_settings_multichannel_dolby_digital_plus(),
            (Pref::MultichannelAudio, MULTI_PCM) => msg::app_settings_multichannel_pcm(),
            (Pref::MultichannelAudio, MULTI_STEREO) => msg::app_settings_multichannel_stereo(),
            (Pref::MultichannelAudio, _) => msg::app_settings_multichannel_dplii(),
            (Pref::TvGuide, 0) => msg::app_settings_switch_off(),
            (Pref::TvGuide, 1) => msg::app_settings_tv_guide_de(),
            (Pref::TvGuide, 2) => msg::app_settings_tv_guide_at(),
            (Pref::TvGuide, 3) => msg::app_settings_tv_guide_ch(),
            (Pref::TvGuide, 4) => msg::app_settings_tv_guide_uk(),
            (Pref::TvGuide, 5) => msg::app_settings_tv_guide_fr(),
            (Pref::TvGuide, 6) => msg::app_settings_tv_guide_it(),
            (Pref::TvGuide, _) => msg::app_settings_tv_guide_es(),
            (Pref::StreamOrder, ORDER_ADDON) => msg::app_settings_stream_order_addon(),
            (Pref::StreamOrder, ORDER_LANGUAGE) => msg::app_settings_stream_order_language(),
            (Pref::StreamOrder, _) => msg::app_settings_stream_order_quality(),
            (Pref::Language, r) => sn_platform::i18n::Preference::ALL[usize::from(r).min(sn_platform::i18n::Preference::ALL.len() - 1)].native_name(),
        }
    }

    /// The rung a fresh install starts on (`sn_media::prefs`'s `#[default]`).
    pub const fn default_rung(self) -> u8 {
        match self {
            Pref::SubtitleSize => 1,
            Pref::SubtitlePosition => 0,
            Pref::SkipIntros => SKIP_BUTTON,
            Pref::DvProfile7 => 0,
            Pref::SubtitleMode => SUBS_OFF,
            Pref::NextEpisode => NEXT_SAME,
            Pref::ForcedFallback => 0,
            Pref::SubtitleColor | Pref::SubtitleBackground | Pref::SubtitleOutline | Pref::SubtitleOpacity => 0,
            Pref::AssStyling => 1,
            Pref::AutoPlay => 1,
            Pref::UpNextAt => UP_NEXT_CREDITS,
            Pref::Language => 0,
            Pref::MultichannelAudio => MULTI_AUTO,
            Pref::TvGuide => 0,
            Pref::StreamOrder => ORDER_ADDON,
        }
    }
}

/// How many preferences [`Pref::ALL`] holds.
pub const PREFS: usize = 18;

/// The pages that show their items as posters or as a list, each remembering its own choice.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ViewPage {
    Search = 0,
    Library = 1,
    Catalog = 2,
    /// Home's catalog rows and Continue Watching (and the Continue Watching page it opens).
    Home = 3,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SettingsStore {
    rungs: [u8; PREFS],
    /// The [`LangList`]s: audio, subtitles, and the stream order (`None` follows audio).
    audio: Vec<String>,
    subtitles: Vec<String>,
    streams: Option<Vec<String>>,
    /// Per [`ViewPage`]: `true` shows the page's items as a list instead of posters.
    lists: [bool; 4],
    /// The Library page's order (a per-device choice, kept in the settings file).
    library_sort: sn_stremio::library::Sort,
    /// The audio sync the viewer chose per stream source ([`SettingsStore::audio_offset`]), oldest
    /// first, at most [`MAX_AUDIO_OFFSETS`].
    audio_offsets: Vec<(String, i64)>,
    gen: u32,
}

/// How many sources' audio sync are kept (the least recently set goes first).
pub const MAX_AUDIO_OFFSETS: usize = 200;

impl Default for SettingsStore {
    fn default() -> Self {
        SettingsStore {
            audio_offsets: Vec::new(),
            rungs: Pref::ALL.map(Pref::default_rung),
            audio: Vec::new(),
            subtitles: Vec::new(),
            streams: None,
            lists: [false; 4],
            library_sort: sn_stremio::library::Sort::LastWatched,
            gen: 0,
        }
    }
}

impl SettingsStore {
    /// A store seeded from what the settings file held (an out-of-range rung is the default).
    pub fn with(size: u8, position: u8, skip: u8) -> Self {
        let mut s = Self::default();
        s.rungs[..3].copy_from_slice(&[size, position, skip]);
        for p in Pref::ALL {
            if s.rungs[p as usize] >= p.rungs() {
                s.rungs[p as usize] = p.default_rung();
            }
        }
        s
    }

    pub fn gen(&self) -> u32 {
        self.gen
    }

    pub fn get(&self, p: Pref) -> u8 {
        self.rungs[p as usize]
    }

    /// Set `p` to `rung`; `false` (and nothing moves) when it already is, or out of range.
    pub fn set(&mut self, p: Pref, rung: u8) -> bool {
        if rung >= p.rungs() || self.rungs[p as usize] == rung {
            return false;
        }
        self.rungs[p as usize] = rung;
        self.gen = self.gen.wrapping_add(1);
        true
    }

    /// The store with the Profile 7 choice the settings file held (seeding: the generation stays;
    /// out of range is the default).
    pub fn with_dv_profile7(mut self, rung: u8) -> Self {
        if rung < Pref::DvProfile7.rungs() {
            self.rungs[Pref::DvProfile7 as usize] = rung;
        }
        self
    }

    /// The store with `p` at the `rung` the settings file held (seeding: the generation stays;
    /// out of range keeps what it was).
    pub fn seeded(mut self, p: Pref, rung: u8) -> Self {
        if rung < p.rungs() {
            self.rungs[p as usize] = rung;
        }
        self
    }

    /// The store with the language lists the settings file held (seeding; a code the app does not
    /// name, or a repeat, is dropped). `streams: None` follows the audio list.
    pub fn with_languages(mut self, audio: &[String], subtitles: &[String], streams: Option<&[String]>) -> Self {
        self.audio = normalized(audio);
        self.subtitles = normalized(subtitles);
        self.streams = streams.map(normalized);
        self
    }

    /// The list `l` as the viewer set it (the stream order: its own, else the audio list it follows).
    pub fn languages(&self, l: LangList) -> &[String] {
        match l {
            LangList::Audio => &self.audio,
            LangList::Subtitles => &self.subtitles,
            LangList::Streams => self.streams.as_deref().unwrap_or(&self.audio),
        }
    }

    /// Does the stream order follow the audio list?
    pub fn streams_follow_audio(&self) -> bool {
        self.streams.is_none()
    }

    /// **Edit a language list**; `false` (and nothing moves) when the edit changes nothing. An edit
    /// of a stream order that follows the audio list starts its own from a copy of it.
    pub fn edit(&mut self, l: LangList, op: LangOp) -> bool {
        if let LangOp::SameAsAudio(follow) = op {
            if l != LangList::Streams || follow == self.streams.is_none() {
                return false;
            }
            self.streams = (!follow).then(|| self.audio.clone());
            self.gen = self.gen.wrapping_add(1);
            return true;
        }
        let was_following = self.streams.is_none();
        let list = match l {
            LangList::Audio => &mut self.audio,
            LangList::Subtitles => &mut self.subtitles,
            LangList::Streams => self.streams.get_or_insert_with(|| self.audio.clone()),
        };
        let changed = match op {
            LangOp::Add(code) => {
                let code = language_code(code);
                let ok = sn_base::tracks::language_index(&code).is_some() && !list.contains(&code);
                if ok {
                    list.push(code);
                }
                ok
            }
            LangOp::Remove(i) if i < list.len() => {
                list.remove(i);
                true
            }
            LangOp::Up(i) if i > 0 && i < list.len() => {
                list.swap(i - 1, i);
                true
            }
            LangOp::Down(i) if i + 1 < list.len() => {
                list.swap(i, i + 1);
                true
            }
            _ => false,
        };
        // an edit that changed nothing leaves a following stream order following
        if !changed && l == LangList::Streams && was_following {
            self.streams = None;
        }
        if changed {
            self.gen = self.gen.wrapping_add(1);
        }
        changed
    }

    /// The audio languages whose streams a stream list puts first, first preferred first
    /// (`sn_stremio::stream_facts::Facts::languages` codes): the stream order, else the audio list,
    /// else the app's language, then English.
    pub fn stream_languages(&self) -> Vec<String> {
        let l = self.languages(LangList::Streams);
        if l.is_empty() { own(app_languages()) } else { l.to_vec() }
    }

    /// **The track preferences a playback starts with**, as the plain data the media layer
    /// picks by ([`TrackPrefs`]): the audio and subtitle lists (no subtitle list: the app's
    /// language, then English), [`Pref::SubtitleMode`] and [`Pref::ForcedFallback`]; Auto's
    /// understood language is the first preferred audio language (the app's language while audio
    /// follows the file).
    pub fn track_prefs(&self) -> TrackPrefs {
        TrackPrefs {
            audio: self.audio.clone(),
            subtitles: if self.subtitles.is_empty() { own(app_languages()) } else { self.subtitles.clone() },
            mode: match self.get(Pref::SubtitleMode) {
                SUBS_OFF => sn_base::tracks::SubsMode::Off,
                SUBS_ON => sn_base::tracks::SubsMode::On,
                SUBS_FORCED => sn_base::tracks::SubsMode::Forced,
                _ => sn_base::tracks::SubsMode::Auto,
            },
            native: self.audio.first().cloned().unwrap_or_else(|| app_languages()[0].to_string()),
            forced_fallback: self.get(Pref::ForcedFallback) != 0,
        }
    }

    /// **The audio and subtitle tracks to start a file on**, by [`Self::track_prefs`]
    /// (`sn_base::tracks`). PURE.
    ///
    /// Audio: the first track in the first preferred language that has one, else the file's
    /// default track, else the first (with the preference at "the file's default", the default
    /// track). Subtitles: Off is none; On the first non-forced track in the first subtitle
    /// language that has one (then a forced one, then the file's default, then the first); Forced
    /// only a forced track in the audio's language (with [`Pref::ForcedFallback`], else one in a
    /// subtitle language; else none); Auto
    /// is Forced only when the chosen audio is in the first preferred audio language (the app's
    /// language while audio follows the file) and On otherwise.
    pub fn pick_tracks(&self, audio: &[Track<'_>], subs: &[Track<'_>]) -> Picked {
        let prefs = self.track_prefs();
        let a = prefs.audio(audio);
        let subtitle = prefs.subtitle(a.map_or("", |i| audio[i].language), subs);
        Picked { audio: a, subtitle }
    }

    /// The store with the list choices the settings file held (seeding: the generation stays).
    pub fn with_lists(mut self, lists: [bool; 4]) -> Self {
        self.lists = lists;
        self
    }

    /// The Library page's order.
    pub fn library_sort(&self) -> sn_stremio::library::Sort {
        self.library_sort
    }

    /// Seed the Library page's order (the settings file); no change.
    pub fn with_library_sort(mut self, sort: sn_stremio::library::Sort) -> Self {
        self.library_sort = sort;
        self
    }

    /// The audio sync (ms) the viewer chose for stream source `key` — a live channel's id, else the
    /// item's — 0 when none. Kept on this device only.
    pub fn audio_offset(&self, key: &str) -> i64 {
        self.audio_offsets.iter().find(|(k, _)| k == key).map_or(0, |(_, ms)| *ms)
    }

    /// Remember `ms` for source `key` (0 forgets it); `false` when nothing moved.
    pub fn set_audio_offset(&mut self, key: &str, ms: i64) -> bool {
        if key.is_empty() || self.audio_offset(key) == ms {
            return false;
        }
        self.audio_offsets.retain(|(k, _)| k != key);
        if ms != 0 {
            self.audio_offsets.push((key.to_string(), ms));
            let over = self.audio_offsets.len().saturating_sub(MAX_AUDIO_OFFSETS);
            self.audio_offsets.drain(..over);
        }
        self.gen = self.gen.wrapping_add(1);
        true
    }

    /// Every remembered audio sync, oldest first (the settings file's).
    pub fn audio_offsets(&self) -> &[(String, i64)] {
        &self.audio_offsets
    }

    /// The store with the audio syncs the settings file held (seeding).
    pub fn with_audio_offsets(mut self, offsets: Vec<(String, i64)>) -> Self {
        self.audio_offsets = offsets.into_iter().filter(|(k, ms)| !k.is_empty() && *ms != 0).collect();
        self
    }

    /// Order the Library page by `sort`; `false` when it already is.
    pub fn set_library_sort(&mut self, sort: sn_stremio::library::Sort) -> bool {
        if self.library_sort == sort {
            return false;
        }
        self.library_sort = sort;
        self.gen = self.gen.wrapping_add(1);
        true
    }

    /// Does `page` show its items as a list?
    pub fn list(&self, page: ViewPage) -> bool {
        self.lists[page as usize]
    }

    /// Every page's list choice, in [`ViewPage`] order (what the settings file keeps).
    pub fn lists(&self) -> [bool; 4] {
        self.lists
    }

    /// Show `page` as a list (`true`) or as posters; `false` when it already is.
    pub fn set_list(&mut self, page: ViewPage, on: bool) -> bool {
        if self.lists[page as usize] == on {
            return false;
        }
        self.lists[page as usize] = on;
        self.gen = self.gen.wrapping_add(1);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_rung_out_of_range_is_the_default_and_a_set_moves_the_generation_once() {
        let mut s = SettingsStore::with(9, 2, 7);
        assert_eq!((s.get(Pref::SubtitleSize), s.get(Pref::SubtitlePosition), s.get(Pref::SkipIntros)), (1, 2, SKIP_BUTTON));
        let g = s.gen();
        assert!(s.set(Pref::SubtitleSize, 4));
        assert!(!s.set(Pref::SubtitleSize, 4), "the same rung is no change");
        assert!(!s.set(Pref::SubtitlePosition, 3), "past the ladder is refused");
        assert_eq!(s.gen(), g.wrapping_add(1));
    }

    /// Up Next appears: rung 0 the credits marker (35 s without one), then the fixed leads; the new
    /// switches default on (stremio's `bingeWatching`) and ASS keeps its styling.
    #[test]
    fn the_up_next_ladder_and_the_new_defaults() {
        assert_eq!(Pref::up_next_lead(UP_NEXT_CREDITS), (35_000, true));
        assert_eq!((1..Pref::UpNextAt.rungs()).map(|r| Pref::up_next_lead(r)).collect::<Vec<_>>(), [(15_000, false), (25_000, false), (35_000, false), (45_000, false)]);
        assert_eq!(Pref::up_next_lead(99), (35_000, true), "out of range is the default");
        let s = SettingsStore::default();
        assert_eq!([Pref::AutoPlay, Pref::AssStyling, Pref::UpNextAt, Pref::SubtitleColor].map(|p| s.get(p)), [1, 1, 0, 0]);
        assert_eq!(Pref::ALL.len(), PREFS);
        assert!(Pref::ALL.iter().enumerate().all(|(i, p)| *p as usize == i), "a pref's discriminant is its slot");
    }

    fn codes(l: &[&str]) -> Vec<String> {
        own(l)
    }

    #[test]
    fn the_stream_languages_follow_audio_then_the_app_until_given_their_own() {
        use sn_platform::i18n::{language_on_this_thread_for_test, Preference};
        let _fr = language_on_this_thread_for_test(Preference::Fr);
        let mut s = SettingsStore::default();
        assert!(s.streams_follow_audio());
        assert_eq!(s.stream_languages(), ["FR", "EN"], "no audio list: the app's language, then English");
        assert!(s.edit(LangList::Audio, LangOp::Add("ger")), "a 639-2 tag is stored as its code");
        assert_eq!(s.stream_languages(), ["DE"], "following the audio list");
        assert!(s.edit(LangList::Streams, LangOp::Add("it")), "an edit gives the order its own copy");
        assert!(!s.streams_follow_audio());
        assert_eq!(s.stream_languages(), ["DE", "IT"]);
        assert_eq!(s.languages(LangList::Audio), ["DE"], "the audio list is untouched");
        assert!(s.edit(LangList::Streams, LangOp::SameAsAudio(true)));
        assert_eq!(s.stream_languages(), ["DE"]);
        assert!(!s.edit(LangList::Streams, LangOp::Up(0)), "a no-op on a following order…");
        assert!(s.streams_follow_audio(), "…leaves it following");
    }

    /// The list editor: add at the end (once, a language the app names), remove, move up and down
    /// within the list; each change moves the generation once, a no-op not at all.
    #[test]
    fn a_language_list_adds_removes_and_reorders() {
        let mut s = SettingsStore::default();
        let g = s.gen();
        for c in ["EN", "de", "fra"] {
            assert!(s.edit(LangList::Subtitles, LangOp::Add(c)));
        }
        assert!(!s.edit(LangList::Subtitles, LangOp::Add("eng")), "listed already");
        assert!(!s.edit(LangList::Subtitles, LangOp::Add("tlh")), "not a language the app names");
        assert_eq!(s.languages(LangList::Subtitles), ["EN", "DE", "FR"]);
        assert!(s.edit(LangList::Subtitles, LangOp::Up(2)));
        assert_eq!(s.languages(LangList::Subtitles), ["EN", "FR", "DE"]);
        assert!(s.edit(LangList::Subtitles, LangOp::Down(0)));
        assert_eq!(s.languages(LangList::Subtitles), ["FR", "EN", "DE"]);
        assert!(!s.edit(LangList::Subtitles, LangOp::Up(0)) && !s.edit(LangList::Subtitles, LangOp::Down(2)), "the ends stay");
        assert!(s.edit(LangList::Subtitles, LangOp::Remove(1)));
        assert!(!s.edit(LangList::Subtitles, LangOp::Remove(5)));
        assert!(!s.edit(LangList::Audio, LangOp::SameAsAudio(false)), "only the stream order follows");
        assert_eq!(s.languages(LangList::Subtitles), ["FR", "DE"]);
        assert_eq!(s.gen(), g.wrapping_add(6));
        assert_eq!(s.track_prefs().subtitles, ["FR", "DE"]);
        let seeded = SettingsStore::default().with_languages(&codes(&["ger", "DE", "xx", "en"]), &[], Some(&[]));
        assert_eq!((seeded.languages(LangList::Audio), seeded.gen()), (&codes(&["DE", "EN"])[..], 0), "seeding normalizes, and is no change");
        assert!(!seeded.streams_follow_audio(), "an own empty order is kept");
    }

    /// An earlier build's rungs become the lists that mean the same.
    #[test]
    fn the_old_language_rungs_migrate_to_equivalent_lists() {
        use sn_platform::i18n::{language_on_this_thread_for_test, Preference};
        let _es = language_on_this_thread_for_test(Preference::Es);
        assert_eq!(migrate_rungs(0, 0, 0), (vec![], vec![], None), "every rung 0: the file's default, the app's languages");
        assert_eq!(
            migrate_rungs(2, 3, 2),
            (codes(&["FR", "EN"]), codes(&["DE", "EN"]), Some(codes(&["DE", "EN"]))),
            "an order is its languages"
        );
        assert_eq!(migrate_rungs(0, 2, 1).2, Some(codes(&["ES", "EN"])), "stream rung 0 beside an audio order: the app's languages, kept");
        assert_eq!(migrate_rungs(1, 0, 99), (vec![], vec![], Some(codes(&["EN"]))), "out of range is rung 0");
        // rung 0's meanings survive: the app's subtitle languages, the app's stream order
        let s = SettingsStore::default().with_languages(&[], &[], None);
        assert_eq!(s.track_prefs().subtitles, ["ES", "EN"]);
        assert_eq!(s.stream_languages(), ["ES", "EN"]);
    }

    fn t(language: &str, forced: bool, default: bool) -> Track<'_> {
        Track { language, forced, default }
    }

    #[test]
    fn the_tracks_follow_the_language_preferences() {
        use sn_platform::i18n::{language_on_this_thread_for_test, Preference};
        let _en = language_on_this_thread_for_test(Preference::En);
        let audio = [t("eng", false, true), t("ger", false, false)];
        let subs = [t("en", true, false), t("eng", false, false), t("de-DE", false, false), t("deu", true, false)];
        let file = SettingsStore::default();
        assert_eq!(file.get(Pref::SubtitleMode), SUBS_OFF, "subtitles default to Off");
        assert_eq!(file.pick_tracks(&audio, &subs), Picked { audio: Some(0), subtitle: None });
        let auto = file.clone().seeded(Pref::SubtitleMode, SUBS_AUTO);
        // audio follows the file (English, the app's language): Auto shows only a forced English track
        assert_eq!(auto.pick_tracks(&audio, &subs), Picked { audio: Some(0), subtitle: Some(0) });
        // German audio first, German then English subtitles
        let de = auto.with_languages(&codes(&["DE", "EN"]), &codes(&["DE", "EN"]), None);
        assert_eq!(de.pick_tracks(&audio, &subs), Picked { audio: Some(1), subtitle: Some(3) }, "Auto in the preferred audio: forced German");
        let on = de.clone().seeded(Pref::SubtitleMode, SUBS_ON);
        assert_eq!(on.pick_tracks(&audio, &subs).subtitle, Some(2), "On: the full German track, not the forced one");
        // German preferred but the file has only English audio: Auto turns German subtitles on
        let en_only = [t("eng", false, true)];
        assert_eq!(de.pick_tracks(&en_only, &subs), Picked { audio: Some(0), subtitle: Some(2) });
        assert_eq!(de.clone().seeded(Pref::SubtitleMode, SUBS_OFF).pick_tracks(&audio, &subs).subtitle, None);
        let forced = de.seeded(Pref::SubtitleMode, SUBS_FORCED);
        assert_eq!(forced.pick_tracks(&en_only, &subs).subtitle, Some(0), "Forced only: the audio's forced track");
        assert_eq!(forced.pick_tracks(&en_only, &[t("en", false, false)]).subtitle, None, "…and none without one");
        // Forced fallback: a forced track in a subtitle language when the audio's has none
        let de_forced = [t("en", false, false), t("de", true, false)];
        assert_eq!(forced.pick_tracks(&en_only, &de_forced).subtitle, None, "off by default");
        let fallback = forced.clone().seeded(Pref::ForcedFallback, 1);
        assert_eq!(fallback.pick_tracks(&en_only, &de_forced).subtitle, Some(1));
        assert_eq!(fallback.pick_tracks(&en_only, &subs).subtitle, Some(0), "the audio's own forced track still comes first");
        // untagged tracks: the file's default, else the first
        assert_eq!(file.pick_tracks(&[t("", false, false), t("", false, true)], &[]), Picked { audio: Some(1), subtitle: None });
        assert_eq!(file.pick_tracks(&[], &[]), Picked { audio: None, subtitle: None });
        assert_eq!((language_code("fre"), language_code("pt-BR"), language_code("")), ("FR".into(), "PT".into(), "".into()));
    }

    #[test]
    fn each_page_keeps_its_own_list_choice_and_a_change_moves_the_generation() {
        let mut s = SettingsStore::default().with_lists([false, true, false, false]);
        assert_eq!(s.gen(), 0, "seeding is not a change");
        assert!(s.list(ViewPage::Library) && !s.list(ViewPage::Search));
        assert!(s.set_list(ViewPage::Search, true));
        assert!(!s.set_list(ViewPage::Search, true), "the same choice is no change");
        assert_eq!((s.lists(), s.gen()), ([true, true, false, false], 1));
    }
}
