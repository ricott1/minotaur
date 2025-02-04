use super::utils::{img_to_lines, RataColor};
use crate::{
    game::{Entity, Game, GameColors, UiOptions},
    AppResult, PlayerId,
};
use anyhow::anyhow;
use image::{Pixel, Rgb};
use itertools::Itertools;
use ratatui::{
    layout::{Constraint, Layout, Margin, Rect},
    style::{Color, Style, Styled},
    symbols::border,
    text::{Line, Span},
    widgets::{Block, BorderType, Paragraph},
    Frame,
};
use std::time::{Duration, Instant};

const MINORADAR: [&'static str; 8] = ["▁", "▂", "▃", "▄", "▅", "▆", "▇", "█"];

const TITLE: [&'static str; 29] = [
    "     ██▓    ▄▄▄          ▄████▄   ▄▄▄        ██████  ▄▄▄            ",
    "     ▓██▒   ▒████▄       ▒██▀  ▀  ▒████▄    ▒██    ▒ ▒████▄         ",
    "     ▒██░   ▒██  ▀█▄     ▒▓█      ▒██  ▀█▄  ░ ▓██▄   ▒██  ▀█▄       ",
    "     ▒██░   ░██▄▄▄▄██    ▒▓▓▄  ▄ ▒░██▄▄▄▄██   ░   ██▒░██▄▄▄▄██      ",
    "     ░██████▒▓█   ▓██▒   ▒ ▓███▀ ░ ▓█   ▓██▒▒██████▒▒ ▓█   ▓██▒     ",
    "     ░ ▒░▓  ░▒▒   ▓▒█░   ░ ░▒ ▒  ░ ▒▒   ▓▒█░▒ ▒▓▒ ▒ ░ ▒▒   ▓▒█░     ",
    "     ░ ░ ▒  ░ ▒   ▒▒ ░     ░  ▒     ▒   ▒▒ ░░ ░▒  ░ ░  ▒   ▒▒ ░     ",
    "       ░ ░    ░   ▒      ░          ░   ▒   ░  ░  ░    ░   ▒        ",
    "         ░  ░     ░  ░   ░ ░            ░  ░      ░        ░  ░     ",
    "                 ░                                                  ",
    "                         ▓█████▄ ▓████▒                             ",
    "                         ▒██▀ ██▌▓█   ▀                             ",
    "                         ░██   █▌▒███                               ",
    "                         ░▓█▄   ▌▒▓█  ▄                             ",
    "                         ░▒████▓ ░▒████▒                            ",
    "                          ▒▒▓  ▒ ░░ ▒░ ░                            ",
    "                          ░ ▒  ▒  ░ ░  ░                            ",
    "                          ░ ░  ░    ░                               ",
    "                            ░       ░  ░             ▄█▓            ",
    "                          ░                         ▀▀▒░            ",
    "▄▄▄        ██████ ▄▄▄█████▓▓█████  ██▀███    █▓  ▒▓███░  ███▄    █  ",
    "▒████▄    ▒██    ▒ ▓  ██▒ ▓▒▓█   ▀ ▓██   ██ ░██ ▒██▒  ██▒ ██ ▀█   █ ",
    "▒██  ▀█▄  ░ ▓██▄   ▒ ▓██░ ▒░▒███   ▓██ ░▄█  ▒██ ▒██░  ██▒▓██  ▀█ ██▒",
    "░██▄▄▄▄██   ▒   ██▒░ ▓██▓ ░ ▒▓█  ▄ ▒██▀▀█▄  ░██░▒██   ██░▓██▒  ▐▌██▒",
    " ▓█   ▓██▒▒██████▒▒  ▒██▒ ░ ░▒████▒░██▓ ▒██ ░██░  ████▓▒░▒██░   ▓██░",
    " ▒▒   ▓▒█░▒ ▒▓▒ ▒ ░  ▒ ░░   ░░ ▒░ ░░ ▒▓ ░▒▓░░▓  ░ ▒░▒░▒░ ░ ▒░   ▒ ▒ ",
    "  ▒   ▒▒ ░░ ░▒  ░ ░    ░     ░ ░  ░  ░▒ ░ ▒░ ▒ ░  ░ ▒ ▒░ ░ ░░   ░ ▒░",
    "  ░   ▒   ░  ░  ░    ░         ░     ░░   ░  ▒ ░░ ░ ░ ▒     ░   ░ ░ ",
    "      ░  ░      ░              ░  ░   ░      ░      ░ ░           ░ ",
];

fn title_paragraph<'a>() -> Paragraph<'a> {
    let lines = TITLE
        .iter()
        .map(|line| {
            let mut spans = vec![];
            for c in line.chars() {
                // █ ▓ ▒ ░
                if c == '█' {
                    spans.push(Span::styled("█", Color::Rgb(138, 3, 3)));
                } else if c == '▀' {
                    spans.push(Span::styled("▀", Color::Rgb(138, 3, 3)));
                } else if c == '▄' {
                    spans.push(Span::styled("▄", Color::Rgb(138, 3, 3)));
                } else if c == '▌' {
                    spans.push(Span::styled("▌", Color::Rgb(138, 3, 3)));
                } else if c == '▐' {
                    spans.push(Span::styled("▐", Color::Rgb(138, 3, 3)));
                } else if c == '▓' {
                    spans.push(Span::styled("▓", Color::Rgb(118, 3, 3)));
                } else if c == '▒' {
                    spans.push(Span::styled("▒", Color::Rgb(98, 2, 2)));
                } else if c == '░' {
                    spans.push(Span::styled("░", Color::Rgb(78, 0, 0)));
                } else {
                    spans.push(Span::styled(c.to_string(), Color::Rgb(255, 255, 255)));
                }
            }
            Line::from(spans)
        })
        .collect::<Vec<Line>>();
    Paragraph::new(lines).centered()
}

fn render_header(frame: &mut Frame, game: &Game, player_id: PlayerId, area: Rect) -> AppResult<()> {
    let number_of_players = game.number_of_players();

    let mut lines = vec![
        Line::from(format!(
            "There {} {} hero{} in the labirynth...",
            if number_of_players == 1 { "is" } else { "are" },
            number_of_players,
            if number_of_players == 1 { "" } else { "es" }
        )),
        Line::default(),
    ];

    if let Some(hero) = game.get_hero(&player_id) {
        let num_minotaurs = game.minotaurs_in_maze(hero.maze_id());

        let alarm_level = game.alarm_level(&hero.id());

        let minoradar: String = MINORADAR.iter().take(alarm_level).map(|s| *s).collect();

        lines.push(Line::from(vec![
            Span::styled(format!("{} - ", hero.name()), GameColors::HERO.to_color()),
            Span::raw(format!(
                "Room {}@{:?} - ",
                hero.maze_id() + 1,
                hero.position()
            )),
            Span::raw(format!(
                "Power up {}collected",
                if let Some(power_up) = hero.power_up_collected_in_maze() {
                    format!("({})", power_up)
                } else {
                    "not ".to_string()
                }
            )),
        ]));

        lines.push(Line::from(vec![
            Span::raw(format!(
                "{} minotaur{}  ",
                num_minotaurs,
                if num_minotaurs == 1 { "" } else { "s" }
            )),
            Span::raw(format!("{}", minoradar)),
        ]));
    }

    frame.render_widget(Paragraph::new(lines), area);

    Ok(())
}

fn render_sidebar(
    frame: &mut Frame,
    game: &Game,
    player_id: PlayerId,
    area: Rect,
) -> AppResult<()> {
    let hero = if let Some(hero) = game.get_hero(&player_id) {
        hero
    } else {
        return Err(anyhow!("Hero non existing"));
    };

    let split = Layout::vertical([
        Constraint::Length(5),
        Constraint::Length(12),
        Constraint::Min(0),
    ])
    .split(area);

    let lines = vec![
        Line::from("←↑→↓: move"),
        Line::from("'a'/'d': rotate"),
        Line::from(format!(
            "{}",
            match hero.ui_options {
                UiOptions::Help => "'l': leaderboard",
                UiOptions::Leaders => "'h': help",
            }
        )),
    ];
    frame.render_widget(
        Paragraph::new(lines).block(Block::bordered().border_set(border::DOUBLE)),
        split[0],
    );

    match hero.ui_options {
        UiOptions::Leaders => {
            let split = Layout::vertical([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)])
                .split(split[1]);

            let lines = game
                .top_heros()
                .iter()
                .take(4)
                .map(|(id, name, maze_id)| {
                    Line::from(Span::styled(
                        format!("{}: r{}", name, maze_id + 1),
                        if game.get_hero(id).is_some() {
                            if *id == hero.id() {
                                Style::new().fg(GameColors::HERO.to_color())
                            } else {
                                Style::new().fg(GameColors::OTHER_HERO.to_color())
                            }
                        } else {
                            Style::new()
                        },
                    ))
                })
                .collect_vec();

            frame.render_widget(
                Paragraph::new(lines).block(
                    Block::bordered()
                        .title("Top Heros")
                        .border_set(border::DOUBLE),
                ),
                split[0],
            );

            let lines = game
                .top_minotaurs()
                .iter()
                .take(4)
                .map(|(_, name, kills)| Line::from(format!("{}: k{}", name, kills)))
                .collect_vec();

            frame.render_widget(
                Paragraph::new(lines).block(
                    Block::bordered()
                        .title("Top Minotaurs")
                        .border_set(border::DOUBLE),
                ),
                split[1],
            );
        }
        UiOptions::Help => {
            let lines = vec![
                Line::from(vec![
                    Span::styled("██", GameColors::HERO.to_color()),
                    Span::raw(format!(" {:12}", "Hero")),
                ]),
                Line::from(vec![
                    Span::styled("██", GameColors::OTHER_HERO.to_color()),
                    Span::raw(format!(" {:12}", "Other heros")),
                ]),
                Line::from(vec![
                    Span::styled("██", GameColors::MINOTAUR.to_color()),
                    Span::raw(format!(" {:12}", "Minotaur")),
                ]),
                Line::from(vec![
                    Span::styled("██", GameColors::CHASING_MINOTAUR.to_color()),
                    Span::raw(format!(" {:12}", "Minotaur (run!)")),
                ]),
                Line::from(vec![
                    Span::styled("██", GameColors::POWER_UP.to_color()),
                    Span::raw(format!(" {:12}", "Power up")),
                ]),
                Line::from(""),
                Line::from(format!("Run from the minotaurs")),
                Line::from(format!("and try to get as far")),
                Line::from(format!("as possible.")),
            ];

            frame.render_widget(
                Paragraph::new(lines).block(Block::bordered().border_set(border::DOUBLE)),
                split[1],
            );
        }
    }

    Ok(())
}

pub fn render(
    frame: &mut Frame,
    game: &Game,
    player_id: PlayerId,
    start_instant: Instant,
) -> AppResult<()> {
    let number_of_players = game.number_of_players();
    let hero = if let Some(hero) = game.get_hero(&player_id) {
        hero
    } else {
        return Err(anyhow!("Missing hero {}", player_id));
    };

    let split = Layout::vertical([Constraint::Length(5), Constraint::Min(1)])
        .split(frame.area().inner(Margin::new(1, 1)));

    let h_split = Layout::horizontal([Constraint::Length(24), Constraint::Min(1)]).split(split[1]);

    if start_instant.elapsed() < Duration::from_millis(1500) {
        frame.render_widget(
            Paragraph::new(format!(
                "There {} {} hero{} in the labirynth...",
                if number_of_players == 1 { "is" } else { "are" },
                number_of_players,
                if number_of_players == 1 { "" } else { "es" }
            )),
            split[0],
        );

        frame.render_widget(title_paragraph(), split[1]);
        return Ok(());
    }

    render_header(frame, game, player_id, split[0])?;
    render_sidebar(frame, game, player_id, h_split[0])?;
    let image = game.draw(player_id)?;

    let maze = game.get_maze(&hero.maze_id()).unwrap();
    let visible_positions =
        maze.get_cached_visible_positions(hero.position(), hero.direction(), hero.view());
    let override_positions = visible_positions
        .iter()
        .filter(|(x, y)| image.get_pixel(*x as u32, *y as u32).to_rgb() == Rgb([0, 0, 0]))
        .map(|&(x, y)| (x as u32, y as u32))
        .collect();

    frame.render_widget(
        Paragraph::new(img_to_lines(&image, (override_positions, '·'))).centered(),
        h_split[1],
    );

    if hero.is_dead() {
        let width = 32;
        let height = 6;
        let popup = Rect::new(
            h_split[1].x + (h_split[1].width - width) / 2,
            h_split[1].y + (h_split[1].height - height) / 2,
            width,
            height,
        );

        frame.render_widget(
            Paragraph::new(vec![
                Line::from(format!("{}", hero.name())),
                Line::from(format!("died while exploring room {}", hero.maze_id() + 1)),
            ])
            .centered()
            .set_style(Style::default().fg(Color::Black).bg(Color::Red))
            .block(Block::bordered().border_type(BorderType::QuadrantOutside)),
            popup,
        );
    }

    Ok(())
}
