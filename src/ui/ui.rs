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
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

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
        let maze = game.get_maze(&hero.maze_id()).unwrap();

        lines.push(Line::from(vec![
            Span::styled(format!("{} - ", hero.name()), GameColors::HERO.to_color()),
            Span::raw(format!(
                "Room {}@{:8} - Success rate {:.2}%",
                hero.maze_id() + 1,
                format!("{:?}", hero.position()),
                maze.success_rate()
            )),
        ]));

        let num_minotaurs = game.minotaurs_in_maze(hero.maze_id());
        let (alarm_level, min_distance_squared) = game.alarm_level(&hero.id());
        let radar_power = 16 * 16 / min_distance_squared.max(1);
        let minoradar: String = MINORADAR.iter().take(radar_power).map(|s| *s).collect();
        lines.push(Line::from(vec![
            Span::raw(format!(
                "{} minotaur{}  ",
                num_minotaurs,
                if num_minotaurs == 1 { "" } else { "s" }
            )),
            Span::styled(
                format!("{:8} ", minoradar),
                Style::new().fg(alarm_level.rgba().to_color()),
            ),
        ]));

        lines.push(Line::from(Span::raw(format!(
            "Power up {}collected",
            if let Some(power_up) = hero.power_up_collected_in_maze() {
                format!("({}) ", power_up)
            } else {
                "not ".to_string()
            }
        ))));

        if hero.vision() > 4 {
            lines.push(Line::from(vec![Span::raw(format!(
                "{}",
                (min_distance_squared as f64).sqrt().round() as usize
            ))]));
        }
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
                        format!("{:14} r{}", name, maze_id + 1),
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
                .map(|(_, name, maze_id, kills)| {
                    Line::from(format!("{:14} k{} r{}", name, kills, maze_id + 1))
                })
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

    // Override empty positions.
    let visible_positions =
        maze.get_cached_visible_positions(hero.position(), hero.direction(), hero.view());
    let mut override_positions = visible_positions
        .iter()
        .filter(|(x, y)| image.get_pixel(*x as u32, *y as u32).to_rgb() == Rgb([0, 0, 0]))
        .map(|&(x, y)| ((x as u32, y as u32), '·'))
        .collect::<HashMap<(u32, u32), char>>();

    for &(x, y) in maze.entrance_positions().iter() {
        if maze.id > 0 {
            for (idx, c) in (maze.id + 1 - 1).to_string().chars().enumerate() {
                override_positions.insert((x as u32 + idx as u32 + 1, y as u32), c);
            }
        }
        override_positions.insert((x as u32, y as u32), '←');
    }

    for &(x, y) in maze.exit_positions().iter() {
        for (idx, c) in (maze.id + 1 + 1).to_string().chars().rev().enumerate() {
            override_positions.insert((x as u32 - idx as u32 - 1, y as u32), c);
        }
        override_positions.insert((x as u32, y as u32), '→');
    }

    frame.render_widget(
        Paragraph::new(img_to_lines(&image, override_positions)).centered(),
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
