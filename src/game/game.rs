use super::{
    entity::Entity,
    hero::{HeroCommand, HeroState, UiOptions},
    minotaur::Minotaur,
    utils::{random_minotaur_name, to_player_name},
    GameColors, Hero, IntoDirection, Maze,
};
use crate::{AppResult, PlayerId};
use anyhow::anyhow;
use image::{Rgba, RgbaImage};
use itertools::Itertools;
use std::{
    collections::{HashMap, HashSet},
    time::{Duration, Instant},
};

const RESPAWN_INTERVAL: Duration = Duration::from_millis(1500);

pub struct Game {
    mazes: HashMap<usize, Maze>,
    taken_names: HashSet<String>,
    heros: HashMap<PlayerId, Hero>,
    hero_rooms: HashMap<usize, Vec<PlayerId>>,
    top_heros_map: HashMap<PlayerId, (String, usize)>,
    top_heros: Vec<(PlayerId, String, usize)>,
    minotaurs: HashMap<PlayerId, Minotaur>,
    minotaur_rooms: HashMap<usize, Vec<PlayerId>>,
    top_minotaurs_map: HashMap<PlayerId, (String, usize)>,
    top_minotaurs: Vec<(PlayerId, String, usize)>,
}

impl Game {
    const TRANSITION_TIME_DELAY: std::time::Duration = std::time::Duration::from_millis(0);

    fn update_top_heros(&mut self) {
        self.top_heros = self
            .top_heros_map
            .iter()
            .map(|(&id, (name, record))| (id, name.clone(), *record))
            .sorted_by(|a, b| {
                if b.2 == a.2 {
                    b.1.cmp(&a.1)
                } else {
                    b.2.cmp(&a.2)
                }
            })
            .collect_vec()
    }

    fn update_top_minotaurs(&mut self) {
        self.top_minotaurs = self
            .minotaurs
            .values()
            .sorted_by(|a, b| {
                if b.kills == a.kills {
                    b.maze_id().cmp(&a.maze_id())
                } else {
                    b.kills.cmp(&a.kills)
                }
            })
            .map(|minotaur| (minotaur.id(), minotaur.name().to_string(), minotaur.kills))
            .collect_vec();
    }

    pub fn update_time_step() -> Duration {
        Duration::from_millis(25)
    }

    pub fn draw_time_step() -> Duration {
        Duration::from_millis(50)
    }

    pub fn new() -> Self {
        let mut mazes = HashMap::new();
        let maze = Maze::random(0);

        let minotaurs = HashMap::new();
        let mut minotaur_rooms = HashMap::new();
        minotaur_rooms.insert(maze.id, vec![]);
        mazes.insert(maze.id, maze);

        Self {
            mazes,
            heros: HashMap::new(),
            hero_rooms: HashMap::new(),
            taken_names: HashSet::new(),
            top_heros_map: HashMap::new(),
            top_heros: vec![],
            minotaurs,
            minotaur_rooms,
            top_minotaurs_map: HashMap::new(),
            top_minotaurs: vec![],
        }
    }

    pub fn top_heros(&self) -> &Vec<(PlayerId, String, usize)> {
        &self.top_heros
    }

    pub fn top_minotaurs(&self) -> &Vec<(PlayerId, String, usize)> {
        &self.top_minotaurs
    }

    pub fn minotaurs_in_maze(&self, maze_id: usize) -> usize {
        if let Some(maze) = self.minotaur_rooms.get(&maze_id) {
            maze.len()
        } else {
            0
        }
    }

    pub fn alarm_level(&self, hero_id: &PlayerId) -> usize {
        if let Some(hero) = self.get_hero(hero_id) {
            if let Some(maze_minotaurs) = self.minotaur_rooms.get(&hero.maze_id()) {
                if let Some(distance_squared) = maze_minotaurs
                    .iter()
                    .map(|minotaur_id| {
                        let minotaur = self.get_minotaur(minotaur_id).unwrap();
                        minotaur
                            .position()
                            .distance_squared(hero.position().into_direction(&hero.direction()))
                    })
                    .min()
                {
                    return 16 * 16 / distance_squared.max(1);
                }
            }
        }

        0
    }

    pub fn add_player(&mut self, player_id: PlayerId, name: &str) {
        let rng = &mut rand::thread_rng();
        let mut player_name = to_player_name(rng, name);
        while self.taken_names.contains(&player_name) {
            player_name = to_player_name(rng, name);
        }
        self.taken_names.insert(player_name.clone());

        let maze = &mut self.mazes.get_mut(&0).unwrap();
        let mut hero = Hero::new(player_id, player_name, maze.hero_starting_position());
        let visible_positions =
            maze.get_and_cache_visible_positions(hero.position(), hero.direction(), hero.view());
        hero.update_past_visible_positions(visible_positions);

        self.hero_rooms
            .entry(maze.id)
            .or_insert(vec![])
            .push(hero.id());

        self.top_heros_map
            .insert(hero.id(), (hero.name().to_string(), hero.record));

        self.heros.insert(player_id, hero);
    }

    pub fn remove_player(&mut self, player_id: &PlayerId) {
        self.heros.remove(&player_id);
    }

    pub fn get_hero(&self, id: &PlayerId) -> Option<&Hero> {
        self.heros.get(id)
    }

    pub fn get_minotaur(&self, id: &PlayerId) -> Option<&Minotaur> {
        self.minotaurs.get(id)
    }

    pub fn get_maze(&self, id: &usize) -> Option<&Maze> {
        self.mazes.get(id)
    }

    pub fn number_of_players(&self) -> usize {
        self.heros.len()
    }

    pub fn update(&mut self) {
        // Update heros
        let mut should_update_top_heros = false;
        for hero in self.heros.values_mut() {
            match hero.state {
                HeroState::Transitioning { to, instant } => {
                    if instant.elapsed() >= Self::TRANSITION_TIME_DELAY {
                        hero.state = HeroState::InMaze;
                        let from = hero.maze_id();
                        let maze = self.mazes.get_mut(&from).unwrap();
                        hero.update_past_visible_positions(maze.get_and_cache_visible_positions(
                            hero.position(),
                            hero.direction(),
                            hero.view(),
                        ));

                        self.hero_rooms
                            .entry(from)
                            .and_modify(|v| v.retain(|id| *id != hero.id()));
                        self.hero_rooms.entry(to).and_modify(|v| v.push(hero.id()));

                        if from < to {
                            for (idx, exit) in self.mazes[&from].exit_positions().iter().enumerate()
                            {
                                if hero.position() == *exit {
                                    hero.set_maze_id(to);

                                    if hero.record < to {
                                        hero.record = to;
                                        self.top_heros_map.insert(
                                            hero.id(),
                                            (hero.name().to_string(), hero.record),
                                        );
                                        should_update_top_heros = true;
                                    }
                                    let maze = self.mazes.get_mut(&to).unwrap();

                                    hero.set_position(maze.entrance_positions()[idx]);
                                    hero.update_past_visible_positions(
                                        maze.get_and_cache_visible_positions(
                                            hero.position(),
                                            hero.direction(),
                                            hero.view(),
                                        ),
                                    );

                                    break;
                                }
                            }
                        } else if from > to {
                            for (idx, exit) in
                                self.mazes[&from].entrance_positions().iter().enumerate()
                            {
                                if hero.position() == *exit {
                                    hero.set_maze_id(to);

                                    let maze = self.mazes.get_mut(&to).unwrap();

                                    hero.set_position(maze.exit_positions()[idx]);
                                    hero.update_past_visible_positions(
                                        maze.get_and_cache_visible_positions(
                                            hero.position(),
                                            hero.direction(),
                                            hero.view(),
                                        ),
                                    );

                                    break;
                                }
                            }
                        }
                    }
                }

                HeroState::Dead { instant } => {
                    if instant.elapsed() > RESPAWN_INTERVAL {
                        let maze = &mut self.mazes.get_mut(&0).unwrap();
                        hero.reset(maze.hero_starting_position());
                        let visible_positions = maze.get_and_cache_visible_positions(
                            hero.position(),
                            hero.direction(),
                            hero.view(),
                        );
                        hero.update_past_visible_positions(visible_positions);
                    }
                }

                _ => {}
            }
        }

        // Update minotaurs
        let mut should_update_top_minotaurs = false;

        for minotaur in self.minotaurs.values_mut() {
            let maze_id = minotaur.maze_id();
            let maze = self.mazes.get_mut(&maze_id).unwrap();

            let visible_positions = maze.get_and_cache_visible_positions(
                minotaur.position(),
                minotaur.direction(),
                minotaur.view(),
            );

            let visible_heros = self
                .heros
                .values()
                .filter(|hero| {
                    !hero.is_dead()
                        && hero.maze_id() == maze_id
                        && visible_positions.contains(&hero.position())
                })
                .collect_vec();

            minotaur.update(maze, visible_heros);

            let catched_heros = self
                .heros
                .values()
                .filter(|hero| {
                    hero.maze_id() == maze_id
                        && hero.position() == minotaur.position()
                        && !hero.is_dead()
                })
                .map(|hero| hero.id())
                .collect_vec();

            for hero_id in catched_heros.iter() {
                if let Some(hero) = self.heros.get_mut(hero_id) {
                    hero.state = HeroState::Dead {
                        instant: Instant::now(),
                    };
                }
            }

            minotaur.kills += catched_heros.len();
            self.top_minotaurs_map
                .insert(minotaur.id(), (minotaur.name().to_string(), minotaur.kills));
            should_update_top_minotaurs = true;
        }

        if should_update_top_heros {
            self.update_top_heros();
        }
        if should_update_top_minotaurs {
            self.update_top_minotaurs();
        }
    }

    pub fn draw(&self, player_id: PlayerId) -> AppResult<RgbaImage> {
        //FIXME: if hero is transitioning from one maze to the other, join the two maze images (and crop them to simulate the exit->entrance transition)
        if let Some(hero) = self.heros.get(&player_id) {
            let (x, y) = hero.position();
            let maze_id = hero.maze_id();
            let maze = &self.mazes[&maze_id];

            let maze_image = maze.image();

            // The base player image is black.
            let mut player_image = RgbaImage::from_pixel(
                maze_image.width(),
                maze_image.height(),
                Rgba([0, 0, 0, 255]),
            );

            let visible_positions =
                maze.get_cached_visible_positions(hero.position(), hero.direction(), hero.view());

            for (&(dx, dy), instant) in hero.past_visible_positions().iter() {
                // Each position in the past_visible_positions is copied from the maze_image, with alpha channel depending on the time passed.
                let base_color = maze_image.get_pixel(dx as u32, dy as u32);

                let is_valid = maze.is_valid_position((dx, dy));

                let base_alpha = if is_valid { 50 } else { 125 };
                let mut alpha = if instant.elapsed() < hero.past_visibility_duration() {
                    base_alpha
                        - (base_alpha as f64 * instant.elapsed().as_millis() as f64
                            / hero.past_visibility_duration().as_millis() as f64)
                            as u8
                } else {
                    0
                };

                if visible_positions.contains(&(dx, dy)) {
                    // Each position in the visible_positions is copied from the maze_image, with alpha channel depending on the distance from the hero.
                    let distance = hero.position().distance((dx, dy));
                    alpha += ((255.0 - alpha as f64)
                        * (1.0 - distance / hero.view().radius() as f64))
                        as u8
                }

                let pixel = Rgba([base_color[0], base_color[1], base_color[2], alpha]);
                player_image.put_pixel(dx as u32, dy as u32, pixel);
            }

            // Add other powerup position
            if let Some((x, y)) = maze.power_up_position {
                if visible_positions.contains(&(x, y)) {
                    player_image.put_pixel(x as u32, y as u32, GameColors::POWER_UP);
                }
            }

            // Add other heros position
            for (p_id, any_hero) in self.heros.iter() {
                if *p_id != player_id {
                    if any_hero.maze_id() == hero.maze_id() {
                        let (ax, ay) = any_hero.position();
                        if visible_positions.contains(&(ax, ay)) {
                            player_image.put_pixel(ax as u32, ay as u32, GameColors::OTHER_HERO);
                        }
                    }
                }
            }

            // Add minotaurs position
            if let Some(maze_minotaurs) = self.minotaur_rooms.get(&hero.maze_id()) {
                for minotaur_id in maze_minotaurs.iter() {
                    if let Some(minotaur) = self.get_minotaur(minotaur_id) {
                        let (mx, my) = minotaur.position();
                        if visible_positions.contains(&(mx, my)) {
                            if minotaur.is_chasing(hero.id()) {
                                player_image.put_pixel(
                                    mx as u32,
                                    my as u32,
                                    GameColors::CHASING_MINOTAUR,
                                );
                            } else {
                                player_image.put_pixel(mx as u32, my as u32, GameColors::MINOTAUR);
                            }
                        }
                    }
                }
            }

            // Add hero position
            player_image.put_pixel(x as u32, y as u32, GameColors::HERO);

            return Ok(player_image);
        }
        return Err(anyhow!("No hero with id {}", player_id));
    }

    pub fn handle_command(&mut self, command: &HeroCommand, player_id: PlayerId) {
        let hero = if let Some(hero) = self.heros.get_mut(&player_id) {
            hero
        } else {
            return;
        };
        match hero.state {
            HeroState::InMaze => {
                let maze_id = hero.maze_id();
                let maze = if let Some(maze) = self.mazes.get_mut(&maze_id) {
                    maze
                } else {
                    return;
                };

                match command {
                    HeroCommand::Move { direction } => {
                        hero.update_past_visible_positions(maze.get_and_cache_visible_positions(
                            hero.position(),
                            hero.direction(),
                            hero.view(),
                        ));

                        if *direction != hero.direction() {
                            hero.set_direction(*direction);
                        }

                        if !hero.can_move() {
                            hero.update_past_visible_positions(
                                maze.get_and_cache_visible_positions(
                                    hero.position(),
                                    hero.direction(),
                                    hero.view(),
                                ),
                            );
                            return;
                        }

                        let (new_x, new_y) = hero.position().into_direction(direction);

                        if !maze.is_valid_position((new_x, new_y)) {
                            hero.update_past_visible_positions(
                                maze.get_and_cache_visible_positions(
                                    hero.position(),
                                    hero.direction(),
                                    hero.view(),
                                ),
                            );
                            return;
                        }

                        hero.set_position((new_x, new_y));
                        hero.update_past_visible_positions(maze.get_and_cache_visible_positions(
                            hero.position(),
                            hero.direction(),
                            hero.view(),
                        ));

                        if let Some(position) = maze.power_up_position {
                            if position == hero.position()
                                && hero.power_up_collected_in_maze().is_none()
                            {
                                hero.apply_power_up();
                            }
                        }

                        if maze.is_entrance_position(hero.position()) {
                            if maze_id > 0 {
                                hero.state = HeroState::Transitioning {
                                    to: maze_id - 1,
                                    instant: Instant::now(),
                                }
                            }
                        } else if maze.is_exit_position(hero.position()) {
                            hero.state = HeroState::Transitioning {
                                to: maze_id + 1,
                                instant: Instant::now(),
                            };

                            if !self.hero_rooms.contains_key(&(maze_id + 1)) {
                                self.hero_rooms.insert(maze.id, vec![]);
                            }

                            if !self.mazes.contains_key(&(maze_id + 1)) {
                                let mut maze = Maze::random(maze_id + 1);
                                let mut maze_minotaurs = vec![];
                                for _ in 0..maze.id {
                                    let rng = &mut rand::thread_rng();
                                    let name = &random_minotaur_name();
                                    let mut minotaur_name = to_player_name(rng, name);
                                    while self.taken_names.contains(&minotaur_name) {
                                        minotaur_name = to_player_name(rng, name);
                                    }
                                    self.taken_names.insert(minotaur_name.clone());

                                    let minotaur = maze.spawn_minotaur(minotaur_name);
                                    maze_minotaurs.push(minotaur.id());
                                    self.minotaurs.insert(minotaur.id(), minotaur);
                                }
                                self.minotaur_rooms.insert(maze.id, maze_minotaurs);
                                self.mazes.insert(maze.id, maze);
                            }
                        }
                    }

                    HeroCommand::TurnClockwise => {
                        hero.update_past_visible_positions(maze.get_and_cache_visible_positions(
                            hero.position(),
                            hero.direction(),
                            hero.view(),
                        ));
                        hero.set_direction(hero.direction().rotate_clockwise());
                        hero.update_past_visible_positions(maze.get_and_cache_visible_positions(
                            hero.position(),
                            hero.direction(),
                            hero.view(),
                        ));
                    }

                    HeroCommand::TurnCounterClockwise => {
                        hero.update_past_visible_positions(maze.get_and_cache_visible_positions(
                            hero.position(),
                            hero.direction(),
                            hero.view(),
                        ));
                        hero.set_direction(hero.direction().rotate_counter_clockwise());
                        hero.update_past_visible_positions(maze.get_and_cache_visible_positions(
                            hero.position(),
                            hero.direction(),
                            hero.view(),
                        ));
                    }

                    HeroCommand::DisplayHelp => hero.ui_options = UiOptions::Help,
                    HeroCommand::DisplayLeaders => hero.ui_options = UiOptions::Leaders,
                }
            }
            _ => {}
        }
    }
}
