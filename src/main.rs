#![feature(const_generics)]
#![feature(const_evaluatable_checked)]
#![feature(proc_macro_hygiene)]

extern crate flame;
#[macro_use]
extern crate flamer;

// use std::io::Write;
// type BasicGrid = Grid<VecProvider>;

// use yew::prelude::*;

use std::{fs::File, io::Write};

use genetic_builder::Bot;
use grid::{Grid, VecProvider};

use crate::grid::{CaseValue, GridProvider, PlayResult};

mod grid;

mod genetic_builder;

enum Msg {
    Click(usize),
}

/* struct Model {
    link: ComponentLink<Self>,
    grid: Grid<VecProvider>,
    play_result: PlayResult,
}

impl Component for Model { */
/*     type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            grid: Grid::new(VecProvider::new(6, 7)),
            play_result: PlayResult::Played,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Click(index) => {
                self.play_result = self.grid.play(index);

                if !matches!(self.play_result, PlayResult::InvalidPosition) {
                    if self.play_result == PlayResult::Played && !self.grid.is_red_turn {
                        self.play_result = self.grid.play(self.grid.where_to_play());
                    }
                    true
                } else {
                    false
                }
            }
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        let win = match self.play_result {
            PlayResult::BlueWin => ("result blue-win", "Blue win !"),
            PlayResult::RedWin => ("result red-win", "Red win !"),
            PlayResult::NobodyWin => ("result draw", "Draw"),
            _ => ("result none", ""),
        };

        html! {
            <>
                <div class=classes!(win.0)>
                    { win.1 }
                </div>
                <div>
                    {
                        for (0..self.grid.height()).map(|y| html_nested! {
                            <div>
                            {
                                for (0..self.grid.width()).map(|x| {
                                    let index = self.grid.x_y_to_index(x, y);
                                    let cell = self.grid.get(index).unwrap();
                                    let cell_class = match cell {
                                        CaseValue::Red => "red",
                                        CaseValue::Blue => "blue",
                                        CaseValue::Black => "wall",
                                        CaseValue::Yellow => "playable",
                                        _ => ""
                                    };

                                    html_nested!{
                                        <div class=cell_class onclick=self.link.callback(move |_| Msg::Click(index))>
                                        </div>
                                    }
                                })
                            }
                            </div>})
                    }
                </div>
            </>
        }
    }
}
 */
/* fn main() {
    yew::start_app::<Model>();
} */

#[derive(Default, Debug)]
pub struct CompareResult {
    win: usize,
    loose: usize,
    none: usize,
}

impl CompareResult {
    fn score(&self) -> isize {
        self.win as isize - self.loose as isize
    }
}

pub fn compare_random(bot: &Bot) -> CompareResult {
    let mut result = CompareResult::default();
    for _ in 0..1000 {
        let mut grid = Grid::new(VecProvider::new(5, 6));
        grid.random_play();
        grid.random_play();
        loop {
            match grid.play(bot.best_play(&grid, CaseValue::Red)) {
                grid::PlayResult::InvalidPosition => {
                    println!("Invalid position!");
                    continue;
                }
                grid::PlayResult::Played => {}
                grid::PlayResult::RedWin => {
                    result.win += 1;
                    break;
                }
                grid::PlayResult::BlueWin => {
                    result.loose += 1;
                    break;
                }
                grid::PlayResult::NobodyWin => {
                    result.none += 1;
                    break;
                }
            }

            match grid.random_play() {
                grid::PlayResult::InvalidPosition => {
                    println!("Invalid position!");
                }
                grid::PlayResult::Played => {}
                grid::PlayResult::RedWin => {
                    result.win += 1;
                    break;
                }
                grid::PlayResult::BlueWin => {
                    result.loose += 1;
                    break;
                }
                grid::PlayResult::NobodyWin => {
                    result.none += 1;
                    break;
                }
            }
        }
    }
    result
}

fn main() {
    let mut bot = Bot::new();
    loop {
        let mut grid = Grid::new(VecProvider::new(5, 6));
        loop {
            term_render(&grid);
            print!("\n\nOù jouer (X Y) : ");
            std::io::stdout().flush().unwrap();
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();
            if input.trim().starts_with("train ") {
                let n = input.trim()[6..].trim();
                let mut o = n.split(' ');
                let n: u32 = o.next().map(|x| x.parse().unwrap()).unwrap_or(1);
                let mut q: f64 = o.next().map(|x| x.parse().unwrap()).unwrap_or(0.5);
                for _ in 0..n {
                    bot = bot.evolve(q);
                    q *= 0.75;
                    println!("Testing against random player...");
                }
                continue;
            }
            if input.trim().starts_with("load ") {
                bot = Bot::load_save(input.trim()[5..].trim().parse().unwrap());
                println!("Loaded save");
                continue;
            }
            if input.contains("test") {
                println!("Testing against random player...");
                println!("{:?}", compare_random(&bot));
                continue;
            }
            //let mut input = input.trim().split(" ").map(|x| x.parse().unwrap());
            //let pos = grid.x_y_to_index(input.next().unwrap(), input.next().unwrap());
            match grid.play(bot.best_play(&grid, CaseValue::Red)) {
                grid::PlayResult::InvalidPosition => {
                    println!("Invalid position!");
                    continue;
                }
                grid::PlayResult::Played => {}
                grid::PlayResult::RedWin => {
                    println!("Red win");
                    break;
                }
                grid::PlayResult::BlueWin => {
                    println!("Blue win");
                    break;
                }
                grid::PlayResult::NobodyWin => {
                    println!("Nobody win");
                    break;
                }
            }

            match grid.random_play() {
                grid::PlayResult::InvalidPosition => {
                    println!("Invalid position!");
                }
                grid::PlayResult::Played => {}
                grid::PlayResult::RedWin => {
                    println!("Red win");
                    break;
                }
                grid::PlayResult::BlueWin => {
                    println!("Blue win");
                    break;
                }
                grid::PlayResult::NobodyWin => {
                    println!("Nobody win");
                    break;
                }
            }
        }
    }
}
fn term_render(grid: &Grid<VecProvider>) {
    for (index, case) in grid.cases.iter().enumerate() {
        if index % grid.width() == 0 {
            print!("\n");
        }
        print!(
            "{}",
            match case {
                grid::CaseValue::Red => 'R',
                grid::CaseValue::Blue => 'B',
                grid::CaseValue::Yellow => '-',
                grid::CaseValue::White => ' ',
                grid::CaseValue::Black => '#',
            }
        );
    }
    println!()
}
