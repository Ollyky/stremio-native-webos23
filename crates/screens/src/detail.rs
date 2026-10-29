//! **Detail** — one item: its description, then either its episodes (a series, a channel) or
//! the streams every addon offers for it, one section per addon. The same screen in its second
//! mode is the **Streams** page of one episode.
//!
//! The page is a `sn_ui::table_screen::TableScreen`: the route header (crumb, title, copy) beside
//! a `TableView`. The table's sections are built by [`build`], a pure function of what the detail
//! store holds, so the per-addon states are graded by the host suite:
//!
//! - only an addon WITH streams is a section; its streams are rows — those this app can play
//!   (HTTP, with the addon's request headers) first, starting the player on OK, then those it
//!   cannot (a torrent, a link elsewhere), dimmed, whose OK raises a toast saying why
//!   ([`unplayable_reason`]);
//! - one closing section says *Still asking N addons…* while some have not answered, and holds
//!   the row *N addons without streams*: OK opens it into each addon that failed or found nothing
//!   with its reason (open at once when nothing was found anywhere);
//! - **the addon dropdown** above the list (two addons or more, `crate::dropdown`'s button saying
//!   *All addons*, the one addon asked, or "3 of 7 addons"): its popover checks each addon asked
//!   for streams, and OK switches one off for stream queries on this TV, or back on — asking it
//!   again (`AddonsStore::stream_filter`, persisted on the device, never the account). The popover
//!   stays open while OK toggles, as Search's addon filter does; an addon switched off is not listed.
//!
//! A stream row is a `sn_ui::table` CHIP row: the stream's own `name` ("[TB⚡] Comet" over
//! "1080p", TvVoo's "Proxy" / "Vavoo"; the addon's only when it sent none) in the side column, over
//! the resolution when the name does not say one, as Stremio shows it; the release's name (two lines at most; the FOCUSED row grows to its whole name
//! and every chip, wrapped, where at rest the chips that do not fit read "+N") and one line of chips under it — the
//! live mark (`behaviorHints.isLive`, or a `tv` item), the debrid cache state, HDR, codec, audio,
//! source, size, languages, seeders
//! ([`stream_chips`], from `sn_stremio::stream_facts`, the one parser of addon stream text). An
//! uncached debrid stream says "Not cached — starts a download" (it plays the service's
//! placeholder clip until the download lands). Settings > Stream order picks how each addon's
//! streams are listed ([`stream_order`]): as the addon sent them (the default), or the ones that
//! play at once first and then the Stream sorting languages (`SettingsStore::stream_languages`)
//! or the highest resolution.
//! An addon's header counts its streams; an upstream failure AIOStreams sends as a stream
//! ("[❌] Comet p2p", `stream_facts::error_note`) is a muted note after them, not counted, as is
//! its optional scrape summary ("🟢 [Comet] Scrape Summary", `stream_facts::statistic_note`, after
//! the failures, at most [`STAT_NOTE_LINES`] lines); an addon that sent errors alone found
//! nothing, and its first error is its reason on the closing row.
//!
//! Rows are focused by IDENTITY ([`RowKeys`]): a stream row's key is its addon and its position
//! in the addon's answer, so neither a slower addon landing above the focused row nor the
//! ordering moves focus.
//!
//! A series lists ONE season at a time, as Stremio's apps do: a season dropdown above the episodes
//! ([`SeasonRow`] as `crate::dropdown`'s button, "Season 1 ▾") picks it — OK opens the popover of
//! seasons (Season 1 … then Specials), a pick lists that season — and with a single season there
//! is no row. The page opens on the season of the episode to watch next, focused on it
//! (`crate::episodes::opening_episode`: the next unwatched episode after the library item's last
//! watched one, else that one), else the first numbered season ([`default_season`]). CH+/CH− are not season
//! keys: they are the global tab hotkeys (`tabs::hotkey`), taken before any page hears them. An
//! episode whose release date is still ahead reads *Upcoming*, dimmed, instead of looking playable.
//!
//! Every mode reads the item's meta (an episode's Streams page holds its show's, which the store
//! shares with a Detail page under it) and the narrative column shows what it says under the
//! title ([`info`], laid out by [`info_flow`]): a facts line (date, runtime, IMDb rating), the
//! synopsis (an episode's own overview, else the show's description; never under
//! [`SYNOPSIS_MIN_LINES`]), and the **link chips**. An item page with a `logo` draws it in the title's place ([`logo_band`]). The art is the
//! page's full-bleed BACKDROP (an episode's still, else the item's background, decoded at
//! [`BACKDROP_DECODE`]) under a page-wide veil, so the column and the list both read over it. A
//! series' episode rows wear their own still (`sn_ui::table::Row::still`, decoded at the row's
//! size and asked for only while the row is drawn) and their overview as a third line. An
//! episode's title reads "S1, E5 · Title". An episode's Streams page carries an **All episodes** button above its streams
//! (UP from the first stream, or the pointer): it pushes the show's page on that episode's season
//! (`AppArg::Overview`), so BACK returns to the episode and then to where the episode was opened.
//!
//! **The link chips** (stremio-web's `MetaLinks`, [`meta_links`], [`LinkColumn`]): a *Genres*, a
//! *Cast* and a *Director* label row, each over its chips (`widgets::BadgeStyle::Link`), wrapping,
//! at most [`LINK_LINES`] lines with a `+N` chip ending the last; focus reaching `+N` shows the group
//! whole until focus leaves it (a short column folds chip lines before the synopsis gives up its
//! minimum). The chips are the meta's `links` (`sn_stremio::links`) where the addon sent the group,
//! else the `genres`, `cast` and `director` fields. OK or a click on a genre pushes Discover on
//! that type, catalog and genre (`AppArg::DiscoverAt`); on a name, Search with the name searched
//! (`AppArg::SearchFor`); BACK returns here with focus on the same chip. UP from the library button
//! climbs the groups, DOWN comes back to it, RIGHT is the list; on a page with no library button
//! LEFT from the list reaches them.
//!
//! An item page (a movie's, a series') carries the **action row** in the narrative column, right
//! under the link chips ([`LibraryButton`], its room kept by [`info_flow`]'s reserve):
//! round icon discs (stremio-web's MetaPreview actions) that unfurl their verb while focused. The
//! library disc — a plus, *Add to library*, or a check, *In your library*, once the account holds
//! it — toggles it on OK (`LibraryCmd::Add` / `Remove`, optimistic in the library store, so it
//! flips at once; a refused write flips it back and the application toasts). Signed out it still
//! reads *Add to library* and OK opens the sign-in page, as the card menu's row says to. LEFT from
//! the list or the season dropdown reaches it, RIGHT walks the row and then goes back to the list,
//! LEFT on its first disc is BACK. An episode's Streams page has none: it is a page about one
//! video, and the library keeps the show (its All episodes page has the row). The row may unfurl
//! into the air up to the stream panel ([`stream_panel_x`]). (Trailer and Share discs are not
//! here: the app has no YouTube playback, and sharing is not a television's job.)
//!
//! **The stream panel**: the list's column stands on its own dark ground
//! ([`theme::DETAIL_STREAM_PANEL`], rounded at its top-left), and each stream row's facts are
//! icon-led lines ([`stream_chips`]: ⚡ cached, 🎞 the picture, ♫ the sound, 🛡 the source and
//! group, the size, 🌐 the languages) drawn as text rather than boxed chips. The narrative column's
//! sections are labelled in caps (SUMMARY, GENRES, CAST, DIRECTOR). An addon still being asked
//! reads its "Looking for streams…" note with an inline spinner, the words breathing
//! (`Row::busy`).
//!
//! **Watched marks** (the account's library, stremio-core's rules, `sn_stremio::library`): an
//! episode row reads *Watched* once its bit is set and the current episode's still wears its resume
//! bar (`crate::episodes::row_marked`). Signed in, a movie's page carries *Mark as Watched* /
//! *Mark as Unwatched* disc beside the library disc (core's `MarkAsWatched`; an eye, a check while watched), a
//! series' chip row a *Mark season as watched / unwatched* button after the season dropdown (core's
//! `MarkSeasonAsWatched`; unwatched once every episode of the season is), and an episode's Streams
//! page a *Mark as Watched / Unwatched* button after All episodes (core's `MarkVideoAsWatched`).
//! Each is `LibraryCmd::Mark`, optimistic; a refusal rolls back and toasts.

use std::borrow::Cow;
use std::cell::RefCell;
use std::ffi::CString;

use sn_machine::machine::{
    Canon, Cx, Effects, EntryId, FocusKey, Fx, GroupId, Handled, Leave, LogicalState, Machine, Measure, NavOp, PartId,
};
use sn_machine::present::Provenance;
use sn_platform::i18n::msg;
use sn_stremio::stream_facts::{self, Facts};
use sn_stremio::streams::{GroupState, StreamGroup};
use sn_stremio::types::{Meta, Playable};
use sn_ui::form::{RowKey, RowKeys};
use sn_ui::route_screen::RouteLayout;
use sn_ui::frame::Budget;
use sn_ui::screen::{
    composed_draw, composed_group_of, composed_groups, composed_neighbour, composed_place, composed_prepare,
    composed_reconcile, composed_seat, Activate, At, AxisMask, Composed, Dir, DrawFrame, EdgeRule, ElemKind, FocusSource,
    Focusable, GroupKind, GroupSpec, HitSource, Hover, Link, Part, Placed, RenderStrategy, Screen, ScreenEvent, Seat, Step, Stop,
};
use sn_stremio::links::{Category, Target};
use sn_ui::text_view::TextView;
use sn_ui::widgets::Button;
use sn_ui::table::{Chip as RowChip, Row, Section, TableView};
use sn_ui::table_screen::{Header, TableScreen};
use sn_ui::{theme, Rect, View};

use crate::registry::{word, AppArg, AppFx, AppMsg, Bundle, ChoiceTarget, DetailCmd, FirstSeat, GenreRef, ItemRef, LibraryCmd, MenuKind, PlayRequest, VideoRef};
use crate::stores::detail::{MetaKey, MetaState, StreamKey};
use crate::stores::library::AddState;
use crate::stores::StoreId;

const STREAM_KEY: u32 = 0x0010_0000;
const EPISODE_KEY: u32 = 0x0800_0000;
const NOTE_KEY: u32 = 0x0F00_0000;
/// The chip row's element keys (`SEASON_KEY + n`), clear of every row key: the season chips, or
/// an episode page's one All episodes button.
const SEASON_KEY: u32 = 0x0E00_0000;
/// The chip row's focus group (the table is group 0).
const SEASON_GROUP: GroupId = GroupId(1);
/// The library button's element key: in the chip band (`SEASON_KEY..NOTE_KEY`, which the first
/// seat does not count as chosen), clear of every chip.
const LIBRARY_KEY: u32 = 0x0E80_0000;
/// The movie's *Mark as Watched* button under it (the same group).
const WATCHED_KEY: u32 = LIBRARY_KEY + 1;
const LIBRARY_GROUP: GroupId = GroupId(2);
/// The addon dropdown over a stream list (in the chip band) and its group.
const ADDON_KEY: u32 = 0x0EC0_0000;
const ADDON_GROUP: GroupId = GroupId(3);
/// Streams shown per addon (an addon answering more is cut, never overflowing a key).
pub const MAX_STREAMS_PER_ADDON: usize = 0x0FFF;
/// The most lines an addon's scrape summary note reads (its name, then its facts, elided): the
/// summary is a dozen lines of status, counts and errors, and a stream list is not the place.
const STAT_NOTE_LINES: u8 = 2;
/// Addons shown on one page.
pub const MAX_STREAM_ADDONS: usize = 0xFF;

/// What a row does when it is activated.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action {
    /// Open this episode's Streams page.
    Episode(VideoRef),
    Play(PlayRequest),
    /// A stream this app cannot play: OK raises a toast saying why.
    Explain(&'static str),
    /// Open or close the closing *N addons without streams* row.
    ToggleQuiet,
}

/// The table's content and, per global row, its key and what it does.
#[derive(Default)]
pub struct Built {
    pub sections: Vec<Section>,
    pub keys: Vec<RowKey>,
    pub actions: Vec<Option<Action>>,
}

impl Built {
    fn note(&mut self, section: &mut Section, text: impl Into<String>) {
        self.note_row(section, Row::note(text));
    }

    fn note_row(&mut self, section: &mut Section, row: Row) {
        section.rows.push(row);
        self.keys.push(RowKey(NOTE_KEY + self.keys.len() as u32));
        self.actions.push(None);
    }

    fn row(&mut self, section: &mut Section, row: Row, key: u32, action: Option<Action>) {
        section.rows.push(row);
        self.keys.push(RowKey(key));
        self.actions.push(action);
    }

    pub fn index_of(&self, key: RowKey) -> Option<usize> {
        self.keys.iter().position(|k| *k == key)
    }
}

impl RowKeys for Built {
    fn key_at(&self, index: usize) -> Option<RowKey> {
        self.keys.get(index).copied()
    }
    fn index_of_key(&self, key: RowKey) -> Option<usize> {
        Built::index_of(self, key)
    }
    /// Keys are identities, so a rebuild never lands the page on a row other than the one the
    /// engine's key names: nothing to re-seat.
    fn reseat(&self) -> Option<RowKey> {
        None
    }
}

/// The page's modes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Mode {
    Item(ItemRef),
    Video(VideoRef),
    /// A Continue Watching item with no recorded video, until its meta names the first episode
    /// ([`resume_video`]); the page is then that episode's Streams page.
    Resume(ItemRef),
}

impl Mode {
    /// Does the item list episodes rather than having streams of its own? A series, an anime
    /// (the Kitsu and MAL addons' type) and a channel do; anything else (a movie, a live channel)
    /// plays its own id. One whose meta lists no video plays its own id too.
    fn lists_episodes(kind: &str) -> bool {
        matches!(kind, "series" | "anime" | "channel")
    }

    /// The item's Stremio type ("movie", "series", "tv", …).
    fn kind(&self) -> &str {
        match self {
            Mode::Item(i) | Mode::Resume(i) => &i.kind,
            Mode::Video(v) => &v.kind,
        }
    }

    fn meta_key(&self) -> Option<MetaKey> {
        match self {
            Mode::Item(i) | Mode::Resume(i) => Some(MetaKey { kind: i.kind.clone(), id: i.id.clone() }),
            Mode::Video(v) if v.item_id.is_empty() => None,
            Mode::Video(v) => Some(MetaKey { kind: v.kind.clone(), id: v.item_id.clone() }),
        }
    }

    /// The episode whose show an All episodes button opens; `None` for a movie (its video is the
    /// item itself) and any other page.
    fn show(&self) -> Option<&VideoRef> {
        match self {
            Mode::Video(v) if Self::lists_episodes(&v.kind) && !v.item_id.is_empty() && v.video_id != v.item_id => Some(v),
            _ => None,
        }
    }

    /// The streams the page opens at once (without waiting for the meta).
    fn stream_key(&self) -> Option<StreamKey> {
        match self {
            Mode::Item(i) if Self::lists_episodes(&i.kind) => None,
            Mode::Item(i) => Some(StreamKey { kind: i.kind.clone(), video_id: i.id.clone() }),
            Mode::Video(v) => Some(StreamKey { kind: v.kind.clone(), video_id: v.video_id.clone() }),
            Mode::Resume(_) => None,
        }
    }
}

/// The Detail page's two columns: a narrower narrative measure than the routes' default (660)
/// gives the stream and episode list room for the stream's own name in its side column
/// ("[TB⚡] Comet", "MediaFusion | ⚡ TB 4K") and longer release names.
pub(crate) const DETAIL_COPY_W: f32 = 580.0;

pub(crate) fn detail_layout() -> RouteLayout {
    RouteLayout::screen_with_copy_w(DETAIL_COPY_W)
}

/// **The video id streams should be asked for**, when the meta says it is not `asked`: `None` keeps
/// it. A TMDB-keyed item (AIOStreams' search, and the library items saved from it) carries
/// `tmdb:157336` or `tmdb:1396:1:1`, and AIOStreams' scrapers answer nothing for those.
/// - a movie: the meta's `behaviorHints.defaultVideoId` (Stremio's own rule);
/// - an episode: the meta's video of the same season and episode when `asked` is not one of the
///   meta's own video ids (the meta lists them under the IMDb id).
fn stream_video_for(kind: &str, asked: &str, meta: &Meta) -> Option<String> {
    if !Mode::lists_episodes(kind) {
        return meta.default_video_id().filter(|d| *d != asked).map(str::to_string);
    }
    if meta.videos.is_empty() || meta.videos.iter().any(|v| v.id == asked) {
        return None;
    }
    let mut tail = asked.rsplitn(3, ':');
    let (episode, season) = (tail.next()?.parse::<u32>().ok()?, tail.next()?.parse::<u32>().ok()?);
    meta.videos.iter().find(|v| v.season == Some(season) && v.episode == Some(episode)).map(|v| v.id.clone())
}

/// What a resume with no recorded video opens once `meta` is in: the first episode
/// (`sn_stremio::library::first_episode`), or the item's own id when the meta lists no video.
/// Never the bare series id while the meta has episodes.
pub fn resume_video(item: &ItemRef, meta: &Meta) -> VideoRef {
    let show = if meta.preview.name.is_empty() { item.name.clone() } else { meta.preview.name.clone() };
    match sn_stremio::library::first_episode(&meta.videos) {
        Some(v) => {
            let address = sn_ui::fmt::episode_ordinal(i64::from(v.season.unwrap_or(0)), i64::from(v.episode.unwrap_or(0)));
            let title = if v.title.trim().is_empty() { address } else { format!("{address} {}", v.title) };
            VideoRef { kind: item.kind.clone(), video_id: v.id.clone(), show, title, resume_ms: 0, item_id: item.id.clone(), from: item.from.clone() }
        }
        None => VideoRef { kind: item.kind.clone(), video_id: item.id.clone(), show, title: String::new(), resume_ms: 0, item_id: item.id.clone(), from: item.from.clone() },
    }
}

/// The table for `mode`, from what the store holds. A series lists only `season` (its first
/// numbered season when `None` or not in the meta); an episode released after `today`
/// (`YYYY-MM-DD`; empty = unknown, nothing is upcoming) reads as upcoming.
pub fn build(mode: &Mode, meta: Option<&MetaState>, streams: Option<&[StreamGroup]>, season: Option<u32>, today: &str) -> Built {
    build_preferring(mode, meta, streams, season, today, &[])
}

/// [`build`], with each addon's streams ordered for the `prefer`red audio languages
/// ([`stream_order`] with `ORDER_LANGUAGE`; `Facts::languages` codes, first preferred first).
pub fn build_preferring(
    mode: &Mode,
    meta: Option<&MetaState>,
    streams: Option<&[StreamGroup]>,
    season: Option<u32>,
    today: &str,
    prefer: &[&str],
) -> Built {
    let view = StreamView { order: crate::stores::settings::ORDER_LANGUAGE, ..StreamView::default() };
    build_marked(mode, meta, streams, season, today, prefer, None, &view)
}

/// [`build_preferring`], the episode rows wearing the library's watched marks (`watched`).
pub fn build_marked(
    mode: &Mode,
    meta: Option<&MetaState>,
    streams: Option<&[StreamGroup]>,
    season: Option<u32>,
    today: &str,
    prefer: &[&str],
    watched: Option<&crate::episodes::Watched<'_>>,
    view: &StreamView,
) -> Built {
    let mut out = Built::default();
    if let Mode::Resume(_) = mode {
        // until the meta names the episode: its state, as the episodes section reads it
        let note = match meta {
            Some(MetaState::Failed(e)) => msg::app_detail_episodes_failed(&e.to_string()),
            Some(MetaState::NoSource) => msg::app_detail_no_meta().to_string(),
            _ => msg::app_detail_episodes_loading().to_string(),
        };
        let mut s = Section::new(msg::app_detail_episodes());
        out.note(&mut s, note);
        out.sections.push(s);
        return out;
    }
    if let Mode::Item(item) = mode {
        if Mode::lists_episodes(&item.kind) {
            match meta {
                Some(MetaState::Ready(m)) if !m.videos.is_empty() => episodes(&mut out, item, m, season, today, watched),
                // a series whose meta lists no video plays its own id: its streams follow
                Some(MetaState::Ready(_)) if streams.is_some() => {}
                Some(MetaState::Ready(_)) => {
                    let mut s = Section::new(msg::app_detail_episodes());
                    out.note(&mut s, msg::app_detail_no_episodes());
                    out.sections.push(s);
                }
                Some(MetaState::Failed(e)) => {
                    let mut s = Section::new(msg::app_detail_episodes());
                    out.note(&mut s, msg::app_detail_episodes_failed(&crate::stores::fetch_reason(e)));
                    out.sections.push(s);
                }
                Some(MetaState::NoSource) => {
                    let mut s = Section::new(msg::app_detail_episodes());
                    out.note(&mut s, msg::app_detail_no_meta());
                    out.sections.push(s);
                }
                Some(MetaState::Loading) | None => {
                    let mut s = Section::new(msg::app_detail_episodes());
                    out.note_row(&mut s, Row::note(msg::app_detail_episodes_loading()).busy());
                    out.sections.push(s);
                }
            }
        }
    }
    if let Some(groups) = streams {
        stream_sections(&mut out, groups, mode, prefer, view);
    }
    out
}

/// The play title for `mode`: what the player page shows.
fn play_title(mode: &Mode) -> String {
    match mode {
        Mode::Item(i) | Mode::Resume(i) => i.name.clone(),
        Mode::Video(v) if v.title.is_empty() => v.show.clone(),
        Mode::Video(v) => format!("{} \u{2014} {}", v.show, v.title),
    }
}

/// What a play from `mode` is of: the player's title lines and next episode, the library item
/// its progress is recorded on.
fn play_item(mode: &Mode) -> Option<crate::registry::PlayItem> {
    match mode {
        Mode::Item(i) => Some(crate::registry::PlayItem {
            kind: i.kind.clone(),
            id: i.id.clone(),
            name: i.name.clone(),
            video_id: i.id.clone(),
            episode: String::new(),
            release: None,
        }),
        Mode::Video(v) if !v.item_id.is_empty() => Some(crate::registry::PlayItem {
            kind: v.kind.clone(),
            id: v.item_id.clone(),
            name: v.show.clone(),
            video_id: v.video_id.clone(),
            episode: v.title.clone(),
            release: None,
        }),
        Mode::Video(_) | Mode::Resume(_) => None,
    }
}

/// What the viewer chose to see of a stream list (the page's own state, not the store's): whether
/// the closing *N addons without streams* row is open.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StreamView {
    /// The closing row shows each addon that found nothing, and why.
    pub quiet_open: bool,
    /// `Pref::StreamOrder`'s rung (`settings::ORDER_*`).
    pub order: u8,
}

/// The closing row's key (*N addons without streams*), clear of every stream key.
pub const QUIET_KEY: u32 = 0x0F80_0000;

/// What an addon's group amounts to on the list.
enum GroupShow {
    /// Streams to list.
    Streams,
    /// Still being asked.
    Asking,
    /// Settled with nothing to play: why.
    Quiet(String),
    /// Switched off for stream queries (unchecked in the addon dropdown): not listed.
    Off,
}

/// How many of an addon's streams are real (an upstream's error or scrape summary is a note).
fn real_streams(streams: &[sn_stremio::types::Stream]) -> usize {
    streams.iter().filter(|s| !stream_facts::is_note(s)).count()
}

fn group_show(g: &StreamGroup) -> GroupShow {
    match &g.state {
        GroupState::Off => GroupShow::Off,
        GroupState::Loading => GroupShow::Asking,
        GroupState::Failed(e) => GroupShow::Quiet(crate::stores::fetch_reason(e)),
        GroupState::Ready(s) if real_streams(s) == 0 => {
            // an addon that sent only its upstreams' failures says the first one
            let why = s.iter().find_map(stream_facts::error_note).unwrap_or_else(|| msg::app_detail_no_streams().to_string());
            GroupShow::Quiet(why)
        }
        GroupState::Ready(_) => GroupShow::Streams,
    }
}

/// Why a stream this app cannot play does not play: the toast OK on its row raises.
pub fn unplayable_reason(p: Option<&Playable>) -> &'static str {
    match p {
        Some(Playable::Torrent { .. }) => msg::app_detail_why_torrent(),
        Some(Playable::YouTube(_)) => msg::app_detail_why_youtube(),
        _ => msg::app_detail_why_external(),
    }
}

/// The stream list: one section per addon WITH streams, headed by its name and how many it offers,
/// in addon order; then ONE closing section: *Still asking N addons…* while some have not
/// answered, and *N addons without streams* — a row that, opened ([`StreamView::quiet_open`]),
/// lists each addon that failed or found nothing with its reason. An addon switched off is not
/// listed. A row's key is its addon's and its own position in what the addon
/// sent, so neither a landing nor the ordering moves a focused row.
fn stream_sections(out: &mut Built, groups: &[StreamGroup], mode: &Mode, prefer: &[&str], view: &StreamView) {
    if groups.is_empty() {
        let mut s = Section::new(msg::app_detail_streams());
        out.note(&mut s, msg::app_detail_no_stream_addons());
        out.sections.push(s);
        return;
    }
    let mut asking = 0;
    let mut quiet: Vec<(String, String)> = Vec::new();
    let mut listed = 0;
    for (gi, group) in groups.iter().enumerate().take(MAX_STREAM_ADDONS) {
        let streams = match (group_show(group), &group.state) {
            (GroupShow::Streams, GroupState::Ready(streams)) => streams,
            (GroupShow::Asking, _) => {
                asking += 1;
                continue;
            }
            (GroupShow::Quiet(why), _) => {
                quiet.push((group.addon_name.clone(), why));
                continue;
            }
            _ => continue,
        };
        listed += 1;
        let mut s = Section::new(group.addon_name.clone()).server_header();
        let streams = &streams[..streams.len().min(MAX_STREAMS_PER_ADDON)];
        // an upstream's failure or scrape summary sent as a stream (AIOStreams' "[❌] Comet
        // p2p", "🟢 [Comet] Scrape Summary") is a note after the streams — failures first,
        // then summaries, where AIOStreams' own order sends them (and a summary told "top"
        // still reads after the streams it sums up) — and no stream the header counts
        let errors: Vec<Option<String>> = streams.iter().map(stream_facts::error_note).collect();
        let stats: Vec<Option<String>> = streams.iter().map(stream_facts::statistic_note).collect();
        let is_note = |si: usize| errors[si].is_some() || stats[si].is_some();
        s = s.accessory(real_streams(streams).to_string());
        let live_item = mode.kind() == "tv";
        let facts: Vec<Facts> = streams.iter().map(stream_facts::facts).map(|f| Facts { live: f.live || live_item, ..f }).collect();
        let playable: Vec<Option<Playable>> = streams.iter().map(sn_stremio::types::Stream::playable).collect();
        let mut order: Vec<usize> = stream_order(&facts, prefer, view.order).into_iter().filter(|si| !is_note(*si)).collect();
        // what plays here before what cannot (a torrent with no debrid, a link elsewhere)
        order.sort_by_key(|si| !matches!(playable[*si], Some(Playable::Http { .. })));
        for si in order {
            let f = &facts[si];
            let label = if f.title.is_empty() { group.addon_name.clone() } else { f.title.clone() };
            // the stream's own name, as Stremio shows it; the addon's only when it sent none
            let name = if f.name.is_empty() { group.addon_name.clone() } else { f.name.clone() };
            let mut row = Row::new(label).server_label().side(name, f.side_tag());
            for c in stream_chips(f) {
                row = row.chip(c);
            }
            let action = match playable[si].clone() {
                Some(Playable::Http { url, headers }) => {
                    let start_ms = if let Mode::Video(v) = mode { v.resume_ms } else { 0 };
                    Action::Play(PlayRequest {
                        url,
                        headers,
                        title: play_title(mode),
                        start_ms,
                        addon: group.addon_name.clone(),
                        item: play_item(mode).map(|it| crate::registry::PlayItem {
                            release: Some(stream_facts::Release {
                                addon_id: group.addon_id.clone(),
                                binge_group: streams[si].behavior_hints.binge_group.clone(),
                                facts: f.clone(),
                            }),
                            ..it
                        }),
                        hints: sn_stremio::subtitles::FileHints::of(&streams[si]),
                    })
                }
                other => {
                    // still a stop: OK says why it does not play here
                    row = row.value(msg::app_detail_not_playable()).dim(true);
                    Action::Explain(unplayable_reason(other.as_ref()))
                }
            };
            out.row(&mut s, row, STREAM_KEY + ((gi as u32) << 12) + si as u32, Some(action));
        }
        for e in errors.iter().zip(&stats).filter_map(|(e, st)| e.clone().filter(|_| st.is_none())) {
            out.note_row(&mut s, Row::note(e).server_label());
        }
        for st in stats.into_iter().flatten() {
            out.note_row(&mut s, Row::note(st).server_label().note_max_lines(STAT_NOTE_LINES));
        }
        out.sections.push(s);
    }
    if asking == 0 && quiet.is_empty() {
        return;
    }
    // with nothing listed the reasons are all there is to read: open
    let open = view.quiet_open || (listed == 0 && asking == 0);
    let mut s = Section::new(if listed == 0 { msg::app_detail_streams() } else { "" });
    if asking > 0 {
        out.note_row(&mut s, Row::note(msg::app_detail_still_asking(asking)).busy());
    }
    if !quiet.is_empty() {
        let row = Row::new(msg::app_detail_quiet_addons(quiet.len() as i64)).chevron(!open).dim(true);
        out.row(&mut s, row, QUIET_KEY, Some(Action::ToggleQuiet));
        if open {
            for (name, why) in quiet {
                out.note_row(&mut s, Row::note(format!("{name} \u{b7} {why}")).server_label());
            }
        }
    }
    out.sections.push(s);
}

/// The order one addon's streams are listed in, for `Pref::StreamOrder`'s rung `by`:
/// - `ORDER_ADDON`: as the addon sent them.
/// - `ORDER_LANGUAGE`: those that play at once (cached, or with no cache to speak of) before those
///   the debrid service must first download; within each, the streams in the first `prefer`red
///   language, then the second, …, then a multi-audio release, then the rest.
/// - `ORDER_QUALITY`: the same split, then the highest resolution first (none named: last).
///
/// The addon's own order breaks every tie.
pub fn stream_order(facts: &[Facts], prefer: &[&str], by: u8) -> Vec<usize> {
    use crate::stores::settings::{ORDER_ADDON, ORDER_LANGUAGE};
    const RESOLUTIONS: [&str; 6] = ["4K", "1440p", "1080p", "720p", "576p", "480p"];
    let rank = |f: &Facts| {
        let download = f.cached == Some(false);
        let then = if by == ORDER_LANGUAGE {
            prefer
                .iter()
                .position(|p| f.languages.contains(p))
                .unwrap_or(if f.languages.contains(&"MULTI") { prefer.len() } else { prefer.len() + 1 })
        } else {
            f.resolution.and_then(|r| RESOLUTIONS.iter().position(|x| *x == r)).unwrap_or(RESOLUTIONS.len())
        };
        (download, then)
    };
    let mut order: Vec<usize> = (0..facts.len()).collect();
    if by != ORDER_ADDON {
        order.sort_by_key(|i| rank(&facts[*i])); // stable: the addon's order breaks ties
    }
    order
}

/// The languages a chip names at most; more read "+N".
const CHIP_LANGUAGES: usize = 3;

/// A stream row's facts, most telling first (the line drops them from the end), each an
/// ICON-LED line as stremio-web's stream rows read (`Chip::icon`, drawn without a ground): ⚡ the
/// cache state, 🎞 the picture (codec · range), ♫ the sound, 🛡 the source (and the release
/// group), the size, 🌐 the languages, the seeders. An uncached stream's caveat stays an outlined
/// quiet chip (a warning, not a fact). Only the cache and live chips are app text that must show;
/// the rest are facts read from the addon's text and may give way.
pub fn stream_chips(f: &Facts) -> Vec<RowChip> {
    use sn_ui::icons::Icon;
    let dot = |parts: &[Option<&str>]| -> Option<String> {
        let v: Vec<&str> = parts.iter().flatten().copied().filter(|p| !p.is_empty()).collect();
        (!v.is_empty()).then(|| v.join(" \u{b7} "))
    };
    let mut out = Vec::new();
    let mark = |mut chip: RowChip, origin: Option<stream_facts::FactOrigin>| {
        if let Some(origin) = origin {
            let evidence = match origin {
                stream_facts::FactOrigin::AddonText => sn_ui::table::ChipEvidence::AddonText,
                stream_facts::FactOrigin::StructuredHint => sn_ui::table::ChipEvidence::AddonHint,
            };
            chip = chip.evidence(evidence);
        }
        chip
    };
    if f.live {
        out.push(mark(RowChip::new(msg::app_detail_stream_live()), f.origins.live));
    }
    match f.cached {
        Some(true) => out.push(mark(RowChip::new(msg::app_detail_stream_cached()).icon(Icon::Bolt), f.origins.cached)),
        Some(false) => out.push(mark(RowChip::new(msg::app_detail_stream_uncached()).quiet(), f.origins.cached)),
        None => {}
    }
    // the picture: the range leads (it is what a viewer chooses by), the codec follows it on the
    // same line as plain text, so the most telling word keeps its room when the line is short
    if let Some(t) = f.range.or(f.codec) {
        out.push(mark(RowChip::new(t).server().icon(Icon::Film), if f.range.is_some() { f.origins.range } else { f.origins.codec }));
    }
    if let (Some(_), Some(c)) = (f.range, f.codec) {
        out.push(mark(RowChip::new(c).server(), f.origins.codec));
    }
    if let Some(a) = &f.audio {
        out.push(mark(RowChip::new(a.clone()).server().icon(Icon::Audio), f.origins.audio));
    }
    if let Some(t) = dot(&[f.source, f.group.as_deref()]) {
        out.push(mark(RowChip::new(t).server().icon(Icon::Shield), f.origins.source.or(f.origins.group)));
    }
    if let Some(b) = f.size {
        out.push(mark(RowChip::new(sn_ui::fmt::bytes(i64::try_from(b).unwrap_or(i64::MAX), (1, 0, 0))).server().icon(Icon::Drive), f.origins.size));
    }
    if !f.languages.is_empty() {
        let mut langs = f.languages.iter().take(CHIP_LANGUAGES).copied().collect::<Vec<_>>().join(" \u{b7} ");
        if f.languages.len() > CHIP_LANGUAGES {
            langs.push_str(&format!(" +{}", f.languages.len() - CHIP_LANGUAGES));
        }
        out.push(mark(RowChip::new(langs).server().icon(Icon::Globe), f.origins.languages));
    }
    if let Some(n) = f.seeders {
        out.push(mark(RowChip::new(msg::app_detail_stream_seeders(i64::from(n))).server().icon(Icon::User), f.origins.seeders));
    }
    out
}

/// What the narrative column says under the title once the meta is in.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Info {
    /// The page's title: an episode's "S1, E5 · Title", else the item's name. `None` keeps the
    /// title the page opened with.
    pub title: Option<String>,
    /// Date (an episode's release day, else the item's release info), runtime, IMDb rating.
    pub facts: String,
    /// An episode's own overview, else the item's description.
    pub synopsis: String,
    /// The genre, cast and director chips ([`meta_links`]), each group when the meta names any.
    pub links: Vec<LinkGroup>,
    /// The full-bleed picture behind the page: an episode page's own still, else the item's
    /// background.
    pub backdrop: Option<String>,
    /// The item's logo (an item page only): drawn in place of the title text.
    pub logo: Option<String>,
    /// The synopsis' section label, in caps ("SUMMARY", stremio-web's), as the column draws it.
    pub summary_caps: String,
}

/// One group of the narrative column's link chips: its label row and its chips.
#[derive(Clone, Debug, PartialEq)]
pub struct LinkGroup {
    pub category: Category,
    pub chips: Vec<LinkChip>,
    /// [`LinkGroup::label`] in caps ("GENRES", "CAST", "DIRECTOR"): the section label the
    /// narrative column draws over the chips, as stremio-web sets its MetaPreview sections.
    pub caps: String,
}

/// One link chip: the name it reads (server text) and the page OK on it pushes.
#[derive(Clone, Debug, PartialEq)]
pub struct LinkChip {
    pub name: String,
    pub to: AppArg,
}

impl LinkGroup {
    /// The label row over the chips (app text).
    pub fn label(&self) -> &'static str {
        match self.category {
            Category::Genres => msg::app_detail_genres_label(),
            Category::Cast => msg::app_detail_cast_label(),
            Category::Director => msg::app_detail_director_label(),
        }
    }
}

/// The chips one group holds at most (its element keys are `LINK_KEY + (group << 8) + n`, the
/// last slot being the `+N` chip's).
pub const LINK_SLOTS: usize = 0xFE;

/// **The genre, cast and director chips** of an item of type `kind`, in that order: the meta's own
/// `links` when the addon sent any of the group's category (`sn_stremio::links`: a discover link
/// opens Discover on its catalog and genre, a search link a Search for its text), else the plain
/// `genres` (Discover on that genre, the first catalog of `kind` offering it), `cast` and
/// `director` fields (a Search for the name). Blank and repeated names are dropped.
pub fn meta_links(kind: &str, meta: &Meta) -> Vec<LinkGroup> {
    let routed = sn_stremio::links::routed(&meta.links);
    let search = |n: &str| AppArg::SearchFor(n.to_string());
    let field = |category: Category| -> Vec<LinkChip> {
        let names: &[String] = match category {
            Category::Genres => &meta.preview.genres,
            Category::Cast => &meta.cast,
            Category::Director => &meta.director,
        };
        names
            .iter()
            .map(|n| n.trim())
            .filter(|n| !n.is_empty())
            .map(|n| LinkChip {
                name: n.to_string(),
                to: match category {
                    Category::Genres => AppArg::DiscoverAt(GenreRef { kind: kind.to_string(), genre: Some(n.to_string()), addon_base: None, catalog: None }),
                    Category::Cast | Category::Director => search(n),
                },
            })
            .collect()
    };
    [Category::Genres, Category::Cast, Category::Director]
        .into_iter()
        .filter_map(|category| {
            let linked: Vec<LinkChip> = routed
                .iter()
                .filter(|(c, _, _)| *c == category)
                .map(|(_, name, target)| LinkChip {
                    name: name.clone(),
                    to: match target {
                        Target::Search(q) => search(q),
                        Target::Discover { transport_url, kind, catalog, genre } => AppArg::DiscoverAt(GenreRef {
                            kind: kind.clone(),
                            genre: genre.clone(),
                            addon_base: Some(sn_stremio::manifest::base_of(transport_url)),
                            catalog: Some(catalog.clone()),
                        }),
                    },
                })
                .collect();
            let mut chips = if linked.is_empty() { field(category) } else { linked };
            let mut seen = std::collections::HashSet::new();
            chips.retain(|c| seen.insert(c.name.clone()));
            chips.truncate(LINK_SLOTS);
            (!chips.is_empty()).then(|| {
                let mut g = LinkGroup { category, chips, caps: String::new() };
                g.caps = g.label().to_uppercase();
                g
            })
        })
        .collect()
}

/// What `meta` says about `mode`'s page ([`Info`]).
pub fn info(mode: &Mode, meta: &Meta) -> Info {
    let p = &meta.preview;
    let text = |s: Option<&str>| s.map(str::trim).filter(|s| !s.is_empty()).map(str::to_string);
    let video = match mode {
        Mode::Video(v) => meta.videos.iter().find(|x| x.id == v.video_id),
        Mode::Item(_) | Mode::Resume(_) => None,
    };
    let title = match video {
        Some(v) => {
            let address = sn_ui::fmt::episode_ordinal(i64::from(v.season.unwrap_or(0)), i64::from(v.episode.unwrap_or(0)));
            Some(match text(Some(&v.title)) {
                Some(t) => format!("{address} \u{b7} {t}"),
                None => address,
            })
        }
        None => text(Some(&p.name)),
    };
    // an episode's air day in the UI language's date; a work's release info as the addon wrote it
    let date = match video {
        Some(v) => text(v.released.as_deref().map(|d| d.get(..10).unwrap_or(d))).map(|d| sn_ui::fmt::pretty_date(&d, 0)),
        None => text(p.release_info.as_deref()),
    };
    // the rating with the UI language's decimal mark (`7,5` in French); text it cannot read as a
    // number stays as the addon sent it
    let rating = text(p.imdb_rating.as_deref()).map(|r| match r.parse::<f64>() {
        Ok(v) => format!("IMDb {}", sn_ui::fmt::decimal(v, 1)),
        Err(_) => format!("IMDb {r}"),
    });
    // the genres are chips of their own ([`meta_links`]), not words on this line
    let facts = [date, text(meta.runtime.as_deref()), rating]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join(" \u{b7} ");
    let synopsis = video.and_then(|v| text(v.overview.as_deref())).or_else(|| text(p.description.as_deref())).unwrap_or_default();
    let backdrop = video.and_then(|v| text(v.thumbnail.as_deref())).or_else(|| text(p.background.as_deref()));
    let logo = match mode {
        Mode::Item(_) => text(p.logo.as_deref()),
        Mode::Video(_) | Mode::Resume(_) => None,
    };
    let summary_caps = msg::app_detail_summary_label().to_uppercase();
    Info { title, facts, synopsis, links: meta_links(mode.kind(), meta), backdrop, logo, summary_caps }
}

/// The size the full-bleed backdrop is decoded to cover: half the panel in each axis (~2 MB of
/// texture, not the ~8 MB a 1920x1080 decode costs). It sits under [`theme::BACKDROP_VEIL_A`],
/// where the upscale does not read; this is the knob if a capture says it does.
pub const BACKDROP_DECODE: (std::os::raw::c_int, std::os::raw::c_int) = (960, 540);

/// The logo's band at the head of the narrative column (an item page with a `logo`), where the
/// title would start: the hero rung's constant layout height, the logo itself bottom-anchored in
/// it and spilling upward as paint (`sn_ui::hero_logo`).
pub fn logo_band() -> Rect {
    let l = detail_layout();
    let h = sn_ui::hero_logo::band_h(sn_ui::hero_logo::LogoRung::Hero);
    // the band's top is where the title's first line would start; a taller logo paints above it
    Rect::new(l.narrative.x, l.narrative_top(true), l.narrative.w, h)
}

/// The full-bleed backdrop under everything: the picture (decoded at [`BACKDROP_DECODE`] and
/// cover-cropped to the panel), the page-wide veil the list and the column both read over, and
/// the hero wedge under the narrative column. A page with no picture draws none of it.
fn draw_backdrop(p: sn_ui::Painter, url: &str) {
    let (tex, tw, th) = sn_ui::widgets::resolve_tex_wh_on(0, url, BACKDROP_DECODE.0, BACKDROP_DECODE.1, 0);
    if tex == 0 {
        sn_ui::placeholder::note(p, sn_ui::placeholder::Reason::DetailBackdrop, url);
    }
    sn_ui::widgets::veiled_backdrop(p, tex, tw, th, theme::BACKDROP_VEIL_A);
}
/// **The stream panel** (stremio-web's MetaDetails streams column): the list's column stands on its
/// own dark ground from near the top of the screen to its foot and past its right edge, so only its
/// top-left corner is rounded. One quad.
fn draw_stream_panel(p: sn_ui::Painter) {
    let (x, y) = (stream_panel_x(), theme::space::LG);
    let rad = theme::DETAIL_STREAM_PANEL_RAD;
    let r = Rect::new(x, y, sn_ui::consts::SCR_W - x + rad, sn_ui::consts::SCR_H - y + rad);
    p.rect(r, rad, theme::DETAIL_STREAM_PANEL, theme::DETAIL_STREAM_PANEL, 0.0);
}

/// The synopsis wraps to at most this many lines (fewer when the column is short), then elides.
pub const SYNOPSIS_MAX_LINES: usize = 6;
/// The synopsis keeps at least this many lines while a link group is expanded: an expanded group
/// gives up lines before the synopsis does.
pub const SYNOPSIS_MIN_LINES: usize = 3;
/// A link group shows at most this many chip lines at rest; the rest give way to a `+N` chip.
pub const LINK_LINES: usize = 2;
const INFO_SIZE: std::os::raw::c_int = theme::size::LABEL;
/// A link group's label row: a one-line de-emphasized label over its chips (the `MICRO` rung's
/// role), set solid (its line is its size) so three groups and the library buttons leave the
/// synopsis its lines under a two-line title.
const LINK_LABEL_SIZE: std::os::raw::c_int = theme::size::MICRO;
/// Between two chips of a line, and between two chip lines.
const LINK_GAP: f32 = theme::space::XS;

/// One text block of the narrative column, where [`info_flow`] put it.
pub struct Block<'a> {
    pub text: &'a str,
    pub ink: [f32; 4],
    pub rect: Rect,
    pub lines: usize,
    /// Its type size: [`INFO_SIZE`], or a link group label's [`LINK_LABEL_SIZE`].
    pub size: std::os::raw::c_int,
}

impl Block<'_> {
    fn view<'m>(&self, measure: &'m dyn Measure) -> TextView<'m>
    where
        Self: 'm,
    {
        match self.size {
            LINK_LABEL_SIZE => label_view(self.text, measure),
            _ => info_view(self.text, self.ink, self.lines, measure),
        }
    }
}

/// A link group's label row ([`LINK_LABEL_SIZE`], one line).
pub fn label_view<'a>(text: &'a str, measure: &'a dyn Measure) -> TextView<'a> {
    TextView::new(text, LINK_LABEL_SIZE, theme::TEXT_SECONDARY).with_measure(measure).leading(LINK_LABEL_SIZE as f32).max_lines(1)
}

/// Where one link group's chips stand ([`flow`]): `group` indexes [`Info::links`]; `chips` are the
/// shown chips' rects, in order; `more` the `+N` chip's rect and how many it stands for.
#[derive(Clone, Debug, PartialEq)]
pub struct GroupPlace {
    pub group: usize,
    pub chips: Vec<Rect>,
    pub more: Option<(Rect, usize)>,
}

fn info_view<'a>(text: &'a str, ink: [f32; 4], lines: usize, measure: &'a dyn Measure) -> TextView<'a> {
    TextView::new(text, INFO_SIZE, ink).with_measure(measure).leading(INFO_SIZE as f32 + theme::space::XS).max_lines(lines)
}

/// Where [`Info`] goes under a page titled `title` (or under its logo's band, [`logo_band`]): the
/// text blocks top-down from the copy anchor (facts, synopsis, then each link group's label).
/// The synopsis takes what the facts and the link groups leave (at most [`SYNOPSIS_MAX_LINES`]),
/// so the groups always show; nothing runs past the column. Pure, so the host suite grades it.
pub fn info_flow<'a>(info: &'a Info, title: &str, measure: &'a dyn Measure) -> Vec<Block<'a>> {
    flow(info, title, measure, 0.0, None).blocks
}

/// The room the action row keeps at the column's foot: a disc and the gap above it.
const ACTION_BAND: f32 = ACTION_D + theme::space::MD;

/// The chips `names` laid on `w`-wide lines from x = 0, `lines` lines at most: each shown chip's
/// `(x, line, width)`, and when some give way the `+N` chip's `(x, line, width, N)`. A chip wider
/// than the line is cut to it.
// ponytail: a name wider than the column (no shipped addon sends one) is drawn at the column's
// width and overhangs its own chip; elide inside `widgets::badge` if one ever does.
pub fn chip_lines(names: &[&str], w: f32, lines: usize, measure: &dyn Measure) -> (Vec<(f32, usize, f32)>, Option<(f32, usize, f32, usize)>) {
    let bw = |t: &str| sn_ui::widgets::badge_w(t, None, measure).min(w);
    let mut placed: Vec<(f32, usize, f32)> = Vec::new();
    let (mut x, mut line) = (0.0, 0);
    for n in names {
        let cw = bw(n);
        if x > 0.0 && x + cw > w {
            (x, line) = (0.0, line + 1);
        }
        if line >= lines {
            break;
        }
        placed.push((x, line, cw));
        x += cw + LINK_GAP;
    }
    if placed.len() == names.len() {
        return (placed, None);
    }
    loop {
        let hidden = names.len() - placed.len();
        let mw = bw(&format!("+{hidden}"));
        let (mx, ml) = placed.last().map_or((0.0, 0), |&(x, l, cw)| (x + cw + LINK_GAP, l));
        if placed.is_empty() || mx + mw <= w {
            return (placed, Some((mx, ml, mw, hidden)));
        }
        placed.pop();
    }
}

/// The height of `n` chip lines.
fn chips_h(n: usize) -> f32 {
    n as f32 * sn_ui::widgets::BADGE_H + n.saturating_sub(1) as f32 * LINK_GAP
}

/// **Link group `group`'s chips placed** from (`x`, `y`) on `w`-wide lines, `lines` at most
/// ([`chip_lines`]): its [`GroupPlace`] and the height its chip lines take. The one chip layout
/// the narrative column and the player's Show info panel share.
pub fn place_chips(group: usize, names: &[&str], x: f32, y: f32, w: f32, lines: usize, measure: &dyn Measure) -> (GroupPlace, f32) {
    let (placed, more) = chip_lines(names, w, lines, measure);
    let rows = placed.last().map_or(0, |p| p.1 + 1).max(more.map_or(0, |m| m.1 + 1));
    let at = |cx: f32, line: usize, cw: f32| Rect::new(x + cx, y + line as f32 * (sn_ui::widgets::BADGE_H + LINK_GAP), cw, sn_ui::widgets::BADGE_H);
    let chips = placed.iter().map(|&(cx, l, cw)| at(cx, l, cw)).collect();
    (GroupPlace { group, chips, more: more.map(|(cx, l, cw, n)| (at(cx, l, cw), n)) }, chips_h(rows))
}

/// [`info_flow`] laid out whole: the text blocks, the link groups' chips, the frame, and the top
/// of the room kept free at the column's foot (`reserve`, the library buttons'), right under the
/// last block or chip line. Group `expanded` shows every chip (wrapped) where the others stop at
/// [`LINK_LINES`]; it gives up lines before the synopsis drops under [`SYNOPSIS_MIN_LINES`].
pub struct Flow<'a> {
    pub blocks: Vec<Block<'a>>,
    pub groups: Vec<GroupPlace>,
    pub frame: Rect,
    pub free: f32,
}

fn flow<'a>(info: &'a Info, title: &str, measure: &'a dyn Measure, reserve: f32, expanded: Option<usize>) -> Flow<'a> {
    let layout = detail_layout();
    let n = layout.narrative;
    let foot = n.y + n.h;
    let lead = INFO_SIZE as f32 + theme::space::XS;
    let gap = theme::space::MD;
    let frame = match info.logo {
        // the logo's constant band stands where the title's lines would
        Some(_) => {
            let band = logo_band();
            let top = band.y + band.h + gap;
            Rect::new(n.x, top, n.w, (foot - top).max(0.0))
        }
        None => layout.narrative_copy_frame(true, title, foot + gap, measure),
    };
    let bottom = frame.y + frame.h - reserve;
    let names: Vec<Vec<&str>> = info.links.iter().map(|g| g.chips.iter().map(|c| c.name.as_str()).collect()).collect();
    let label_h = |g: &LinkGroup| label_view(&g.caps, measure).measure_h(frame.w);
    // the synopsis' own section label over it (SUMMARY), when there is a synopsis to label
    let summary_h = if info.synopsis.is_empty() || info.summary_caps.is_empty() {
        0.0
    } else {
        label_view(&info.summary_caps, measure).measure_h(frame.w) + theme::space::XS
    };
    let lines_of = |i: usize, cap: usize| -> usize {
        let (placed, more) = chip_lines(&names[i], frame.w, cap, measure);
        placed.last().map_or(0, |p| p.1 + 1).max(more.map_or(0, |m| m.1 + 1))
    };
    let mut caps: Vec<usize> = (0..info.links.len()).map(|i| if expanded == Some(i) { lines_of(i, usize::MAX) } else { LINK_LINES }).collect();
    let groups_h = |caps: &[usize]| -> f32 {
        info.links.iter().enumerate().map(|(i, g)| label_h(g) + theme::space::XS + chips_h(lines_of(i, caps[i])) + theme::space::SM).sum()
    };
    let facts_h = if info.facts.is_empty() { 0.0 } else { info_view(&info.facts, theme::TEXT_SECONDARY, 1, measure).measure_h(frame.w) + gap };
    let synopsis_lines = |caps: &[usize]| ((bottom - frame.y - facts_h - summary_h - groups_h(caps) - gap) / lead).floor().max(0.0) as usize;
    // a short column folds chip lines before the synopsis drops under its minimum: the last group
    // with more than one line first, the expanded group last
    while !info.synopsis.is_empty() && synopsis_lines(&caps) < SYNOPSIS_MIN_LINES {
        let lines: Vec<usize> = (0..caps.len()).map(|i| lines_of(i, caps[i])).collect();
        let fold = (0..caps.len()).rev().filter(|i| lines[*i] > 1).min_by_key(|i| expanded == Some(*i));
        let Some(i) = fold else { break };
        caps[i] = lines[i] - 1;
    }
    let mut blocks = Vec::new();
    let mut groups = Vec::new();
    let mut y = frame.y;
    let push = |text: &'a str, ink: [f32; 4], lines: usize, y: &mut f32, after: f32, blocks: &mut Vec<Block<'a>>| -> bool {
        let h = info_view(text, ink, lines, measure).measure_h(frame.w);
        if *y + h > bottom {
            return false;
        }
        blocks.push(Block { text, ink, rect: Rect::new(frame.x, *y, frame.w, h), lines, size: INFO_SIZE });
        *y += h + after;
        true
    };
    if !info.facts.is_empty() {
        push(&info.facts, theme::TEXT_SECONDARY, 1, &mut y, gap, &mut blocks);
    }
    if !info.synopsis.is_empty() {
        let lines = synopsis_lines(&caps).min(SYNOPSIS_MAX_LINES);
        if lines > 0 {
            if summary_h > 0.0 {
                let lh = summary_h - theme::space::XS;
                blocks.push(Block { text: &info.summary_caps, ink: theme::TEXT_SECONDARY, rect: Rect::new(frame.x, y, frame.w, lh), lines: 1, size: LINK_LABEL_SIZE });
                y += summary_h;
            }
            push(&info.synopsis, theme::TEXT_READING, lines, &mut y, gap, &mut blocks);
        }
    }
    let mut last = blocks.last().map_or(frame.y, |b| b.rect.y + b.rect.h);
    for (i, g) in info.links.iter().enumerate() {
        let lh = label_h(g);
        let (place, ch) = place_chips(i, &names[i], frame.x, y + lh + theme::space::XS, frame.w, caps[i], measure);
        if y + lh + theme::space::XS + ch > bottom {
            break;
        }
        blocks.push(Block { text: &g.caps, ink: theme::TEXT_SECONDARY, rect: Rect::new(frame.x, y, frame.w, lh), lines: 1, size: LINK_LABEL_SIZE });
        y += lh + theme::space::XS;
        groups.push(place);
        last = y + ch;
        y = last + theme::space::SM;
    }
    let free = if blocks.is_empty() { frame.y } else { last + theme::space::MD };
    Flow { blocks, groups, frame, free }
}

pub use crate::episodes::seasons;
use crate::episodes::{season_label, today};

/// The season a series opens on: the season of `last_video` (the library item's last-watched
/// video) when the meta lists it, else the first numbered season (specials only when that is all
/// there is). `None` for a meta with no video.
// ponytail: "the first season with an unwatched released episode" needs the library's watched
// bitfield decoded per video; add it when a viewer with no last-watched video asks for it.
pub fn default_season(meta: &Meta, last_video: Option<&str>) -> Option<u32> {
    last_video
        .and_then(|id| meta.videos.iter().find(|v| v.id == id))
        .map(|v| v.season.unwrap_or(0))
        .or_else(|| seasons(meta).first().copied())
}

/// One season of a series' episodes (`season`, else the first in picker order), in episode order.
fn episodes(out: &mut Built, item: &ItemRef, meta: &Meta, season: Option<u32>, today: &str, watched: Option<&crate::episodes::Watched<'_>>) {
    let all = seasons(meta);
    let Some(season) = season.filter(|s| all.contains(s)).or_else(|| all.first().copied()) else { return };
    let show = if meta.preview.name.is_empty() { item.name.clone() } else { meta.preview.name.clone() };
    // the season dropdown above the list names the season; with one season there is none
    let mut s = Section::new(if all.len() > 1 { String::new() } else { season_label(season) });
    for (i, v) in crate::episodes::of_season(meta, season) {
        let address = crate::episodes::address(season, v);
        let title = if v.title.trim().is_empty() { address.clone() } else { v.title.clone() };
        let row = match watched {
            Some(w) => crate::episodes::row_marked(v, season, today, w.of_video(v)),
            None => crate::episodes::row(v, season, today),
        };
        let action = Action::Episode(VideoRef {
            kind: item.kind.clone(),
            video_id: v.id.clone(),
            show: show.clone(),
            title: if v.title.trim().is_empty() { address } else { format!("{address} {title}") },
            resume_ms: 0,
            item_id: item.id.clone(),
            from: String::new(),
        });
        out.row(&mut s, row, EPISODE_KEY + (i as u32).min(0x00FF_FFFF), Some(action));
    }
    out.sections.push(s);
}

/// The chip row above the table, in the content column at `frame`'s top: a series' season
/// dropdown (one button saying the season listed, `dropdown`) or an episode page's one All
/// episodes button (nothing checked); or a stream list's addon dropdown. The row slides so the
/// focused chip stays on screen (the type chips' `sn_ui::value_chip::row_xs`), and never reaches
/// past the content column.
pub struct SeasonRow<'a> {
    pub labels: &'a [CString],
    /// The row's first element key and its focus group: the chip row's ([`SEASON_KEY`],
    /// [`SEASON_GROUP`]) or the addon dropdown's ([`ADDON_KEY`], [`ADDON_GROUP`]).
    pub base: u32,
    pub group: GroupId,
    /// The checked chip's index, which also seats the row.
    pub chosen: Option<usize>,
    pub entry: EntryId,
    /// The content column (its top is the row's).
    pub frame: Rect,
    /// The chips' widths, measured once per chip list and choice (layout asks every frame).
    pub widths: &'a RefCell<Option<Vec<f32>>>,
    /// The first label is a dropdown's (`crate::dropdown`'s button: a chevron, no check); the rest
    /// are buttons.
    pub dropdown: bool,
}

impl SeasonRow<'_> {
    fn index_of(&self, elem: u32) -> Option<usize> {
        elem.checked_sub(self.base).map(|n| n as usize).filter(|n| *n < self.labels.len())
    }

    fn key(&self, n: usize) -> FocusKey<u32> {
        FocusKey { entry: self.entry, elem: self.base + n as u32 }
    }

    fn chosen_index(&self) -> usize {
        self.chosen.filter(|c| *c < self.labels.len()).unwrap_or(0)
    }

    fn rects(&self, measure: &dyn Measure, focused: Option<usize>) -> Vec<Rect> {
        let mut memo = self.widths.borrow_mut();
        let widths = memo.get_or_insert_with(|| {
            let size = theme::size::BODY;
            self.labels
                .iter()
                .enumerate()
                .map(|(n, l)| match self.dropdown && n == 0 {
                    true => crate::dropdown::face_w(&l.to_string_lossy(), measure),
                    false => Button::pill_w_measured(l, size, self.chosen == Some(n), false, measure),
                })
                // a face wider than the column elides inside it
                .map(|w| w.min(self.frame.w))
                .collect()
        });
        // row_xs lays the row from the safe margin; this one starts at the content column
        let off = self.frame.x - sn_ui::consts::MARGIN_X;
        let mut xs: Vec<f32> = sn_ui::value_chip::row_xs(widths, focused, theme::space::SM).into_iter().map(|x| x + off).collect();
        if let Some(i) = focused.filter(|i| *i < xs.len()) {
            let over = (xs[i] + widths[i] - (self.frame.x + self.frame.w)).max(0.0);
            xs.iter_mut().for_each(|x| *x -= over);
        }
        xs.into_iter().zip(widths.iter()).map(|(x, w)| Rect::new(x, self.frame.y, *w, crate::type_filter::CHIP_H)).collect()
    }

    fn focused(&self, focus: Option<FocusKey<u32>>) -> Option<usize> {
        focus.filter(|k| k.entry == self.entry).and_then(|k| self.index_of(k.elem))
    }

    fn extent(&self) -> Rect {
        Rect::new(self.frame.x, self.frame.y, self.frame.w, crate::type_filter::CHIP_H)
    }
}

impl Focusable<Bundle> for SeasonRow<'_> {
    fn groups(&self, _cx: &Cx<'_, Bundle>, out: &mut Vec<GroupSpec>) {
        out.push(GroupSpec {
            id: self.group,
            kind: GroupKind::Row { wrap: false },
            seat: Seat::Remembered,
            reachable: AxisMask::BOTH,
            // [up, down, left, right]: DOWN to the episodes, LEFT to the library button (an item
            // page's; with none it stops), RIGHT stops at the row's end
            edge: [EdgeRule::Geometric, EdgeRule::Geometric, EdgeRule::Geometric, EdgeRule::Stop],
            extent: self.extent(),
            len: self.labels.len(),
            elem: ElemKind::Control,
        });
    }
    fn group_of(&self, key: &u32, _cx: &Cx<'_, Bundle>) -> Option<GroupId> {
        self.index_of(*key).map(|_| self.group)
    }
    fn neighbour(&self, key: FocusKey<u32>, dir: Dir, _cx: &Cx<'_, Bundle>) -> Step<u32> {
        let Some(n) = self.index_of(key.elem) else { return Step::Edge };
        match dir {
            Dir::Left if n > 0 => Step::Move(self.key(n - 1)),
            Dir::Right if n + 1 < self.labels.len() => Step::Move(self.key(n + 1)),
            _ => Step::Edge,
        }
    }
    fn place(&self, key: &u32, cx: &Cx<'_, Bundle>, _at: At) -> Option<Placed> {
        let n = self.index_of(*key)?;
        let r = *self.rects(cx.measure, self.focused(cx.focus.current)).get(n)?;
        Some(Placed { rect: r, rest_rect: r, clip: Rect::FULL, index: Some(n as u32) })
    }
    fn reconcile(&self, want: FocusKey<u32>, _cx: &Cx<'_, Bundle>) -> FocusKey<u32> {
        self.key(self.index_of(want.elem).unwrap_or_else(|| self.chosen_index()))
    }
    fn seat(&self, _g: GroupId, _from: Placed, _cx: &Cx<'_, Bundle>) -> FocusKey<u32> {
        self.key(self.chosen_index())
    }
}

impl Part<Bundle> for SeasonRow<'_> {
    fn prepare(&mut self, _b: &mut Budget, _cx: &Cx<'_, Bundle>) {}
    fn draw(&mut self, f: &mut DrawFrame<'_, '_, Bundle>, _rect: Rect) {
        let p = f.painter;
        let focused = self.focused(f.focus.current);
        for (n, rect) in self.rects(f.measure, focused).into_iter().enumerate() {
            if self.dropdown && n == 0 {
                crate::dropdown::draw_face(p, rect, &self.labels[n].to_string_lossy(), focused == Some(n), f.measure);
            } else {
                let mut b = Button::new(self.labels[n].as_ptr(), theme::size::BODY, rect).focused(focused == Some(n));
                if self.chosen == Some(n) {
                    b = b.icon(sn_ui::icons::Icon::Check);
                }
                b.draw(&sn_ui::Env::inert(), p);
            }
            f.stop(
                p,
                // a click activates at once (a `Press` stop only arms, and a click has no release)
                Stop { key: self.key(n), rect, rest_rect: rect, clip: Rect::FULL, hover: Hover::Focus, activate: Activate::Immediate },
            );
        }
    }
}

/// The narrative column's link chips ([`flow`]'s groups): one focus group per link group shown,
/// each chip a stop. OK (or a click) pushes the chip's page; the `+N` chip, once focused, expands its
/// group ([`DetailScreen`] holds which), and focus moves on to the first chip it stood for.
pub struct LinkColumn<'a> {
    pub info: &'a Info,
    pub title: &'a str,
    /// The library buttons' room at the column's foot.
    pub reserve: f32,
    /// The expanded group and the first chip its `+N` stood for.
    pub expanded: Option<(usize, usize)>,
    pub entry: EntryId,
    /// Where the groups stand, measured once per layout (focus asks every frame).
    pub places: &'a RefCell<Option<Vec<GroupPlace>>>,
}

/// The `+N` chip's slot in its group's keys.
const MORE_SLOT: u32 = 0xFF;
/// The link chips' element keys: `LINK_KEY + (group << 8) + slot`, in the chip band
/// (`SEASON_KEY..NOTE_KEY`, which the first seat does not count as chosen).
const LINK_KEY: u32 = 0x0E40_0000;
/// The first link group's focus group; group `g` is `LINK_GROUP + g`.
const LINK_GROUP: u32 = 0x10;

fn link_group(g: usize) -> GroupId {
    GroupId(LINK_GROUP + g as u32)
}

/// The link group and slot an element key names.
fn link_slot(elem: u32) -> Option<(usize, u32)> {
    let off = elem.checked_sub(LINK_KEY).filter(|o| *o < 0x1_0000)?;
    Some(((off >> 8) as usize, off & 0xFF))
}

impl LinkColumn<'_> {
    fn places(&self, measure: &dyn Measure) -> Vec<GroupPlace> {
        self.places
            .borrow_mut()
            .get_or_insert_with(|| flow(self.info, self.title, measure, self.reserve, self.expanded.map(|e| e.0)).groups)
            .clone()
    }

    fn key(&self, g: usize, slot: u32) -> FocusKey<u32> {
        FocusKey { entry: self.entry, elem: LINK_KEY + ((g as u32) << 8) + slot }
    }

    /// A group's stops in order: its chips, then its `+N`.
    fn stops(place: &GroupPlace) -> Vec<(u32, Rect)> {
        place.chips.iter().enumerate().map(|(i, r)| (i as u32, *r)).chain(place.more.map(|(r, _)| (MORE_SLOT, r))).collect()
    }

    fn rect_of(&self, elem: u32, measure: &dyn Measure) -> Option<Rect> {
        let (g, slot) = link_slot(elem)?;
        let places = self.places(measure);
        let place = places.iter().find(|p| p.group == g)?;
        Self::stops(place).into_iter().find(|(s, _)| *s == slot).map(|(_, r)| r)
    }

    /// The group's whole extent.
    fn extent(place: &GroupPlace) -> Rect {
        let stops = Self::stops(place);
        stops.iter().skip(1).fold(stops.first().map_or(Rect::new(0.0, 0.0, 0.0, 0.0), |s| s.1), |u, s| u.union(s.1))
    }

    /// The whole column's extent, when a group shows.
    fn bounds(&self, measure: &dyn Measure) -> Option<Rect> {
        self.places(measure).iter().map(Self::extent).reduce(|a, b| a.union(b))
    }
}

impl Focusable<Bundle> for LinkColumn<'_> {
    fn groups(&self, cx: &Cx<'_, Bundle>, out: &mut Vec<GroupSpec>) {
        for place in self.places(cx.measure) {
            out.push(GroupSpec {
                id: link_group(place.group),
                kind: GroupKind::Row { wrap: true },
                seat: Seat::Nearest,
                reachable: AxisMask::BOTH,
                // [up, down, left, right]: UP/DOWN are the screen's links (the group above, the
                // group or the library button below); LEFT off a line is BACK, as on the library
                // button; RIGHT the screen's link back to the list (geometric when it has no row)
                edge: [EdgeRule::Stop, EdgeRule::Stop, EdgeRule::Nav(sn_machine::machine::NavOpKind::Back), EdgeRule::Geometric],
                extent: Self::extent(&place),
                len: Self::stops(&place).len(),
                elem: ElemKind::Control,
            });
        }
    }
    fn group_of(&self, key: &u32, cx: &Cx<'_, Bundle>) -> Option<GroupId> {
        let (g, _) = link_slot(*key)?;
        self.places(cx.measure).iter().any(|p| p.group == g).then(|| link_group(g))
    }
    fn neighbour(&self, key: FocusKey<u32>, dir: Dir, cx: &Cx<'_, Bundle>) -> Step<u32> {
        let Some((g, slot)) = link_slot(key.elem) else { return Step::Edge };
        let places = self.places(cx.measure);
        let Some(place) = places.iter().find(|p| p.group == g) else { return Step::Edge };
        let stops = Self::stops(place);
        let Some(i) = stops.iter().position(|(s, _)| *s == slot) else { return Step::Edge };
        let at = stops[i].1;
        let pick = match dir {
            Dir::Left => i.checked_sub(1).filter(|j| stops[*j].1.y == at.y),
            Dir::Right => Some(i + 1).filter(|j| *j < stops.len() && stops[*j].1.y == at.y),
            Dir::Up | Dir::Down => {
                // the nearest stop of the line above / below
                let line = stops.iter().map(|s| s.1.y).filter(|y| if dir == Dir::Up { *y < at.y } else { *y > at.y });
                let line = if dir == Dir::Up { line.fold(f32::MIN, f32::max) } else { line.fold(f32::MAX, f32::min) };
                (0..stops.len()).filter(|j| stops[*j].1.y == line).min_by(|a, b| {
                    let d = |j: &usize| (stops[*j].1.cx() - at.cx()).abs();
                    d(a).total_cmp(&d(b))
                })
            }
        };
        pick.map_or(Step::Edge, |j| Step::Move(self.key(g, stops[j].0)))
    }
    fn place(&self, key: &u32, cx: &Cx<'_, Bundle>, _at: At) -> Option<Placed> {
        let r = self.rect_of(*key, cx.measure)?;
        Some(Placed { rect: r, rest_rect: r, clip: Rect::FULL, index: link_slot(*key).map(|s| s.1) })
    }
    fn reconcile(&self, want: FocusKey<u32>, cx: &Cx<'_, Bundle>) -> FocusKey<u32> {
        let Some((g, slot)) = link_slot(want.elem) else { return want };
        if self.rect_of(want.elem, cx.measure).is_some() {
            return want;
        }
        // the `+N` of the expanded group: the first chip it stood for; a chip that gave way: the `+N`
        if let Some((e, from)) = self.expanded.filter(|e| e.0 == g && slot == MORE_SLOT) {
            return self.key(e, from as u32);
        }
        let places = self.places(cx.measure);
        match places.iter().find(|p| p.group == g).and_then(|p| Self::stops(p).last().map(|s| s.0)) {
            Some(last) => self.key(g, last),
            None => want,
        }
    }
    fn seat(&self, g: GroupId, from: Placed, cx: &Cx<'_, Bundle>) -> FocusKey<u32> {
        let places = self.places(cx.measure);
        let place = places.iter().find(|p| link_group(p.group) == g).or(places.first());
        let Some(place) = place else { return self.key(0, 0) };
        // the stop nearest the press's source: its line first, then across
        let (fx, fy) = (from.rect.cx(), from.rect.cy());
        let best = Self::stops(place).into_iter().min_by(|a, b| {
            let d = |r: Rect| (r.cy() - fy).abs() * 4.0 + (r.cx() - fx).abs();
            d(a.1).total_cmp(&d(b.1))
        });
        self.key(place.group, best.map_or(0, |s| s.0))
    }
}

impl Part<Bundle> for LinkColumn<'_> {
    fn prepare(&mut self, _b: &mut Budget, _cx: &Cx<'_, Bundle>) {}
    fn draw(&mut self, f: &mut DrawFrame<'_, '_, Bundle>, _rect: Rect) {
        let p = f.painter;
        for place in self.places(f.measure) {
            let Some(group) = self.info.links.get(place.group) else { continue };
            for (slot, rect) in Self::stops(&place) {
                let key = self.key(place.group, slot);
                let text = match (slot, place.more) {
                    (MORE_SLOT, Some((_, n))) => format!("+{n}"),
                    _ => group.chips.get(slot as usize).map(|c| c.name.clone()).unwrap_or_default(),
                };
                let style = sn_ui::widgets::BadgeStyle::Link { focused: f.focus.current == Some(key) };
                sn_ui::widgets::badge(p, rect.x, rect.cy(), &text, None, style, f.measure);
                f.stop(p, Stop { key, rect, rest_rect: rect, clip: Rect::FULL, hover: Hover::Focus, activate: Activate::Immediate });
            }
        }
    }
}

/// **The action row** at the foot of an item page's narrative column (stremio-web's MetaPreview
/// action buttons): round icon discs — the library toggle (a plus, or a check once the account
/// holds the item), then a movie's watched mark (an eye, or a check once watched) — that UNFURL
/// their verb while focused (`CircleButton::label`, `open` the unfurl springs). LEFT/RIGHT walk
/// the row; RIGHT off its end is the list, LEFT off its start is BACK.
pub struct LibraryButton<'a> {
    pub label: &'a CString,
    /// The account holds the item (or is being told to): the disc wears a check.
    pub held: bool,
    /// A movie's *Mark as Watched / Unwatched* disc beside it, and whether it is watched (a check).
    pub watched: Option<(&'a CString, bool)>,
    pub entry: EntryId,
    pub info: &'a Info,
    pub title: &'a str,
    /// The expanded link group ([`LinkColumn::expanded`]'s group).
    pub expanded: Option<usize>,
    /// How far each disc's verb has unfurled (0 shut, 1 open): the library disc's, the watched one's.
    pub open: [f32; 2],
    /// Where the row starts, measured once per column (focus asks every frame).
    pub rect: &'a RefCell<Option<Rect>>,
}

/// Where the stream panel's ground starts ([`theme::DETAIL_STREAM_PANEL`]): the list's column, less
/// its air. The narrative column's action row may unfurl into the gap up to it.
pub fn stream_panel_x() -> f32 {
    detail_layout().content.x - theme::space::LG
}

/// An action disc's diameter (`CircleButton`'s own).
pub const ACTION_D: f32 = 60.0;
/// Between two action discs.
const ACTION_GAP: f32 = theme::space::XS;

impl LibraryButton<'_> {
    /// The room the row keeps at the column's foot.
    fn band(&self) -> f32 {
        ACTION_BAND
    }

    /// Where the row's first disc stands: the column's foot.
    fn origin(&self, measure: &dyn Measure) -> Rect {
        *self.rect.borrow_mut().get_or_insert_with(|| {
            let f = flow(self.info, self.title, measure, self.band(), self.expanded);
            // right under the column's last block, so the actions read with what they act on
            // rather than alone at the screen's foot (owner, 2026-10-18); never past the foot
            let foot = f.frame.y + f.frame.h;
            Rect::new(f.frame.x, f.free.min(foot - ACTION_D), ACTION_D, ACTION_D)
        })
    }

    /// Disc `i`'s width with its verb `label` unfurled `e`.
    fn disc_w(label: &CString, e: f32, measure: &dyn Measure) -> f32 {
        let lw = measure.width(label, theme::size::BODY, true);
        sn_ui::widgets::CircleButton::cap_w(ACTION_D, e, lw)
    }

    fn rect(&self, measure: &dyn Measure) -> Rect {
        let o = self.origin(measure);
        Rect::new(o.x, o.y, Self::disc_w(self.label, self.open[0], measure), ACTION_D)
    }

    /// The watched disc's rect: after the library disc.
    fn watched_rect(&self, measure: &dyn Measure) -> Option<Rect> {
        let (label, _) = self.watched?;
        let lib = self.rect(measure);
        Some(Rect::new(lib.x + lib.w + ACTION_GAP, lib.y, Self::disc_w(label, self.open[1], measure), ACTION_D))
    }

    fn rect_of(&self, elem: u32, measure: &dyn Measure) -> Option<Rect> {
        match elem {
            LIBRARY_KEY => Some(self.rect(measure)),
            WATCHED_KEY => self.watched_rect(measure),
            _ => None,
        }
    }

    fn key(&self) -> FocusKey<u32> {
        FocusKey { entry: self.entry, elem: LIBRARY_KEY }
    }
}

impl Focusable<Bundle> for LibraryButton<'_> {
    fn groups(&self, cx: &Cx<'_, Bundle>, out: &mut Vec<GroupSpec>) {
        out.push(GroupSpec {
            id: LIBRARY_GROUP,
            kind: GroupKind::Row { wrap: false },
            seat: Seat::Remembered,
            reachable: AxisMask::BOTH,
            // [up, down, left, right]: LEFT off the row is BACK, as on the list; RIGHT off it the list
            edge: [EdgeRule::Stop, EdgeRule::Stop, EdgeRule::Nav(sn_machine::machine::NavOpKind::Back), EdgeRule::Geometric],
            extent: match self.watched_rect(cx.measure) {
                Some(w) => self.rect(cx.measure).union(w),
                None => self.rect(cx.measure),
            },
            len: 1 + usize::from(self.watched.is_some()),
            elem: ElemKind::Control,
        });
    }
    fn group_of(&self, key: &u32, _cx: &Cx<'_, Bundle>) -> Option<GroupId> {
        (*key == LIBRARY_KEY || *key == WATCHED_KEY && self.watched.is_some()).then_some(LIBRARY_GROUP)
    }
    fn neighbour(&self, key: FocusKey<u32>, dir: Dir, _cx: &Cx<'_, Bundle>) -> Step<u32> {
        match (key.elem, dir) {
            (LIBRARY_KEY, Dir::Right) if self.watched.is_some() => Step::Move(FocusKey { entry: self.entry, elem: WATCHED_KEY }),
            (WATCHED_KEY, Dir::Left) => Step::Move(self.key()),
            _ => Step::Edge,
        }
    }
    fn place(&self, key: &u32, cx: &Cx<'_, Bundle>, _at: At) -> Option<Placed> {
        let r = self.rect_of(*key, cx.measure)?;
        Some(Placed { rect: r, rest_rect: r, clip: Rect::FULL, index: Some(u32::from(*key == WATCHED_KEY)) })
    }
    fn reconcile(&self, want: FocusKey<u32>, _cx: &Cx<'_, Bundle>) -> FocusKey<u32> {
        if want.elem == WATCHED_KEY && self.watched.is_some() {
            return want;
        }
        self.key()
    }
    fn seat(&self, _g: GroupId, _from: Placed, _cx: &Cx<'_, Bundle>) -> FocusKey<u32> {
        self.key()
    }
}

impl Part<Bundle> for LibraryButton<'_> {
    fn prepare(&mut self, _b: &mut Budget, _cx: &Cx<'_, Bundle>) {}
    fn draw(&mut self, f: &mut DrawFrame<'_, '_, Bundle>, _rect: Rect) {
        use sn_ui::icons::Icon;
        let p = f.painter;
        let mut discs = vec![(LIBRARY_KEY, self.rect(f.measure), self.label, if self.held { Icon::Check } else { Icon::Plus }, self.open[0])];
        if let (Some((label, on)), Some(rect)) = (self.watched, self.watched_rect(f.measure)) {
            discs.push((WATCHED_KEY, rect, label, if on { Icon::Check } else { Icon::Eye }, self.open[1]));
        }
        for (elem, rect, label, icon, e) in discs {
            let key = FocusKey { entry: self.entry, elem };
            sn_ui::widgets::CircleButton::new(c"".as_ptr())
                .frame(rect)
                .icon(icon)
                .focused(f.focus.current == Some(key))
                .label(label.as_ptr(), e)
                .draw(&sn_ui::Env::inert(), p);
            f.stop(p, Stop { key, rect, rest_rect: rect, clip: Rect::FULL, hover: Hover::Focus, activate: Activate::Immediate });
        }
    }
}

/// The page as parts: the route's header and table, the chip row above the table (a series'
/// season chips when it has more than one season, an episode's All episodes button), the
/// narrative column's link chips and an item page's library button.
struct DetailView<'a> {
    screen: TableScreen<'a>,
    seasons: Option<SeasonRow<'a>>,
    /// The addon dropdown over a stream list.
    addons: Option<SeasonRow<'a>>,
    links: Option<LinkColumn<'a>>,
    library: Option<LibraryButton<'a>>,
}

const SEASONS_PART: PartId = PartId(3);
const LIBRARY_PART: PartId = PartId(4);
const LINKS_PART: PartId = PartId(5);
const ADDONS_PART: PartId = PartId(6);

impl Composed<Bundle> for DetailView<'_> {
    fn layout(&self, cx: &Cx<'_, Bundle>) -> Vec<(PartId, Rect)> {
        let mut v = self.screen.layout(cx);
        if let Some(row) = &self.seasons {
            v.push((SEASONS_PART, row.extent()));
        }
        if let Some(row) = &self.addons {
            v.push((ADDONS_PART, row.extent()));
        }
        if let Some(b) = &self.library {
            v.push((LIBRARY_PART, b.watched_rect(cx.measure).map_or(b.rect(cx.measure), |w| b.rect(cx.measure).union(w))));
        }
        if let Some(r) = self.links.as_ref().and_then(|l| l.bounds(cx.measure)) {
            v.push((LINKS_PART, r));
        }
        v
    }
    fn part(&self, id: PartId) -> &dyn Part<Bundle> {
        if let (ADDONS_PART, Some(row)) = (id, self.addons.as_ref()) {
            return row;
        }
        match (id, self.seasons.as_ref(), self.library.as_ref(), self.links.as_ref()) {
            (SEASONS_PART, Some(row), _, _) => row,
            (LIBRARY_PART, _, Some(b), _) => b,
            (LINKS_PART, _, _, Some(l)) => l,
            _ => self.screen.part(id),
        }
    }
    fn part_mut(&mut self, id: PartId) -> &mut dyn Part<Bundle> {
        if let (ADDONS_PART, Some(row)) = (id, self.addons.as_mut()) {
            return row;
        }
        match (id, self.seasons.as_mut(), self.library.as_mut(), self.links.as_mut()) {
            (SEASONS_PART, Some(row), _, _) => row,
            (LIBRARY_PART, _, Some(b), _) => b,
            (LINKS_PART, _, _, Some(l)) => l,
            _ => self.screen.part_mut(id),
        }
    }
}

impl Focusable<Bundle> for DetailView<'_> {
    fn groups(&self, cx: &Cx<'_, Bundle>, out: &mut Vec<GroupSpec>) {
        let n = out.len();
        composed_groups(self, cx, out);
        // LEFT off the list is the library button or the link chips beside it, not BACK (their
        // LEFT is; with a library button the screen's link names it, [`DetailScreen::links`])
        if self.library.is_some() || self.links.as_ref().is_some_and(|l| l.bounds(cx.measure).is_some()) {
            if let Some(table) = out[n..].iter_mut().find(|g| g.id == GroupId(0)) {
                table.edge[2] = EdgeRule::Geometric;
            }
        }
    }
    fn group_of(&self, key: &u32, cx: &Cx<'_, Bundle>) -> Option<GroupId> {
        composed_group_of(self, key, cx)
    }
    fn neighbour(&self, key: FocusKey<u32>, dir: Dir, cx: &Cx<'_, Bundle>) -> Step<u32> {
        composed_neighbour(self, key, dir, cx)
    }
    fn place(&self, key: &u32, cx: &Cx<'_, Bundle>, at: At) -> Option<Placed> {
        composed_place(self, key, cx, at)
    }
    fn reconcile(&self, want: FocusKey<u32>, cx: &Cx<'_, Bundle>) -> FocusKey<u32> {
        composed_reconcile(self, want, cx)
    }
    fn seat(&self, g: GroupId, from: Placed, cx: &Cx<'_, Bundle>) -> FocusKey<u32> {
        composed_seat(self, g, from, cx)
    }
}

impl Part<Bundle> for DetailView<'_> {
    fn prepare(&mut self, b: &mut Budget, cx: &Cx<'_, Bundle>) {
        composed_prepare(self, b, cx)
    }
    fn draw(&mut self, f: &mut DrawFrame<'_, '_, Bundle>, _rect: Rect) {
        composed_draw(self, f)
    }
}

/// Does the library button show the item as held? Optimistic: an add on its way already is, a
/// removal on its way no longer is. Signed out nothing is.
fn library_held((signed_in, add): (bool, AddState)) -> bool {
    signed_in && matches!(add, AddState::In | AddState::Adding)
}

pub struct DetailScreen {
    entry: EntryId,
    mode: Mode,
    /// What the viewer chose to see of the stream list (the closing row open).
    stream_view: StreamView,
    /// The addon dropdown's one label, what it says (two serving addons at least; none otherwise).
    addon_label: Vec<CString>,
    addon_widths: RefCell<Option<Vec<f32>>>,
    table: TableView,
    built: Built,
    /// What the store held at the last build, and the settings it was ordered by.
    built_gen: Option<(u32, u32, u32)>,
    /// The streams this page holds open: the item's own, or a series' that listed no video.
    streams: Option<StreamKey>,
    /// The meta this page holds open.
    meta: Option<MetaKey>,
    /// The focused row's identity.
    sel: Option<RowKey>,
    title: String,
    /// What the narrative column shows under the title ([`info`]); empty until the meta lands.
    info: Info,
    crumb: String,
    closed: bool,
    /// The page mounts on read-outs (the meta and the streams still out): the first row that
    /// lands takes focus.
    seat: FirstSeat,
    /// A series' seasons (picker order) and the one listed; chips only with two or more.
    seasons: Vec<u32>,
    season: Option<u32>,
    season_labels: Vec<CString>,
    season_widths: RefCell<Option<Vec<f32>>>,
    /// A season was just chosen: the next build starts the list at its top.
    season_moved: bool,
    /// An overview page's episode: the list opens on its season.
    from_video: Option<String>,
    /// The viewer moved the focus (a key or the pointer). Until then a seat the engine gave the
    /// chip row on mount (the streams still out) yields to the first row once one lands.
    moved: bool,
    /// An episode page's All episodes button label (one chip), empty on any other page.
    overview_label: Vec<CString>,
    /// The library button's look (signed in, where the item stands) and label, read off the
    /// stores each tick; `None` until the first tick and on a page with no button.
    library: Option<(bool, AddState)>,
    library_label: CString,
    library_rect: RefCell<Option<Rect>>,
    /// The action discs' unfurl springs (the library disc's, the watched disc's): open on focus.
    action_open: [sn_ui::Spring; 2],
    /// A movie page's watched button (signed in): whether its entry is watched, and its label.
    watched: Option<bool>,
    watched_label: CString,
    /// The chip row: what each chip does and its label (the season dropdown or All episodes, then
    /// the season's or the episode's watched button), rebuilt each tick.
    chips: Vec<Chip>,
    chip_labels: Vec<CString>,
    /// The episode the page opened on (its row key), focused once its row is in.
    focus_episode: Option<u32>,
    /// The link group whose `+N` took focus, shown whole while focus stays in it, and the first
    /// chip the `+N` stood for.
    expanded: Option<(usize, usize)>,
    /// Where the link groups stand ([`LinkColumn::places`]).
    link_places: RefCell<Option<Vec<GroupPlace>>>,
}

/// What a chip of the row does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Chip {
    /// The season dropdown: its popover.
    Seasons,
    Overview,
    /// Mark the listed season watched (`true`) or unwatched.
    MarkSeason(bool),
    /// Mark the episode watched (`true`) or unwatched.
    MarkEpisode(bool),
}

impl DetailScreen {
    pub fn item(entry: EntryId, item: ItemRef) -> Self {
        Self::new(entry, Mode::Item(item))
    }

    pub fn video(entry: EntryId, video: VideoRef) -> Self {
        Self::new(entry, Mode::Video(video))
    }

    /// A resume with no recorded video: the first episode's streams once the meta names it.
    pub fn resume(entry: EntryId, item: ItemRef) -> Self {
        Self::new(entry, Mode::Resume(item))
    }

    /// A series' page opened from its episode `video`'s All episodes button: it lists that
    /// episode's season, and its crumb is the episode page BACK returns to.
    pub fn overview(entry: EntryId, video: VideoRef) -> Self {
        let item = ItemRef { kind: video.kind.clone(), id: video.item_id.clone(), name: video.show.clone(), from: String::new() };
        let mut page = Self::new(entry, Mode::Item(item));
        page.crumb = if video.title.is_empty() { video.show } else { video.title };
        page.from_video = Some(video.video_id);
        page
    }

    fn new(entry: EntryId, mode: Mode) -> Self {
        let (title, crumb) = match &mode {
            // the crumb names the page BACK returns to: the opener, else Home (an item) or the show
            // (an episode pushed from its series page)
            Mode::Item(i) | Mode::Resume(i) => (i.name.clone(), if i.from.is_empty() { msg::app_nav_home().to_string() } else { i.from.clone() }),
            Mode::Video(v) => (if v.title.is_empty() { v.show.clone() } else { v.title.clone() }, if v.from.is_empty() { v.show.clone() } else { v.from.clone() }),
        };
        let mut table = TableView::new();
        table.list_focused = true;
        // the list sits on the full-bleed backdrop: each addon's streams (a season's episodes) on
        // a card of its own, so the rows read over bright art
        table.grouped = true;
        let overview_label = Self::overview_label(&mode);
        DetailScreen {
            entry,
            mode,
            stream_view: StreamView::default(),
            addon_label: Vec::new(),
            addon_widths: RefCell::new(None),
            table,
            built: Built::default(),
            built_gen: None,
            streams: None,
            meta: None,
            sel: None,
            title,
            info: Info::default(),
            crumb,
            closed: false,
            seat: FirstSeat::new(),
            seasons: Vec::new(),
            season: None,
            season_labels: Vec::new(),
            season_widths: RefCell::new(None),
            season_moved: false,
            from_video: None,
            moved: false,
            overview_label,
            library: None,
            library_label: CString::default(),
            library_rect: RefCell::new(None),
            action_open: [sn_ui::Spring::at(0.0), sn_ui::Spring::at(0.0)],
            watched: None,
            watched_label: CString::default(),
            chips: Vec::new(),
            chip_labels: Vec::new(),
            focus_episode: None,
            expanded: None,
            link_places: RefCell::new(None),
        }
    }

    /// The narrative column moved (its text, the buttons under it, an expansion): measure again.
    fn relayout(&self) {
        self.library_rect.replace(None);
        self.link_places.replace(None);
    }

    /// The room the library buttons keep at the column's foot.
    fn reserve(&self) -> f32 {
        if self.library.is_some() { ACTION_BAND } else { 0.0 }
    }

    /// The link chips, as the suite reads them: each shown group's chip count and `+N`.
    pub fn link_places(&self) -> Vec<GroupPlace> {
        self.link_places.borrow().clone().unwrap_or_default()
    }

    /// The link chip `ev` presses (a click's element, or the focused one for OK): its group and slot.
    fn link_of(&self, ev: &ScreenEvent<Bundle>, cx: &Cx<'_, Bundle>) -> Option<(usize, u32)> {
        let elem = match ev {
            ScreenEvent::Activate(e) => *e,
            ScreenEvent::PressCommit(_) => cx.focus.current.filter(|k| k.entry == self.entry)?.elem,
            _ => return None,
        };
        link_slot(elem).filter(|(g, _)| *g < self.info.links.len())
    }

    /// Focus reached group `g`'s `+N` (a key, the pointer or a click): show the group whole and
    /// move on to the first chip it stood for.
    fn expand(&mut self, g: usize, fx: &mut Effects<'_, Bundle>) {
        // already open (a column too short to show more leaves focus on its `+N`)
        if self.expanded.is_some_and(|e| e.0 == g) {
            return;
        }
        let Some(from) = self.link_places.borrow().as_ref().and_then(|p| p.iter().find(|p| p.group == g)).map(|p| p.chips.len()) else { return };
        self.expanded = Some((g, from));
        self.relayout();
        let key = FocusKey { entry: self.entry, elem: LINK_KEY + ((g as u32) << 8) + from as u32 };
        crate::registry::reseat(sn_ui::screen::FocusTarget::Elem(key), fx);
        fx.invalidate(Provenance::Input);
    }

    /// The All episodes button's label, for a page that has one.
    fn overview_label(mode: &Mode) -> Vec<CString> {
        mode.show().map(|_| CString::new(msg::app_detail_all_episodes()).unwrap_or_default()).into_iter().collect()
    }

    pub fn mode(&self) -> &Mode {
        &self.mode
    }

    /// The action of the row `key` names.
    pub fn action_of(&self, key: u32) -> Option<&Action> {
        let i = self.built.index_of(RowKey(key))?;
        self.built.actions.get(i)?.as_ref()
    }

    /// The built table, as the test suite reads it.
    pub fn sections(&self) -> &[Section] {
        &self.table.sections
    }

    /// The season the episode list shows (`None` until the meta lands).
    pub fn season(&self) -> Option<u32> {
        self.season
    }

    /// The page's narrative column, as the suite reads it.
    pub fn info(&self) -> &Info {
        &self.info
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    /// The library button's label, as the suite reads it (`None`: no button).
    pub fn library_label(&self) -> Option<String> {
        self.library.map(|_| self.library_label.to_string_lossy().into_owned())
    }

    /// The item an item page's library button adds or removes.
    fn library_item(&self) -> Option<&ItemRef> {
        match &self.mode {
            Mode::Item(i) => Some(i),
            Mode::Video(_) | Mode::Resume(_) => None,
        }
    }

    /// Read where the item stands; relabel the button when that moved.
    fn sync_library(&mut self, cx: &Cx<'_, Bundle>, fx: &mut Effects<'_, Bundle>) {
        let Some(item) = self.library_item() else { return };
        let look = (cx.views.session.signed_in, cx.views.library.add_state(&item.id));
        // a movie's watched button (core's `MetaDetails` `MarkAsWatched`): signed in only
        let watched = (look.0 && !Mode::lists_episodes(&item.kind)).then(|| cx.views.library.item(&item.id).is_some_and(|i| i.watched()));
        if watched != self.watched {
            self.watched = watched;
            let verb = if watched == Some(true) { sn_ui::widgets::mark_unwatched_verb() } else { sn_ui::widgets::mark_watched_verb() };
            self.watched_label = CString::new(verb).unwrap_or_default();
            self.relayout();
            fx.invalidate(Provenance::Landing(fx.from()));
        }
        if self.library == Some(look) {
            return;
        }
        self.library = Some(look);
        self.library_label = CString::new(if library_held(look) { msg::app_menu_in_library() } else { msg::app_menu_add_to_library() }).unwrap_or_default();
        self.relayout();
        fx.invalidate(Provenance::Landing(fx.from()));
    }

    /// OK on the library button: add, remove, or (signed out) the sign-in page. Nothing while a
    /// write for the item is on its way.
    fn press_library(&self, cx: &Cx<'_, Bundle>, fx: &mut Effects<'_, Bundle>) {
        let Some(item) = self.library_item() else { return };
        if !cx.views.session.signed_in {
            fx.push(Fx::Nav(NavOp::Push(AppArg::SignIn)));
            return;
        }
        let cmd = match cx.views.library.add_state(&item.id) {
            AddState::Absent => {
                let preview = match self.meta.as_ref().and_then(|k| cx.views.detail.meta(k)) {
                    Some(MetaState::Ready(m)) => Some(&m.preview),
                    _ => None,
                };
                LibraryCmd::Add(sn_stremio::library::NewEntry {
                    id: item.id.clone(),
                    kind: item.kind.clone(),
                    name: preview.map(|p| p.name.clone()).filter(|n| !n.is_empty()).unwrap_or_else(|| item.name.clone()),
                    poster: preview.and_then(|p| p.poster.clone()),
                })
            }
            AddState::In => LibraryCmd::Remove(item.id.clone()),
            AddState::Adding | AddState::Removing => return,
        };
        fx.push(Fx::App(AppFx::Library(cmd)));
    }

    /// The library entry a mark on this page writes: the item (an episode page's show), named and
    /// pictured by its meta when it is in.
    fn mark_entry(&self, cx: &Cx<'_, Bundle>) -> Option<sn_stremio::library::NewEntry> {
        let (id, kind, name) = match &self.mode {
            Mode::Item(i) => (i.id.clone(), i.kind.clone(), i.name.clone()),
            Mode::Video(v) if !v.item_id.is_empty() => (v.item_id.clone(), v.kind.clone(), v.show.clone()),
            _ => return None,
        };
        let preview = match self.meta.as_ref().and_then(|k| cx.views.detail.meta(k)) {
            Some(MetaState::Ready(m)) => Some(&m.preview),
            _ => None,
        };
        Some(sn_stremio::library::NewEntry {
            id,
            kind,
            name: preview.map(|p| p.name.clone()).filter(|n| !n.is_empty()).unwrap_or(name),
            poster: preview.and_then(|p| p.poster.clone()),
        })
    }

    /// The meta this page holds, once it is in.
    fn ready_meta<'a>(&self, cx: &Cx<'a, Bundle>) -> Option<&'a Meta> {
        match self.meta.as_ref().and_then(|k| cx.views.detail.meta(k)) {
            Some(MetaState::Ready(m)) => Some(m),
            _ => None,
        }
    }

    /// The watched marks over this page's meta.
    fn watched_marks<'a>(&self, cx: &Cx<'a, Bundle>, meta: &Meta) -> crate::episodes::Watched<'a> {
        let id = self.meta.as_ref().map_or("", |k| k.id.as_str());
        crate::episodes::Watched::of(cx.views.library.items(), id, meta)
    }

    /// A watched mark: the movie's button, the season's, the episode's.
    fn press_mark(&self, edit: sn_stremio::library::Edit, cx: &Cx<'_, Bundle>, fx: &mut Effects<'_, Bundle>) {
        if let Some(entry) = self.mark_entry(cx).filter(|_| cx.views.session.signed_in) {
            fx.push(Fx::App(AppFx::Library(LibraryCmd::Mark(entry, edit))));
        }
    }

    /// Does `ev` press the movie's watched button?
    fn watched_pressed(&self, ev: &ScreenEvent<Bundle>, cx: &Cx<'_, Bundle>) -> bool {
        self.watched.is_some()
            && match ev {
                ScreenEvent::Activate(e) => *e == WATCHED_KEY,
                ScreenEvent::PressCommit(_) => cx.focus.current == Some(FocusKey { entry: self.entry, elem: WATCHED_KEY }),
                _ => false,
            }
    }

    /// The stream list's groups (none before it is open).
    fn stream_groups<'v>(&self, cx: &Cx<'v, Bundle>) -> &'v [StreamGroup] {
        self.streams.as_ref().and_then(|k| cx.views.detail.streams(k)).unwrap_or(&[])
    }

    /// The addon dropdown as the stream list stands: shown with two serving addons or more,
    /// saying which are asked.
    fn sync_addons(&mut self, cx: &Cx<'_, Bundle>, fx: &mut Effects<'_, Bundle>) {
        let groups = self.stream_groups(cx);
        let label = (groups.len() >= 2).then(|| crate::dropdown::stream_addon_summary(groups, cx.views.addons.stream_filter()));
        let label: Vec<CString> = label.into_iter().map(|l| CString::new(l.replace('\0', "")).unwrap_or_default()).collect();
        if label != self.addon_label {
            self.addon_label = label;
            self.addon_widths.replace(None);
            fx.invalidate(Provenance::Landing(fx.from()));
        }
    }

    /// Does `ev` press the addon dropdown (a click on it, or OK while it has focus)?
    fn addons_pressed(&self, ev: &ScreenEvent<Bundle>, cx: &Cx<'_, Bundle>) -> bool {
        !self.addon_label.is_empty()
            && match ev {
                ScreenEvent::Activate(e) => *e == ADDON_KEY,
                ScreenEvent::PressCommit(_) => cx.focus.current == Some(FocusKey { entry: self.entry, elem: ADDON_KEY }),
                _ => false,
            }
    }

    /// The addon dropdown's popover, under it: each serving addon, checked while it is asked.
    fn open_addons(&self, cx: &Cx<'_, Bundle>, fx: &mut Effects<'_, Bundle>) {
        let view = self.view();
        let Some(rect) = view.addons.as_ref().and_then(|row| row.place(&ADDON_KEY, cx, At::Drawn)).map(|p| p.rect) else { return };
        let kind = crate::dropdown::stream_addon_menu(self.stream_groups(cx), cx.views.addons.stream_filter());
        crate::dropdown::open(self.entry, cx, fx, rect, kind);
    }

    /// The chip row as it stands: the season dropdown or All episodes, then (signed in, the meta
    /// in) the listed season's or the episode's watched button.
    fn sync_chips(&mut self, cx: &Cx<'_, Bundle>, fx: &mut Effects<'_, Bundle>) {
        let mut chips = Vec::new();
        let mut labels: Vec<String> = Vec::new();
        if self.season_chips() {
            chips.push(Chip::Seasons);
            labels.push(self.season.map(season_label).unwrap_or_default());
        } else if self.mode.show().is_some() {
            chips.push(Chip::Overview);
            labels.push(msg::app_detail_all_episodes().to_string());
        }
        if let Some(m) = self.ready_meta(cx).filter(|_| cx.views.session.signed_in) {
            let w = self.watched_marks(cx, m);
            match &self.mode {
                Mode::Item(i) if Mode::lists_episodes(&i.kind) && !m.videos.is_empty() => {
                    if let Some(season) = self.season {
                        let all = w.season_watched(m, season);
                        chips.push(Chip::MarkSeason(!all));
                        labels.push(if all { msg::app_watched_unmark_season() } else { msg::app_watched_mark_season() }.to_string());
                    }
                }
                Mode::Video(v) if self.mode.show().is_some() => {
                    let on = w.bits.get_video(&v.video_id);
                    chips.push(Chip::MarkEpisode(!on));
                    labels.push(if on { sn_ui::widgets::mark_unwatched_verb() } else { sn_ui::widgets::mark_watched_verb() }.to_string());
                }
                _ => {}
            }
        }
        let labels: Vec<CString> = labels.iter().map(|l| CString::new(l.replace('\0', "")).unwrap_or_default()).collect();
        if chips != self.chips || labels != self.chip_labels {
            self.chips = chips;
            self.chip_labels = labels;
            self.season_widths.replace(None);
            fx.invalidate(Provenance::Landing(fx.from()));
        }
    }

    /// Does `ev` press the library button (a click on it, or OK while it has focus)?
    fn library_pressed(&self, ev: &ScreenEvent<Bundle>, cx: &Cx<'_, Bundle>) -> bool {
        self.library.is_some()
            && match ev {
                ScreenEvent::Activate(e) => *e == LIBRARY_KEY,
                ScreenEvent::PressCommit(_) => cx.focus.current == Some(FocusKey { entry: self.entry, elem: LIBRARY_KEY }),
                _ => false,
            }
    }

    fn season_chips(&self) -> bool {
        self.seasons.len() > 1 && matches!(self.mode, Mode::Item(_))
    }

    /// The listed season's position among the seasons.
    fn season_index(&self) -> usize {
        self.season.and_then(|c| self.seasons.iter().position(|s| *s == c)).unwrap_or(0)
    }

    /// The chip row's labels and checked chip: the season dropdown's one label (the season
    /// listed), an episode page's All episodes button, or `None` (no row).
    fn chip_row(&self) -> Option<(&[CString], Option<usize>)> {
        (!self.chip_labels.is_empty()).then_some((&self.chip_labels[..], None))
    }

    /// The table's frame: the route's sectioned frame, under the chip row when it shows.
    fn table_frame(&self) -> Rect {
        let f = detail_layout().sectioned_table();
        let rows = usize::from(self.chip_row().is_some()) + usize::from(!self.addon_label.is_empty());
        let band = crate::type_filter::BAND * rows as f32;
        Rect::new(f.x, f.y + band, f.w, f.h - band)
    }

    fn view(&self) -> DetailView<'_> {
        // a page with a logo draws the logo in the title's place ([`logo_band`])
        let title = if self.info.logo.is_some() { "" } else { self.title.as_str() };
        let screen = TableScreen::new(
            Header::new(detail_layout(), Some(self.crumb.as_str()), title, ""),
            &self.table,
            GroupId(0),
            self.entry,
        )
        .keyed(&self.built)
        .with_frame(self.table_frame());
        let seasons = self.chip_row().map(|(labels, chosen)| SeasonRow {
            labels,
            base: SEASON_KEY,
            group: SEASON_GROUP,
            chosen,
            entry: self.entry,
            frame: detail_layout().sectioned_table(),
            widths: &self.season_widths,
            dropdown: self.chips.first() == Some(&Chip::Seasons),
        });
        let addons = (!self.addon_label.is_empty()).then(|| {
            let f = detail_layout().sectioned_table();
            let top = if self.chip_row().is_some() { crate::type_filter::BAND } else { 0.0 };
            SeasonRow {
                labels: &self.addon_label,
                base: ADDON_KEY,
                group: ADDON_GROUP,
                chosen: None,
                entry: self.entry,
                frame: Rect::new(f.x, f.y + top, f.w, f.h - top),
                widths: &self.addon_widths,
                dropdown: true,
            }
        });
        let library = self.library.map(|look| LibraryButton {
            label: &self.library_label,
            held: library_held(look),
            watched: self.watched.map(|on| (&self.watched_label, on)),
            entry: self.entry,
            info: &self.info,
            title: &self.title,
            expanded: self.expanded.map(|e| e.0),
            open: [self.action_open[0].pos.clamp(0.0, 1.0), self.action_open[1].pos.clamp(0.0, 1.0)],
            rect: &self.library_rect,
        });
        let links = (!self.info.links.is_empty()).then(|| LinkColumn {
            info: &self.info,
            title: &self.title,
            reserve: self.reserve(),
            expanded: self.expanded,
            entry: self.entry,
            places: &self.link_places,
        });
        DetailView { screen, seasons, addons, links, library }
    }

    /// The chip `ev` presses: a click's element, or the focused one for OK.
    fn chip_of(&self, ev: &ScreenEvent<Bundle>, cx: &Cx<'_, Bundle>) -> Option<Chip> {
        let elem = match ev {
            ScreenEvent::Activate(e) => *e,
            ScreenEvent::PressCommit(_) => cx.focus.current.filter(|k| k.entry == self.entry)?.elem,
            _ => return None,
        };
        let n = elem.checked_sub(SEASON_KEY)? as usize;
        self.chips.get(n).copied()
    }

    /// The All episodes button's press: the show's page on this episode's season.
    fn open_overview(&self, fx: &mut Effects<'_, Bundle>) {
        if let Some(v) = self.mode.show() {
            fx.push(Fx::Nav(NavOp::Push(AppArg::Overview(v.clone()))));
        }
    }

    /// The season dropdown's popover, under it: every season, the listed one checked.
    fn open_seasons(&self, cx: &Cx<'_, Bundle>, fx: &mut Effects<'_, Bundle>) {
        let view = self.view();
        let Some(rect) = view.seasons.as_ref().and_then(|row| row.place(&SEASON_KEY, cx, At::Drawn)).map(|p| p.rect) else { return };
        let options = self.seasons.iter().map(|s| season_label(*s)).collect();
        let kind = MenuKind::Choice { title: msg::app_detail_seasons().to_string(), options, current: self.season_index(), target: ChoiceTarget::DetailSeason };
        crate::dropdown::open(self.entry, cx, fx, rect, kind);
    }

    /// List season `season` (a pick from the season dropdown).
    fn choose_season(&mut self, season: u32, cx: &Cx<'_, Bundle>, fx: &mut Effects<'_, Bundle>) {
        if self.season == Some(season) {
            return;
        }
        self.season = Some(season);
        self.season_widths.replace(None);
        self.season_moved = true;
        self.built_gen = None;
        self.sync(cx, fx);
        self.sync_chips(cx, fx);
    }

    fn open(&mut self, fx: &mut Effects<'_, Bundle>) {
        if let Some(key) = self.mode.meta_key() {
            fx.push(Fx::App(AppFx::Detail(DetailCmd::OpenMeta(key.clone()))));
            self.meta = Some(key);
        }
        if let Some(key) = self.mode.stream_key() {
            fx.push(Fx::App(AppFx::Detail(DetailCmd::OpenStreams(key.clone()))));
            self.streams = Some(key);
        }
    }

    fn close(&mut self, fx: &mut Effects<'_, Bundle>) {
        if std::mem::replace(&mut self.closed, true) {
            return;
        }
        if let Some(key) = self.meta.take() {
            fx.push(Fx::App(AppFx::Detail(DetailCmd::CloseMeta(key))));
        }
        if let Some(key) = self.streams.take() {
            fx.push(Fx::App(AppFx::Detail(DetailCmd::CloseStreams(key))));
        }
    }

    /// Rebuild the table from the store when it moved; keep the focused row by identity.
    fn sync(&mut self, cx: &Cx<'_, Bundle>, fx: &mut Effects<'_, Bundle>) {
        let store = cx.views.detail;
        // a resume's meta landed: this is now the first episode's Streams page
        if let (Mode::Resume(item), Some(MetaState::Ready(m))) = (&self.mode, self.meta.as_ref().and_then(|k| store.meta(k))) {
            if !self.closed {
                let video = resume_video(item, m);
                let key = StreamKey { kind: video.kind.clone(), video_id: video.video_id.clone() };
                fx.push(Fx::App(AppFx::Detail(DetailCmd::OpenStreams(key.clone()))));
                self.streams = Some(key);
                if !video.title.is_empty() {
                    self.title = video.title.clone();
                }
                self.mode = Mode::Video(video);
                self.overview_label = Self::overview_label(&self.mode);
                self.built_gen = None;
            }
        }
        // the meta names the video its streams are asked for (`behaviorHints.defaultVideoId`, as
        // Stremio reads it): a TMDB-keyed movie (`tmdb:157336`) opened on its own id at once, and
        // AIOStreams' scrapers answer only for the IMDb id the hint names
        let default_video = match self.meta.as_ref().and_then(|k| store.meta(k)) {
            Some(MetaState::Ready(m)) => m.default_video_id().map(str::to_string),
            _ => None,
        };
        if let (Some(MetaState::Ready(m)), Some(asked)) = (self.meta.as_ref().and_then(|k| store.meta(k)), self.streams.as_ref()) {
            let kind = match &self.mode {
                Mode::Item(i) => Some(i.kind.as_str()),
                Mode::Video(v) => Some(v.kind.as_str()),
                Mode::Resume(_) => None,
            };
            if let Some(better) = kind.and_then(|k| stream_video_for(k, &asked.video_id, m)) {
                if !self.closed {
                    let key = StreamKey { kind: asked.kind.clone(), video_id: better };
                    fx.push(Fx::App(AppFx::Detail(DetailCmd::OpenStreams(key.clone()))));
                    self.streams = Some(key);
                    self.built_gen = None;
                }
            }
        }
        // a series whose meta lists no video plays its default video, else its own id
        let no_videos = matches!(self.meta.as_ref().and_then(|k| store.meta(k)), Some(MetaState::Ready(m)) if m.videos.is_empty());
        if no_videos && self.streams.is_none() && !self.closed {
            if let Mode::Item(item) = &self.mode {
                let key = StreamKey { kind: item.kind.clone(), video_id: default_video.unwrap_or_else(|| item.id.clone()) };
                fx.push(Fx::App(AppFx::Detail(DetailCmd::OpenStreams(key.clone()))));
                self.streams = Some(key);
            }
        }
        let gens = (store.gen(), cx.views.settings.gen(), cx.views.library.gen());
        if self.built_gen == Some(gens) {
            return;
        }
        self.built_gen = Some(gens);
        let meta = self.meta.as_ref().and_then(|k| store.meta(k));
        if let Some(MetaState::Ready(m)) = meta {
            if !matches!(self.mode, Mode::Resume(_)) {
                self.info = info(&self.mode, m);
                if let Some(t) = &self.info.title {
                    self.title = t.clone();
                }
                self.relayout();
            }
            let seasons = seasons(m);
            if seasons != self.seasons {
                self.season_labels = seasons.iter().map(|s| CString::new(season_label(*s).replace('\0', "")).unwrap_or_default()).collect();
                self.season_widths.replace(None);
                self.seasons = seasons;
            }
            if self.season.is_none_or(|s| !self.seasons.contains(&s)) {
                // the episode an overview was opened from, else the one to watch next (after the
                // library item's last watched, whichever id the item is filed under), focused
                let watched = self.watched_marks(cx, m);
                let next = crate::episodes::opening_episode(m, &watched, &crate::episodes::now()).map(|v| v.id.clone());
                let open = self.from_video.clone().filter(|id| m.videos.iter().any(|v| v.id == *id)).or(next);
                if self.from_video.is_none() && self.sel.is_none() {
                    self.focus_episode = open.as_deref().and_then(|id| m.videos.iter().position(|v| v.id == id)).map(|i| EPISODE_KEY + (i as u32).min(0x00FF_FFFF));
                }
                self.season = default_season(m, open.as_deref());
            }
        }
        let streams = self.streams.as_ref().and_then(|k| store.streams(k));
        let prefer_owned = cx.views.settings.stream_languages();
        let prefer: Vec<&str> = prefer_owned.iter().map(String::as_str).collect();
        let marks = match meta {
            Some(MetaState::Ready(m)) => Some(self.watched_marks(cx, m)),
            _ => None,
        };
        let view = StreamView { order: cx.views.settings.get(crate::stores::settings::Pref::StreamOrder), ..self.stream_view.clone() };
        let built = build_marked(&self.mode, meta, streams, self.season, &today(), &prefer, marks.as_ref(), &view);
        let moved = std::mem::take(&mut self.season_moved);
        let mut want = self.sel.or_else(|| self.focus_episode.map(RowKey));
        // until the viewer moves, a stream list that grows re-seats on its first row: a slower
        // addon landing above must not leave focus on a row it pushed down the list
        let first_stream = built.keys.iter().zip(&built.actions).find(|(k, a)| a.is_some() && (STREAM_KEY..EPISODE_KEY).contains(&k.0)).map(|(k, _)| *k);
        let on_list = cx.focus.current.is_some_and(|k| k.entry == self.entry && self.built.index_of(RowKey(k.elem)).is_some());
        let reseat = first_stream.filter(|f| !self.moved && on_list && self.focus_episode.is_none() && want != Some(*f));
        if let Some(f) = reseat {
            want = Some(f);
            self.sel = Some(f);
            crate::registry::reseat(sn_ui::screen::FocusTarget::Elem(FocusKey { entry: self.entry, elem: f.0 }), fx);
        }
        let sel = if moved { None } else { want.and_then(|k| built.index_of(k)).map(|i| i as i32) };
        if moved {
            self.sel = None;
        }
        let fresh = self.table.sections.is_empty();
        self.built = built;
        let sections = std::mem::take(&mut self.built.sections);
        match sel {
            // the viewer's row keeps its place on the screen while the list grows around it
            Some(i) if !fresh && !moved && reseat.is_none() => self.table.set_sections_anchored(sections, i),
            _ => self.table.set_sections_or_open(sections, sel, !fresh && !moved),
        }
        fx.invalidate(Provenance::Landing(fx.from()));
    }
}

impl LogicalState for DetailScreen {
    fn write(&self, c: &mut Canon) {
        c.u32(self.entry.0);
        c.option(self.sel, |c, k| {
            c.u32(k.0);
        });
        c.bool(self.closed);
        c.u32(self.season.unwrap_or(u32::MAX));
        c.u32(self.expanded.map_or(u32::MAX, |e| e.0 as u32));
        self.table.write_motion(c);
    }
    fn probe(&self, out: &mut String) {
        out.push_str(&format!("detail rows={} sel={:?}", self.table.n_rows(), self.sel));
    }
}

impl Machine<Bundle> for DetailScreen {
    type Ev = ScreenEvent<Bundle>;
    fn step(&mut self, ev: &Self::Ev, cx: &Cx<'_, Bundle>, fx: &mut Effects<'_, Bundle>) -> Handled {
        match ev {
            ScreenEvent::Mount => {
                self.open(fx);
                self.sync(cx, fx);
                self.sync_library(cx, fx);
                Handled::Yes
            }
            ScreenEvent::StoreChanged(ord, _) if *ord == StoreId::Detail.ord() => {
                self.sync(cx, fx);
                self.sync_chips(cx, fx);
                self.sync_addons(cx, fx);
                Handled::Yes
            }
            ScreenEvent::Tick(t) => {
                self.sync(cx, fx);
                self.sync_library(cx, fx);
                self.sync_chips(cx, fx);
                self.sync_addons(cx, fx);
                let selectable = self.built.actions.iter().any(Option::is_some);
                // the episode the page opened on, once its row is in; else the list's first
                let focus = self.focus_episode.filter(|k| self.built.index_of(RowKey(*k)).is_some());
                let target = match focus {
                    Some(k) => Some(sn_ui::screen::FocusTarget::Elem(FocusKey { entry: self.entry, elem: k })),
                    None => selectable.then_some(sn_ui::screen::FocusTarget::FirstInGroup(GroupId(0))),
                };
                let seated = cx.focus.current.is_some_and(|k| k.entry == self.entry && (self.moved || k.elem < SEASON_KEY || k.elem >= NOTE_KEY));
                if seated {
                    self.focus_episode = None;
                }
                self.seat.tick(seated, target, fx);
                self.table.update(t.dt(), self.table_frame().h);
                for (spring, elem) in self.action_open.iter_mut().zip([LIBRARY_KEY, WATCHED_KEY]) {
                    let on = cx.focus.current == Some(FocusKey { entry: self.entry, elem });
                    let before = spring.pos;
                    spring.step(if on { 1.0 } else { 0.0 }, 300.0, t.dt());
                    if spring.pos != before {
                        fx.invalidate(Provenance::Landing(fx.from()));
                    }
                }
                Handled::Yes
            }
            ScreenEvent::FocusMoved { to, by, .. } => {
                self.moved |= matches!(by, sn_machine::machine::By::Dir | sn_machine::machine::By::Pointer);
                // the season dropdown / All episodes holds focus: no row draws as focused under it
                if to.entry == self.entry {
                    self.table.list_focused = self.built.index_of(RowKey(to.elem)).is_some();
                }
                if let Some(i) = self.built.index_of(RowKey(to.elem)) {
                    self.table.sel = i as i32;
                    self.sel = Some(RowKey(to.elem));
                }
                let link = link_slot(to.elem).filter(|_| to.entry == self.entry);
                match (link, self.expanded) {
                    (Some((g, MORE_SLOT)), _) => self.expand(g, fx),
                    // focus left the expanded group: it folds back to its lines
                    (l, Some((e, _))) if l.is_none_or(|(g, _)| g != e) && to.entry == self.entry => {
                        self.expanded = None;
                        self.relayout();
                    }
                    _ => {}
                }
                fx.invalidate(Provenance::Input);
                Handled::Yes
            }
            // a link chip pushes its page (BACK returns here, focus on the chip); a click on `+N`
            // expands its group
            ScreenEvent::PressCommit(_) | ScreenEvent::Activate(_) if self.link_of(ev, cx).is_some() => {
                match self.link_of(ev, cx) {
                    Some((g, MORE_SLOT)) => self.expand(g, fx),
                    Some((g, n)) => {
                        if let Some(chip) = self.info.links.get(g).and_then(|l| l.chips.get(n as usize)) {
                            fx.push(Fx::Nav(NavOp::Push(chip.to.clone())));
                        }
                    }
                    None => {}
                }
                fx.invalidate(Provenance::Input);
                Handled::Yes
            }
            ScreenEvent::PressCommit(_) | ScreenEvent::Activate(_) if self.library_pressed(ev, cx) => {
                self.press_library(cx, fx);
                fx.invalidate(Provenance::Input);
                Handled::Yes
            }
            ScreenEvent::PressCommit(_) | ScreenEvent::Activate(_) if self.watched_pressed(ev, cx) => {
                self.press_mark(sn_stremio::library::Edit::MarkWatched(self.watched != Some(true)), cx, fx);
                fx.invalidate(Provenance::Input);
                Handled::Yes
            }
            // a chip is a control: OK commits it as a press, a click activates it
            ScreenEvent::PressCommit(_) | ScreenEvent::Activate(_) if self.chip_of(ev, cx).is_some() => {
                match self.chip_of(ev, cx) {
                    Some(Chip::Seasons) => self.open_seasons(cx, fx),
                    Some(Chip::Overview) => self.open_overview(fx),
                    Some(chip @ (Chip::MarkSeason(watched) | Chip::MarkEpisode(watched))) => {
                        let target = match (chip, &self.mode) {
                            (Chip::MarkEpisode(_), Mode::Video(v)) => Some(sn_stremio::library::MarkTarget::Video(v.video_id.clone())),
                            (Chip::MarkSeason(_), _) => self.season.map(sn_stremio::library::MarkTarget::Season),
                            _ => None,
                        };
                        if let (Some(target), Some(m)) = (target, self.ready_meta(cx)) {
                            let mark = sn_stremio::library::VideoMark { videos: m.videos.clone(), target, watched };
                            self.press_mark(sn_stremio::library::Edit::MarkVideos(mark), cx, fx);
                        }
                    }
                    None => {}
                }
                fx.invalidate(Provenance::Input);
                Handled::Yes
            }
            ScreenEvent::PressCommit(_) | ScreenEvent::Activate(_) if self.addons_pressed(ev, cx) => {
                self.open_addons(cx, fx);
                fx.invalidate(Provenance::Input);
                Handled::Yes
            }
            ScreenEvent::App(AppMsg::Chose(ChoiceTarget::DetailStreamAddons, n)) => {
                // the popover stays open and re-reads its checks (`dropdown::live`); the store
                // drops the addon's answer, or asks it again
                if let Some(id) = crate::dropdown::stream_addon_toggles(self.stream_groups(cx), *n) {
                    fx.push(Fx::App(AppFx::Addons(crate::registry::AddonsCmd::ToggleStreams(id))));
                }
                fx.invalidate(Provenance::Input);
                Handled::Yes
            }
            ScreenEvent::App(AppMsg::Chose(ChoiceTarget::DetailSeason, n)) => {
                if let Some(season) = self.seasons.get(*n).copied() {
                    self.choose_season(season, cx, fx);
                }
                fx.invalidate(Provenance::Input);
                Handled::Yes
            }
            ScreenEvent::Activate(elem) => {
                match self.action_of(*elem).cloned() {
                    Some(Action::Episode(video)) => fx.push(Fx::Nav(NavOp::Push(AppArg::Streams(video)))),
                    Some(Action::Play(mut req)) => {
                        // a stream picked here resumes a saved position, as Continue Watching does
                        if let (0, Some(it)) = (req.start_ms, &req.item) {
                            let saved = cx.views.library.items().iter().find(|i| i.id == it.id);
                            req.start_ms = saved.map_or(0, |i| i.saved_offset(&it.video_id));
                        }
                        fx.push(Fx::App(AppFx::Play(req)))
                    }
                    Some(Action::Explain(why)) => fx.push(Fx::App(AppFx::Toast(why.to_string()))),
                    Some(Action::ToggleQuiet) => {
                        self.stream_view.quiet_open = !self.stream_view.quiet_open;
                        self.built_gen = None;
                        self.sync(cx, fx);
                    }
                    None => {}
                }
                Handled::Yes
            }
            ScreenEvent::WillLeave(Leave::ForGood) | ScreenEvent::Unmount => {
                self.close(fx);
                Handled::Yes
            }
            ScreenEvent::Enter(_) | ScreenEvent::Uncover => {
                fx.invalidate(Provenance::Nav);
                Handled::Yes
            }
            _ => Handled::No,
        }
    }
}

sn_ui::focusable_via_view!(DetailScreen, Bundle, view);

impl Screen<Bundle> for DetailScreen {
    fn name(&self) -> &'static str {
        match self.mode {
            Mode::Item(_) => word::DETAIL,
            Mode::Video(_) => word::STREAMS,
            Mode::Resume(_) => word::RESUME,
        }
    }
    fn state(&self) -> &dyn LogicalState {
        self
    }
    fn crumb(&self, _cx: &Cx<'_, Bundle>) -> Option<Cow<'_, str>> {
        Some(Cow::Borrowed(&self.crumb))
    }
    /// The narrative column's vertical path: the link groups top-down, then the library button;
    /// RIGHT off a chip is the list; LEFT off the list or the season row is the library button (the
    /// chips sit above it).
    fn links(&self, out: &mut Vec<Link>) {
        let shown: Vec<usize> = match &*self.link_places.borrow() {
            Some(p) => p.iter().map(|g| g.group).collect(),
            None => (0..self.info.links.len()).collect(),
        };
        for g in &shown {
            out.push(Link { from: link_group(*g), dir: Dir::Right, to: GroupId(0) });
        }
        for w in shown.windows(2) {
            out.push(Link { from: link_group(w[0]), dir: Dir::Down, to: link_group(w[1]) });
            out.push(Link { from: link_group(w[1]), dir: Dir::Up, to: link_group(w[0]) });
        }
        if self.library.is_some() {
            out.push(Link { from: GroupId(0), dir: Dir::Left, to: LIBRARY_GROUP });
            out.push(Link { from: SEASON_GROUP, dir: Dir::Left, to: LIBRARY_GROUP });
            if let Some(&last) = shown.last() {
                out.push(Link { from: link_group(last), dir: Dir::Down, to: LIBRARY_GROUP });
                out.push(Link { from: LIBRARY_GROUP, dir: Dir::Up, to: link_group(last) });
            }
        }
    }
    fn prepare(&mut self, _b: &mut sn_ui::frame::Budget, _cx: &Cx<'_, Bundle>) {}
    fn draw(&mut self, f: &mut DrawFrame<'_, '_, Bundle>) {
        sn_gfx::gfx::frame_clear(theme::CLEAR_RGB.0, theme::CLEAR_RGB.1, theme::CLEAR_RGB.2);
        if let Some(url) = self.info.backdrop.as_deref() {
            draw_backdrop(f.painter, url);
        }
        draw_stream_panel(f.painter);
        let mut v = self.view();
        Part::<Bundle>::draw(&mut v, f, Rect::FULL);
        if let Some(url) = self.info.logo.as_deref() {
            sn_ui::hero_logo::HeroLogo::url(0, url, &self.title, sn_ui::hero_logo::LogoRung::Hero).draw(f.painter, logo_band(), f.measure);
        }
        for b in &flow(&self.info, &self.title, f.measure, self.reserve(), self.expanded.map(|e| e.0)).blocks {
            b.view(f.measure).draw(f.painter, b.rect);
        }
    }
    fn render(&self) -> RenderStrategy {
        RenderStrategy::Page
    }
    fn focus_source(&self) -> FocusSource {
        FocusSource::Engine
    }
    fn hit_source(&self) -> HitSource {
        HitSource::Engine
    }
    fn as_any(&self) -> Option<&dyn std::any::Any> {
        Some(self)
    }
    fn as_any_mut(&mut self) -> Option<&mut dyn std::any::Any> {
        Some(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sn_stremio::types::Video;
    use sn_stremio::types::{MetaPreview, ProxyHeaders, Stream, StreamHints};
    use sn_stremio::FetchError;

    fn movie() -> Mode {
        Mode::Item(ItemRef { kind: "movie".into(), id: "tt1".into(), name: "Film".into(), from: String::new() })
    }

    fn group(name: &str, state: GroupState) -> StreamGroup {
        StreamGroup { addon_id: name.to_lowercase(), addon_name: name.into(), url: String::new(), state }
    }

    fn http(name: &str, title: &str, url: &str) -> Stream {
        Stream { url: Some(url.into()), name: Some(name.into()), title: Some(title.into()), ..Default::default() }
    }

    fn texts(b: &Built) -> Vec<(String, Vec<String>)> {
        b.sections.iter().map(|s| (s.header.clone(), s.rows.iter().map(|r| r.label.clone()).collect())).collect()
    }

    #[test]
    fn every_addon_shows_its_own_state() {
        let _en = sn_platform::i18n::language_on_this_thread_for_test(sn_platform::i18n::Preference::En);
        // a failed and an empty addon sink below the ones with streams or still asking
        let groups = vec![
            group("Fast", GroupState::Ready(vec![http("Fast\n4K", "Film 2160p\n👤 12", "https://cdn.example/a.mkv")])),
            group("Broken", GroupState::Failed(FetchError::Status(503))),
            group("Slow", GroupState::Loading),
            group("Empty", GroupState::Ready(Vec::new())),
        ];
        let b = build(&movie(), None, Some(&groups), None, "");
        // only the addons WITH streams are sections; the rest is one closing section: the addons
        // still asked, and one row standing for those that found nothing
        assert_eq!(
            texts(&b),
            vec![
                ("Fast".into(), vec!["Film 2160p".into()]),
                (String::new(), vec![msg::app_detail_still_asking(1), msg::app_detail_quiet_addons(2)]),
            ]
        );
        let row = &b.sections[0].rows[0];
        assert_eq!(row.side, Some(("Fast\n4K".into(), String::new())), "the stream's own name, which says its quality");
        assert_eq!(row.chips.iter().map(|c| c.text.as_str()).collect::<Vec<_>>(), ["12 seeders"]);
        assert_eq!(b.sections[0].accessory, "1", "the header counts the addon's streams");
        assert!(b.sections[1].rows[0].is_note(), "the wait is a note, never a focusable row");
        assert_eq!(b.actions[2], Some(Action::ToggleQuiet), "the closing row opens");
        // opened, it names each addon and why
        let open = StreamView { quiet_open: true, ..StreamView::default() };
        let b = build_marked(&movie(), None, Some(&groups), None, "", &[], None, &open);
        assert_eq!(
            texts(&b)[1].1[2..],
            [format!("Broken \u{b7} HTTP 503"), format!("Empty \u{b7} {}", msg::app_detail_no_streams())]
        );
        // an addon switched off (unchecked in the addon dropdown) is neither listed nor counted
        let mut off = groups.clone();
        off[0].state = GroupState::Off;
        off[1].state = GroupState::Off;
        let b = build(&movie(), None, Some(&off), None, "");
        assert_eq!(texts(&b), vec![(msg::app_detail_streams().into(), vec![msg::app_detail_still_asking(1), msg::app_detail_quiet_addons(1)])]);
    }

    /// Nothing anywhere: the reasons are all there is to read, so the closing row stands open.
    #[test]
    fn with_no_stream_anywhere_the_reasons_show_at_once() {
        let _en = sn_platform::i18n::language_on_this_thread_for_test(sn_platform::i18n::Preference::En);
        let groups = vec![group("Broken", GroupState::Failed(FetchError::Status(503))), group("Empty", GroupState::Ready(Vec::new()))];
        let b = build(&movie(), None, Some(&groups), None, "");
        assert_eq!(texts(&b)[0].1.len(), 3, "the row and both reasons: {:?}", texts(&b));
    }

    /// An AIOStreams-over-TorBox list as it arrives: an uncached German 4K first, a cached English
    /// 1080p, a cached German 720p, a cached multi-audio release, a plain row.
    fn tb_streams() -> Vec<Stream> {
        vec![
            http("[TB download] AIOStreams\n4K", "Film.2024.German.2160p.WEB-DL.HEVC\n💾 20 GB", "https://example.com/1"),
            http("[TB+] AIOStreams\n1080p", "Film.2024.1080p.BluRay.x264\n🇬🇧", "https://example.com/2"),
            http("[TB+] AIOStreams\n720p", "Film.2024.GERMAN.720p.WEB.x264", "https://example.com/3"),
            http("[TB+] AIOStreams\n1080p", "Film.2024.MULTi.1080p.WEB-DL", "https://example.com/4"),
            http("[TB+] AIOStreams\n480p", "Film.2024.480p", "https://example.com/5"),
        ]
    }

    #[test]
    fn the_stream_order_reads_the_settings_language_list() {
        use crate::stores::settings::{LangList, LangOp, SettingsStore};
        let facts: Vec<Facts> = tb_streams().iter().map(stream_facts::facts).collect();
        let order = |s: &SettingsStore| {
            let l = s.stream_languages();
            stream_order(&facts, &l.iter().map(String::as_str).collect::<Vec<_>>(), crate::stores::settings::ORDER_LANGUAGE)
        };
        let mut s = SettingsStore::default();
        s.edit(LangList::Audio, LangOp::Add("en"));
        assert_eq!(order(&s), [1, 3, 2, 4, 0], "following the audio list: English first");
        s.edit(LangList::Streams, LangOp::Add("de"));
        s.edit(LangList::Streams, LangOp::Up(1));
        assert_eq!(s.stream_languages(), ["DE", "EN"]);
        assert_eq!(order(&s), [2, 1, 3, 4, 0], "its own order: German, then English");
    }

    #[test]
    fn cached_streams_come_first_then_the_preferred_languages_in_order() {
        let facts: Vec<Facts> = tb_streams().iter().map(stream_facts::facts).collect();
        use crate::stores::settings::{ORDER_ADDON, ORDER_LANGUAGE as LANG, ORDER_QUALITY};
        assert_eq!(stream_order(&facts, &["DE", "EN"], LANG), [2, 1, 3, 4, 0], "cached German, English, multi, the rest; the download last");
        assert_eq!(stream_order(&facts, &["EN"], LANG), [1, 3, 2, 4, 0]);
        assert_eq!(stream_order(&facts, &[], LANG), [3, 1, 2, 4, 0], "no preference: multi-audio first, else the addon's order");
        assert_eq!(stream_order(&facts, &["DE", "EN"], ORDER_ADDON), [0, 1, 2, 3, 4], "the addon's order, untouched");
        let res: Vec<_> = facts.iter().map(|f| f.resolution).collect();
        let by_quality = stream_order(&facts, &[], ORDER_QUALITY);
        assert_eq!(by_quality.last(), Some(&0), "the download still last: {res:?}");
        let ranks: Vec<_> = by_quality[..4].iter().map(|i| ["4K", "1440p", "1080p", "720p", "576p", "480p"].iter().position(|r| Some(*r) == res[*i]).unwrap_or(6)).collect();
        assert!(ranks.windows(2).all(|w| w[0] <= w[1]), "highest resolution first: {ranks:?} {res:?}");
        let b = build_preferring(&movie(), None, Some(&[group("AIO", GroupState::Ready(tb_streams()))]), None, "", &["DE", "EN"]);
        let last = b.sections[0].rows.last().unwrap();
        assert_eq!(last.chips[0].text, msg::app_detail_stream_uncached(), "the download says so, first");
        assert!(last.chips[0].quiet, "…in the quiet ink");
        assert!(b.actions[4].is_some(), "an uncached stream still plays");
        assert_eq!(b.keys[0], RowKey(STREAM_KEY + 2), "a row keeps the key of its place in the addon's answer");
    }

    /// Every piece of text a stream row draws: the side column, the title, the chips.
    fn row_texts(r: &Row) -> Vec<String> {
        let (src, tag) = r.side.clone().unwrap_or_default();
        [src, tag, r.label.clone()].into_iter().chain(r.chips.iter().map(|c| c.text.clone())).collect()
    }

    /// No row reads "none"/"null", and no chip is empty.
    fn assert_no_placeholder_text(b: &Built) {
        for r in b.sections.iter().flat_map(|s| &s.rows) {
            for t in row_texts(r) {
                let l = t.to_lowercase();
                assert!(!l.contains("none") && !l.contains("null") && !l.contains("undefined"), "{t:?} in {:?}", row_texts(r));
            }
            assert!(r.chips.iter().all(|c| !c.text.trim().is_empty()), "an empty chip in {:?}", row_texts(r));
        }
    }

    /// TvVoo's two streams of one channel as the addon sends them: they differ only in their
    /// `name` (and the emoji in their title), so the name is what the side column must say.
    #[test]
    fn tvvoo_s_proxy_and_direct_streams_read_apart() {
        let _en = sn_platform::i18n::language_on_this_thread_for_test(sn_platform::i18n::Preference::En);
        let parse = |j: &str| serde_json::from_str::<Stream>(j).unwrap();
        let proxy = parse(r#"{"name":"Proxy","title":"[🛰️] SKY 1","url":"https://example.com/p.m3u8","behaviorHints":{}}"#);
        let direct = parse(r#"{"name":"Vavoo","title":"[🏠] SKY 1","url":"https://example.com/v.m3u8","behaviorHints":{"isLive":true}}"#);
        let b = build(&movie(), None, Some(&[group("TvVoo", GroupState::Ready(vec![proxy, direct]))]), None, "");
        assert_eq!(b.sections[0].header, "TvVoo", "the group header stays the addon");
        let rows = &b.sections[0].rows;
        assert_eq!(rows[0].side, Some(("Proxy".into(), String::new())), "the stream's own name, no quality to tell");
        assert_eq!(rows[1].side, Some(("Vavoo".into(), String::new())));
        assert_eq!((rows[0].label.as_str(), rows[1].label.as_str()), ("[🛰️] SKY 1", "[🏠] SKY 1"), "the emoji stay");
        assert!(rows[0].chips.iter().all(|c| c.text != msg::app_detail_stream_live()), "the proxy is not marked live");
        assert_eq!(rows[1].chips.first().map(|c| c.text.as_str()), Some(msg::app_detail_stream_live()));
        assert_ne!(row_texts(&rows[0]), row_texts(&rows[1]));
        assert_no_placeholder_text(&b);
        // a live-TV item's streams are live whatever their hints say
        let tv = Mode::Item(ItemRef { kind: "tv".into(), id: "ch1".into(), name: "SKY 1".into(), from: String::new() });
        let b = build(&tv, None, Some(&[group("TvVoo", GroupState::Ready(vec![http("Proxy", "[🛰️] SKY 1", "https://example.com/p")]))]), None, "");
        assert_eq!(b.sections[0].rows[0].chips[0].text, msg::app_detail_stream_live());
    }

    /// AIOStreams' report of a timed-out upstream is a muted note after the addon's streams, not a
    /// row and not a stream the header counts; an addon that sent nothing else sinks, dimmed.
    #[test]
    fn an_aiostreams_error_stream_is_a_note_after_the_streams() {
        let _en = sn_platform::i18n::language_on_this_thread_for_test(sn_platform::i18n::Preference::En);
        let parse = |j: &str| serde_json::from_str::<Stream>(j).unwrap();
        let error = || parse(r#"{"name":"[❌] Comet p2p","description":"The operation was aborted due to timeout","externalUrl":"https://example.com/repo","streamData":{"type":"error"}}"#);
        let real = http("[TB\u{26a1}] Comet\n1080p", "Film.2024.1080p.WEB-DL", "https://example.com/1");
        let groups = [group("Failing", GroupState::Ready(vec![error()])), group("AIOStreams", GroupState::Ready(vec![error(), real]))];
        let b = build(&movie(), None, Some(&groups), None, "");
        let aio = &b.sections[0];
        assert_eq!(aio.header, "AIOStreams", "the addon with a real stream first");
        assert_eq!(aio.accessory, "1", "the error is no stream");
        assert_eq!(aio.rows[0].label, "Film.2024.1080p.WEB-DL");
        assert!(aio.rows[1].is_note() && aio.rows[1].dim, "a muted note, after the streams");
        assert_eq!(aio.rows[1].label, "Comet p2p: The operation was aborted due to timeout");
        assert_eq!(b.actions.iter().filter(|a| matches!(a, Some(Action::Play(_)))).count(), 1, "the note plays nothing");
        // an addon of errors alone is no section: it is counted on the closing row, and its first
        // error is its reason there
        assert_eq!(texts(&b)[1].1, [msg::app_detail_quiet_addons(1)]);
        let open = StreamView { quiet_open: true, ..StreamView::default() };
        let b = build_marked(&movie(), None, Some(&groups), None, "", &[], None, &open);
        assert_eq!(texts(&b)[1].1[1], "Failing \u{b7} Comet p2p: The operation was aborted due to timeout");
    }

    /// AIOStreams' scrape summaries (`streamData.type` "statistic", sent before the streams when
    /// its position is "top") are muted notes too: after the streams AND after the failures,
    /// capped at two lines, not counted, playing nothing. An addon of notes alone sinks, dimmed.
    #[test]
    fn aiostreams_statistic_streams_are_notes_after_the_failures() {
        let _en = sn_platform::i18n::language_on_this_thread_for_test(sn_platform::i18n::Preference::En);
        let parse = |j: &str| serde_json::from_str::<Stream>(j).unwrap();
        let stat = |up: &str| {
            parse(&format!(
                r#"{{"name":"🟢 [{up}] Scrape Summary","description":"✔ Status      : SUCCESS\n📦 Streams    : 1\n📋 Details    : Successfully fetched streams.\n⏱️ Time       : 3.2s\n","externalUrl":"https://example.com/repo","streamData":{{"type":"statistic"}}}}"#
            ))
        };
        let error = parse(r#"{"name":"[❌] Comet p2p","description":"No results","externalUrl":"https://example.com/repo","streamData":{"type":"error"}}"#);
        let a = http("[TB\u{26a1}] Comet\n1080p", "Film.2024.1080p.WEB-DL", "https://example.com/1");
        let b2 = http("[RD+] Torrentio\n4k", "Film.2024.2160p.BluRay", "https://example.com/2");
        let groups = [
            group("Summaries", GroupState::Ready(vec![stat("Comet")])),
            group("AIOStreams", GroupState::Ready(vec![stat("Comet"), stat("Torrentio"), a, error, b2])),
        ];
        let b = build(&movie(), None, Some(&groups), None, "");
        let aio = &b.sections[0];
        assert_eq!(aio.header, "AIOStreams", "an addon of summaries alone sinks below");
        assert_eq!(aio.accessory, "2", "neither summaries nor the failure count");
        let labels: Vec<&str> = aio.rows.iter().map(|r| r.label.as_str()).collect();
        let summary = |up: &str| format!("🟢 [{up}] Scrape Summary\n✔ Status : SUCCESS · 📦 Streams : 1 · 📋 Details : Successfully fetched streams. · ⏱️ Time : 3.2s");
        assert_eq!(labels, ["Film.2024.1080p.WEB-DL", "Film.2024.2160p.BluRay", "Comet p2p: No results", &summary("Comet"), &summary("Torrentio")]);
        assert!(aio.rows[2..].iter().all(|r| r.is_note() && r.dim), "muted notes, never focusable");
        assert_eq!(aio.rows[2].note_max, 0, "a failure reads whole");
        assert!(aio.rows[3..].iter().all(|r| r.note_max == STAT_NOTE_LINES), "a summary is capped");
        assert_eq!(b.actions.iter().filter(|a| matches!(a, Some(Action::Play(_)))).count(), 2, "the notes play nothing");
        assert_eq!(texts(&b)[1].1, [msg::app_detail_quiet_addons(1)], "summaries alone: no section, counted on the closing row");
        // laid out, a summary takes its two lines, elided, however long its text
        let mut t = TableView::new();
        t.set_sections(b.sections, 0, false);
        t.fit_notes(620.0, &sn_base::fontcov::advances::ShippedMeasure);
        assert_eq!(t.sections[0].rows[3].note_lines.get(), 2);
    }

    /// AIOStreams/Comet and Torrentio keep their title and chips; the side column now reads the
    /// stream's own name, and the quality tag only when the name does not already say it.
    #[test]
    fn aiostreams_and_torrentio_rows_read_their_own_names() {
        let _en = sn_platform::i18n::language_on_this_thread_for_test(sn_platform::i18n::Preference::En);
        let mut comet = http("[TB\u{26a1}] Comet\n1080p", "🎞️ HEVC 🏷️ GRP\n🎧 DD 🔊 5.1\n📦 4.2 GB", "https://example.com/c");
        comet.behavior_hints.filename = Some("Film.2024.1080p.WEB-DL.HEVC.DD5.1-GRP.mkv".into());
        let torrentio = http("[RD+] Torrentio\n4k", "Film.2024.2160p.BluRay.REMUX.HEVC-GRP\n👤 12 💾 40 GB", "https://example.com/t");
        let bare = http("", "Film.2024.720p.WEB", "https://example.com/b");
        let b = build(&movie(), None, Some(&[group("AIOStreams", GroupState::Ready(vec![comet, torrentio, bare]))]), None, "");
        let rows = &b.sections[0].rows;
        assert_eq!(rows[0].label, "Film.2024.1080p.WEB-DL.HEVC.DD5.1-GRP.mkv", "the filename hint is the release");
        assert_eq!(rows[0].side, Some(("[TB\u{26a1}] Comet\n1080p".into(), String::new())), "the name carries its quality");
        let chips: Vec<&str> = rows[0].chips.iter().map(|c| c.text.as_str()).collect();
        assert_eq!(chips[..4], ["Cached", "HEVC", "DD 5.1", "WEB-DL \u{b7} GRP"], "the source carries the release group");
        assert_eq!(rows[1].label, "Film.2024.2160p.BluRay.REMUX.HEVC-GRP");
        assert_eq!(rows[1].side, Some(("[RD+] Torrentio\n4k".into(), String::new())));
        assert_eq!(rows[2].side, Some(("AIOStreams".into(), "720p".into())), "no name: the addon, and the quality it does not say");
        assert_no_placeholder_text(&b);
    }

    #[test]
    fn a_stream_row_reads_as_chips() {
        let _en = sn_platform::i18n::language_on_this_thread_for_test(sn_platform::i18n::Preference::En);
        let f = stream_facts::facts(&http(
            "[RD+] Torrentio\n4k DV",
            "Film.2023.2160p.BluRay.REMUX.DV.HEVC.TrueHD.Atmos.7.1-GRP\n👤 42 💾 61 GB ⚙️ TPB\n🇬🇧 / 🇩🇪 / 🇫🇷 / 🇮🇹",
            "https://example.com/r",
        ));
        let chips: Vec<String> = stream_chips(&f).into_iter().map(|c| c.text).collect();
        assert_eq!(
            chips,
            ["Cached", "DV", "HEVC", "TrueHD Atmos 7.1", "REMUX \u{b7} GRP", &sn_ui::fmt::bytes(61 << 30, (1, 0, 0)), "EN \u{b7} DE \u{b7} FR +1", "42 seeders"]
        );
        use sn_ui::icons::Icon;
        let icons: Vec<Option<Icon>> = stream_chips(&f).into_iter().map(|c| c.icon).collect();
        assert_eq!(
            icons,
            [Some(Icon::Bolt), Some(Icon::Film), None, Some(Icon::Audio), Some(Icon::Shield), Some(Icon::Drive), Some(Icon::Globe), Some(Icon::User)],
            "every fact leads with its icon (the codec rides on the picture's)"
        );
        let facts_chips = stream_chips(&f);
        assert_eq!(facts_chips[1].evidence, Some(sn_ui::table::ChipEvidence::AddonText));
        assert_eq!(facts_chips[5].evidence, Some(sn_ui::table::ChipEvidence::AddonText));
    }

    #[test]
    fn a_structured_size_hint_is_marked_as_an_addon_hint() {
        let mut s = http("Example", "Watch now", "https://example.com/hint");
        s.behavior_hints.video_size = Some(2_000_000_000);
        let facts = stream_facts::facts(&s);
        let chips = stream_chips(&facts);
        assert_eq!(chips.len(), 1);
        assert_eq!(chips[0].evidence, Some(sn_ui::table::ChipEvidence::AddonHint));

    }

    /// The stream table fits at the page's width in every shipped language: the cache chips (app
    /// text) always show, a long release wraps to two lines, the rest may give way.
    #[test]
    fn the_stream_list_fits_in_every_shipped_language() {
        use sn_platform::i18n::{language_on_this_thread_for_test, SHIPPED};
        let m = sn_base::fontcov::advances::ShippedMeasure;
        let w = detail_layout().sectioned_table().w;
        let mut out = Vec::new();
        for language in SHIPPED {
            let _g = language_on_this_thread_for_test(language);
            let mut streams = tb_streams();
            streams.push(http(
                "[TB download] Torrentio\n4k",
                "A.Very.Long.Release.Name.That.Goes.On.And.On.2023.2160p.UHD.BluRay.REMUX.DV.HDR10.HEVC.TrueHD.Atmos.7.1-SOMEGROUP\n👤 1234 💾 80.2 GB\nMulti Audio / 🇬🇧 / 🇩🇪 / 🇫🇷 / 🇪🇸",
                "https://example.com/6",
            ));
            // both app chips on one line: a live stream the debrid service must first fetch
            let mut live = http("[TB download] Vavoo\n4k", "[🏠] A Channel With A Long Name HD", "https://example.com/7");
            live.behavior_hints.is_live = true;
            streams.push(live);
            // the closing section too: the wait, and the row standing for the quiet addons
            let groups = [
                group("AIOStreams", GroupState::Ready(streams)),
                group("Slow", GroupState::Loading),
                group("Broken", GroupState::Failed(FetchError::Status(503))),
                group("Empty", GroupState::Ready(Vec::new())),
            ];
            let b = build_preferring(&movie(), None, Some(&groups), None, "", &["EN"]);
            let mut t = TableView::new();
            t.set_sections(b.sections, 0, false);
            out.extend(t.app_fit_failures(w, language.tag()));
            t.fit_notes(w, &m);
            let long = t.sections[0].rows.iter().find(|r| r.label.starts_with("A.Very")).unwrap();
            assert_eq!(long.note_lines.get(), 2, "{}: the long release wraps to two lines", language.tag());
            let (shown, _) = sn_ui::table::chip_run(&long.chips, t.label_width(long, w, &m), &m);
            assert!(shown >= 2, "{}: the cache chip and at least one fact show", language.tag());
        }
        sn_ui::table::assert_no_fit_failures(&out);
    }

    #[test]
    fn an_http_stream_plays_with_its_request_headers_and_a_torrent_does_not() {
        let mut live = http("Live", "", "https://live.example/ch1.m3u8");
        live.behavior_hints = StreamHints {
            proxy_headers: Some(ProxyHeaders {
                request: [("Referer".to_string(), "https://live.example/".to_string())].into(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let torrent = Stream { info_hash: Some("ABC".into()), name: Some("Torrent".into()), ..Default::default() };
        let groups = vec![group("TV", GroupState::Ready(vec![live, torrent]))];
        let b = build(&movie(), None, Some(&groups), None, "");
        match &b.actions[0] {
            Some(Action::Play(req)) => {
                assert_eq!(req.url, "https://live.example/ch1.m3u8");
                assert_eq!(req.headers, vec![("Referer".to_string(), "https://live.example/".to_string())]);
                assert_eq!(req.title, "Film");
            }
            other => panic!("expected a play action, got {other:?}"),
        }
        assert_eq!(b.actions[1], Some(Action::Explain(msg::app_detail_why_torrent())), "a torrent needs a streaming server: OK says so");
        assert!(b.sections[0].rows[1].dim);
        assert_eq!(b.sections[0].rows[1].value.as_deref(), Some(msg::app_detail_not_playable()));
    }

    #[test]
    fn a_landing_above_the_focused_row_keeps_its_key() {
        let fast = group("Fast", GroupState::Ready(vec![http("A", "", "https://x/a"), http("B", "", "https://x/b")]));
        let before = build(&movie(), None, Some(&[group("Slow", GroupState::Loading), fast.clone()]), None, "");
        let focused = before.keys[1]; // "B" of the addon with streams
        let slow = group("Slow", GroupState::Ready(vec![http("C", "", "https://x/c"), http("D", "", "https://x/d")]));
        let after = build(&movie(), None, Some(&[slow, fast]), None, "");
        let at = after.index_of(focused).expect("the focused stream keeps its key");
        assert_eq!(after.sections[1].rows[1].label, "B");
        assert_eq!(at, 3, "and is found at its new index");
    }

    #[test]
    fn no_streaming_addon_says_so_rather_than_showing_nothing() {
        let b = build(&movie(), None, Some(&[]), None, "");
        assert_eq!(texts(&b), vec![(msg::app_detail_streams().into(), vec![msg::app_detail_no_stream_addons().into()])]);
    }

    fn ready(meta: Meta) -> MetaState {
        MetaState::Ready(Box::new(meta))
    }

    fn fixture(name: &str) -> Meta {
        let body = match name {
            "cinemeta_breaking_bad" => include_str!("../../stremio/src/fixtures/cinemeta_breaking_bad.json"),
            "cinemeta_wild_last_boss" => include_str!("../../stremio/src/fixtures/cinemeta_wild_last_boss.json"),
            "tmdb_wild_last_boss" => include_str!("../../stremio/src/fixtures/tmdb_wild_last_boss.json"),
            "kitsu_wild_last_boss_s2" => include_str!("../../stremio/src/fixtures/kitsu_wild_last_boss_s2.json"),
            "cinemeta_breaking_bad_meta" => include_str!("../../stremio/src/fixtures/cinemeta_breaking_bad_meta.json"),
            "cinemeta_shawshank" => include_str!("../../stremio/src/fixtures/cinemeta_shawshank.json"),
            _ => unreachable!("{name}"),
        };
        serde_json::from_str::<sn_stremio::types::MetaResponse>(body).unwrap().meta
    }

    fn details(b: &Built) -> Vec<(String, Vec<String>)> {
        b.sections.iter().map(|s| (s.header.clone(), s.rows.iter().map(|r| r.detail.clone()).collect())).collect()
    }

    #[test]
    fn a_series_lists_one_season_at_a_time_with_specials_last() {
        let item = ItemRef { kind: "series".into(), id: "tt0903747".into(), name: "Show".into(), from: String::new() };
        let v = |id: &str, s: u32, e: u32, t: &str| Video {
            id: id.into(),
            title: t.into(),
            season: Some(s),
            episode: Some(e),
            released: Some("2008-01-20T00:00:00.000Z".into()),
            ..Default::default()
        };
        let meta = Meta {
            preview: MetaPreview { id: "tt0903747".into(), kind: "series".into(), name: "Breaking".into(), ..Default::default() },
            videos: vec![v("x:0:1", 0, 1, "Special"), v("x:1:2", 1, 2, "Two"), v("x:1:1", 1, 1, "Pilot"), v("x:2:1", 2, 1, "")],
            ..Default::default()
        };
        assert_eq!(seasons(&meta), [1, 2, 0], "specials last");
        let loading = build(&Mode::Item(item.clone()), Some(&MetaState::Loading), None, None, "");
        assert_eq!(texts(&loading), vec![(msg::app_detail_episodes().into(), vec![msg::app_detail_episodes_loading().into()])]);
        let mode = Mode::Item(item.clone());
        let state = ready(meta);
        let b = build(&mode, Some(&state), None, None, "");
        assert_eq!(texts(&b), vec![(String::new(), vec!["Pilot".to_string(), "Two".to_string()])], "only season 1, which the dropdown names");
        assert_eq!(b.sections[0].rows[0].detail, format!("{} \u{b7} 1/20/2008", msg::core_episode(1)));
        match &b.actions[0] {
            Some(Action::Episode(video)) => {
                assert_eq!(video.video_id, "x:1:1");
                assert_eq!(video.show, "Breaking");
                assert_eq!(video.kind, "series");
            }
            other => panic!("expected an episode, got {other:?}"),
        }
        let s2 = build(&mode, Some(&state), None, Some(2), "");
        assert_eq!(texts(&s2), vec![(String::new(), vec![sn_ui::fmt::episode_ordinal(2, 1)])], "an untitled episode reads as its address");
        let sp = build(&mode, Some(&state), None, Some(0), "");
        assert_eq!(texts(&sp), vec![(String::new(), vec!["Special".to_string()])]);
        assert_eq!(texts(&build(&mode, Some(&state), None, Some(9), "")), texts(&b), "a season the meta lacks lists the first");
        // an episode keeps its key whichever season shows it
        assert_eq!(s2.keys[0], RowKey(EPISODE_KEY + 3));
        let failed = build(&Mode::Item(item), Some(&MetaState::Failed(FetchError::Status(404))), None, None, "");
        assert_eq!(failed.sections[0].rows[0].label, msg::app_detail_episodes_failed("HTTP 404"));
    }

    /// Real Cinemeta meta (trimmed, `sn_stremio`'s fixture): every season is offered and every
    /// row reads its own season and episode — not none (a video with both `episode` and `number`
    /// was dropped) and not swapped (the generated formatter takes `(episode, season)`).
    #[test]
    fn a_cinemeta_series_reads_its_real_seasons_and_episodes() {
        let _en = sn_platform::i18n::language_on_this_thread_for_test(sn_platform::i18n::Preference::En);
        let meta = fixture("cinemeta_breaking_bad");
        assert_eq!(seasons(&meta), [1, 2, 5, 0]);
        let mode = Mode::Item(ItemRef { kind: "series".into(), id: "tt0903747".into(), name: "Breaking Bad".into(), from: String::new() });
        let state = ready(meta);
        let got: Vec<_> = [1, 2, 5, 0].into_iter().flat_map(|s| details(&build(&mode, Some(&state), None, Some(s), ""))).collect();
        assert_eq!(
            got,
            [
                (String::new(), vec!["E1 \u{b7} 1/21/2008".to_string(), "E2 \u{b7} 1/28/2008".to_string()]),
                (String::new(), vec!["E1 \u{b7} 3/9/2009".to_string()]),
                (String::new(), vec!["E16 \u{b7} 9/30/2013".to_string()]),
                (String::new(), vec!["E1 \u{b7} 2/18/2009".to_string()]),
            ]
        );
    }

    /// An episode row wears its own still (decoded small by the table, `Row::still`) and its
    /// overview as a third line; one the addon sent no still for keeps the slot, empty, so the
    /// list's column stays straight.
    #[test]
    fn an_episode_row_carries_its_still_and_its_overview() {
        let mode = Mode::Item(ItemRef { kind: "series".into(), id: "tt0903747".into(), name: "Breaking Bad".into(), from: String::new() });
        let mut meta = fixture("cinemeta_breaking_bad");
        let state = ready(meta.clone());
        let b = build(&mode, Some(&state), None, Some(1), "");
        let pilot = &b.sections[0].rows[0];
        assert_eq!(pilot.still.as_deref(), Some("https://episodes.metahub.space/tt0903747/1/1/w780.jpg"));
        assert!(pilot.blurb.starts_with("When an unassuming high school chemistry teacher"), "{}", pilot.blurb);
        assert!(!pilot.blurb.contains('\n'));
        for v in meta.videos.iter_mut() {
            (v.thumbnail, v.overview) = (None, Some("  Two\nlines  ".into()));
        }
        let b = build(&mode, Some(&ready(meta)), None, Some(1), "");
        let row = &b.sections[0].rows[0];
        assert_eq!((row.still.as_deref(), row.blurb.as_str()), (Some(""), "Two lines"));
    }

    /// The page opens on the last-watched video's season (real Cinemeta meta of *A Wild Last Boss
    /// Appeared!*: S1 E1–12 and a placeholder S2 E1), else the first numbered season.
    #[test]
    fn the_default_season_is_the_last_watched_videos_else_the_first_numbered_one() {
        let meta = fixture("cinemeta_wild_last_boss");
        assert_eq!(seasons(&meta), [1, 2]);
        assert_eq!(default_season(&meta, Some("tt33258199:2:1")), Some(2));
        assert_eq!(default_season(&meta, Some("tt33258199:1:12")), Some(1));
        assert_eq!(default_season(&meta, None), Some(1));
        assert_eq!(default_season(&meta, Some("tt0903747:2:1")), Some(1), "another show's video says nothing");
        let specials_only = Meta { videos: vec![Video { id: "x:0:1".into(), season: Some(0), ..Default::default() }], ..Default::default() };
        assert_eq!(default_season(&specials_only, None), Some(0));
        assert_eq!(default_season(&Meta::default(), None), None);
    }

    /// The TMDB addon's meta for `tmdb:280042` files both cours under ONE season (TMDB's own
    /// numbering: S1 E1–24) with IMDb-based ids, `tt33258199:1:13` for the second cour's first
    /// episode. The fixture is that shape rebuilt from the addon's `getEpisodes` (ids
    /// `${imdb}:${season}:${index+1}`, `released` = air date + season ms) over TMDB's season page
    /// (the public instance answered 504 when this was written). Episodes airing after today read
    /// "Upcoming", dimmed, with no chevron; one airing today does not.
    #[test]
    fn an_episode_released_after_today_reads_upcoming() {
        let _en = sn_platform::i18n::language_on_this_thread_for_test(sn_platform::i18n::Preference::En);
        let meta = fixture("tmdb_wild_last_boss");
        assert_eq!(seasons(&meta), [1], "one season: no chips");
        let mode = Mode::Item(ItemRef { kind: "series".into(), id: "tmdb:280042".into(), name: "A Wild Last Boss Appeared!".into(), from: String::new() });
        let state = ready(meta);
        let b = build(&mode, Some(&state), None, None, "2026-10-17");
        let rows: Vec<(&str, Option<&str>, bool)> =
            b.sections[0].rows.iter().map(|r| (r.detail.as_str(), r.value.as_deref(), r.dim)).collect();
        assert_eq!(
            rows,
            [
                ("E1 \u{b7} 10/4/2025", None, false),
                ("E12 \u{b7} 12/20/2025", None, false),
                ("E13 \u{b7} 10/3/2026", None, false),
                ("E15 \u{b7} 10/17/2026", None, false),
                ("E16 \u{b7} 10/24/2026", Some("Upcoming"), true),
                ("E24 \u{b7} 12/19/2026", Some("Upcoming"), true),
            ]
        );
        assert_eq!(b.sections[0].rows[4].ticon, None, "an upcoming episode carries no chevron");
        assert!(b.actions[4].is_some(), "…but still opens its Streams page");
        match &b.actions[2] {
            Some(Action::Episode(v)) => assert_eq!(v.video_id, "tt33258199:1:13", "TMDB's numbering under the IMDb id"),
            other => panic!("{other:?}"),
        }
        let unknown = build(&mode, Some(&state), None, None, "");
        assert!(unknown.sections[0].rows.iter().all(|r| !r.dim), "no clock: nothing is upcoming");
        // Kitsu's second season: every episode dated 2026-09-26 (a placeholder start date), all past
        let kitsu = Mode::Item(ItemRef { kind: "series".into(), id: "kitsu:50404".into(), name: "2nd Season".into(), from: String::new() });
        let b = build(&kitsu, Some(&ready(fixture("kitsu_wild_last_boss_s2"))), None, None, "2026-10-17");
        assert!(b.sections[0].rows.iter().all(|r| !r.dim));
    }

    fn episode(id: &str, title: &str) -> VideoRef {
        VideoRef {
            kind: "series".into(),
            video_id: id.into(),
            show: "Breaking Bad".into(),
            title: title.into(),
            resume_ms: 0,
            item_id: "tt0903747".into(),
            from: String::new(),
        }
    }

    /// Real Cinemeta meta (trimmed): an episode page reads the episode's title line, its own
    /// overview, its still, its release day and the show's runtime, rating, genres and cast; one
    /// without an overview falls back to the show's description. A movie reads its year, cast and
    /// director and its background.
    #[test]
    fn the_narrative_column_reads_the_meta() {
        let _en = sn_platform::i18n::language_on_this_thread_for_test(sn_platform::i18n::Preference::En);
        let mut show = fixture("cinemeta_breaking_bad_meta");
        let ep = info(&Mode::Video(episode("tt0903747:1:5", "S1, E5 Gray Matter")), &show);
        assert_eq!(ep.title.as_deref(), Some("S1, E5 \u{b7} Gray Matter"));
        assert_eq!(ep.facts, "2/25/2008 \u{b7} 49 min \u{b7} IMDb 9.5", "the genres are chips, not words on the line");
        assert!(ep.synopsis.starts_with("Walter and Skyler attend a former colleague"), "{}", ep.synopsis);
        assert_eq!(
            link_names(&ep),
            [(Category::Genres, vec!["Crime", "Drama", "Thriller"]), (Category::Cast, vec!["Bryan Cranston", "Aaron Paul", "Anna Gunn"])],
            "Cinemeta's null director is no group"
        );
        assert_eq!(ep.backdrop.as_deref(), Some("https://episodes.metahub.space/tt0903747/1/5/w780.jpg"), "the episode's own still, full-bleed");
        assert_eq!(ep.logo, None, "an episode page is titled by its own line, not the show's logo");
        // no overview, no still: the show's description and background
        let v = show.videos.iter_mut().find(|v| v.id == "tt0903747:1:5").unwrap();
        (v.overview, v.thumbnail) = (Some("  ".into()), None);
        let ep = info(&Mode::Video(episode("tt0903747:1:5", "")), &show);
        assert!(ep.synopsis.starts_with("A chemistry teacher diagnosed"), "{}", ep.synopsis);
        assert_eq!(ep.backdrop, show.preview.background, "no still: the show's background");
        // the series' own page
        let page = info(&Mode::Item(ItemRef { kind: "series".into(), id: "tt0903747".into(), name: "x".into(), from: String::new() }), &show);
        assert_eq!(page.title.as_deref(), Some("Breaking Bad"));
        assert_eq!(page.logo.as_deref(), Some("https://images.metahub.space/logo/medium/tt0903747/img"));
        assert_eq!(page.backdrop, show.preview.background);
        assert!(page.facts.starts_with("2008\u{2013}2013 \u{b7} 49 min"), "{}", page.facts);
        // every cast name is a chip (the column's `+N` folds what does not fit)
        show.cast = ["A", "B", "C", "D", "E", "F", "G"].map(String::from).to_vec();
        show.links.clear();
        assert_eq!(info(&Mode::Video(episode("tt0903747:1:1", "")), &show).links[1].chips.len(), 7);
        let movie = fixture("cinemeta_shawshank");
        for mode in [movie_mode(), Mode::Video(VideoRef { kind: "movie".into(), video_id: "tt0111161".into(), show: "S".into(), title: "1994".into(), resume_ms: 0, item_id: "tt0111161".into(), from: String::new() })] {
            let m = info(&mode, &movie);
            assert_eq!(m.title.as_deref(), Some("The Shawshank Redemption"));
            assert_eq!(m.facts, "1994 \u{b7} 142 min \u{b7} IMDb 9.3");
            assert_eq!(
                link_names(&m),
                [
                    (Category::Genres, vec!["Drama"]),
                    (Category::Cast, vec!["Tim Robbins", "Morgan Freeman", "Bob Gunton"]),
                    (Category::Director, vec!["Frank Darabont"])
                ]
            );
            assert!(m.synopsis.starts_with("After a banker"));
            assert_eq!(m.backdrop, movie.preview.background);
            assert_eq!(m.logo.is_some(), matches!(mode, Mode::Item(_)), "a movie's own page wears its logo");
            assert_eq!(mode.show(), None, "a movie has no All episodes button");
        }
    }

    fn movie_mode() -> Mode {
        Mode::Item(ItemRef { kind: "movie".into(), id: "tt0111161".into(), name: "The Shawshank Redemption".into(), from: String::new() })
    }

    fn link_names(i: &Info) -> Vec<(Category, Vec<&str>)> {
        i.links.iter().map(|g| (g.category, g.chips.iter().map(|c| c.name.as_str()).collect())).collect()
    }

    /// A show whose column is as full as a real one gets: a synopsis far longer than the column,
    /// three genres, twelve cast names (more than two chip lines hold) and a director.
    fn crowded_show() -> Meta {
        let mut show = fixture("cinemeta_breaking_bad_meta");
        show.preview.description = Some("A chemistry teacher turns to crime. ".repeat(40));
        show.director = vec!["Vince Gilligan".into()];
        show.cast = (1..=12).map(|i| format!("Actor Number {i}")).collect();
        // the plain fields, not Cinemeta's links (three cast names, no director)
        show.links.clear();
        show
    }


    /// Every chip of `fl` inside the column `n`, under its group's label.
    fn assert_chips_inside(fl: &Flow<'_>, n: Rect, tag: &str) {
        for g in &fl.groups {
            for r in g.chips.iter().chain(g.more.iter().map(|m| &m.0)) {
                assert!(r.x >= n.x && r.x + r.w <= n.x + n.w + 0.5, "{tag}: chip {r:?} leaves the column");
                assert!(r.y + r.h <= n.y + n.h, "{tag}: chip {r:?} runs off the column");
            }
        }
    }

    /// The narrative column fits: in every shipped language, under a two-line title and over a
    /// synopsis far longer than the column, the synopsis keeps at least [`SYNOPSIS_MIN_LINES`] and
    /// elides, every link group shows under its label on at most [`LINK_LINES`] chip lines (the cast
    /// folding the rest into `+N`), and nothing runs off the column, whether the page wears a logo
    /// or a title. The cast expanded shows every name and the synopsis still keeps its lines. The
    /// All episodes button and the group labels are app text: they fit, unelided.
    #[test]
    fn the_narrative_column_and_its_chips_fit_in_every_shipped_language() {
        use sn_platform::i18n::{language_on_this_thread_for_test, SHIPPED};
        let m = sn_base::fontcov::advances::ShippedMeasure;
        let layout = detail_layout();
        let n = layout.narrative;
        for language in SHIPPED {
            let _g = language_on_this_thread_for_test(language);
            let show = crowded_show();
            let series = Mode::Item(ItemRef { kind: "series".into(), id: "tt0903747".into(), name: "x".into(), from: String::new() });
            let mut page = info(&series, &show);
            page.title = Some("The Remarkably Long Title of a Series".into());
            let mut titled = page.clone();
            titled.logo = None;
            let mut ep = info(&Mode::Video(episode("tt0903747:1:5", "")), &show);
            ep.synopsis = page.synopsis.clone();
            ep.links = page.links.clone();
            ep.title = page.title.clone();
            // an item page wears its logo (the band where the title was) or its wrapped title; an
            // episode page its own title
            for (what, inf) in [("logo", &page), ("title", &titled), ("episode", &ep)] {
                let tag = format!("{}/{what}", language.tag());
                let fl = flow(inf, inf.title.as_deref().unwrap(), &m, 0.0, None);
                let texts: Vec<&str> = fl.blocks.iter().map(|b| b.text).collect();
                assert_eq!(texts.len(), 6, "{tag}: facts, SUMMARY, synopsis, the three labels: {texts:?}");
                assert_eq!(texts[1], msg::app_detail_summary_label().to_uppercase(), "{tag}: the synopsis' label, in caps");
                let syn = &fl.blocks[2];
                assert!(syn.lines >= SYNOPSIS_MIN_LINES && syn.lines <= SYNOPSIS_MAX_LINES, "{tag}: {} lines", syn.lines);
                assert!(info_view(syn.text, syn.ink, syn.lines, &m).truncates(syn.rect.w), "the long synopsis elides");
                if inf.logo.is_some() {
                    assert!(fl.blocks[0].rect.y >= logo_band().y + logo_band().h, "{tag}: the facts run under the logo");
                }
                for b in &fl.blocks {
                    assert!(b.rect.x >= n.x && b.rect.x + b.rect.w <= n.x + n.w, "{tag}: {:?}", b.text);
                    assert!(b.rect.y + b.rect.h <= n.y + n.h, "{tag}: {:?} runs off the column", b.text);
                }
                assert_eq!(fl.groups.len(), 3, "{tag}: genres, cast, director");
                for (g, label) in fl.groups.iter().zip(&fl.blocks[3..]) {
                    let lines: std::collections::BTreeSet<u32> = g.chips.iter().chain(g.more.iter().map(|m| &m.0)).map(|r| r.y as u32).collect();
                    assert!(lines.len() <= LINK_LINES, "{tag}: {} chip lines", lines.len());
                    assert!(g.chips.iter().all(|r| r.y >= label.rect.y + label.rect.h), "{tag}: chips under their label");
                }
                let cast = &fl.groups[1];
                let (more, hidden) = cast.more.expect("twelve names fold into +N");
                assert_eq!(cast.chips.len() + hidden, 12, "{tag}: +N counts the rest");
                assert!(more.x > cast.chips.last().unwrap().x, "{tag}: +N ends the last line");
                assert_chips_inside(&fl, n, &tag);
                // the cast expanded shows more names; with both library buttons under it (the most
                // crowded column) it gives way first, and the synopsis keeps its minimum either way
                for reserve in [0.0, ACTION_BAND] {
                    let open = flow(inf, inf.title.as_deref().unwrap(), &m, reserve, Some(1));
                    assert!(open.blocks[2].lines >= SYNOPSIS_MIN_LINES, "{tag}/{reserve}: expanded, the synopsis keeps {} lines", open.blocks[2].lines);
                    assert_eq!(open.groups.len(), 3, "{tag}/{reserve}: expanded, every group still shows");
                    let c = &open.groups[1];
                    assert_eq!(c.chips.len() + c.more.map_or(0, |m| m.1), 12, "{tag}/{reserve}: expanded");
                    if reserve == 0.0 {
                        assert!(c.chips.len() > cast.chips.len(), "{tag}: expanded shows more names");
                    }
                    assert_chips_inside(&open, n, &tag);
                    assert!(open.free + reserve <= n.y + n.h + theme::space::MD, "{tag}/{reserve}: the buttons still fit under it");
                }
            }
            let label = CString::new(msg::app_detail_all_episodes()).unwrap();
            let w = Button::pill_w_measured(&label, theme::size::BODY, false, false, &m);
            assert!(w <= layout.content.w / 2.0, "{}: All episodes is {w}px", language.tag());
            for g in &page.links {
                assert!(!label_view(&g.caps, &m).truncates(n.w), "{}: {:?} elides", language.tag(), g.caps);
            }
            let summary = msg::app_detail_summary_label().to_uppercase();
            assert!(!label_view(&summary, &m).truncates(n.w), "{}: {summary:?} elides", language.tag());
            {
            }
        }
    }

    /// The real dispatcher: Continue Watching opens S2 E1's Streams page; the show's meta and one
    /// addon's streams land; the first stream row is focused, the title is the episode's line and
    /// the column reads the episode. UP reaches All episodes, OK opens the show on Season 2; BACK
    /// returns to the episode, BACK again to Home. The pointer presses the button too.
    #[test]
    fn all_episodes_opens_the_show_on_the_episodes_season_and_back_returns() {
        use crate::stores::detail::{DetailLanding, DetailStore};
        use crate::stores::StoreId;
        use sn_machine::machine::{InputKind, Key, MachineId};
        use sn_stremio::manifest::{Addon, Manifest};
        let _g = sn_base::testlock::serial();
        let _en = sn_platform::i18n::language_on_this_thread_for_test(sn_platform::i18n::Preference::En);
        let cinemeta = Addon::new(
            "https://v3-cinemeta.strem.io/manifest.json",
            Manifest::parse(br#"{"id":"com.linvo.cinemeta","resources":["catalog","meta"],"types":["movie","series"],"idPrefixes":["tt"]}"#).unwrap(),
        );
        let streams = Addon::new(
            "https://one.example/manifest.json",
            Manifest::parse(br#"{"id":"one","name":"One","resources":["stream"],"types":["movie","series"],"idPrefixes":["tt"]}"#).unwrap(),
        );
        let addons = [cinemeta.clone(), streams];
        let mut app = crate::test_support::TestApp::boot(crate::stores::catalog::CatalogStore::from_addons(vec![cinemeta]));
        let video = episode("tt0903747:2:1", "S2, E1 Seven Thirty-Seven");
        let meta_key = MetaKey { kind: "series".into(), id: "tt0903747".into() };
        let stream_key = StreamKey { kind: "series".into(), video_id: video.video_id.clone() };
        app.rig.detail = DetailStore::new();
        app.rig.detail.open_meta(&addons, meta_key.clone());
        app.rig.detail.open_streams(&addons, &Default::default(), stream_key.clone());
        app.d.request(MachineId::Nav, NavOp::Push(AppArg::Streams(video.clone())));
        app.frames(40);
        app.rig.detail.land(DetailLanding::Meta { key: meta_key, result: Ok(fixture("cinemeta_breaking_bad_meta")) });
        let ready = GroupState::Ready(vec![http("1080p", "", "https://cdn.example/a.mkv"), http("720p", "", "https://cdn.example/b.mkv")]);
        app.rig.detail.land(DetailLanding::Streams { key: stream_key, index: 0, state: ready });
        let gen = app.rig.detail.gen();
        app.notify(StoreId::Detail, gen);
        app.frames(40);
        let page = detail_page(&app);
        assert_eq!(page.title(), "S2, E1 \u{b7} Seven Thirty-Seven");
        assert!(page.info().synopsis.starts_with("Walt and Jesse are reminded"));
        assert_eq!(link_names(page.info())[1], (Category::Cast, vec!["Bryan Cranston", "Aaron Paul", "Anna Gunn"]));
        assert!(page.addon_label.is_empty(), "one stream addon: no addon dropdown");
        assert_eq!(app.d.focus().map(|k| k.elem), Some(STREAM_KEY), "the first stream row is focused on arrival");
        app.press(Key::Up);
        assert_eq!(app.d.focus().map(|k| k.elem), Some(SEASON_KEY), "UP reaches All episodes");
        assert!(!detail_page(&app).table.list_focused, "All episodes has focus: no stream row draws as focused");
        app.press(Key::Ok);
        assert_eq!(app.top_arg(), Some(&AppArg::Overview(video.clone())));
        assert_eq!(app.top_name(), "detail");
        app.frames(10);
        let show = detail_page(&app);
        assert_eq!(show.season(), Some(2), "the episode's season");
        assert_eq!(show.title(), "Breaking Bad");
        assert_eq!(show.crumb, "S2, E1 Seven Thirty-Seven", "BACK names the episode page");
        assert_eq!(app.d.focus().map(|k| k.elem), Some(EPISODE_KEY + 2), "S2 E1 is seated");
        app.press(Key::Back);
        assert_eq!(app.top_arg(), Some(&AppArg::Streams(video.clone())), "BACK returns to the episode");
        // the pointer: a click on the button
        app.frames(60);
        let row = detail_layout().sectioned_table();
        app.send(InputKind::Click { x: row.x + 30.0, y: row.y + crate::type_filter::CHIP_H * 0.5, hit: None });
        app.frames(40);
        assert_eq!(app.top_arg(), Some(&AppArg::Overview(video.clone())), "a click presses All episodes");
        app.press(Key::Back);
        app.press(Key::Back);
        assert_eq!(app.top_name(), "home", "then Home");
    }

    fn detail_page(app: &crate::test_support::TestApp) -> &DetailScreen {
        app.d.top_screen().and_then(|s| s.as_any()).and_then(|a| a.downcast_ref::<DetailScreen>()).expect("the detail page")
    }

    /// Boss (a series last watched at S2 E1) open on its page, everything landed.
    fn boss_detail() -> crate::test_support::TestApp {
        boss_detail_with(r#"[{"_id":"tt33258199","type":"series","name":"Boss","state":{"video_id":"tt33258199:2:1"}}]"#)
    }

    fn boss_detail_with(library_json: &str) -> crate::test_support::TestApp {
        use crate::stores::account::Secret;
        use crate::stores::detail::{DetailLanding, DetailStore};
        use crate::stores::library::{LibraryLanding, LibraryStore};
        use crate::stores::StoreId;
        use sn_machine::machine::MachineId;
        use sn_stremio::manifest::{Addon, Manifest};
        let cinemeta = Addon::new(
            "https://v3-cinemeta.strem.io/manifest.json",
            Manifest::parse(br#"{"id":"com.linvo.cinemeta","resources":["catalog","meta"],"types":["movie","series"],"idPrefixes":["tt"]}"#).unwrap(),
        );
        let mut app = crate::test_support::TestApp::boot(crate::stores::catalog::CatalogStore::from_addons(vec![cinemeta.clone()]));
        let mut library = LibraryStore::new();
        library.sign_in("u1", &Secret::new("k"));
        let _ = library.land(LibraryLanding::Fetched {
            user_id: "u1".into(),
            result: Ok(serde_json::from_str(library_json).unwrap()),
        });
        app.rig.library = library;
        let item = ItemRef { kind: "series".into(), id: "tt33258199".into(), name: "Boss".into(), from: String::new() };
        let key = MetaKey { kind: item.kind.clone(), id: item.id.clone() };
        app.rig.detail = DetailStore::new();
        app.rig.detail.open_meta(&[cinemeta], key.clone());
        app.d.request(MachineId::Nav, NavOp::Push(AppArg::Detail(item)));
        app.frames(40);
        app.rig.detail.land(DetailLanding::Meta { key, result: Ok(fixture("cinemeta_wild_last_boss")) });
        let gen = app.rig.detail.gen();
        app.notify(StoreId::Detail, gen);
        app.frames(10);
        app
    }

    /// Continue Watching keeps the ids an item was saved under: a TMDB movie resumes on its
    /// default video, a TMDB episode on the meta's episode of the same number.
    #[test]
    fn a_tmdb_saved_item_asks_streams_under_the_metas_ids() {
        let movie: Meta = serde_json::from_str(r#"{"id":"tmdb:157336","type":"movie","name":"I","behaviorHints":{"defaultVideoId":"tt0816692"}}"#).unwrap();
        assert_eq!(stream_video_for("movie", "tmdb:157336", &movie).as_deref(), Some("tt0816692"));
        assert_eq!(stream_video_for("movie", "tt0816692", &movie), None, "already there");
        let show: Meta = serde_json::from_str(
            r#"{"id":"tmdb:1396","type":"series","name":"B","videos":[{"id":"tt0903747:1:1","season":1,"episode":1},{"id":"tt0903747:1:2","season":1,"episode":2}]}"#,
        )
        .unwrap();
        assert_eq!(stream_video_for("series", "tmdb:1396:1:2", &show).as_deref(), Some("tt0903747:1:2"));
        assert_eq!(stream_video_for("series", "tt0903747:1:1", &show), None, "one of the meta's own");
        assert_eq!(stream_video_for("series", "tmdb:1396:9:9", &show), None, "no such episode: ask as saved");
        assert_eq!(stream_video_for("series", "tmdb:1396", &show), None, "not an episode id");
    }

    /// A movie from a TMDB-keyed catalog (AIOStreams' search lists Interstellar as
    /// `tmdb:157336`) asks for its streams under the IMDb id its meta names in
    /// `behaviorHints.defaultVideoId`: the scrapers answer nothing for the TMDB id.
    #[test]
    fn a_tmdb_movie_asks_its_streams_under_the_meta_default_video() {
        use crate::stores::detail::{DetailLanding, DetailStore};
        use crate::stores::StoreId;
        use sn_machine::machine::MachineId;
        use sn_stremio::manifest::{Addon, Manifest};
        let _g = sn_base::testlock::serial();
        let _en = sn_platform::i18n::language_on_this_thread_for_test(sn_platform::i18n::Preference::En);
        let aio = Addon::new(
            "https://aio.example/manifest.json",
            Manifest::parse(br#"{"id":"aio","name":"AIO","resources":["catalog",{"name":"meta","types":["movie"],"idPrefixes":["tmdb:","tt"]},{"name":"stream","types":["movie"],"idPrefixes":["tt","tmdb:"]}],"types":["movie"]}"#).unwrap(),
        );
        let mut app = crate::test_support::TestApp::boot(crate::stores::catalog::CatalogStore::from_addons(vec![aio.clone()]));
        let item = ItemRef { kind: "movie".into(), id: "tmdb:157336".into(), name: "Interstellar".into(), from: String::new() };
        let key = MetaKey { kind: item.kind.clone(), id: item.id.clone() };
        app.rig.detail = DetailStore::new();
        app.rig.detail.open_meta(&[aio], key.clone());
        app.d.request(MachineId::Nav, NavOp::Push(AppArg::Detail(item)));
        app.frames(20);
        let opened = |fx: &[AppFx]| -> Vec<String> {
            fx.iter()
                .filter_map(|f| match f {
                    AppFx::Detail(DetailCmd::OpenStreams(k)) => Some(k.video_id.clone()),
                    _ => None,
                })
                .collect()
        };
        assert_eq!(opened(&app.take_effects()), ["tmdb:157336"], "the streams open at once, on the catalog id");
        let meta = serde_json::from_str(r#"{"id":"tmdb:157336","type":"movie","name":"Interstellar","behaviorHints":{"defaultVideoId":"tt0816692"}}"#).unwrap();
        app.rig.detail.land(DetailLanding::Meta { key, result: Ok(meta) });
        let gen = app.rig.detail.gen();
        app.notify(StoreId::Detail, gen);
        app.frames(10);
        assert_eq!(opened(&app.take_effects()), ["tt0816692"], "then once more under the video the meta names");
        app.frames(10);
        assert!(opened(&app.take_effects()).is_empty(), "and only once");
    }

    /// The real dispatcher: the Wild Last Boss series opened from Cinemeta with its season 2
    /// last watched opens on Season 2; UP from the episodes reaches the season dropdown, OK opens
    /// its popover on Season 2, BACK closes it unchanged, a pick lists that season, DOWN goes back
    /// to the episodes.
    #[test]
    fn the_season_dropdown_opens_and_chooses_through_the_dispatcher() {
        use sn_machine::machine::Key;
        let _g = sn_base::testlock::serial();
        let _en = sn_platform::i18n::language_on_this_thread_for_test(sn_platform::i18n::Preference::En);
        let mut app = boss_detail();
        let page = detail_page(&app);
        assert_eq!(page.season(), Some(2), "the last-watched video's season");
        assert_eq!(page.sections().iter().map(|s| s.header.as_str()).collect::<Vec<_>>(), [""], "the dropdown names the season");
        let focus = || app.d.focus().map(|k| k.elem);
        assert_eq!(focus(), Some(EPISODE_KEY + 3), "the first row seats: S2 E1 (the fixture's 4th video)");
        app.press(Key::Up);
        assert_eq!(app.d.focus().map(|k| k.elem), Some(SEASON_KEY), "UP reaches the season dropdown");
        assert!(!detail_page(&app).table.list_focused, "the dropdown has focus: no episode row draws as focused");
        assert!(app.drawn_text().iter().any(|t| t == "Season 2"), "it says the season listed");
        app.press(Key::Left);
        assert_eq!(app.d.focus().map(|k| k.elem), Some(LIBRARY_KEY), "LEFT reaches the library button");
        assert_eq!(app.top_name(), "detail", "…and is not BACK");
        app.press(Key::Right);
        assert_eq!(app.d.focus().map(|k| k.elem), Some(EPISODE_KEY + 3), "RIGHT returns to the list");
        app.press(Key::Up);
        assert_eq!(app.d.focus().map(|k| k.elem), Some(SEASON_KEY), "back on the dropdown");
        app.press(Key::Ok);
        let MenuKind::Choice { options, current, target, .. } = crate::discover_tests::menu_of(&app) else { panic!("a choice") };
        assert_eq!((options, current, target), (vec!["Season 1".to_string(), "Season 2".into()], 1, ChoiceTarget::DetailSeason), "Season 2 checked");
        app.press(Key::Back);
        assert_eq!(detail_page(&app).season(), Some(2), "BACK closes it, nothing chosen");
        app.press(Key::Ok);
        crate::discover_tests::pick(&mut app, 0);
        assert_eq!(app.d.focus().map(|k| k.elem), Some(SEASON_KEY), "focus returns to the dropdown");
        let page = detail_page(&app);
        assert_eq!(page.season(), Some(1));
        assert_eq!(page.sections().iter().map(|s| s.rows.len()).collect::<Vec<_>>(), [3]);
        app.press(Key::Down);
        assert_eq!(app.d.focus().map(|k| k.elem), Some(EPISODE_KEY), "DOWN lands on S1 E1");
        assert!(detail_page(&app).table.list_focused, "the row's highlight returns with the focus");
        app.press(Key::Ok);
        assert_eq!(app.top_name(), "streams");
    }

    /// The season dropdown stays on the page while its popover is up and after a pick.
    #[test]
    fn the_dropdown_button_stays_drawn_while_open_and_after_a_pick() {
        let _g = sn_base::testlock::serial();
        let _en = sn_platform::i18n::language_on_this_thread_for_test(sn_platform::i18n::Preference::En);
        let mut app = boss_detail();
        app.press(sn_machine::machine::Key::Up);
        assert_eq!(app.d.focus().map(|k| k.elem), Some(SEASON_KEY));
        crate::nav_tests::the_dropdown_button_stays_drawn(&mut app, "Season 2", 0, "Season 1");
    }

    /// An `anime`-typed item (the Kitsu addon's and MAL addons' catalog type) lists the episodes
    /// its meta carries, as a series does — real Kitsu meta, trimmed.
    #[test]
    fn an_anime_item_lists_its_episodes() {
        let _en = sn_platform::i18n::language_on_this_thread_for_test(sn_platform::i18n::Preference::En);
        let meta: sn_stremio::types::MetaResponse =
            serde_json::from_str(include_str!("../../stremio/src/fixtures/kitsu_aot_s2.json")).unwrap();
        let item = ItemRef { kind: "anime".into(), id: "kitsu:8671".into(), name: "Attack on Titan Season 2".into(), from: String::new() };
        assert_eq!(Mode::Item(item.clone()).stream_key(), None, "no streams for the bare show id");
        let b = build(&Mode::Item(item), Some(&MetaState::Ready(Box::new(meta.meta))), None, None, "");
        let rows: Vec<&str> = b.sections.iter().flat_map(|s| s.rows.iter().map(|r| r.detail.as_str())).collect();
        assert_eq!(rows, ["E1 \u{b7} 3/31/2017", "E2 \u{b7} 4/7/2017", "E12 \u{b7} 6/16/2017"]);
    }

    /// The stream panel's ground starts in the air between the narrative column and the list, so
    /// the list (and its season row) stand wholly on it and the column wholly off it.
    #[test]
    fn the_stream_panel_stands_between_the_column_and_the_list() {
        let l = detail_layout();
        let x = stream_panel_x();
        assert!(x > l.narrative.x + l.narrative.w + theme::space::LG, "clear of the column's text");
        assert!(x < l.sectioned_table().x, "under the whole list");
    }

    /// The action row keeps its room at the column's foot: in every shipped language, under a long
    /// title or a logo and over a synopsis longer than the column, the link chips still show, the
    /// discs sit under them inside the column, and whichever disc unfurls its verb (one at a time:
    /// focus is on one) keeps the whole row inside the column.
    #[test]
    fn the_action_row_fits_under_the_link_chips_in_every_shipped_language() {
        use sn_platform::i18n::{language_on_this_thread_for_test, SHIPPED};
        let m = sn_base::fontcov::advances::ShippedMeasure;
        let n = detail_layout().narrative;
        for language in SHIPPED {
            let _g = language_on_this_thread_for_test(language);
            let show = crowded_show();
            let mut page = info(&Mode::Item(ItemRef { kind: "series".into(), id: "tt0903747".into(), name: "x".into(), from: String::new() }), &show);
            page.title = Some("The Remarkably Long Title of a Series".into());
            let mut titled = page.clone();
            titled.logo = None;
            for (what, inf) in [("logo", &page), ("title", &titled)] {
                let verbs = [None, Some((sn_ui::widgets::mark_watched_verb(), false)), Some((sn_ui::widgets::mark_unwatched_verb(), true))];
                for (label, verb) in [msg::app_menu_add_to_library(), msg::app_menu_in_library()].into_iter().flat_map(|l| verbs.map(|v| (l, v))) {
                    let label = CString::new(label).unwrap();
                    let verb_c = verb.map(|(v, on)| (CString::new(v).unwrap(), on));
                    let memo = RefCell::new(None);
                    let held = label.to_str().unwrap() == msg::app_menu_in_library();
                    let watched = verb_c.as_ref().map(|(c, on)| (c, *on));
                    let tag = format!("{}/{what}/{label:?}/{verb:?}", language.tag());
                    for open in [[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]] {
                        let b = LibraryButton { label: &label, held, watched, entry: EntryId(1), info: inf, title: inf.title.as_deref().unwrap(), expanded: None, open, rect: &memo };
                        let lib = b.rect(&m);
                        let row = b.watched_rect(&m).map_or(lib, |w| lib.union(w));
                        // the row may unfurl into the air between the column and the stream panel
                        assert!(lib.x == n.x && row.x + row.w <= stream_panel_x() - theme::space::SM, "{tag}/{open:?}: the row {row:?} reaches the stream panel");
                        assert!(row.y + row.h <= n.y + n.h, "{tag}: inside the column {row:?}");
                        let blocks = flow(inf, inf.title.as_deref().unwrap(), &m, b.band(), None).blocks;
                        assert_eq!(blocks.len(), 6, "{tag}: facts, SUMMARY, synopsis, the three labels");
                        assert!(blocks[2].lines >= SYNOPSIS_MIN_LINES, "{tag}: the synopsis keeps {} lines", blocks[2].lines);
                        let chips = flow(inf, inf.title.as_deref().unwrap(), &m, b.band(), None).groups;
                        let last = chips.last().and_then(|g| g.chips.last()).unwrap();
                        assert!(lib.y >= last.y + last.h, "{tag}: under the link chips");
                    }
                }
            }
        }
    }

    fn sign_in(app: &mut crate::test_support::TestApp) {
        use crate::stores::account::{Account, AccountLanding, AccountStore, Secret, Stored};
        let mut account = AccountStore::new();
        let _ = account.land(AccountLanding::Restored(Stored::Session(Account { user_id: "u1".into(), email: "v@example.com".into(), auth_key: Secret::new("k") })), 0);
        app.rig.account = account;
        app.notify(StoreId::Account, 7);
        app.frames(3);
    }

    fn library_cmds(app: &mut crate::test_support::TestApp) -> Vec<LibraryCmd> {
        app.take_effects()
            .into_iter()
            .filter_map(|fx| match fx {
                AppFx::Library(c) => Some(c),
                _ => None,
            })
            .collect()
    }

    /// The real dispatcher: Boss (in the library) open on its page, signed in. LEFT from the
    /// episodes reaches the button, which reads In your library; OK asks for the removal and,
    /// the store optimistic, the button flips at once; a refusal flips it back (and is queued for
    /// the toast); OK again with the removal landed, then OK adds it back with the meta's name
    /// and poster. Nothing is asked while a write is on its way; RIGHT returns to the list.
    #[test]
    fn the_library_button_toggles_the_item_and_a_refusal_rolls_it_back() {
        use crate::stores::library::{LibraryLanding, Refused};
        use sn_machine::machine::Key;
        let _g = sn_base::testlock::serial();
        let _en = sn_platform::i18n::language_on_this_thread_for_test(sn_platform::i18n::Preference::En);
        let mut app = boss_detail();
        sign_in(&mut app);
        let label = |app: &crate::test_support::TestApp| detail_page(app).library_label();
        assert_eq!(label(&app).as_deref(), Some(msg::app_menu_in_library()));
        app.press(Key::Left);
        assert_eq!(app.d.focus().map(|k| k.elem), Some(LIBRARY_KEY), "LEFT from the episodes reaches the button");
        assert!(!detail_page(&app).table.list_focused, "the button has focus: no row draws as focused");
        assert!(app.drawn_text().iter().any(|t| t == msg::app_menu_in_library()), "it is drawn");
        app.take_effects();
        app.press(Key::Ok);
        let cmds = library_cmds(&mut app);
        assert_eq!(cmds, [LibraryCmd::Remove("tt33258199".into())]);
        // the rig does not run the store; the application's bridge does — stand in for it
        assert_eq!(app.rig.library.remove("tt33258199").len(), 1);
        app.frames(2);
        assert_eq!(label(&app).as_deref(), Some(msg::app_menu_add_to_library()), "optimistic: flipped at once");
        app.press(Key::Ok);
        assert!(library_cmds(&mut app).is_empty(), "nothing while the write is on its way");
        let refused = Err(sn_stremio::account::ApiError::Api { code: 1, message: "no".into() });
        let _ = app.rig.library.land(LibraryLanding::Edited { user_id: "u1".into(), id: "tt33258199".into(), result: refused });
        app.frames(2);
        assert_eq!(label(&app).as_deref(), Some(msg::app_menu_in_library()), "a refusal flips it back");
        assert_eq!(app.rig.library.take_failed(), [Refused::Remove], "and is the application's toast");
        app.press(Key::Ok);
        assert_eq!(library_cmds(&mut app), [LibraryCmd::Remove("tt33258199".into())], "tried again");
        let _ = app.rig.library.remove("tt33258199");
        let _ = app.rig.library.land(LibraryLanding::Edited { user_id: "u1".into(), id: "tt33258199".into(), result: Ok(()) });
        app.frames(2);
        assert_eq!(label(&app).as_deref(), Some(msg::app_menu_add_to_library()));
        app.press(Key::Ok);
        let cmds = library_cmds(&mut app);
        let [LibraryCmd::Add(entry)] = &cmds[..] else { panic!("an add: {cmds:?}") };
        assert_eq!((entry.id.as_str(), entry.kind.as_str()), ("tt33258199", "series"));
        assert!(!entry.name.is_empty());
        let _ = app.rig.library.add(entry.clone());
        app.frames(2);
        assert_eq!(label(&app).as_deref(), Some(msg::app_menu_in_library()), "added: held at once");
        app.press(Key::Right);
        assert!(app.d.focus().is_some_and(|k| k.elem >= EPISODE_KEY && k.elem < NOTE_KEY), "RIGHT returns to the episodes");
        assert_eq!(app.top_name(), "detail");
    }

    /// Signed out the button still offers Add to library; OK opens the sign-in page and writes
    /// nothing.
    #[test]
    fn signed_out_the_library_button_opens_the_sign_in_page() {
        use sn_machine::machine::Key;
        let _g = sn_base::testlock::serial();
        let _en = sn_platform::i18n::language_on_this_thread_for_test(sn_platform::i18n::Preference::En);
        let mut app = boss_detail();
        assert_eq!(detail_page(&app).library_label().as_deref(), Some(msg::app_menu_add_to_library()), "signed out nothing is held");
        app.press(Key::Left);
        assert_eq!(app.d.focus().map(|k| k.elem), Some(LIBRARY_KEY));
        app.take_effects();
        app.press(Key::Ok);
        assert!(library_cmds(&mut app).is_empty(), "nothing is written");
        assert_eq!(app.top_arg(), Some(&AppArg::SignIn));
    }

    /// An episode's Streams page has no library button (the show's page does).
    #[test]
    fn an_episode_page_has_no_library_button() {
        let _g = sn_base::testlock::serial();
        let mut app = crate::test_support::TestApp::boot(crate::stores::catalog::CatalogStore::from_addons(Vec::new()));
        app.d.request(sn_machine::machine::MachineId::Nav, NavOp::Push(AppArg::Streams(episode("tt0903747:2:1", "S2, E1"))));
        app.frames(10);
        assert_eq!(app.top_name(), "streams");
        assert_eq!(detail_page(&app).library_label(), None);
    }

    /// Boss with S1 E1–E12 watched (the account's bits) and E12 the last played: the page opens
    /// on Season 2 focused on its first episode, S1's rows read Watched, and the chip row's
    /// season button marks the listed season — a write only on its press.
    #[test]
    fn a_series_opens_on_its_next_unwatched_episode_and_marks_a_season() {
        use sn_machine::machine::Key;
        use sn_stremio::library::{Edit, MarkTarget};
        let _g = sn_base::testlock::serial();
        let _en = sn_platform::i18n::language_on_this_thread_for_test(sn_platform::i18n::Preference::En);
        let meta = fixture("cinemeta_wild_last_boss");
        let mut bits = sn_stremio::watched::WatchedBitField::of_state(None, &meta.videos);
        for v in meta.videos.iter().filter(|v| v.season == Some(1)) {
            bits.set_video(&v.id, true);
        }
        let lib = format!(r#"[{{"_id":"tt33258199","type":"series","name":"Boss","state":{{"video_id":"tt33258199:1:12","watched":"{bits}"}}}}]"#);
        let mut app = boss_detail_with(&lib);
        sign_in(&mut app);
        app.frames(5);
        let page = detail_page(&app);
        assert_eq!(page.season(), Some(2), "the next unwatched episode's season");
        let s2e1 = meta.videos.iter().position(|v| v.id == "tt33258199:2:1").unwrap() as u32;
        assert_eq!(app.d.focus().map(|k| k.elem), Some(EPISODE_KEY + s2e1), "focused on it");
        assert!(app.drawn_text().iter().any(|t| t == msg::app_watched_mark_season()), "the season button");
        app.take_effects();
        app.press(Key::Up);
        app.press(Key::Right);
        assert_eq!(app.d.focus().map(|k| k.elem), Some(SEASON_KEY + 1));
        assert!(library_cmds(&mut app).is_empty(), "nothing is written by moving");
        app.press(Key::Ok);
        let cmds = library_cmds(&mut app);
        let [LibraryCmd::Mark(entry, Edit::MarkVideos(mark))] = &cmds[..] else { panic!("{cmds:?}") };
        assert_eq!((entry.id.as_str(), &mark.target, mark.watched), ("tt33258199", &MarkTarget::Season(2), true));
        // season 1 reads watched
        app.press(Key::Left);
        app.press(Key::Ok);
        crate::discover_tests::pick(&mut app, 0);
        app.frames(3);
        let rows = &detail_page(&app).sections()[0].rows;
        assert!(rows.iter().all(|r| r.value.as_deref() == Some(msg::app_watched_watched())), "every S1 row is watched");
        assert!(app.drawn_text().iter().any(|t| t == msg::app_watched_unmark_season()), "so its button unmarks");
    }

    /// A movie's page, signed in: *Mark as Unwatched* (checked) under the library button while
    /// its entry is watched; a press asks for core's `MarkAsWatched(false)` and nothing else.
    #[test]
    fn a_watched_movie_offers_mark_as_unwatched_under_the_library_button() {
        use crate::stores::account::Secret;
        use crate::stores::library::{LibraryLanding, LibraryStore};
        use sn_machine::machine::{Key, MachineId};
        let _g = sn_base::testlock::serial();
        let _en = sn_platform::i18n::language_on_this_thread_for_test(sn_platform::i18n::Preference::En);
        let mut app = crate::test_support::TestApp::boot(crate::stores::catalog::CatalogStore::from_addons(Vec::new()));
        let mut library = LibraryStore::new();
        library.sign_in("u1", &Secret::new("k"));
        let _ = library.land(LibraryLanding::Fetched {
            user_id: "u1".into(),
            result: Ok(serde_json::from_str(r#"[{"_id":"tt1","type":"movie","name":"Film","state":{"timesWatched":1}}]"#).unwrap()),
        });
        app.rig.library = library;
        sign_in(&mut app);
        app.d.request(MachineId::Nav, NavOp::Push(AppArg::Detail(ItemRef { kind: "movie".into(), id: "tt1".into(), name: "Film".into(), from: String::new() })));
        app.frames(10);
        assert_eq!(detail_page(&app).watched, Some(true));
        assert!(!app.drawn_text().iter().any(|t| t == sn_ui::widgets::mark_unwatched_verb()), "at rest the disc is its glyph alone");
        app.press(Key::Left);
        assert_eq!(app.d.focus().map(|k| k.elem), Some(LIBRARY_KEY));
        let verb = detail_page(&app).library_label().unwrap();
        assert!(app.drawn_text().iter().any(|t| *t == verb), "focused, the library disc unfurls its verb");
        app.press(Key::Right);
        assert_eq!(app.d.focus().map(|k| k.elem), Some(WATCHED_KEY), "RIGHT along the action row");
        assert!(app.drawn_text().iter().any(|t| t == sn_ui::widgets::mark_unwatched_verb()), "the watched disc says what it does");
        app.take_effects();
        app.press(Key::Ok);
        let cmds = library_cmds(&mut app);
        assert!(
            matches!(&cmds[..], [LibraryCmd::Mark(e, sn_stremio::library::Edit::MarkWatched(false))] if e.id == "tt1"),
            "{cmds:?}"
        );
    }

    // ---- the link chips ----------------------------------------------------------------------

    const LINK_CATALOG: &str = r#"{"id":"example.catalog","name":"Example","resources":["catalog","meta"],"types":["movie","series"],"idPrefixes":["tt"],"catalogs":[
        {"type":"movie","id":"top","name":"Popular","extra":[{"name":"genre","options":["Drama","Comedy"]},{"name":"skip"}]},
        {"type":"movie","id":"fresh","name":"Fresh","extra":[{"name":"genre","options":["Action","Comedy"]}]},
        {"type":"series","id":"top","name":"Popular","extra":[{"name":"genre","options":["Drama"]}]}]}"#;

    fn link_key(g: usize, slot: u32) -> u32 {
        LINK_KEY + ((g as u32) << 8) + slot
    }

    /// The addon's `links` are read per group, plain fields filling a group it sent none for:
    /// a discover link names its addon, catalog and genre, a search link its text; the writers,
    /// the IMDb rating and the share link are no chips.
    #[test]
    fn the_links_come_from_the_metas_links_else_its_fields() {
        let meta: Meta = serde_json::from_str(
            r#"{"id":"tt1","type":"movie","name":"Film","genres":["Comedy"],"cast":["Field Actor"],"director":["Cy Example"],"links":[
                {"name":"Drama","category":"Genres","url":"stremio:///discover/https%3A%2F%2Fcatalog.example.com%2Fmanifest.json/movie/top?genre=Drama"},
                {"name":"Ann Example","category":"Cast","url":"stremio:///search?search=Ann%20Example"},
                {"name":"Wes Writer","category":"Writers","url":"stremio:///search?search=Wes%20Writer"},
                {"name":"7.1","category":"imdb","url":"https://imdb.com/title/tt1"},
                {"name":"Film","category":"share","url":"https://www.example.com/s/film"}]}"#,
        )
        .unwrap();
        let groups = meta_links("movie", &meta);
        let got: Vec<(Category, Vec<(&str, &AppArg)>)> = groups.iter().map(|g| (g.category, g.chips.iter().map(|c| (c.name.as_str(), &c.to)).collect())).collect();
        let drama = AppArg::DiscoverAt(GenreRef {
            kind: "movie".into(),
            genre: Some("Drama".into()),
            addon_base: Some("https://catalog.example.com".into()),
            catalog: Some("top".into()),
        });
        assert_eq!(
            got,
            [
                (Category::Genres, vec![("Drama", &drama)]),
                (Category::Cast, vec![("Ann Example", &AppArg::SearchFor("Ann Example".into()))]),
                (Category::Director, vec![("Cy Example", &AppArg::SearchFor("Cy Example".into()))]),
            ],
            "the links win where they name a group; the director falls back to the field"
        );
        // no links at all: the fields, a genre opening Discover on the item's type
        let plain: Meta = serde_json::from_str(r#"{"id":"tt2","type":"series","name":"Show","genres":[" Drama ","","Drama"],"cast":["A"]}"#).unwrap();
        let groups = meta_links("series", &plain);
        assert_eq!(groups.len(), 2, "no director: no group");
        assert_eq!(groups[0].chips.len(), 1, "blank and repeated genres are dropped");
        assert_eq!(groups[0].chips[0].to, AppArg::DiscoverAt(GenreRef { kind: "series".into(), genre: Some("Drama".into()), addon_base: None, catalog: None }));
    }

    /// A genre link opens the catalog it names when that offers the genre, else the type's first
    /// catalog that does; a genre nobody offers opens no catalog.
    #[test]
    fn a_genre_link_finds_the_catalog_that_offers_it() {
        use sn_stremio::manifest::{Addon, Manifest};
        let addons = [Addon::new("https://catalog.example.com/manifest.json", Manifest::parse(LINK_CATALOG.as_bytes()).unwrap())];
        let all = crate::discover::catalogs(&addons);
        let to = |genre: &str, catalog: Option<&str>| GenreRef {
            kind: "movie".into(),
            genre: Some(genre.into()),
            addon_base: catalog.map(|_| "https://catalog.example.com/manifest.json".into()),
            catalog: catalog.map(str::to_string),
        };
        let id = |g: &GenreRef| crate::discover::genre_catalog(&all, &addons, g).map(|c| c.catalog.id.clone());
        assert_eq!(id(&to("Comedy", Some("fresh"))).as_deref(), Some("fresh"), "the named catalog");
        assert_eq!(id(&to("Action", Some("top"))).as_deref(), Some("fresh"), "the named one lacks it: the first that offers it");
        assert_eq!(id(&to("Drama", None)).as_deref(), Some("top"), "a plain genre: the type's first that offers it");
        assert_eq!(id(&to("Western", None)), None, "nobody offers it");
    }

    /// A movie page over the example addon, its meta (`cast` names) and one stream landed: focus is
    /// on the stream row. Its genre comes as a Cinemeta-style discover link.
    fn link_movie(cast: &[&str]) -> crate::test_support::TestApp {
        use crate::stores::detail::{DetailLanding, DetailStore};
        use sn_machine::machine::MachineId;
        use sn_stremio::manifest::{Addon, Manifest};
        let catalog = Addon::new("https://catalog.example.com/manifest.json", Manifest::parse(LINK_CATALOG.as_bytes()).unwrap());
        let streams = Addon::new(
            "https://streams.example.com/manifest.json",
            Manifest::parse(br#"{"id":"one","name":"One","resources":["stream"],"types":["movie"],"idPrefixes":["tt"]}"#).unwrap(),
        );
        let addons = [catalog.clone(), streams];
        let mut app = crate::test_support::TestApp::boot(crate::stores::catalog::CatalogStore::from_addons(vec![catalog]));
        let item = ItemRef { kind: "movie".into(), id: "tt1".into(), name: "Film".into(), from: String::new() };
        let meta_key = MetaKey { kind: "movie".into(), id: "tt1".into() };
        let stream_key = StreamKey { kind: "movie".into(), video_id: "tt1".into() };
        app.rig.detail = DetailStore::new();
        app.rig.detail.open_meta(&addons, meta_key.clone());
        app.rig.detail.open_streams(&addons, &Default::default(), stream_key.clone());
        app.d.request(MachineId::Nav, NavOp::Push(AppArg::Detail(item)));
        app.frames(20);
        let cast: Vec<String> = cast.iter().map(|c| format!("{c:?}")).collect();
        let meta = format!(
            r#"{{"id":"tt1","type":"movie","name":"Film","description":"A film.","cast":[{}],"director":["Cy Example"],"links":[
                {{"name":"Drama","category":"Genres","url":"stremio:///discover/https%3A%2F%2Fcatalog.example.com%2Fmanifest.json/movie/top?genre=Drama"}}]}}"#,
            cast.join(",")
        );
        app.rig.detail.land(DetailLanding::Meta { key: meta_key, result: Ok(serde_json::from_str(&meta).unwrap()) });
        let ready = GroupState::Ready(vec![http("1080p", "", "https://cdn.example.com/a.mkv")]);
        app.rig.detail.land(DetailLanding::Streams { key: stream_key, index: 0, state: ready });
        let gen = app.rig.detail.gen();
        app.notify(StoreId::Detail, gen);
        app.frames(20);
        app
    }

    /// The real dispatcher: the chips sit between the list and the library button. LEFT from the
    /// list still reaches the library button; UP from it climbs the groups (director, cast,
    /// genres) and stops at the top; DOWN walks back to the button; RIGHT from a chip returns to
    /// the list; LEFT/RIGHT walk a group's line.
    #[test]
    fn the_chips_are_reached_from_the_library_button_and_the_list() {
        use sn_machine::machine::Key;
        let _g = sn_base::testlock::serial();
        let _en = sn_platform::i18n::language_on_this_thread_for_test(sn_platform::i18n::Preference::En);
        let mut app = link_movie(&["Ann Example", "Bob Example"]);
        let focus = |app: &crate::test_support::TestApp| app.d.focus().map(|k| k.elem);
        assert_eq!(focus(&app), Some(STREAM_KEY), "the stream row is seated");
        assert!(app.drawn_text().iter().any(|t| *t == msg::app_detail_cast_label().to_uppercase()), "the CAST label is drawn, in caps");
        app.press(Key::Left);
        assert_eq!(focus(&app), Some(LIBRARY_KEY), "LEFT from the list: the library button, as before");
        app.press(Key::Up);
        assert_eq!(focus(&app), Some(link_key(2, 0)), "UP: the director's chip");
        app.press(Key::Up);
        assert_eq!(focus(&app), Some(link_key(1, 0)), "UP: the cast");
        app.press(Key::Right);
        assert_eq!(focus(&app), Some(link_key(1, 1)), "RIGHT along the line");
        app.press(Key::Up);
        assert_eq!(focus(&app), Some(link_key(0, 0)), "UP: the genres");
        app.press(Key::Up);
        assert_eq!(focus(&app), Some(link_key(0, 0)), "UP at the top stops");
        assert_eq!(app.top_name(), "detail");
        app.press(Key::Down);
        app.press(Key::Down);
        app.press(Key::Down);
        assert_eq!(focus(&app), Some(LIBRARY_KEY), "DOWN walks back to the button");
        app.press(Key::Up);
        app.press(Key::Right);
        assert_eq!(focus(&app), Some(STREAM_KEY), "RIGHT from a chip: the list");
        assert!(detail_page(&app).table.list_focused, "the row's highlight returns");
    }

    /// OK on a genre chip pushes Discover on that type, catalog and genre; BACK returns to the
    /// page with focus on the same chip.
    #[test]
    fn a_genre_chip_opens_discover_on_its_genre_and_back_restores_the_chip() {
        use sn_machine::machine::Key;
        let _g = sn_base::testlock::serial();
        let _en = sn_platform::i18n::language_on_this_thread_for_test(sn_platform::i18n::Preference::En);
        let mut app = link_movie(&["Ann Example"]);
        for k in [Key::Left, Key::Up, Key::Up, Key::Up] {
            app.press(k);
        }
        assert_eq!(app.d.focus().map(|k| k.elem), Some(link_key(0, 0)));
        app.take_effects();
        app.press(Key::Ok);
        let want = GenreRef { kind: "movie".into(), genre: Some("Drama".into()), addon_base: Some("https://catalog.example.com".into()), catalog: Some("top".into()) };
        assert_eq!(app.top_arg(), Some(&AppArg::DiscoverAt(want)));
        app.frames(10);
        assert_eq!(app.top_name(), "discover");
        let discover = app.d.top_screen().and_then(|s| s.as_any()).and_then(|a| a.downcast_ref::<crate::discover::DiscoverScreen>()).unwrap();
        let key = crate::stores::grid::GridKey { addon_id: "example.catalog".into(), kind: "movie".into(), catalog_id: "top".into(), genre: Some("Drama".into()), search: None };
        assert_eq!(discover.key(), Some(&key), "the catalog the link names, on its genre");
        assert!(app.take_effects().contains(&AppFx::Grid(crate::registry::GridCmd::Open(key))), "and asks for it");
        app.press(Key::Back);
        app.frames(10);
        assert_eq!(app.top_name(), "detail", "BACK returns to the page");
        assert_eq!(app.d.focus().map(|k| k.elem), Some(link_key(0, 0)), "focus on the same chip");
    }

    /// OK on a cast chip pushes Search with the name in the field, searched at once; BACK returns
    /// to the page on that chip. A click on the director's chip opens its search too.
    #[test]
    fn a_cast_chip_searches_its_name_and_back_restores_the_chip() {
        use crate::registry::SearchCmd;
        use sn_machine::machine::{InputKind, Key};
        let _g = sn_base::testlock::serial();
        let _en = sn_platform::i18n::language_on_this_thread_for_test(sn_platform::i18n::Preference::En);
        let mut app = link_movie(&["Ann Example", "Bob Example"]);
        for k in [Key::Left, Key::Up, Key::Up, Key::Right] {
            app.press(k);
        }
        assert_eq!(app.d.focus().map(|k| k.elem), Some(link_key(1, 1)));
        app.take_effects();
        app.press(Key::Ok);
        assert_eq!(app.top_arg(), Some(&AppArg::SearchFor("Bob Example".into())));
        app.frames(5);
        assert_eq!(app.top_name(), "search");
        assert!(app.take_effects().contains(&AppFx::Search(SearchCmd::Submit("Bob Example".into()))), "searched on arrival");
        let search = app.d.top_screen().and_then(|s| s.as_any()).and_then(|a| a.downcast_ref::<crate::search::SearchScreen>()).unwrap();
        assert_eq!(search.query(), "Bob Example", "in the field");
        app.press(Key::Back);
        app.frames(10);
        assert_eq!(app.top_name(), "detail", "BACK returns to the page");
        assert_eq!(app.d.focus().map(|k| k.elem), Some(link_key(1, 1)), "focus on the same chip");
        // the pointer: a click on the director's chip
        let at = detail_page(&app).link_places().iter().find(|p| p.group == 2).map(|p| p.chips[0]).unwrap();
        app.send(InputKind::Click { x: at.cx(), y: at.cy(), hit: None });
        app.frames(5);
        assert_eq!(app.top_arg(), Some(&AppArg::SearchFor("Cy Example".into())), "a click opens it");
        app.press(Key::Back);
        app.frames(10);
        assert_eq!(app.top_name(), "detail");
    }

    /// A cast too long for two lines ends in `+N`; focus reaching it shows the group whole and
    /// moves on to the first name it stood for; leaving the group folds it back.
    #[test]
    fn the_more_chip_expands_its_group_on_focus() {
        use sn_machine::machine::Key;
        let _g = sn_base::testlock::serial();
        let _en = sn_platform::i18n::language_on_this_thread_for_test(sn_platform::i18n::Preference::En);
        let cast: Vec<String> = (1..=14).map(|i| format!("Example Actor {i}")).collect();
        let mut app = link_movie(&cast.iter().map(String::as_str).collect::<Vec<_>>());
        let cast_of = |app: &crate::test_support::TestApp| detail_page(app).link_places().into_iter().find(|p| p.group == 1).unwrap();
        let folded = cast_of(&app);
        let (_, hidden) = folded.more.expect("+N");
        assert_eq!(folded.chips.len() + hidden, 14);
        for k in [Key::Left, Key::Up, Key::Up] {
            app.press(k);
        }
        assert_eq!(app.d.focus().and_then(|k| link_slot(k.elem)).map(|s| s.0), Some(1), "in the cast");
        // walk to the `+N` at the end of the last line
        for _ in 0..20 {
            if app.d.focus().map(|k| k.elem) == Some(link_key(1, MORE_SLOT)) || detail_page(&app).expanded.is_some() {
                break;
            }
            let line_end = app.d.focus().map(|k| k.elem);
            app.press(Key::Right);
            if app.d.focus().map(|k| k.elem) == line_end {
                app.press(Key::Down);
            }
        }
        app.frames(3);
        assert_eq!(detail_page(&app).expanded.map(|e| e.0), Some(1), "the group is open");
        assert_eq!(app.d.focus().map(|k| k.elem), Some(link_key(1, folded.chips.len() as u32)), "focus on the first name +N stood for");
        let open = cast_of(&app);
        assert!(open.chips.len() > folded.chips.len(), "more names show");
        app.press(Key::Right);
        app.press(Key::Right);
        app.press(Key::Right);
        app.press(Key::Right);
        app.press(Key::Right);
        // out of the group: DOWN to the director (from the last line)
        for _ in 0..8 {
            app.press(Key::Down);
        }
        app.frames(3);
        assert_eq!(detail_page(&app).expanded, None, "leaving the group folds it");
        assert_eq!(cast_of(&app), folded);
    }

    /// An episode's Streams page has no library button: LEFT from its stream list reaches the
    /// chips, RIGHT goes back.
    #[test]
    fn without_a_library_button_left_from_the_list_reaches_the_chips() {
        use crate::stores::detail::{DetailLanding, DetailStore};
        use sn_machine::machine::{Key, MachineId};
        use sn_stremio::manifest::{Addon, Manifest};
        let _g = sn_base::testlock::serial();
        let _en = sn_platform::i18n::language_on_this_thread_for_test(sn_platform::i18n::Preference::En);
        let meta_addon = Addon::new("https://catalog.example.com/manifest.json", Manifest::parse(LINK_CATALOG.as_bytes()).unwrap());
        let streams = Addon::new(
            "https://streams.example.com/manifest.json",
            Manifest::parse(br#"{"id":"one","name":"One","resources":["stream"],"types":["series"],"idPrefixes":["tt"]}"#).unwrap(),
        );
        let addons = [meta_addon.clone(), streams];
        let mut app = crate::test_support::TestApp::boot(crate::stores::catalog::CatalogStore::from_addons(vec![meta_addon]));
        let video = episode("tt0903747:2:1", "S2, E1 Seven Thirty-Seven");
        let meta_key = MetaKey { kind: "series".into(), id: "tt0903747".into() };
        let stream_key = StreamKey { kind: "series".into(), video_id: video.video_id.clone() };
        app.rig.detail = DetailStore::new();
        app.rig.detail.open_meta(&addons, meta_key.clone());
        app.rig.detail.open_streams(&addons, &Default::default(), stream_key.clone());
        app.d.request(MachineId::Nav, NavOp::Push(AppArg::Streams(video)));
        app.frames(20);
        app.rig.detail.land(DetailLanding::Meta { key: meta_key, result: Ok(fixture("cinemeta_breaking_bad_meta")) });
        let ready = GroupState::Ready(vec![http("1080p", "", "https://cdn.example.com/a.mkv")]);
        app.rig.detail.land(DetailLanding::Streams { key: stream_key, index: 0, state: ready });
        let gen = app.rig.detail.gen();
        app.notify(StoreId::Detail, gen);
        app.frames(20);
        assert_eq!(detail_page(&app).library_label(), None);
        assert_eq!(app.d.focus().map(|k| k.elem), Some(STREAM_KEY));
        app.press(Key::Left);
        assert!(app.d.focus().and_then(|k| link_slot(k.elem)).is_some(), "LEFT reaches a chip: {:?}", app.d.focus());
        assert_eq!(app.top_name(), "streams", "…and is not BACK");
        app.press(Key::Right);
        assert_eq!(app.d.focus().map(|k| k.elem), Some(STREAM_KEY), "RIGHT returns to the list");
    }

    /// A movie whose streams two addons answer: One with a torrent first and a link that plays,
    /// Two with a link. The page is the real dispatcher's.
    fn two_addon_movie() -> crate::test_support::TestApp {
        use crate::stores::detail::{DetailLanding, DetailStore};
        use sn_machine::machine::MachineId;
        use sn_stremio::manifest::{Addon, Manifest};
        let addon = |id: &str, name: &str| {
            let body = format!(r#"{{"id":"{id}","name":"{name}","resources":["stream"],"types":["movie"],"idPrefixes":["tt"]}}"#);
            Addon::new(&format!("https://{id}.example/manifest.json"), Manifest::parse(body.as_bytes()).unwrap())
        };
        let addons = [addon("one", "One"), addon("two", "Two")];
        let mut app = crate::test_support::TestApp::boot(crate::stores::catalog::CatalogStore::from_addons(addons.to_vec()));
        let stream_key = StreamKey { kind: "movie".into(), video_id: "tt1".into() };
        app.rig.detail = DetailStore::new();
        app.rig.detail.open_streams(&addons, &Default::default(), stream_key.clone());
        app.d.request(MachineId::Nav, NavOp::Push(AppArg::Detail(ItemRef { kind: "movie".into(), id: "tt1".into(), name: "Film".into(), from: String::new() })));
        app.frames(20);
        let torrent = Stream { info_hash: Some("ABC".into()), name: Some("Torrent".into()), title: Some("Film.Torrent".into()), ..Default::default() };
        let one = GroupState::Ready(vec![torrent, http("One 1080p", "Film.One", "https://cdn.example/one.mkv")]);
        app.rig.detail.land(DetailLanding::Streams { key: stream_key.clone(), index: 0, state: one });
        app.rig.detail.land(DetailLanding::Streams { key: stream_key, index: 1, state: GroupState::Ready(vec![http("Two 720p", "Film.Two", "https://cdn.example/two.mkv")]) });
        let gen = app.rig.detail.gen();
        app.notify(StoreId::Detail, gen);
        app.frames(20);
        app
    }

    /// What plays comes first; OK on what does not says why (the television's toast), never
    /// nothing.
    #[test]
    fn playable_streams_come_first_and_ok_on_a_torrent_says_why() {
        use sn_machine::machine::Key;
        let _g = sn_base::testlock::serial();
        let _en = sn_platform::i18n::language_on_this_thread_for_test(sn_platform::i18n::Preference::En);
        let mut app = two_addon_movie();
        let page = detail_page(&app);
        assert_eq!(page.sections()[0].rows.iter().map(|r| r.label.as_str()).collect::<Vec<_>>(), ["Film.One", "Film.Torrent"]);
        assert_eq!(app.d.focus().map(|k| k.elem), Some(STREAM_KEY + 1), "the stream that plays is the first seat");
        app.press(Key::Down);
        assert_eq!(app.d.focus().map(|k| k.elem), Some(STREAM_KEY), "the torrent is a stop");
        app.take_effects();
        app.press(Key::Ok);
        let effects = app.take_effects();
        assert!(effects.contains(&AppFx::Toast(msg::app_detail_why_torrent().into())), "{effects:?}");
        assert!(!effects.iter().any(|e| matches!(e, AppFx::Play(_))));
    }

    /// The addon dropdown over the list: UP from the first stream reaches it, it says which addons
    /// are asked, and its popover (under it) checks each one; OK switches an addon off for stream
    /// queries (the addons store's, persisted on the device) — its streams leave the list — and on
    /// again, which asks it again; the popover stays open, its checks following, and nothing is
    /// written to the account. DOWN returns to the list.
    #[test]
    fn the_addon_dropdown_switches_an_addons_stream_queries() {
        use crate::registry::{AddonsCmd, ChoiceTarget};
        use crate::stores::detail::DetailJob;
        use sn_machine::machine::Key;
        let _g = sn_base::testlock::serial();
        let _en = sn_platform::i18n::language_on_this_thread_for_test(sn_platform::i18n::Preference::En);
        let mut app = two_addon_movie();
        let headers = |app: &crate::test_support::TestApp| detail_page(app).sections().iter().map(|s| s.header.clone()).collect::<Vec<_>>();
        let checks = |app: &crate::test_support::TestApp| match crate::discover_tests::menu_of(app) {
            MenuKind::List { rows, target: ChoiceTarget::DetailStreamAddons, .. } => rows.into_iter().map(|r| (r.label, r.switch.map(|s| s.1))).collect::<Vec<_>>(),
            other => panic!("the addon dropdown: {other:?}"),
        };
        assert_eq!(headers(&app), ["One", "Two"]);
        assert!(app.drawn_text().iter().any(|t| t == "All addons"), "the button says what is asked");
        app.press(Key::Up);
        assert_eq!(app.d.focus().map(|k| k.elem), Some(ADDON_KEY), "UP from the list reaches the dropdown");
        app.press(Key::Ok);
        assert_eq!(app.top_name(), "menu");
        assert_eq!(checks(&app), [("One".to_string(), Some(true)), ("Two".to_string(), Some(true))]);
        let (anchor, panel) = crate::discover_tests::menu_panel(&app);
        assert!(panel.is_some_and(|p| p.y >= anchor.y + anchor.h), "under its button");
        app.press(Key::Down);
        app.take_effects();
        app.press(Key::Ok);
        let effects = app.take_effects();
        assert!(effects.contains(&AppFx::Addons(AddonsCmd::ToggleStreams("two".into()))), "{effects:?}");
        assert!(!effects.iter().any(|e| matches!(e, AppFx::Account(_) | AppFx::Library(_) | AppFx::Toast(_))), "{effects:?}");
        assert!(!app.rig.addons.stream_filter().allows("two"), "the switch is the addons store's (persisted)");
        app.frames(5);
        assert_eq!(app.top_name(), "menu", "the popover stays open");
        assert_eq!(checks(&app)[1], ("Two".to_string(), Some(false)), "and shows the switch");
        assert_eq!(headers(&app), ["One"], "an addon switched off is not listed");
        assert!(app.drawn_text().iter().any(|t| t == "One"), "the button names the one addon asked");
        // on again: the store asks it again
        app.press(Key::Ok);
        app.frames(5);
        assert!(app.rig.addons.stream_filter().allows("two"));
        assert_eq!(checks(&app)[1].1, Some(true));
        assert!(app.rig.detail_jobs.iter().any(|j| matches!(j, DetailJob::Streams { group, .. } if group.addon_id == "two")), "asked again");
        app.press(Key::Back);
        assert_eq!(app.top_name(), "detail");
        assert_eq!(app.d.focus().map(|k| k.elem), Some(ADDON_KEY), "focus returns to the button");
        app.press(Key::Down);
        assert!(app.d.focus().is_some_and(|k| k.elem >= STREAM_KEY && k.elem < NOTE_KEY), "DOWN returns to the list: {:?}", app.d.focus());
    }

    /// The button says which addons are asked: all, a count, or the one addon's name.
    #[test]
    fn the_addon_dropdown_says_which_addons_are_asked() {
        use sn_stremio::search::AddonFilter;
        let _en = sn_platform::i18n::language_on_this_thread_for_test(sn_platform::i18n::Preference::En);
        let groups = [group("A", GroupState::Loading), group("B", GroupState::Loading), group("C", GroupState::Loading)];
        let mut f = AddonFilter::default();
        assert_eq!(crate::dropdown::stream_addon_summary(&groups, &f), "All addons");
        f.toggle("a");
        assert_eq!(crate::dropdown::stream_addon_summary(&groups, &f), msg::app_search_addons_some(2, 3));
        f.toggle("b");
        assert_eq!(crate::dropdown::stream_addon_summary(&groups, &f), "C");
        assert_eq!(crate::dropdown::stream_addon_toggles(&groups, 1).as_deref(), Some("b"));
    }

    /// The addon dropdown's button stands inside the stream list's column in every shipped
    /// language, whatever it says (the longest summary, or one addon's long name, elides).
    #[test]
    fn the_addon_dropdown_stays_inside_the_stream_column() {
        use sn_platform::i18n::{language_on_this_thread_for_test, SHIPPED};
        let m = sn_base::fontcov::advances::ShippedMeasure;
        let f = detail_layout().sectioned_table();
        for language in SHIPPED {
            let _g = language_on_this_thread_for_test(language);
            let long = "A stream addon with a very long name that goes on and on and on and on and on and on";
            for label in [msg::app_search_addons_all().to_string(), msg::app_search_addons_some(12, 12), long.to_string()] {
                let labels = [CString::new(label.clone()).unwrap()];
                let widths = RefCell::new(None);
                let row = SeasonRow { labels: &labels, base: ADDON_KEY, group: ADDON_GROUP, chosen: None, entry: EntryId(1), frame: f, widths: &widths, dropdown: true };
                for focused in [None, Some(0)] {
                    let r = row.rects(&m, focused)[0];
                    assert!(r.x >= f.x && r.x + r.w <= f.x + f.w, "{}: {label:?} at {r:?} leaves {f:?}", language.tag());
                }
                if label != long {
                    let natural = crate::dropdown::face_w(&label, &m);
                    assert!(natural <= f.w.min(crate::dropdown::BUTTON_W_CAP), "{}: {label:?} is {natural}px", language.tag());
                }
            }
        }
    }

    /// A slower addon landing ABOVE the first seat: until the viewer moves, focus re-seats on the
    /// list's first row (the new one); once they moved, their row stays theirs, at its place on the
    /// screen.
    #[test]
    fn a_landing_above_reseats_the_first_row_until_the_viewer_moves() {
        use crate::stores::detail::{DetailLanding, DetailStore};
        use sn_machine::machine::{Key, MachineId};
        use sn_stremio::manifest::{Addon, Manifest};
        let _g = sn_base::testlock::serial();
        let addon = |id: &str| {
            let body = format!(r#"{{"id":"{id}","name":"{id}","resources":["stream"],"types":["movie"],"idPrefixes":["tt"]}}"#);
            Addon::new(&format!("https://{id}.example/manifest.json"), Manifest::parse(body.as_bytes()).unwrap())
        };
        let addons = [addon("a"), addon("b"), addon("c")];
        let mut app = crate::test_support::TestApp::boot(crate::stores::catalog::CatalogStore::from_addons(addons.to_vec()));
        let key = StreamKey { kind: "movie".into(), video_id: "tt1".into() };
        app.rig.detail = DetailStore::new();
        app.rig.detail.open_streams(&addons, &Default::default(), key.clone());
        app.d.request(MachineId::Nav, NavOp::Push(AppArg::Detail(ItemRef { kind: "movie".into(), id: "tt1".into(), name: "Film".into(), from: String::new() })));
        app.frames(20);
        let land = |app: &mut crate::test_support::TestApp, index: usize, n: usize| {
            let streams = (0..n).map(|i| http(&format!("S{index}{i}"), "", &format!("https://cdn.example/{index}/{i}.mkv"))).collect();
            app.rig.detail.land(DetailLanding::Streams { key: key.clone(), index, state: GroupState::Ready(streams) });
            let gen = app.rig.detail.gen();
            app.notify(StoreId::Detail, gen);
            app.frames(20);
        };
        land(&mut app, 2, 12);
        assert_eq!(app.d.focus().map(|k| k.elem), Some(STREAM_KEY + (2 << 12)), "c's first stream takes the seat");
        land(&mut app, 1, 2);
        assert_eq!(app.d.focus().map(|k| k.elem), Some(STREAM_KEY + (1 << 12)), "b landed above: its first row is the seat now");
        app.press(Key::Down);
        let mine = app.d.focus().map(|k| k.elem);
        assert_eq!(mine, Some(STREAM_KEY + (1 << 12) + 1));
        let y = |app: &crate::test_support::TestApp| app.d.last_stops().iter().find(|s| Some(s.key.elem) == mine).map(|s| s.rect.y);
        let before = y(&app);
        land(&mut app, 0, 4);
        assert_eq!(app.d.focus().map(|k| k.elem), mine, "moved: the viewer's row stays focused");
        let after = y(&app);
        assert!(before.zip(after).is_some_and(|(b, a)| (a - b).abs() < 1.0), "and stays at its place on the screen: {before:?} -> {after:?}");
    }

    /// The facts line speaks the UI language: the rating's decimal mark and an episode's air day.
    #[test]
    fn the_facts_line_is_localised() {
        let _fr = sn_platform::i18n::language_on_this_thread_for_test(sn_platform::i18n::Preference::Fr);
        let mut movie = fixture("cinemeta_shawshank");
        movie.preview.imdb_rating = Some("7.5".into());
        assert!(info(&movie_mode(), &movie).facts.ends_with("IMDb 7,5"), "{}", info(&movie_mode(), &movie).facts);
        let show = fixture("cinemeta_breaking_bad_meta");
        let ep = info(&Mode::Video(episode("tt0903747:1:5", "")), &show);
        assert!(ep.facts.starts_with("25/02/2008"), "{}", ep.facts);
    }
}
