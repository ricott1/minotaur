use super::{direction::Direction, minotaur::Minotaur, IntoDirection, Position, View};
use crate::AppResult;
use image::{Rgba, RgbaImage};
use itertools::Itertools;
use rand::{
    seq::{IteratorRandom, SliceRandom},
    Rng, SeedableRng,
};
use rand_chacha::ChaCha8Rng;
use std::{
    collections::{HashMap, HashSet},
    path::Path,
};
use strum::{Display, IntoEnumIterator};
use strum_macros::EnumIter;

const MAX_NUMBER_OF_WALLS: usize = 8; // We work at most in a octagone-lattice, so we have at most 8 walls.

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Cell {
    x: usize,
    y: usize,
    walls: [bool; MAX_NUMBER_OF_WALLS],
}

impl Cell {
    fn remove_wall(&mut self, direction: Direction) {
        self.walls[direction as usize] = false;
    }

    pub fn new(x: usize, y: usize) -> Self {
        Self {
            x,
            y,
            walls: [true; MAX_NUMBER_OF_WALLS],
        }
    }

    pub fn x(&self) -> usize {
        self.x
    }

    pub fn y(&self) -> usize {
        self.y
    }

    pub fn has_wall(&self, direction: Direction) -> bool {
        self.walls[direction as usize]
    }

    pub fn number_of_walls(&self) -> usize {
        self.walls.iter().filter(|&w| *w).count()
    }
}

#[derive(Debug, Clone, Copy, Display, EnumIter, PartialEq)]
pub enum MazeGenerationAlgorithm {
    DepthFirstSearch,
    Wilson,
}

#[derive(Debug, Clone, Copy, Display, EnumIter, PartialEq)]
pub enum MazeImageStyle {
    Straight,
    Seasaw,
}

#[derive(Debug, Clone, Copy, Display, EnumIter, PartialEq)]
pub enum MazeTopology {
    Orthogonal,
    Chessboard,
}

impl MazeTopology {
    fn number_of_linked_cells(&self) -> usize {
        match self {
            Self::Orthogonal => 4,
            Self::Chessboard => 8,
        }
    }

    pub fn available_directions(&self) -> Vec<Direction> {
        Direction::iter()
            .filter(|&d| (d as usize) < self.number_of_linked_cells())
            .collect_vec()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Maze {
    pub id: usize,
    random_seed: u64,
    pub width: usize,
    pub height: usize,
    generation_algorithm: MazeGenerationAlgorithm,
    topology: MazeTopology,
    image: RgbaImage,
    image_style: MazeImageStyle,
    valid_positions: HashSet<Position>,
    entrance: Vec<Position>,
    exit: Vec<Position>,
    pub power_up_position: Option<Position>,
    visible_positions_cache: HashMap<(usize, usize, Direction, View), HashSet<Position>>, // (x, y, direction, type) -> visible positions
    success_rate: (usize, usize), //pass/attempted
}

impl Maze {
    const CELL_SIZE: usize = 6;
    const WALL_SIZE: usize = 2;

    fn cell_image_position(&self, position: Position) -> Position {
        // The cell position indicate the top-left corner.
        match self.image_style {
            MazeImageStyle::Straight => {
                let x = position.0 * (Self::CELL_SIZE - Self::WALL_SIZE);
                let y = position.1 * (Self::CELL_SIZE - Self::WALL_SIZE);
                (x, y)
            }
            MazeImageStyle::Seasaw => {
                let x = position.0 * (Self::CELL_SIZE - Self::WALL_SIZE)
                    + Self::WALL_SIZE * (position.1 % 2);
                let y = position.1 * (Self::CELL_SIZE - Self::WALL_SIZE);
                (x, y)
            }
        }
    }
    fn generate_cells(&mut self) -> Vec<Cell> {
        let rng = &mut ChaCha8Rng::seed_from_u64(self.random_seed);
        let mut cells = Vec::with_capacity(self.width * self.height);
        for y in 0..self.height {
            for x in 0..self.width {
                let cell = Cell::new(x, y);
                cells.push(cell);
            }
        }

        match self.generation_algorithm {
            MazeGenerationAlgorithm::DepthFirstSearch => {
                //     Randomized depth-first search
                let mut stack = Vec::new();
                let mut visited_cells = vec![false; cells.len()];

                //     Choose the initial cell, mark it as visited and push it to the stack
                let initial_cell_index = 0;

                stack.push(initial_cell_index);
                visited_cells[0] = true;
                let available_directions = self.topology.available_directions();

                // While the stack is not empty
                while !stack.is_empty() {
                    // Pop a cell from the stack and make it a current cell
                    let current_cell_index = stack.pop().unwrap();
                    let current_cell = cells.get_mut(current_cell_index).unwrap();

                    // Get neighbours of the current cell

                    let mut unvisited_neighbours = vec![];
                    for &direction in available_directions.iter() {
                        let (dx, dy) = direction.as_offset();
                        let x = current_cell.x as isize + dx;
                        let y = current_cell.y as isize + dy;
                        if x < 0 || x >= self.width as isize || y < 0 || y >= self.height as isize {
                            continue;
                        }
                        let x = x as usize;
                        let y = y as usize;
                        if !visited_cells[y * self.width + x] {
                            unvisited_neighbours.push(direction);
                        }
                    }
                    // If the current cell has any neighbours which have not been visited
                    if unvisited_neighbours.len() > 0 {
                        // Push the current cell to the stack
                        stack.push(current_cell_index);
                        // Choose one of the unvisited neighbours
                        let chosen_neighbour_direction: Direction =
                            unvisited_neighbours[rng.gen::<usize>() % unvisited_neighbours.len()];
                        // Remove the wall between the current cell and the chosen cell
                        let opposite = chosen_neighbour_direction.opposite();
                        current_cell.remove_wall(chosen_neighbour_direction);

                        let neighbour_index = (current_cell.y as isize
                            + chosen_neighbour_direction.as_offset().1)
                            as usize
                            * self.width
                            + (current_cell.x as isize + chosen_neighbour_direction.as_offset().0)
                                as usize;
                        let neighbour = cells.get_mut(neighbour_index).unwrap();
                        neighbour.remove_wall(opposite);

                        // Mark the chosen cell as visited and push it to the stack
                        visited_cells[neighbour.y * self.width + neighbour.x] = true;

                        stack.push(neighbour_index);
                    }
                }
            }
            MazeGenerationAlgorithm::Wilson => {}
        }

        cells
    }

    fn generate_maze(&mut self, cells: Vec<Cell>, entrance: Option<usize>, exit: Option<usize>) {
        let available_directions = self.topology.available_directions();

        self.valid_positions.clear();
        for cell in cells.iter() {
            let (x, y) = self.cell_image_position((cell.x(), cell.y()));

            // If the cell has less than MAX_NUMBER_OF_WALLS walls, the 4 central pixels are empty.
            if cell.number_of_walls() < MAX_NUMBER_OF_WALLS {
                for dy in Self::WALL_SIZE..Self::CELL_SIZE - Self::WALL_SIZE {
                    for dx in Self::WALL_SIZE..Self::CELL_SIZE - Self::WALL_SIZE {
                        self.valid_positions.insert((x + dx, y + dy));
                    }
                }
            }
        }

        for cell in cells.iter() {
            let (x, y) = self.cell_image_position((cell.x(), cell.y()));

            // Remove walls.
            for &direction in available_directions.iter() {
                if !cell.has_wall(direction) {
                    match direction {
                        Direction::North => {
                            for dy in 0..Self::WALL_SIZE {
                                for dx in Self::WALL_SIZE..Self::CELL_SIZE - Self::WALL_SIZE {
                                    self.valid_positions.insert((x + dx, y + dy));
                                }
                            }
                        }
                        Direction::East => {
                            for dy in Self::WALL_SIZE..Self::CELL_SIZE - Self::WALL_SIZE {
                                for dx in
                                    Self::CELL_SIZE - Self::WALL_SIZE..Self::CELL_SIZE as usize
                                {
                                    self.valid_positions.insert((x + dx, y + dy));
                                }
                            }
                        }
                        Direction::South => {
                            for dy in Self::CELL_SIZE - Self::WALL_SIZE..Self::CELL_SIZE {
                                for dx in Self::WALL_SIZE..Self::CELL_SIZE - Self::WALL_SIZE {
                                    self.valid_positions.insert((x + dx, y + dy));
                                }
                            }
                        }
                        Direction::West => {
                            for dy in Self::WALL_SIZE..Self::CELL_SIZE - Self::WALL_SIZE {
                                for dx in 0..Self::WALL_SIZE {
                                    self.valid_positions.insert((x + dx, y + dy));
                                }
                            }
                        }

                        Direction::NorthEast => {
                            for dy in 0..Self::WALL_SIZE {
                                for dx in Self::CELL_SIZE - Self::WALL_SIZE..Self::CELL_SIZE {
                                    self.valid_positions.insert((x + dx, y + dy));
                                }
                            }
                            self.valid_positions.insert((x + 3, y + 1));
                            self.valid_positions.insert((x + 4, y + 2));
                        }
                        Direction::SouthEast => {
                            for dy in Self::CELL_SIZE - Self::WALL_SIZE..Self::CELL_SIZE {
                                for dx in Self::CELL_SIZE - Self::WALL_SIZE..Self::CELL_SIZE {
                                    self.valid_positions.insert((x + dx, y + dy));
                                }
                            }
                            self.valid_positions.insert((x + 3, y + 4));
                            self.valid_positions.insert((x + 4, y + 3));
                        }
                        Direction::SouthWest => {
                            for dy in Self::CELL_SIZE - Self::WALL_SIZE..Self::CELL_SIZE {
                                for dx in 0..Self::WALL_SIZE {
                                    self.valid_positions.insert((x + dx, y + dy));
                                }
                            }
                            self.valid_positions.insert((x + 1, y + 3));
                            self.valid_positions.insert((x + 2, y + 4));
                        }
                        Direction::NorthWest => {
                            for dy in 0..Self::WALL_SIZE {
                                for dx in 0..Self::WALL_SIZE {
                                    self.valid_positions.insert((x + dx, y + dy));
                                }
                            }
                            self.valid_positions.insert((x + 2, y + 1));
                            self.valid_positions.insert((x + 1, y + 2));
                        }
                    }
                }
            }
        }

        let rng = &mut ChaCha8Rng::seed_from_u64(self.random_seed);

        // create entrance
        let entrance_y = if let Some(y) = entrance {
            y
        } else {
            rng.gen_range(
                Self::WALL_SIZE as usize
                    ..self.height * (Self::CELL_SIZE - Self::WALL_SIZE) as usize
                        - Self::WALL_SIZE as usize
                        - 1,
            ) / 2
                * 2
        };
        self.entrance = {
            let mut x = 0;
            loop {
                if self.is_valid_position((x, entrance_y))
                    && self.is_valid_position((x, entrance_y + 1))
                {
                    break;
                }

                // Starting maze has no entrance
                if self.id > 0 {
                    self.valid_positions.insert((x, entrance_y));
                    self.valid_positions.insert((x, entrance_y + 1));
                }
                x += 1;
            }

            vec![(0, entrance_y), (0, entrance_y + 1)]
        };

        // create exit
        let exit_y = if let Some(y) = exit {
            y
        } else {
            rng.gen_range(
                Self::WALL_SIZE as usize
                    ..self.height * (Self::CELL_SIZE - Self::WALL_SIZE) as usize
                        - Self::WALL_SIZE as usize
                        - 1,
            ) / 2
                * 2
        };
        self.exit = {
            let max_x = (self.width - 1) * (Self::CELL_SIZE - Self::WALL_SIZE) as usize
                + Self::CELL_SIZE as usize
                + 1;
            let mut x = max_x;

            loop {
                if self.is_valid_position((x, exit_y)) && self.is_valid_position((x, exit_y + 1)) {
                    break;
                }

                self.valid_positions.insert((x, exit_y));
                self.valid_positions.insert((x, exit_y + 1));
                x -= 1;
            }

            vec![(max_x, exit_y), (max_x, exit_y + 1)]
        };

        // Add random rooms. The number of rooms deoends on the maze size.
        let number_of_rooms = rng.gen_range(4..=((self.width + self.height) / 3).max(5));
        for _ in 0..number_of_rooms {
            let room_width = rng.gen_range(4..=((self.width + self.height) / 8).max(5));
            let room_height = rng.gen_range(4..=((self.width + self.height) / 8).max(5));
            let (room_x, room_y) = self.cell_image_position((
                rng.gen_range(
                    Self::WALL_SIZE
                        ..self
                            .width
                            .saturating_sub(room_width + Self::WALL_SIZE)
                            .max(Self::WALL_SIZE + 1),
                ),
                rng.gen_range(
                    Self::WALL_SIZE
                        ..self
                            .height
                            .saturating_sub(room_height + Self::WALL_SIZE)
                            .max(Self::WALL_SIZE + 1),
                ),
            ));

            for y in room_y..room_y + room_height {
                for x in room_x..room_x + room_width {
                    self.valid_positions.insert((x, y));
                }
            }
        }
    }

    fn random_valid_position(&self) -> Position {
        self.valid_positions
            .iter()
            .choose(&mut rand::thread_rng())
            .copied()
            .unwrap()
    }

    fn random_valid_power_up_position(&self) -> Option<Position> {
        self.valid_positions
            .iter()
            .filter(|&&position| {
                self.entrance
                    .iter()
                    .all(|entrance| entrance.distance(position) > 6.0)
                    && self.exit.iter().all(|exit| exit.distance(position) > 6.0)
            })
            .choose(&mut rand::thread_rng())
            .copied()
    }

    fn color(id: usize) -> Rgba<u8> {
        let a = (id.min(10) as f64) / 10.0;
        // red = Rgba([208, 28, 28, 125]);
        // whiteblueish = Rgba([210, 240, 255, 125]);

        Rgba([
            (a * 208.0 + (1.0 - a) * 210.0) as u8,
            (a * 28.0 + (1.0 - a) * 240.0) as u8,
            (a * 28.0 + (1.0 - a) * 255.0) as u8,
            125,
        ])
    }

    fn entrance_color(id: usize) -> Rgba<u8> {
        if id == 0 {
            return Self::color(id);
        }
        Rgba([0; 4])
    }

    fn exit_color(_id: usize) -> Rgba<u8> {
        Rgba([0; 4])
    }

    pub fn random(id: usize, entrance: Option<usize>) -> Self {
        let random_seed = ChaCha8Rng::from_entropy().gen();
        let rng = &mut ChaCha8Rng::seed_from_u64(random_seed);
        let width = rng.gen_range(10 + 2 * (id / 4)..=(12 + 2 * (id / 2)).min(32));
        let height = rng.gen_range(4 + 2 * (id / 4)..=(6 + 2 * (id / 2)).min(20));
        let generation_algorithm = MazeGenerationAlgorithm::DepthFirstSearch;
        let topology = MazeTopology::Orthogonal;
        let image_style = MazeImageStyle::Straight;

        Self::new(
            id,
            random_seed,
            width,
            height,
            entrance,
            None,
            generation_algorithm,
            topology,
            image_style,
        )
    }

    pub fn new(
        id: usize,
        random_seed: u64,
        width: usize,
        height: usize,
        entrance: Option<usize>,
        exit: Option<usize>,
        generation_algorithm: MazeGenerationAlgorithm,
        topology: MazeTopology,
        image_style: MazeImageStyle,
    ) -> Self {
        // Initialize empty image with maze size.
        let image = RgbaImage::from_pixel(
            (width * (Self::CELL_SIZE - Self::WALL_SIZE) + 2 * Self::WALL_SIZE) as u32,
            (height * (Self::CELL_SIZE - Self::WALL_SIZE) + Self::WALL_SIZE) as u32,
            Self::color(id),
        );

        let valid_positions = HashSet::new();

        let mut maze = Self {
            id,
            random_seed,
            width,
            height,
            generation_algorithm,
            topology,
            image,
            image_style,
            valid_positions,
            entrance: Vec::new(),
            exit: Vec::new(),
            power_up_position: None,
            visible_positions_cache: HashMap::new(),
            success_rate: (0, 0),
        };

        let cells = maze.generate_cells();
        maze.generate_maze(cells, entrance, exit);
        maze.power_up_position = maze.random_valid_power_up_position();
        maze.generate_image();

        println!("Generated maze {}", maze.random_seed);

        maze
    }

    pub fn spawn_minotaur(&mut self, name: String) -> Minotaur {
        let view = View::minotaur(6 + self.id / 3);
        let mut position = self.random_valid_position();
        while position.distance(self.entrance[0]) < 10.0 {
            position = self.random_valid_position()
        }
        let direction = Direction::North;
        self.get_and_cache_visible_positions(position, direction, view);

        let speed = (self.id as u64 / 3).min(5);
        let aggression = (0.45 + 0.05 * (self.id / 2) as f64).min(1.0);
        let vision = (3 + self.id / 3).min(6);
        Minotaur::new(name, self.id, position, speed, vision, aggression)
    }

    pub fn get_and_cache_visible_positions(
        &mut self,
        position: Position,
        direction: Direction,
        view: View,
    ) -> HashSet<Position> {
        let cache_key = (position.0, position.1, direction, view);
        if let Some(visible_positions) = self.visible_positions_cache.get(&cache_key) {
            return visible_positions.clone();
        }

        if view == View::Full {
            let mut visible_positions = HashSet::new();
            for &(x, y) in self.valid_positions.iter() {
                visible_positions.insert((x, y));
            }

            self.visible_positions_cache
                .insert(cache_key, visible_positions.clone());
            return visible_positions;
        }

        let (x, y) = position;
        let view_radius = view.radius();

        let mut visible_positions = HashSet::new();
        for dy in
            y.saturating_sub(view_radius)..=(y + view_radius).min(self.image().height() as usize)
        {
            for dx in
                x.saturating_sub(view_radius)..=(x + view_radius).min(self.image().width() as usize)
            {
                // Origin is always visible
                if x == dx && y == dy {
                    visible_positions.insert((dx, dy));
                    continue;
                }

                if visible_positions.contains(&(dx, dy)) {
                    continue;
                }

                // Position must be unobstructed by walls.
                // We check this by drawing a line from the position to (x, y) and check that the positions on the line are valid positions.

                // vertical line
                if x == dx {
                    let iter = if y < dy {
                        (y..=dy).collect_vec()
                    } else {
                        (dy..=y).rev().collect_vec()
                    };

                    'inner: for ly in iter {
                        // The wall visible as well.
                        visible_positions.insert((x, ly));
                        if !self.is_valid_position((x, ly)) {
                            break 'inner;
                        }
                    }
                }
                //horizontal line
                else if y == dy {
                    let iter = if x < dx {
                        (x..=dx).collect_vec()
                    } else {
                        (dx..=x).rev().collect_vec()
                    };

                    'inner: for lx in iter {
                        // The wall visible as well.
                        visible_positions.insert((lx, y));
                        if !self.is_valid_position((lx, y)) {
                            break 'inner;
                        }
                    }
                }
                // generic line
                else {
                    let mut line = bresenham_line((x as i32, y as i32), (dx as i32, dy as i32));

                    if line[0] != (x, y) {
                        line.reverse();
                    };

                    'inner: for index in 0..line.len() {
                        let (lx, ly) = line[index];
                        // The wall visible as well.
                        visible_positions.insert((lx, ly));
                        if !self.is_valid_position((lx, ly)) {
                            break 'inner;
                        }

                        if index < line.len() - 1 {
                            // Check if we are moving through a wall in a diagonal
                            let (next_x, next_y) = line[index + 1];

                            // 4 cases
                            if next_x == lx + 1 && next_y + 1 == ly {
                                if self.is_valid_position((next_x, next_y))
                                    && !self.is_valid_position((lx + 1, ly))
                                    && !self.is_valid_position((lx, ly - 1))
                                {
                                    break 'inner;
                                }
                            }

                            if next_x == lx + 1 && next_y == ly + 1 {
                                if self.is_valid_position((next_x, next_y))
                                    && !self.is_valid_position((lx + 1, ly))
                                    && !self.is_valid_position((lx, ly + 1))
                                {
                                    break 'inner;
                                }
                            }

                            if next_x + 1 == lx && next_y + 1 == ly {
                                if self.is_valid_position((next_x, next_y))
                                    && !self.is_valid_position((lx - 1, ly))
                                    && !self.is_valid_position((lx, ly - 1))
                                {
                                    break 'inner;
                                }
                            }

                            if next_x + 1 == lx && next_y == ly + 1 {
                                if self.is_valid_position((next_x, next_y))
                                    && !self.is_valid_position((lx - 1, ly))
                                    && !self.is_valid_position((lx, ly + 1))
                                {
                                    break 'inner;
                                }
                            }
                        }
                    }
                }
            }
        }

        // Filter out-of-bounds positions.
        visible_positions = visible_positions
            .iter()
            .filter(|(x, y)| {
                *x < self.image().width() as usize && *y < self.image().height() as usize
            })
            .map(|(x, y)| (*x, *y))
            .collect();

        // Limit view to relevant cone depending on the direction.
        visible_positions = visible_positions
            .iter()
            .filter(|(x, y)| {
                let dx = *x as i32 - position.0 as i32;
                let dy = *y as i32 - position.1 as i32;
                match view {
                    View::Cone { .. } => match direction {
                        Direction::North => dx >= dy && dx <= -dy,
                        Direction::East => dy <= dx && dy >= -dx,
                        Direction::South => dx <= dy && dx >= -dy,
                        Direction::West => dy >= dx && dy <= -dx,
                        Direction::NorthEast => dx >= 0 && dy <= 0,
                        Direction::SouthEast => dx >= 0 && dy >= 0,
                        Direction::SouthWest => dx <= 0 && dy >= 0,
                        Direction::NorthWest => dx <= 0 && dy <= 0,
                    },
                    View::Plane { .. } => match direction {
                        Direction::North => dy < 0,
                        Direction::East => dx > 0,
                        Direction::South => dy > 0,
                        Direction::West => dx < 0,
                        Direction::NorthEast => dx > dy,
                        Direction::SouthEast => dx > -dy,
                        Direction::SouthWest => dx < dy,
                        Direction::NorthWest => dx < -dy,
                    },
                    View::Circle { .. } => true,
                    _ => unreachable!(),
                }
            })
            .map(|(x, y)| (*x, *y))
            .collect();

        self.visible_positions_cache
            .insert(cache_key, visible_positions.clone());

        visible_positions
    }

    pub fn get_cached_visible_positions(
        &self,
        position: Position,
        direction: Direction,
        view: View,
    ) -> HashSet<Position> {
        let cache_key = (position.0, position.1, direction, view);
        self.visible_positions_cache
            .get(&cache_key)
            .expect("Visible positions should have been cached")
            .clone()
    }

    pub fn image(&self) -> &RgbaImage {
        &self.image
    }

    pub fn save_image(&self, name: &str) -> AppResult<()> {
        image::save_buffer(
            &Path::new(name),
            &self.image,
            self.image.width(),
            self.image.height(),
            image::ColorType::Rgba8,
        )?;
        Ok(())
    }

    pub fn generate_image(&mut self) {
        // Initialize empty image with maze size.
        self.image = RgbaImage::from_pixel(
            (self.width * (Self::CELL_SIZE - Self::WALL_SIZE) + 2 * Self::WALL_SIZE) as u32,
            (self.height * (Self::CELL_SIZE - Self::WALL_SIZE) + Self::WALL_SIZE) as u32,
            Self::color(self.id),
        );

        for &(x, y) in self.valid_positions.iter() {
            self.image
                .put_pixel(x as u32, y as u32, Rgba([0, 0, 0, 255]));
        }

        // color entrance
        for &(x, y) in self.entrance.iter() {
            self.image
                .put_pixel(x as u32, y as u32, Self::entrance_color(self.id));
        }

        //color exit
        for &(x, y) in self.exit.iter() {
            self.image
                .put_pixel(x as u32, y as u32, Self::exit_color(self.id));
        }
    }

    pub fn is_valid_position(&self, position: Position) -> bool {
        self.valid_positions.get(&position).is_some()
    }

    pub fn is_valid_minotaur_position(&self, position: Position) -> bool {
        let entrances = self.entrance_positions();
        self.valid_positions.get(&position).is_some()
            && entrances.iter().all(|p| p.distance(position) > 6.0)
    }

    pub fn is_entrance_position(&self, position: Position) -> bool {
        self.entrance.contains(&position)
    }

    pub fn is_exit_position(&self, position: Position) -> bool {
        self.exit.contains(&position)
    }

    pub fn entrance_positions(&self) -> &Vec<Position> {
        &self.entrance
    }

    pub fn exit_positions(&self) -> &Vec<Position> {
        &self.exit
    }

    pub fn hero_starting_position(&self) -> Position {
        let rng = &mut rand::thread_rng();
        let &(x, y) = self.entrance.choose(rng).unwrap();
        (x + 1, y)
    }

    pub fn increase_attempted(&mut self) {
        self.success_rate.1 += 1;
    }

    pub fn decrease_attempted(&mut self) {
        self.success_rate.1 -= 1;
    }

    pub fn increase_passed(&mut self) {
        self.success_rate.0 += 1;
    }

    pub fn decrease_passed(&mut self) {
        self.success_rate.0 -= 1;
    }
}

// Returns the list of points from (x0, y0) to (x1, y1)
fn bresenham_line(from: (i32, i32), to: (i32, i32)) -> Vec<Position> {
    let mut result = Vec::new();

    let (mut x0, mut y0) = from;
    let (mut x1, mut y1) = to;

    let steep = (y1 - y0).abs() > (x1 - x0).abs();
    if steep {
        (x0, y0) = (y0, x0);
        (x1, y1) = (y1, x1);
    }
    if x0 > x1 {
        (x0, x1) = (x1, x0);
        (y0, y1) = (y1, y0);
    }

    let delta_x = x1 - x0;
    let delta_y = (y1 - y0).abs();
    let mut error = 0;
    let ystep = if y0 < y1 { 1 } else { -1 };
    let mut y = y0;

    for x in x0..=x1 {
        if steep {
            result.push((y as usize, x as usize))
        } else {
            result.push((x as usize, y as usize))
        }
        error += delta_y;
        if 2 * error >= delta_x {
            y += ystep;
            error -= delta_x;
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::{Maze, MazeGenerationAlgorithm, MazeImageStyle, MazeTopology};
    use crate::AppResult;
    use strum::IntoEnumIterator;

    #[test]
    fn test_maze_image() -> AppResult<()> {
        for generation_algorithm in MazeGenerationAlgorithm::iter() {
            for topology in MazeTopology::iter() {
                for image_style in MazeImageStyle::iter() {
                    let maze = Maze::new(
                        0,
                        0,
                        20,
                        12,
                        None,
                        None,
                        generation_algorithm,
                        topology,
                        image_style,
                    );
                    let name = format!(
                        "images/maze_{}_{}_{}.png",
                        maze.generation_algorithm, maze.topology, maze.image_style
                    );
                    maze.save_image(&name)?;
                }
            }
        }

        Ok(())
    }

    #[test]
    fn test_random_mazes_image() -> AppResult<()> {
        for id in 0..10 {
            let maze = Maze::random(id, None);
            let name = format!("images/random_{}.png", id);
            maze.save_image(&name)?;
        }

        Ok(())
    }
}
