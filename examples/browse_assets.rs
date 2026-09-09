use anyhow::Result;
use byteorder::{LittleEndian, ReadBytesExt};
use crossterm::{
    event::{self, DisableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    prelude::*,
    widgets::{
        Block, Borders, List, ListItem, ListState, Paragraph, Scrollbar, ScrollbarOrientation,
        ScrollbarState, Tabs, Wrap,
    },
};
use regnumassets::{AssetBookmark, AssetContent, AssetData, AssetType, ResourceIndex};
use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
    path::{Path, PathBuf},
};

/// Maximum amount of payload bytes shown in the hexdump pane
const HEXDUMP_MAX_BYTES: u64 = 1024;

const ALL_TYPES: [Option<AssetType>; 17] = [
    None,
    Some(AssetType::Material),
    Some(AssetType::Animation),
    Some(AssetType::Mesh),
    Some(AssetType::Image),
    Some(AssetType::Text),
    Some(AssetType::Binary),
    Some(AssetType::Texture),
    Some(AssetType::Font),
    Some(AssetType::Effect),
    Some(AssetType::Music),
    Some(AssetType::Sound),
    Some(AssetType::Character),
    Some(AssetType::Auth),
    Some(AssetType::MapObject),
    Some(AssetType::TerrainRegion),
    Some(AssetType::WorldMap),
];

fn type_label(filter: &Option<AssetType>) -> String {
    match filter {
        None => "All".into(),
        Some(t) => {
            let s: &str = t.clone().into();
            s.to_string()
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Focus {
    List,
    Hexdump,
}

#[derive(Clone)]
struct Hexdump {
    /// raw payload bytes (may be capped to HEXDUMP_MAX_BYTES)
    bytes: Vec<u8>,
    /// file offset of bytes[0]
    base: u64,
    /// full payload size
    total: u32,
    /// first visible row; clamped to [0, max_scroll]
    scroll: usize,
    /// highest scroll position (data_rows - visible), refreshed on
    /// every frame by the renderer since it depends on pane size
    max_scroll: usize,
}

struct Database {
    sdb_path: PathBuf,
}

struct Entry {
    bookmark: AssetBookmark,
    db: usize,
}

struct App {
    databases: Vec<Database>,
    entries: Vec<Entry>,
    filter_index: usize,
    list_state: ListState,
    focus: Focus,
    detail: Option<Vec<Line<'static>>>,
    hexdump: Option<Result<Hexdump, String>>,
    status: String,
}

impl App {
    fn load(dir: &Path) -> Result<Self> {
        let mut paths: Vec<PathBuf> = std::fs::read_dir(dir)?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().map(|ext| ext == "idx").unwrap_or(false))
            .collect();
        paths.sort();

        if paths.is_empty() {
            anyhow::bail!("no index files found in {}", dir.display());
        }

        let mut databases = vec![];
        let mut entries = vec![];
        for idx_path in paths {
            let sdb_path = idx_path.with_extension("sdb");
            if !sdb_path.exists() {
                continue;
            }
            let index = ResourceIndex::read(File::open(&idx_path)?)?;
            for bookmark in &index.bookmarks {
                entries.push(Entry {
                    bookmark: bookmark.clone(),
                    db: databases.len(),
                });
            }
            databases.push(Database { sdb_path });
        }

        let status = format!("loaded {} assets", entries.len());
        let mut app = App {
            databases,
            entries,
            filter_index: 0,
            list_state: ListState::default(),
            focus: Focus::List,
            detail: None,
            hexdump: None,
            status,
        };
        app.apply_filter();
        Ok(app)
    }

    fn filtered(&self) -> Vec<&Entry> {
        let filter = &ALL_TYPES[self.filter_index];
        self.entries
            .iter()
            .filter(|e| filter.as_ref().map_or(true, |t| e.bookmark.asset_type == *t))
            .collect()
    }

    fn apply_filter(&mut self) {
        let len = self.filtered().len();
        self.list_state = ListState::default();
        if len > 0 {
            self.list_state.select(Some(0));
        }
        self.detail = None;
        self.hexdump = None;
        self.focus = Focus::List;
        self.status = format!("{} assets ({} shown)", self.entries.len(), len);
    }

    fn selected(&self) -> Option<&Entry> {
        self.list_state
            .selected()
            .and_then(|i| self.filtered().get(i).copied())
    }

    fn move_cursor(&mut self, delta: isize) {
        let len = self.filtered().len();
        if len == 0 {
            return;
        }
        let current = self.list_state.selected().unwrap_or(0) as isize;
        let next = (current + delta).clamp(0, len as isize - 1);
        self.list_state.select(Some(next as usize));
        self.detail = None;
        self.hexdump = None;
    }

    fn toggle_focus(&mut self) {
        if self.hexdump.is_some() {
            self.focus = match self.focus {
                Focus::List => Focus::Hexdump,
                Focus::Hexdump => Focus::List,
            };
        }
    }

    fn scroll_hexdump(&mut self, delta: isize) {
        if let Some(Ok(hex)) = self.hexdump.as_mut() {
            let next = hex.scroll as isize + delta;
            hex.scroll = next.clamp(0, hex.max_scroll as isize) as usize;
        }
    }

    fn read_selected(&mut self) {
        let Some(entry) = self.selected() else {
            return;
        };
        let bookmark = entry.bookmark.clone();
        let closure_bookmark = bookmark.clone();
        let db_path = self.databases[entry.db].sdb_path.clone();
        let closure_db_path = db_path.clone();

        // the asset parsers use assertions internally, so a badly-parsed
        // asset may panic; catch it instead of taking the whole TUI down
        let default_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(|_| {}));
        let result: Result<AssetData> = std::panic::catch_unwind(move || {
            File::open(&closure_db_path)
                .map_err(anyhow::Error::from)
                .and_then(|f| AssetData::read(f, &closure_bookmark))
        })
        .unwrap_or_else(|_| Err(anyhow::anyhow!("the asset parser panicked")));
        std::panic::set_hook(default_hook);

        // hexdump does not need the parser; read the raw payload directly
        self.hexdump = Some(
            read_payload_head(&db_path, &bookmark).map_err(|e| e.to_string()),
        );

        self.status = if result.is_ok() { "asset read".into() } else { "asset read failed".into() };
        let lines: Vec<Line<'static>> = match result {
            Err(e) => vec![
                Line::from("Failed to read asset".bold().red()),
                Line::from(e.to_string()),
            ],
            Ok(data) => {
                let asset_type: &str = data.asset_type.clone().into();
                vec![
                Line::from("Details".bold()),
                Line::from(format!("resource id: {}", data.resource_id)),
                Line::from(format!("asset type:  {}", asset_type)),
                Line::from(format!("asset name:  {}", data.asset_name)),
                Line::from(format!("uid:         {}", data.uid)),
                Line::from(format!("size:        {}", format_size(bookmark.size))),
                Line::from(""),
                Line::from("Content".bold()),
                Line::from(describe_content(&data.content)),
            ]
            }
        };
        self.detail = Some(lines);
    }

    fn next_filter(&mut self, delta: isize) {
        let len = ALL_TYPES.len() as isize;
        self.filter_index = ((self.filter_index as isize + delta).rem_euclid(len)) as usize;
        self.apply_filter();
    }
}

/// Reads the first HEXDUMP_MAX_BYTES of an asset payload directly from the
/// database file, walking the asset node header without using the parser.
fn read_payload_head(db_path: &Path, bookmark: &AssetBookmark) -> Result<Hexdump> {
    fn skip<T: Read>(reader: &mut T, n: u64) -> Result<()> {
        std::io::copy(&mut reader.by_ref().take(n), &mut std::io::sink())?;
        Ok(())
    }

    let mut reader = File::open(db_path)?;
    reader.seek(SeekFrom::Start(bookmark.node_end as u64))?;

    // node marker
    let mut marker = [0u8; 4];
    reader.read_exact(&mut marker)?;
    if &marker != b"PAIR" {
        anyhow::bail!("unexpected node marker");
    }

    skip(&mut reader, 4)?; // unknown u32
    let uid_length = reader.read_u8()?;
    skip(&mut reader, 16)?; // unknown
    skip(&mut reader, uid_length as u64)?; // uid
    let resource_name_length = reader.read_u8()?;
    skip(&mut reader, resource_name_length as u64)?; // resource name
    skip(&mut reader, 4)?; // separator
    skip(&mut reader, 4)?; // payload size (we already know it)
    skip(&mut reader, 16)?; // unknown
    skip(&mut reader, 4)?; // unknown u32
    skip(&mut reader, 4)?; // resource id
    skip(&mut reader, 4)?; // unknown u32
    let asset_type_length = reader.read_u32::<LittleEndian>()?;
    skip(&mut reader, asset_type_length as u64)?; // asset type
    let asset_name_length = reader.read_u32::<LittleEndian>()?;
    skip(&mut reader, asset_name_length as u64)?; // asset name
    skip(&mut reader, 16)?; // unknown

    let base = reader.stream_position()?;
    let to_read = (bookmark.size as u64).min(HEXDUMP_MAX_BYTES);
    let mut bytes = Vec::with_capacity(to_read as usize);
    reader
        .by_ref()
        .take(to_read)
        .read_to_end(&mut bytes)?;

    if bytes.is_empty() {
        anyhow::bail!("empty payload");
    }

    Ok(Hexdump {
        bytes,
        base,
        total: bookmark.size,
        scroll: 0,
        max_scroll: 0,
    })
}

fn describe_content(content: &AssetContent) -> String {
    match content {
        AssetContent::Sound { filename, size, .. } => {
            format!("Ogg Vorbis: '{}' ({} bytes)", filename, size)
        }
        AssetContent::Texture { width, height, .. } => {
            format!("DDS texture: {}x{}", width, height)
        }
        AssetContent::Text { contents } => {
            format!("Text asset: {} component(s)", contents.len())
        }
        AssetContent::Image { bytes } => {
            format!("JPEG image ({} bytes)", bytes.len())
        }
        AssetContent::Font(font) => match font {
            regnumassets::asset::font::Font::FontBlob { height, .. } => {
                format!("Game font blob (height {})", height)
            }
            regnumassets::asset::font::Font::TrueType(bytes) => {
                format!("TrueType font ({} bytes)", bytes.len())
            }
            regnumassets::asset::font::Font::OpenType(bytes) => {
                format!("OpenType font, CFF outlines ({} bytes)", bytes.len())
            }
            regnumassets::asset::font::Font::FontCollection { faces, .. } => {
                format!("TrueType collection ({} faces)", faces)
            }
        },
        AssetContent::NotSupported => "Content not supported by this crate".into(),
    }
}

fn format_size(size: u32) -> String {
    if size >= 1024 * 1024 {
        format!("{:.1} MiB", size as f64 / (1024.0 * 1024.0))
    } else if size >= 1024 {
        format!("{:.1} KiB", size as f64 / 1024.0)
    } else {
        format!("{} bytes", size)
    }
}

fn type_tabs<'a>(app: &'a App) -> Tabs<'a> {
    let labels: Vec<String> = ALL_TYPES
        .iter()
        .map(|filter| {
            let count = app
                .entries
                .iter()
                .filter(|e| {
                    filter
                        .as_ref()
                        .map_or(true, |t| e.bookmark.asset_type == *t)
                })
                .count();
            format!("{} ({})", type_label(filter), count)
        })
        .collect();

    Tabs::new(labels)
        .block(Block::default().borders(Borders::ALL).title(" Asset type "))
        .select(app.filter_index)
        .highlight_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
}

/// The three hexdump columns, rendered as borderless sub-panes of fixed
/// width so every column stays aligned regardless of row length.
struct HexdumpColumns {
    offsets: Vec<Line<'static>>,
    hex: Vec<Line<'static>>,
    ascii: Vec<Line<'static>>,
    /// bytes shown per row; the hex sub-pane is `3 * bytes_per_row` wide
    bytes_per_row: usize,
    /// total data rows in the payload (regardless of what is visible)
    data_rows: usize,
    /// highest scroll position (data_rows - visible)
    max_scroll: usize,
    /// first visible row after clamping to the pane height
    scroll: usize,
}

fn hexdump_columns(hex: &Hexdump, width: u16, height: u16) -> HexdumpColumns {
    // bytes per row: 8 offset chars + 2 spaces, then 'XX ' per byte
    let bytes_per_row = ((width.saturating_sub(12)) / 3).clamp(1, 16) as usize;
    let data_rows = hex.bytes.len().div_ceil(bytes_per_row);

    // clamp scroll to what fits right now
    let visible = height as usize;
    let max_scroll = data_rows.saturating_sub(visible);
    let scroll = hex.scroll.min(max_scroll);

    let mut columns = HexdumpColumns {
        offsets: vec![],
        hex: vec![],
        ascii: vec![],
        bytes_per_row,
        data_rows,
        max_scroll,
        scroll,
    };
    for row in scroll..(scroll + visible).min(data_rows) {
        let start = row * bytes_per_row;
        let chunk = &hex.bytes[start..(start + bytes_per_row).min(hex.bytes.len())];

        let mut hex_part = String::with_capacity(bytes_per_row * 3);
        let mut ascii_part = String::with_capacity(bytes_per_row);
        for b in chunk {
            hex_part.push_str(&format!("{:02X} ", b));
            if b.is_ascii_graphic() || *b == b' ' {
                ascii_part.push(*b as char);
            } else {
                ascii_part.push('·');
            }
        }

        let offset = hex.base + start as u64;
        columns
            .offsets
            .push(Line::from(format!("{:08X}  ", offset)).fg(Color::DarkGray));
        columns.hex.push(Line::from(hex_part).fg(Color::Cyan));
        columns.ascii.push(Line::from(ascii_part).fg(Color::Gray));
    }

    columns
}

fn ui(f: &mut Frame, app: &mut App) {
    let main = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(1),
        ])
        .split(f.area());

    f.render_widget(type_tabs(app), main[0]);

    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
        .split(main[1]);

    // right column: hexdump on top, details below
    let right = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(body[1]);

    let focused = Style::default().fg(Color::Cyan);

    let items: Vec<ListItem> = app
        .filtered()
        .into_iter()
        .map(|e| {
            let bookmark = &e.bookmark;
            ListItem::new(Line::from(format!(
                "#{:<8} {:<24} {}",
                bookmark.resource_id.unwrap_or(0),
                truncate(bookmark.name.as_deref().unwrap_or("(unnamed)"), 24),
                format_size(bookmark.size),
            )))
        })
        .collect();

    let list_block = Block::default()
        .borders(Borders::ALL)
        .title(" Assets ")
        .border_style(if app.focus == Focus::List { focused } else { Style::default() });
    let list = List::new(items)
        .block(list_block)
        .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
        .highlight_symbol("> ");
    f.render_stateful_widget(list, body[0], &mut app.list_state);

    let hex_title = match app.hexdump.as_ref() {
        Some(Ok(hex)) => format!(
            " Hexdump 0x{:08X}-0x{:08X} / {} ",
            hex.base,
            hex.base + hex.bytes.len() as u64 - 1,
            format_size(hex.total)
        ),
        _ => " Hexdump ".to_string(),
    };

    let hex_block = Block::default()
        .borders(Borders::ALL)
        .title(hex_title)
        .border_style(if app.focus == Focus::Hexdump { focused } else { Style::default() });
    let hex_inner = hex_block.inner(right[0]);
    f.render_widget(hex_block, right[0]);

    // the truncated-payload notice gets its own bottom row so it stays
    // pinned and full-width instead of scrolling with the data
    let has_footer = matches!(
        app.hexdump.as_ref(),
        Some(Ok(hex)) if hex.total as u64 > HEXDUMP_MAX_BYTES
    );
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(u16::from(has_footer)),
        ])
        .split(hex_inner);

    let view = match app.hexdump.as_ref() {
        Some(Ok(hex)) => Some(hexdump_columns(hex, hex_inner.width, rows[0].height)),
        _ => None,
    };

    // write back the effective scroll position and bound so key handling
    // clamps against the current pane size; this also snaps a stale
    // scroll value after a terminal resize
    if let (Some(Ok(hex)), Some(view)) = (app.hexdump.as_mut(), view.as_ref()) {
        hex.max_scroll = view.max_scroll;
        hex.scroll = view.scroll;
    }

    match app.hexdump.as_ref() {
        None => {
            let p = Paragraph::new(Line::from("no asset loaded").fg(Color::DarkGray));
            f.render_widget(p, hex_inner);
        }
        Some(Err(e)) => {
            let lines = vec![
                Line::from("could not read payload".bold().red()),
                Line::from(e.clone()),
            ];
            f.render_widget(Paragraph::new(lines), hex_inner);
        }
        Some(Ok(hex)) => {
            let view = view.expect("hexdump view present for a loaded asset");
            let cols = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Length(10),
                    Constraint::Length(view.bytes_per_row as u16 * 3),
                    Constraint::Min(0),
                ])
                .split(rows[0]);
            f.render_widget(Paragraph::new(view.offsets), cols[0]);
            f.render_widget(Paragraph::new(view.hex), cols[1]);
            f.render_widget(Paragraph::new(view.ascii), cols[2]);

            // vertical scrollbar on the pane's right edge, only when the
            // payload has more rows than fit on screen; sits on the data
            // area so it never covers the truncated-payload footer. The
            // content length is the maximum scroll position plus one so
            // the thumb touches the track bottom at max scroll (the pane
            // does not overscroll past the last row)
            if view.max_scroll > 0 && rows[0].height > 0 {
                let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                    .begin_symbol(None)
                    .end_symbol(None);
                let mut state = ScrollbarState::new(view.max_scroll + 1).position(view.scroll);
                f.render_stateful_widget(scrollbar, rows[0], &mut state);
            }

            if has_footer {
                let footer = Line::from(format!(
                    "… first {} of {} bytes shown",
                    hex.bytes.len(),
                    hex.total
                ))
                .fg(Color::DarkGray);
                f.render_widget(Paragraph::new(footer), rows[1]);
            }
        }
    }

    let detail_text = app.detail.clone().unwrap_or_else(|| {
        vec![Line::from(vec![
            "Press ".into(),
            "Enter".bold(),
            " to read the selected asset".into(),
        ])]
    });

    let detail = Paragraph::new(detail_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Detail ")
                .border_style(Style::default()),
        )
        .wrap(Wrap { trim: false });
    f.render_widget(detail, right[1]);

    let help = Line::from(vec![
        "←/→ filter type  ↑/↓ navigate  Enter read asset  Tab focus  q quit ".fg(Color::DarkGray),
    ]);
    f.render_widget(Paragraph::new(help), main[2]);
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let mut out: String = s.chars().take(max - 1).collect();
        out.push('…');
        out
    }
}

fn restore_terminal() {
    let _ = disable_raw_mode();
    let _ = execute!(std::io::stdout(), LeaveAlternateScreen, DisableMouseCapture);
}

fn main() -> Result<()> {
    let dir = std::env::args().nth(1).unwrap_or_else(|| "examples/regnum".into());
    let app = App::load(Path::new(&dir));

    // build UI before entering raw mode so load errors show up normally
    let mut app = match app {
        Ok(app) => app,
        Err(e) => {
            eprintln!("failed to load assets: {}", e);
            std::process::exit(1);
        }
    };

    enable_raw_mode()?;
    execute!(std::io::stdout(), EnterAlternateScreen)?;
    // restore the terminal even if the app panics
    let guard = TerminalGuard;

    let backend = CrosstermBackend::new(std::io::stdout());
    let mut terminal = Terminal::new(backend)?;

    let mut quit = false;
    while !quit {
        terminal.draw(|f| ui(f, &mut app))?;

        if !event::poll(std::time::Duration::from_millis(100))? {
            continue;
        }

        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => quit = true,
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    quit = true
                }
                KeyCode::Tab => app.toggle_focus(),
                KeyCode::Left | KeyCode::Char('h') => app.next_filter(-1),
                KeyCode::Right | KeyCode::Char('l') => app.next_filter(1),
                KeyCode::Up | KeyCode::Char('k') => match app.focus {
                    Focus::List => app.move_cursor(-1),
                    Focus::Hexdump => app.scroll_hexdump(-1),
                },
                KeyCode::Down | KeyCode::Char('j') => match app.focus {
                    Focus::List => app.move_cursor(1),
                    Focus::Hexdump => app.scroll_hexdump(1),
                },
                KeyCode::PageUp => match app.focus {
                    Focus::List => app.move_cursor(-10),
                    Focus::Hexdump => app.scroll_hexdump(-10),
                },
                KeyCode::PageDown => match app.focus {
                    Focus::List => app.move_cursor(10),
                    Focus::Hexdump => app.scroll_hexdump(10),
                },
                KeyCode::Enter | KeyCode::Char(' ') => app.read_selected(),
                _ => {}
            }
        }
    }

    drop(guard);
    terminal.show_cursor()?;
    Ok(())
}

/// Restores the terminal when dropped (also on panic via unwind).
struct TerminalGuard;

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        restore_terminal();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ascii_gutter_starts_at_fixed_column_in_rendered_buffer() {
        // 201 bytes -> 12 full rows of 16 + one partial row of 9
        let hex = Hexdump {
            bytes: (0..=200).collect(),
            base: 0x11A4_F03E,
            total: 201,
            scroll: 0,
            max_scroll: 0,
        };
        let area = Rect::new(0, 0, 75, 15);
        let view = hexdump_columns(&hex, area.width, area.height);
        assert_eq!(view.bytes_per_row, 16);
        assert_eq!(view.offsets.len(), 13);

        // render the three columns into a buffer using the same layout
        // split as the hexdump pane
        let mut buf = Buffer::empty(area);
        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(10),
                Constraint::Length(view.bytes_per_row as u16 * 3),
                Constraint::Min(0),
            ])
            .split(area);
        Paragraph::new(view.offsets.clone()).render(cols[0], &mut buf);
        Paragraph::new(view.hex.clone()).render(cols[1], &mut buf);
        Paragraph::new(view.ascii.clone()).render(cols[2], &mut buf);

        // no ascii content may bleed into the hex pane's blank area:
        // between the end of the hex text and the gutter's left edge
        // every cell must stay blank, on every row including the
        // partial last one
        let total = hex.bytes.len();
        for y in 0..view.offsets.len() {
            let chunk_len = view.bytes_per_row.min(total - y * view.bytes_per_row);
            let hex_end = 10 + chunk_len * 3;
            for x in hex_end..cols[2].x as usize {
                let cell = buf.get(x as u16, y as u16);
                assert_eq!(
                    cell.symbol(),
                    " ",
                    "row {y}: non-blank cell at column {x} between hex text and gutter"
                );
            }
        }

        // the scroll reported for the scrollbar stays within the
        // content, even when the app state runs past the end
        assert_eq!(view.scroll, 0);
        let mut hex = hex.clone();
        hex.scroll = 999;

        // pane shorter than the content: scroll clamps to max_scroll
        let view = hexdump_columns(&hex, area.width, 8);
        assert_eq!(view.max_scroll, view.data_rows - 8);
        assert_eq!(view.scroll, view.max_scroll);

        // at max scroll the scrollbar thumb must touch the track bottom
        let mut sb_buf = Buffer::empty(area);
        let mut state = ScrollbarState::new(view.max_scroll + 1).position(view.scroll);
        Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(None)
            .end_symbol(None)
            .render(area, &mut sb_buf, &mut state);
        let bottom = sb_buf.get(area.width - 1, area.height - 1);
        assert_eq!(bottom.symbol(), "█", "thumb must reach the track bottom");

        // pane taller than the content: nothing to scroll
        let view = hexdump_columns(&hex, area.width, area.height);
        assert_eq!(view.max_scroll, 0);
        assert_eq!(view.scroll, 0);
    }
}
