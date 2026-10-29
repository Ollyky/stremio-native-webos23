//! **The settings file** — the viewer's preferences, kept across launches: the subtitle size and
//! position, the Dolby Vision Profile 7 and the multichannel audio choices (published to the player
//! through `sn_media::prefs`), Skip intros, the audio, subtitle and stream-sorting language lists and the
//! recent searches. (The
//! search addon filter is THE addons store's, persisted beside the account's other files,
//! `account::SEARCH_FILE`; a `search_disabled` an earlier build wrote here is ignored.)
//!
//! One small JSON document, `settings.json` under the persistent state root. It is read ONCE at
//! boot, before the frame loop (the boot's synchronous loads run outside the frame scope), and
//! written whenever the settings or the search store's settings generation moves — never on the
//! frame thread: the snapshot is serialised there and the write (to a temporary file, then a
//! rename, so a crash mid-write leaves the old file) runs on the shared storage worker.
//!
//! A file that is missing, unreadable or malformed is the defaults: a preference is never worth
//! refusing to boot over. Unknown fields are ignored and missing ones default, so the format can
//! grow.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use sn_base::eventlog::log;
use sn_media::prefs::{DvProfile7, MultichannelAudio, SubtitleBackground, SubtitleColor, SubtitleOpacity, SubtitleOutline, SubtitlePosition, SubtitleSize};
use sn_screens::stores::search::SearchStore;
use sn_screens::stores::settings::{LangList, Pref, SettingsStore};
use sn_platform::i18n::Preference;
use sn_stremio::library::Sort;

/// The file's shape. Field names are a file-format contract.
#[derive(Serialize, Deserialize, Default, Debug, PartialEq)]
struct File {
    #[serde(default)]
    subtitle_size: SubtitleSize,
    #[serde(default)]
    subtitle_position: SubtitlePosition,
    #[serde(default)]
    skip_intros: SkipIntros,
    /// How a Dolby Vision Profile 7 file plays (`hdr10` / `try_dolby_vision`).
    #[serde(default)]
    dv_profile7: DvProfile7,
    /// What a >2-channel track the app decodes becomes (`auto` / `dolby_digital` / `pcm` /
    /// `stereo_dplii`).
    #[serde(default)]
    multichannel_audio: MultichannelAudio,
    /// An earlier build's language rungs (0 = the app's language / the file's default; past 0 an
    /// order), read once to migrate (`settings::migrate_rungs`) and never written again.
    #[serde(default, skip_serializing)]
    stream_languages: u8,
    #[serde(default, skip_serializing)]
    audio_language: u8,
    #[serde(default, skip_serializing)]
    subtitle_language: u8,
    /// The audio languages, first preferred first (`LangList::Audio`; codes such as `"DE"`, empty =
    /// the file's default track). Absent: a file an earlier build wrote, whose rungs migrate.
    #[serde(default)]
    audio_languages: Option<Vec<String>>,
    /// The subtitle languages (`LangList::Subtitles`; empty = the app's language, then English).
    #[serde(default)]
    subtitle_languages: Vec<String>,
    /// The stream-sorting languages (`LangList::Streams`); `null` follows the audio languages.
    #[serde(default)]
    stream_sorting: Option<Vec<String>>,
    /// `Pref::ForcedFallback` (0 off, 1 on).
    #[serde(default)]
    forced_fallback: u8,
    /// `Pref::SubtitleMode` (0 off, 1 on, 2 forced only, 3 auto); absent is the store's default.
    #[serde(default)]
    subtitle_mode: Option<u8>,
    /// `Pref::NextEpisode` (0 the same release, 1 always the stream list).
    #[serde(default)]
    next_episode: u8,
    /// The text caption's look (`Pref::SubtitleColor`, `SubtitleBackground`, `SubtitleOutline`,
    /// `SubtitleOpacity`), by the media layer's names.
    #[serde(default)]
    subtitle_color: SubtitleColor,
    #[serde(default)]
    subtitle_background: SubtitleBackground,
    #[serde(default)]
    subtitle_outline: SubtitleOutline,
    #[serde(default)]
    subtitle_opacity: SubtitleOpacity,
    /// `Pref::AssStyling`: an ASS track keeps its own styling (absent: yes).
    #[serde(default = "yes")]
    ass_styling: bool,
    /// `Pref::AutoPlay`: an episode's end plays the next one (absent: yes).
    #[serde(default = "yes")]
    auto_play: bool,
    /// `Pref::UpNextAt` (0 at the credits marker, else 15 / 25 / 35 / 45 s before the end).
    #[serde(default)]
    up_next_at: u8,
    /// Newest first.
    #[serde(default)]
    recent_searches: Vec<String>,
    /// The pages shown as a list instead of posters (`ViewPage` order: search, library, catalog, home).
    #[serde(default)]
    list_view: ListView,
    /// The Library page's order, by stremio-core's name for it (`lastwatched`, `name`, …); absent
    /// or unknown is the default.
    #[serde(default)]
    library_sort: String,
    /// The interface language by its tag (`en`, `it`, …; absent or `system`: the TV's own), read
    /// before the locale is set up at boot ([`language`]).
    #[serde(default)]
    language: String,
    /// The audio sync per stream source (a live channel's id, else the item's), in ms, oldest first.
    #[serde(default)]
    audio_offsets: Vec<(String, i64)>,
    /// `Pref::TvGuide` by region name (`de`, `uk`, …; absent, empty or unknown: off).
    #[serde(default)]
    tv_guide: String,
    /// `Pref::StreamOrder`'s rung (absent: the addon's order).
    #[serde(default)]
    stream_order: u8,
}

fn yes() -> bool {
    true
}

impl File {
    /// What an absent file means: every field at its serde default (the derived `Default` would
    /// put the switches that default on, `ass_styling` and `auto_play`, off).
    fn fresh() -> Self {
        serde_json::from_str("{}").expect("every field has a default")
    }
}

/// Which pages show their items as a list, by name.
#[derive(Serialize, Deserialize, Default, Debug, PartialEq, Clone, Copy)]
struct ListView {
    #[serde(default)]
    search: bool,
    #[serde(default)]
    library: bool,
    #[serde(default)]
    catalog: bool,
    #[serde(default)]
    home: bool,
}

/// [`Pref::SkipIntros`] in the file, by name.
#[derive(Serialize, Deserialize, Default, Debug, PartialEq, Clone, Copy)]
#[serde(rename_all = "lowercase")]
enum SkipIntros {
    Off,
    #[default]
    Button,
    Auto,
}

impl SkipIntros {
    const LADDER: [SkipIntros; 3] = [SkipIntros::Off, SkipIntros::Button, SkipIntros::Auto];
}

/// Where the settings live: the first private data directory that is writable (the same choice
/// the account cache makes), else the state root.
pub fn path() -> PathBuf {
    sn_base::paths::private_data_candidates()
        .into_iter()
        .find(|dir| crate::account::private_dir(dir))
        .unwrap_or_else(sn_base::paths::persistent_state_root)
        .join("settings.json")
}

/// The stores the file seeds.
pub struct Loaded {
    pub settings: SettingsStore,
    pub search: SearchStore,
}

/// Read `path` (boot, before the frame loop). Anything wrong with it is the defaults, logged.
pub fn load(path: &Path) -> Loaded {
    let file = match std::fs::read(path) {
        Ok(bytes) => serde_json::from_slice::<File>(&bytes).unwrap_or_else(|e| {
            log(&format!("settings: unreadable ({e}); using the defaults"));
            File::fresh()
        }),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => File::fresh(),
        Err(e) => {
            log(&format!("settings: {e}; using the defaults"));
            File::fresh()
        }
    };
    of(file)
}

fn of(file: File) -> Loaded {
    let (audio, subtitles, streams) = match file.audio_languages {
        Some(audio) => (audio, file.subtitle_languages, file.stream_sorting),
        None => sn_screens::stores::settings::migrate_rungs(file.stream_languages, file.audio_language, file.subtitle_language),
    };
    Loaded {
        settings: SettingsStore::with(
            file.subtitle_size.index(),
            file.subtitle_position.index(),
            SkipIntros::LADDER.iter().position(|s| *s == file.skip_intros).unwrap_or(0) as u8,
        )
        .with_lists([file.list_view.search, file.list_view.library, file.list_view.catalog, file.list_view.home])
        .with_library_sort(Sort::named(&file.library_sort))
        .with_dv_profile7(file.dv_profile7.index())
        .seeded(Pref::MultichannelAudio, file.multichannel_audio.index())
        .with_languages(&audio, &subtitles, streams.as_deref())
        .seeded(Pref::SubtitleMode, file.subtitle_mode.unwrap_or(Pref::SubtitleMode.default_rung()))
        .seeded(Pref::ForcedFallback, file.forced_fallback)
        .seeded(Pref::NextEpisode, file.next_episode)
        .seeded(Pref::SubtitleColor, file.subtitle_color.index())
        .seeded(Pref::SubtitleBackground, file.subtitle_background.index())
        .seeded(Pref::SubtitleOutline, file.subtitle_outline.index())
        .seeded(Pref::SubtitleOpacity, file.subtitle_opacity.index())
        .seeded(Pref::AssStyling, u8::from(file.ass_styling))
        .seeded(Pref::AutoPlay, u8::from(file.auto_play))
        .seeded(Pref::UpNextAt, file.up_next_at)
        .seeded(Pref::Language, language_rung(&file.language))
        .seeded(Pref::TvGuide, sn_screens::stores::settings::tv_guide_rung(&file.tv_guide))
        .seeded(Pref::StreamOrder, file.stream_order)
        .with_audio_offsets(file.audio_offsets),
        search: SearchStore::new(file.recent_searches),
    }
}

/// The file's bytes for the stores as they stand.
pub fn snapshot(settings: &SettingsStore, search: &SearchStore) -> Vec<u8> {
    let file = File {
        subtitle_size: SubtitleSize::from_index(settings.get(Pref::SubtitleSize)),
        subtitle_position: SubtitlePosition::from_index(settings.get(Pref::SubtitlePosition)),
        skip_intros: SkipIntros::LADDER.get(usize::from(settings.get(Pref::SkipIntros))).copied().unwrap_or_default(),
        dv_profile7: DvProfile7::from_index(settings.get(Pref::DvProfile7)),
        multichannel_audio: MultichannelAudio::from_index(settings.get(Pref::MultichannelAudio)),
        stream_languages: 0,
        audio_language: 0,
        subtitle_language: 0,
        audio_languages: Some(settings.languages(LangList::Audio).to_vec()),
        subtitle_languages: settings.languages(LangList::Subtitles).to_vec(),
        stream_sorting: (!settings.streams_follow_audio()).then(|| settings.languages(LangList::Streams).to_vec()),
        forced_fallback: settings.get(Pref::ForcedFallback),
        subtitle_mode: Some(settings.get(Pref::SubtitleMode)),
        next_episode: settings.get(Pref::NextEpisode),
        subtitle_color: SubtitleColor::from_index(settings.get(Pref::SubtitleColor)),
        subtitle_background: SubtitleBackground::from_index(settings.get(Pref::SubtitleBackground)),
        subtitle_outline: SubtitleOutline::from_index(settings.get(Pref::SubtitleOutline)),
        subtitle_opacity: SubtitleOpacity::from_index(settings.get(Pref::SubtitleOpacity)),
        ass_styling: settings.get(Pref::AssStyling) != 0,
        auto_play: settings.get(Pref::AutoPlay) != 0,
        up_next_at: settings.get(Pref::UpNextAt),
        recent_searches: search.recents().to_vec(),
        list_view: {
            let [search, library, catalog, home] = settings.lists();
            ListView { search, library, catalog, home }
        },
        library_sort: settings.library_sort().name().to_string(),
        language: language_of(settings.get(Pref::Language)).tag().to_string(),
        audio_offsets: settings.audio_offsets().to_vec(),
        tv_guide: sn_screens::stores::settings::tv_guide_name(settings.get(Pref::TvGuide)).to_string(),
        stream_order: settings.get(Pref::StreamOrder),
    };
    serde_json::to_vec_pretty(&file).unwrap_or_default()
}

fn language_rung(tag: &str) -> u8 {
    let p = Preference::from_tag(tag);
    Preference::ALL.iter().position(|x| *x == p).unwrap_or(0) as u8
}

fn language_of(rung: u8) -> Preference {
    Preference::ALL.get(usize::from(rung)).copied().unwrap_or_default()
}

/// The interface language the settings file names (boot, before the locale exists: nothing here
/// may read a localized string). Anything wrong with the file is the TV's own language.
pub fn language(path: &Path) -> Preference {
    #[derive(Deserialize)]
    struct Only {
        #[serde(default)]
        language: String,
    }
    let tag = std::fs::read(path).ok().and_then(|b| serde_json::from_slice::<Only>(&b).ok()).map(|o| o.language).unwrap_or_default();
    language_of(language_rung(&tag))
}

/// Write `bytes` to `path` on the storage worker: a temporary file beside it, then a rename.
pub fn save(path: PathBuf, bytes: Vec<u8>) {
    let _ticket = sn_base::storage_worker::submit_retained(move || {
        let result = write_atomic(&path, &bytes);
        if let Err(e) = &result {
            log(&format!("settings: not saved ({e})"));
        }
        result.is_ok()
    });
}

fn write_atomic(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, bytes)?;
    std::fs::rename(&tmp, path)
}

/// Hand the player the subtitle preferences in force (boot, and after every change).
pub fn publish(settings: &SettingsStore) {
    sn_media::prefs::set_subtitle_size(SubtitleSize::from_index(settings.get(Pref::SubtitleSize)));
    sn_media::prefs::set_subtitle_position(SubtitlePosition::from_index(settings.get(Pref::SubtitlePosition)));
    sn_media::prefs::set_dv_profile7(DvProfile7::from_index(settings.get(Pref::DvProfile7)));
    sn_media::prefs::set_multichannel_audio(MultichannelAudio::from_index(settings.get(Pref::MultichannelAudio)));
    sn_media::prefs::set_subtitle_color(SubtitleColor::from_index(settings.get(Pref::SubtitleColor)));
    sn_media::prefs::set_subtitle_background(SubtitleBackground::from_index(settings.get(Pref::SubtitleBackground)));
    sn_media::prefs::set_subtitle_outline(SubtitleOutline::from_index(settings.get(Pref::SubtitleOutline)));
    sn_media::prefs::set_subtitle_opacity(SubtitleOpacity::from_index(settings.get(Pref::SubtitleOpacity)));
    sn_media::prefs::set_ass_styling(settings.get(Pref::AssStyling) != 0);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The store's ladders ARE the media layer's: the same number of rungs and the same default,
    /// so an index the Settings page writes is the rung the player draws.
    #[test]
    fn the_settings_ladders_are_the_media_layers() {
        assert_eq!(usize::from(Pref::SubtitleSize.rungs()), SubtitleSize::LADDER.len());
        assert_eq!(usize::from(Pref::SubtitlePosition.rungs()), SubtitlePosition::LADDER.len());
        assert_eq!(Pref::SubtitleSize.default_rung(), SubtitleSize::default().index());
        assert_eq!(Pref::SubtitlePosition.default_rung(), SubtitlePosition::default().index());
        assert_eq!(Pref::SubtitleMode.default_rung(), sn_screens::stores::settings::SUBS_OFF);
        assert_eq!(usize::from(Pref::SkipIntros.rungs()), SkipIntros::LADDER.len());
        assert_eq!(SkipIntros::LADDER[usize::from(Pref::SkipIntros.default_rung())], SkipIntros::default());
        assert_eq!(usize::from(Pref::DvProfile7.rungs()), DvProfile7::LADDER.len());
        assert_eq!(Pref::DvProfile7.default_rung(), DvProfile7::default().index());
        assert_eq!(usize::from(Pref::MultichannelAudio.rungs()), MultichannelAudio::LADDER.len());
        assert_eq!(Pref::MultichannelAudio.default_rung(), MultichannelAudio::default().index());
        assert_eq!(MultichannelAudio::LADDER[usize::from(sn_screens::stores::settings::MULTI_DPLII)], MultichannelAudio::StereoDplii);
        assert_eq!(MultichannelAudio::LADDER[usize::from(sn_screens::stores::settings::MULTI_STEREO)], MultichannelAudio::Stereo);
        assert_eq!(usize::from(Pref::SubtitleColor.rungs()), SubtitleColor::LADDER.len());
        assert_eq!(usize::from(Pref::SubtitleBackground.rungs()), SubtitleBackground::LADDER.len());
        assert_eq!(usize::from(Pref::SubtitleOutline.rungs()), SubtitleOutline::LADDER.len());
        assert_eq!(usize::from(Pref::SubtitleOpacity.rungs()), SubtitleOpacity::LADDER.len());
        assert_eq!(Pref::SubtitleColor.default_rung(), SubtitleColor::default().index());
        assert_eq!(Pref::SubtitleBackground.default_rung(), SubtitleBackground::default().index());
        assert_eq!(Pref::SubtitleOutline.default_rung(), SubtitleOutline::default().index());
        assert_eq!(Pref::SubtitleOpacity.default_rung(), SubtitleOpacity::default().index());
        assert_eq!(Pref::AssStyling.default_rung(), u8::from(yes()), "ASS keeps its styling by default");
        assert_eq!(Pref::AutoPlay.default_rung(), u8::from(yes()), "auto-play is on by default");
    }

    /// The install directory is read-only to the running app on a Homebrew Channel install
    /// (`settings: not saved (Permission denied)` on the first beta), so the settings live in the
    /// private data directory the account cache already proved writable.
    #[test]
    fn the_settings_live_in_the_writable_private_data_directory() {
        let p = path();
        assert_eq!(p.file_name().unwrap(), "settings.json");
        let root = sn_base::paths::private_data_candidates().remove(0);
        assert_eq!(p.parent().unwrap(), root, "{p:?}");
    }

    #[test]
    fn the_file_round_trips_the_recents_and_the_subtitles() {
        let dir = sn_base::testscratch::process_dir("settings-file");
        let path = dir.join("settings.json");
        let mut settings = SettingsStore::default();
        settings.set(Pref::SubtitleSize, 3);
        settings.set(Pref::SubtitlePosition, 2);
        settings.set(Pref::SkipIntros, sn_screens::stores::settings::SKIP_AUTO);
        settings.set(Pref::DvProfile7, 1);
        settings.set(Pref::MultichannelAudio, sn_screens::stores::settings::MULTI_DPLII);
        use sn_screens::stores::settings::LangOp;
        for (l, c) in [(LangList::Audio, "FR"), (LangList::Audio, "EN"), (LangList::Subtitles, "DE"), (LangList::Streams, "IT")] {
            settings.edit(l, LangOp::Add(c));
        }
        settings.set(Pref::SubtitleMode, sn_screens::stores::settings::SUBS_FORCED);
        settings.set(Pref::ForcedFallback, 1);
        settings.set(Pref::NextEpisode, sn_screens::stores::settings::NEXT_CHOOSE);
        for (p, r) in [(Pref::SubtitleColor, 1), (Pref::SubtitleBackground, 3), (Pref::SubtitleOutline, 2), (Pref::SubtitleOpacity, 2), (Pref::AssStyling, 0), (Pref::AutoPlay, 0), (Pref::UpNextAt, 4)] {
            settings.set(p, r);
        }
        let mut search = SearchStore::default();
        search.remember("batman");
        search.remember("news");
        write_atomic(&path, &snapshot(&settings, &search)).unwrap();
        let back = load(&path);
        assert_eq!(back.settings.get(Pref::SubtitleSize), 3);
        assert_eq!(back.settings.get(Pref::SubtitlePosition), 2);
        assert_eq!(back.settings.get(Pref::SkipIntros), sn_screens::stores::settings::SKIP_AUTO);
        assert_eq!(back.settings.get(Pref::DvProfile7), 1, "the Profile 7 choice is kept");
        assert_eq!(back.settings.get(Pref::MultichannelAudio), sn_screens::stores::settings::MULTI_DPLII, "the multichannel choice is kept");
        assert!(String::from_utf8(snapshot(&settings, &search)).unwrap().contains(r#""multichannel_audio": "stereo_dplii""#));
        assert_eq!(back.settings.languages(LangList::Audio), ["FR", "EN"], "the audio languages are kept, in order");
        assert_eq!(back.settings.languages(LangList::Subtitles), ["DE"]);
        assert_eq!(back.settings.languages(LangList::Streams), ["FR", "EN", "IT"], "the stream order is its own");
        assert_eq!(
            [Pref::SubtitleMode, Pref::ForcedFallback].map(|p| back.settings.get(p)),
            [sn_screens::stores::settings::SUBS_FORCED, 1],
            "the subtitle mode and the forced fallback are kept"
        );
        let text = String::from_utf8(snapshot(&settings, &search)).unwrap();
        assert!(text.contains(r#""audio_languages": ["#) && !text.contains(r#""audio_language""#), "the lists are written, never the rungs: {text}");
        // a following stream order is written as following
        let mut follow = SettingsStore::default();
        follow.edit(LangList::Audio, LangOp::Add("DE"));
        write_atomic(&path, &snapshot(&follow, &search)).unwrap();
        assert!(load(&path).settings.streams_follow_audio());
        assert_eq!(back.settings.get(Pref::NextEpisode), sn_screens::stores::settings::NEXT_CHOOSE, "the next episode choice is kept");
        assert!(String::from_utf8(snapshot(&settings, &search)).unwrap().contains(r#""dv_profile7": "try_dolby_vision""#));
        assert!(String::from_utf8(snapshot(&settings, &search)).unwrap().contains(r#""skip_intros": "auto""#));
        assert_eq!(back.search.recents(), ["news", "batman"]);
        assert_eq!(
            [Pref::SubtitleColor, Pref::SubtitleBackground, Pref::SubtitleOutline, Pref::SubtitleOpacity, Pref::AssStyling, Pref::AutoPlay, Pref::UpNextAt]
                .map(|p| back.settings.get(p)),
            [1, 3, 2, 2, 0, 0, 4],
            "the caption look, the ASS switch, auto-play and the Up Next lead are kept"
        );
        assert!(String::from_utf8(snapshot(&settings, &search)).unwrap().contains(r#""subtitle_background": "black""#));
        let mut lists = SettingsStore::default();
        lists.set_list(sn_screens::stores::settings::ViewPage::Library, true);
        write_atomic(&path, &snapshot(&lists, &search)).unwrap();
        assert_eq!(load(&path).settings.lists(), [false, true, false, false], "the list choice is kept per page");
        // an earlier build's filter in this file is read past (the addons store holds it now)
        std::fs::write(&path, br#"{"search_disabled":["x"],"recent_searches":["old"]}"#).unwrap();
        assert_eq!(load(&path).search.recents(), ["old"]);
    }

    /// The multichannel choice is kept by NAME: a file the four-rung build wrote reads back as the
    /// same choice although Stereo now sits where Pro Logic II was; an unknown name is the defaults.
    #[test]
    fn the_multichannel_choice_reads_back_by_name_across_ladder_changes() {
        use sn_screens::stores::settings::{MULTI_AUTO, MULTI_DOLBY_DIGITAL, MULTI_DOLBY_DIGITAL_PLUS, MULTI_DPLII, MULTI_PCM, MULTI_STEREO};
        let dir = sn_base::testscratch::process_dir("settings-multichannel");
        let path = dir.join("settings.json");
        for (name, rung) in [("auto", MULTI_AUTO), ("dolby_digital", MULTI_DOLBY_DIGITAL), ("dolby_digital_plus", MULTI_DOLBY_DIGITAL_PLUS), ("pcm", MULTI_PCM), ("stereo_dplii", MULTI_DPLII), ("stereo", MULTI_STEREO)] {
            std::fs::write(&path, format!(r#"{{"multichannel_audio":"{name}"}}"#)).unwrap();
            assert_eq!(load(&path).settings.get(Pref::MultichannelAudio), rung, "{name}");
        }
        let mut s = SettingsStore::default();
        s.set(Pref::MultichannelAudio, MULTI_STEREO);
        assert!(String::from_utf8(snapshot(&s, &SearchStore::new(Vec::new()))).unwrap().contains(r#""multichannel_audio": "stereo""#));
    }

    /// The TV guide is kept by its region's name; absent or unknown is off.
    #[test]
    fn the_tv_guide_is_kept_by_its_region_name() {
        let dir = sn_base::testscratch::process_dir("settings-tv-guide");
        let path = dir.join("settings.json");
        let mut s = SettingsStore::default();
        assert_eq!(s.get(Pref::TvGuide), 0, "off on a fresh install");
        assert!(s.set(Pref::TvGuide, sn_screens::stores::settings::tv_guide_rung("uk")));
        write_atomic(&path, &snapshot(&s, &SearchStore::new(Vec::new()))).unwrap();
        assert!(std::fs::read_to_string(&path).unwrap().contains(r#""tv_guide": "uk""#));
        let back = load(&path).settings.get(Pref::TvGuide);
        assert_eq!(sn_screens::stores::settings::tv_guide_url(back), Some("https://epgshare01.online/epgshare01/epg_ripper_UK1.xml.gz"));
        std::fs::write(&path, br#"{"tv_guide":"xx"}"#).unwrap();
        assert_eq!(load(&path).settings.get(Pref::TvGuide), 0, "an unknown region is off");
    }

    /// The interface language is kept by its tag, and boot reads it on its own (before the locale
    /// exists); a missing or unknown one is the TV's own language.
    #[test]
    fn the_interface_language_is_kept_and_read_before_the_locale() {
        let dir = sn_base::testscratch::process_dir("settings-language");
        let path = dir.join("settings.json");
        assert_eq!(language(&path), Preference::System, "no file: the TV's language");
        let mut settings = SettingsStore::default();
        let it = Preference::ALL.iter().position(|p| *p == Preference::It).unwrap() as u8;
        assert!(settings.set(Pref::Language, it));
        write_atomic(&path, &snapshot(&settings, &SearchStore::new(Vec::new()))).unwrap();
        assert!(std::fs::read_to_string(&path).unwrap().contains(r#""language": "it""#));
        assert_eq!(language(&path), Preference::It);
        assert_eq!(load(&path).settings.get(Pref::Language), it);
        std::fs::write(&path, br#"{"language":"xx"}"#).unwrap();
        assert_eq!(language(&path), Preference::System);
    }

    /// The audio sync is kept per stream source in the settings file (the device's, never the
    /// account), and only the sources the viewer moved; the oldest go past the cap.
    #[test]
    fn the_audio_sync_is_kept_per_source() {
        use sn_screens::stores::settings::MAX_AUDIO_OFFSETS;
        let dir = sn_base::testscratch::process_dir("settings-audio-sync");
        let path = dir.join("settings.json");
        let mut s = SettingsStore::default();
        assert!(s.set_audio_offset("vavoo_SKY1", -1_500));
        assert!(s.set_audio_offset("tt0111161", 200));
        assert!(!s.set_audio_offset("", 300), "no source, nothing kept");
        write_atomic(&path, &snapshot(&s, &SearchStore::new(Vec::new()))).unwrap();
        let back = load(&path).settings;
        assert_eq!((back.audio_offset("vavoo_SKY1"), back.audio_offset("tt0111161"), back.audio_offset("other")), (-1_500, 200, 0));
        let mut s = back;
        assert!(s.set_audio_offset("vavoo_SKY1", 0));
        assert_eq!(s.audio_offsets().len(), 1, "back to 0 forgets it");
        for i in 0..MAX_AUDIO_OFFSETS + 5 {
            s.set_audio_offset(&format!("ch{i}"), 100);
        }
        assert_eq!(s.audio_offsets().len(), MAX_AUDIO_OFFSETS);
        assert_eq!(s.audio_offset("tt0111161"), 0, "the oldest went first");
    }

    /// A file an earlier build wrote keeps its meaning: its language rungs become the lists.
    #[test]
    fn an_earlier_files_language_rungs_migrate() {
        let dir = sn_base::testscratch::process_dir("settings-file-migrate");
        let path = dir.join("settings.json");
        std::fs::write(&path, br#"{"stream_languages":3,"audio_language":2,"subtitle_language":5,"subtitle_mode":1}"#).unwrap();
        let s = load(&path).settings;
        assert_eq!(s.languages(LangList::Audio), ["DE", "EN"]);
        assert_eq!(s.languages(LangList::Subtitles), ["IT", "EN"]);
        assert_eq!(s.languages(LangList::Streams), ["FR", "EN"]);
        assert!(!s.streams_follow_audio());
        assert_eq!(s.get(Pref::SubtitleMode), sn_screens::stores::settings::SUBS_ON);
        std::fs::write(&path, br#"{"subtitle_mode":3}"#).unwrap();
        assert_eq!(load(&path).settings.get(Pref::SubtitleMode), sn_screens::stores::settings::SUBS_AUTO, "an explicit saved Auto remains Auto");
        std::fs::write(&path, br#"{"stream_languages":0,"audio_language":0,"subtitle_language":0}"#).unwrap();
        let s = load(&path).settings;
        assert!(s.languages(LangList::Audio).is_empty() && s.languages(LangList::Subtitles).is_empty() && s.streams_follow_audio());
        // once written, the file holds the lists and the rungs are read past
        std::fs::write(&path, br#"{"audio_language":2,"audio_languages":["it"],"subtitle_languages":["fre"]}"#).unwrap();
        let s = load(&path).settings;
        assert_eq!((s.languages(LangList::Audio), s.languages(LangList::Subtitles)), (&["IT".to_string()][..], &["FR".to_string()][..]));
    }

    #[test]
    fn the_library_sort_is_kept_by_core_s_name() {
        let dir = sn_base::testscratch::process_dir("settings-sort");
        let path = dir.join("settings.json");
        let mut settings = SettingsStore::default();
        assert!(settings.set_library_sort(Sort::NotWatched));
        write_atomic(&path, &snapshot(&settings, &SearchStore::new(Vec::new()))).unwrap();
        assert!(std::fs::read_to_string(&path).unwrap().contains(r#""library_sort": "notwatched""#));
        assert_eq!(load(&path).settings.library_sort(), Sort::NotWatched);
        assert_eq!(Sort::named("bogus"), Sort::LastWatched, "unknown is the default");
        for s in Sort::ALL {
            assert_eq!(Sort::named(s.name()), s);
        }
    }

    #[test]
    fn a_missing_or_broken_file_is_the_defaults() {
        let dir = sn_base::testscratch::process_dir("settings-file-broken");
        let missing = load(&dir.join("absent.json"));
        assert_eq!(missing.settings, SettingsStore::default());
        let broken = dir.join("broken.json");
        std::fs::write(&broken, b"{not json").unwrap();
        assert_eq!(load(&broken).settings, SettingsStore::default());
        let partial = dir.join("partial.json");
        std::fs::write(&partial, br#"{"subtitle_size":"huge","unknown":1}"#).unwrap();
        let p = load(&partial);
        assert_eq!(p.settings.get(Pref::SubtitleSize), 4, "a known field is read, an unknown one ignored");
        assert_eq!(p.settings.get(Pref::SubtitlePosition), Pref::SubtitlePosition.default_rung());
        assert_eq!(p.settings.get(Pref::SubtitleMode), sn_screens::stores::settings::SUBS_OFF, "a missing mode uses the new install default");
        assert_eq!(p.settings.get(Pref::MultichannelAudio), sn_screens::stores::settings::MULTI_AUTO, "an earlier file is Automatic");
        assert_eq!([Pref::AssStyling, Pref::AutoPlay].map(|x| p.settings.get(x)), [1, 1], "an earlier file keeps ASS styled and auto-play on");
    }
}
