/* <const WIDTH: usize, const HEIGHT: usize>
where
    [(); WIDTH * HEIGHT]: Sized, */
#[derive(Clone)]
pub struct Grid<T: GridProvider + Clone> {
    pub cases: T, /* [CaseValue; WIDTH * HEIGHT] */
    last_play: Option<usize>,
    pub is_red_turn: bool,
    was_everything_yellow: bool,
}

impl<T: GridProvider + Clone> Display for Grid<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (index, case) in self.cases.iter().enumerate() {
            if index % self.width() == 0 {
                write!(f, "\n")?;
            }
            write!(
                f,
                "{}",
                match case {
                    CaseValue::Red => 'R',
                    CaseValue::Blue => 'B',
                    CaseValue::Yellow => '-',
                    CaseValue::White => ' ',
                    CaseValue::Black => '#',
                }
            )?;
        }
        writeln!(f)
    }
}

#[derive(Clone)]
pub struct VecProvider {
    height: usize,
    width: usize,
    cases: Vec<CaseValue>,
}

#[derive(Clone)]
pub struct ArrayProvider<const WIDTH: usize, const HEIGHT: usize> {
    cases: Vec<CaseValue>,
}

impl<const WIDTH: usize, const HEIGHT: usize> GridProvider for ArrayProvider<WIDTH, HEIGHT> {
    fn get(&self, index: usize) -> Option<&CaseValue> {
        self.cases.get(index)
    }

    fn get_mut(&mut self, index: usize) -> Option<&mut CaseValue> {
        self.cases.get_mut(index)
    }

    unsafe fn get_unchecked(&self, index: usize) -> &CaseValue {
        self.cases.get_unchecked(index)
    }

    fn iter_mut(&mut self) -> IterMut<CaseValue> {
        self.cases.iter_mut()
    }

    unsafe fn get_unchecked_mut(&mut self, index: usize) -> &mut CaseValue {
        self.cases.get_unchecked_mut(index)
    }

    fn width(&self) -> usize {
        WIDTH
    }

    fn height(&self) -> usize {
        HEIGHT
    }

    fn iter(&self) -> Iter<CaseValue> {
        self.cases.iter()
    }
}

impl VecProvider {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            height,
            width,
            cases: (0..(width * height)).map(|_| CaseValue::Yellow).collect(),
        }
    }
}

impl GridProvider for VecProvider {
    fn get(&self, index: usize) -> Option<&CaseValue> {
        self.cases.get(index)
    }

    fn get_mut(&mut self, index: usize) -> Option<&mut CaseValue> {
        self.cases.get_mut(index)
    }

    unsafe fn get_unchecked(&self, index: usize) -> &CaseValue {
        self.cases.get_unchecked(index)
    }

    fn iter_mut(&mut self) -> IterMut<CaseValue> {
        self.cases.iter_mut()
    }

    fn iter(&self) -> Iter<CaseValue> {
        self.cases.iter()
    }

    unsafe fn get_unchecked_mut(&mut self, index: usize) -> &mut CaseValue {
        self.cases.get_unchecked_mut(index)
    }

    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.height
    }
}

pub trait GridProvider {
    fn get(&self, index: usize) -> Option<&CaseValue>;
    fn get_mut(&mut self, index: usize) -> Option<&mut CaseValue>;
    unsafe fn get_unchecked(&self, index: usize) -> &CaseValue;
    fn iter_mut(&mut self) -> IterMut<CaseValue>;
    unsafe fn get_unchecked_mut(&mut self, index: usize) -> &mut CaseValue;
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn iter(&self) -> Iter<CaseValue>;
}

impl<T: GridProvider + Clone> Grid<T> {
    pub fn new(grid_provider: T) -> Self {
        Self {
            cases: grid_provider,
            last_play: None,
            is_red_turn: true,
            was_everything_yellow: true,
        }
    }

    pub fn place_blocks(&mut self, i: usize, rand_func: impl Fn() -> usize) {
        for _ in 0..i {
            self.set(
                rand_func() % (self.height() * self.width()),
                CaseValue::Black,
            )
        }
    }

    pub fn width(&self) -> usize {
        self.cases.width()
    }

    pub fn height(&self) -> usize {
        self.cases.height()
    }

    pub fn x_y_to_index(&self, x: usize, y: usize) -> usize {
        x + y * self.width()
    }

    pub fn get(&self, index: usize) -> Option<&CaseValue> {
        self.cases.get(index)
    }

    pub fn set(&mut self, index: usize, value: CaseValue) {
        if let Some(e) = self.cases.get_mut(index) {
            *e = value;
        }
    }

    fn follow(&self, index: usize, dir: Direction) -> Option<usize> {
        let u = (index as isize) + dir.offset(self.width());
        if u < 0 {
            None
        } else if u >= ((self.width() * self.height()) as isize) {
            None
        } else {
            let u = u as usize;
            match dir {
                Direction::North => Some(u),
                Direction::East | Direction::SouthEast | Direction::NorthEast => {
                    if u % self.width() == 0 {
                        None
                    } else {
                        Some(u)
                    }
                }
                Direction::West | Direction::NorthWest | Direction::SouthWest => {
                    if (u + 1) % self.width() == 0 {
                        None
                    } else {
                        Some(u)
                    }
                }
                Direction::South => Some(u),
            }
        }
    }

    // TODO: Optimize this function
    fn replace_around_if(&mut self, index: usize, value: CaseValue, condition: CaseValue) {
        for dir in Direction::iter() {
            if let Some(u) = self.follow(index, dir) {
                // Perfectly safe since `follow` check bounds.
                let o = unsafe { self.cases.get_unchecked_mut(u) };
                if *o == condition {
                    *o = value;
                }
            }
        }
    }

    fn check_direction_and_yellow_first(&mut self, index: usize, compare: CaseValue) -> PlayResult {
        let mut p = false;
        let mut dirs: HashSet<Direction> = HashSet::new();
        for dir in Direction::iter() {
            if let Some(u) = self.follow(index, dir) {
                // Perfectly safe since `follow` check bounds.
                let o = unsafe { self.cases.get_unchecked_mut(u) };
                if *o == CaseValue::White {
                    p = true;
                    *o = CaseValue::Yellow;
                } else if *o == compare {
                    dirs.insert(dir);
                    if dirs.contains(&dir.mirror()) {
                        if compare == CaseValue::Blue {
                            return PlayResult::RedWin;
                        } else {
                            return PlayResult::BlueWin;
                        }
                    }
                    if let Some(u) = self.follow(u, dir) {
                        let o = unsafe { self.cases.get_unchecked_mut(u) };
                        if *o == compare {
                            if compare == CaseValue::Blue {
                                return PlayResult::RedWin;
                            } else {
                                return PlayResult::BlueWin;
                            }
                        }
                    }
                }
            }
        }
        if !p {
            self.cases.iter_mut().for_each(|x| {
                if matches!(x, CaseValue::White) {
                    p = true;
                    *x = CaseValue::Yellow
                }
            });
            self.was_everything_yellow = true;
            if !p {
                return PlayResult::NobodyWin;
            }
        } else {
            self.was_everything_yellow = false;
        }
        PlayResult::Played
    }

    pub fn random_play(&mut self) -> PlayResult {
        self.play(*self.get_yellows().choose(&mut rand::thread_rng()).unwrap())
    }

    pub fn play(&mut self, index: usize) -> PlayResult {
        match self.get(index) {
            Some(CaseValue::Yellow) => {
                if self.was_everything_yellow {
                    // TODO: Optimization tip | Instead of removing everything only remove something that is not at
                    // 1 of distance of the new play position
                    self.cases.iter_mut().for_each(|x| {
                        if matches!(x, CaseValue::Yellow) {
                            *x = CaseValue::White
                        }
                    });
                } else {
                    if let Some(e) = &self.last_play {
                        self.replace_around_if(*e, CaseValue::White, CaseValue::Yellow);
                    }
                }
                let turn_case_color = if self.is_red_turn {
                    CaseValue::Red
                } else {
                    CaseValue::Blue
                };
                self.set(index, turn_case_color);
                self.is_red_turn = !self.is_red_turn;

                self.last_play = Some(index);
                self.check_direction_and_yellow_first(index, turn_case_color)
            }
            _ => PlayResult::InvalidPosition,
        }
    }

    pub fn follow_and_get(&self, index: usize, dir: Direction) -> Option<(usize, &CaseValue)> {
        if let Some(u) = self.follow(index, dir) {
            // Perfectly safe since `follow` check bounds.
            Some((u, unsafe { self.cases.get_unchecked(u) }))
        } else {
            None
        }
    }

    fn choose_best_indexes(&self, turn_case_color: CaseValue) -> Vec<(usize, f32)> {
        let mut i = self
            .cases
            .iter()
            .enumerate()
            .flat_map(|(index, value)| {
                if *value == CaseValue::Yellow {
                    Some((index, self.evaluate_play(index, turn_case_color.invert())))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        i.sort_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap());
        i.into_iter().take(3).collect()
    }

    pub fn get_yellows(&self) -> Vec<usize> {
        self.cases
            .iter()
            .enumerate()
            .filter(|(_, value)| matches!(value, CaseValue::Yellow))
            .map(|(a, _)| a)
            .collect()
    }

    fn begin_simul(&self, turn_case_color: CaseValue, max_depth: u8) -> Option<(usize, PathIssue)> {
        if max_depth == 0 {
            return None;
        }
        let mut max_points = 0;
        let mut current = None;
        for (i, s) in self.choose_best_indexes(turn_case_color) {
            let mut current_issue = PathIssue {
                win: 0,
                none: 0,
                lose: 0,
                nowin: 0,
            };
            if s < OUR_OUR_COLOR {
                let mut new = self.clone();
                new.play(i);
                for p in self.get_yellows() {
                    let mut new = self.clone();
                    match new.play(p) {
                        PlayResult::RedWin | PlayResult::BlueWin => {
                            current_issue.win += 1;
                        }
                        PlayResult::NobodyWin => {
                            current_issue.nowin += 1;
                        }
                        _ => (),
                    }
                    if let Some((
                        _,
                        PathIssue {
                            win,
                            none,
                            lose,
                            nowin,
                        },
                    )) = new.begin_simul(turn_case_color, max_depth - 1)
                    {
                        current_issue.win += win;
                        current_issue.none += none;
                        current_issue.lose += lose;
                        current_issue.nowin += nowin;
                    }
                }
            } else {
                current_issue.lose += (max_depth * max_depth) as usize;
            }
            let cc = current_issue.count();
            if max_points < cc {
                current = Some((i, current_issue));
                max_points = cc;
            }
        }

        current
    }

    pub fn where_to_play(&self) -> usize {
        let p = self.begin_simul(
            if self.is_red_turn {
                CaseValue::Blue
            } else {
                CaseValue::Red
            },
            5,
        );
        //ConsoleService::log(&format!("{:?}", p));
        p.unwrap().0
    }

    pub fn evaluate_play(&self, index: usize, color: CaseValue) -> f32 {
        let mut score = 0.;
        let other = color.invert();
        let mut play_everywhere = true;
        for dir in Direction::iter() {
            if let Some((u, o)) = self.follow_and_get(index, dir) {
                if o.empty() {
                    play_everywhere = false;
                    if let Some((_, o)) = self.follow_and_get(u, dir) {
                        if o.empty() {
                            score += EMPTY_EMPTY;
                        } else if *o == color {
                            score += EMPTY_OUR_COLOR;
                        }
                    }
                    if let Some((_, o)) = self.follow_and_get(index, dir.mirror()) {
                        if o.empty() {
                            score += EMPTY_EMPTY;
                        } else if *o == color {
                            score += EMPTY_OUR_COLOR;
                        }
                    }
                } else if *o == color {
                    if let Some((_, o)) = self.follow_and_get(u, dir) {
                        if o.empty() {
                            score += EMPTY_OUR_COLOR;
                        } else if *o == color {
                            score += OUR_OUR_COLOR;
                        }
                    }
                    if let Some((_, o)) = self.follow_and_get(index, dir.mirror()) {
                        if o.empty() {
                            score += EMPTY_OUR_COLOR;
                        } else if *o == color {
                            score += OUR_OUR_COLOR;
                        }
                    }
                } else if *o == other {
                    if let Some((_, o)) = self.follow_and_get(u, dir) {
                        if *o == other {
                            score += THEIR_THEIR_COLOR;
                        }
                    }
                    if let Some((_, o)) = self.follow_and_get(index, dir.mirror()) {
                        if *o == other {
                            score += THEIR_THEIR_COLOR;
                        }
                    }
                }
            }
        }
        if play_everywhere {
            score += PLAY_EVERYWHERE;
        }
        score
    }
}

#[derive(Debug)]
struct PathIssue {
    win: usize,
    none: usize,
    lose: usize,
    nowin: usize,
}

impl PathIssue {
    fn count(&self) -> usize {
        usize::MAX / 2 + self.win * 1 - self.lose * 50
    }
}

const EMPTY_EMPTY: f32 = 0.05;
const EMPTY_OUR_COLOR: f32 = 2.;
const THEIR_THEIR_COLOR: f32 = 0.5;
const OUR_OUR_COLOR: f32 = 1000.;
const PLAY_EVERYWHERE: f32 = 4.;

// X X X X X
// X X X X X
// X X X   X
// X X X O X
// X X X B X

use std::{
    collections::HashSet,
    fmt::Display,
    slice::{Iter, IterMut},
};

use rand::prelude::SliceRandom;
use strum::IntoEnumIterator; // 0.17.1
use strum_macros::EnumIter;
// use yew::services::ConsoleService; // 0.17.1

#[derive(EnumIter, Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum Direction {
    North,
    East,
    West,
    South,
    NorthEast,
    NorthWest,
    SouthEast,
    SouthWest,
}

impl Direction {
    pub fn mirror(&self) -> Self {
        match self {
            Self::North => Self::South,
            Self::South => Self::North,
            Self::East => Self::West,
            Self::West => Self::East,

            Self::NorthEast => Self::SouthWest,
            Self::NorthWest => Self::SouthEast,
            Self::SouthEast => Self::NorthWest,
            Self::SouthWest => Self::NorthEast,
        }
    }
    pub fn offset(&self, width: usize) -> isize {
        match self {
            Self::North => -(width as isize),
            Self::East => 1,
            Self::West => -1,
            Self::South => width as isize,
            Self::NorthEast => -(width as isize) + 1,
            Self::NorthWest => -(width as isize) - 1,
            Self::SouthEast => width as isize + 1,
            Self::SouthWest => width as isize - 1,
        }
    }
}

#[derive(PartialEq)]
pub enum PlayResult {
    InvalidPosition,
    Played,
    RedWin,
    BlueWin,
    NobodyWin,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum CaseValue {
    Red,
    Blue,
    Yellow,
    White,
    Black,
}

impl CaseValue {
    fn empty(&self) -> bool {
        matches!(self, Self::White | Self::Yellow)
    }
    fn invert(&self) -> Self {
        match self {
            Self::Red => Self::Blue,
            Self::Blue => Self::Red,
            _ => unreachable!(),
        }
    }
}
