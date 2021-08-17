use std::{
    convert::TryInto,
    io::Write,
    sync::{
        atomic::AtomicBool,
        mpsc::{channel, sync_channel},
        Arc, Mutex,
    },
    thread::JoinHandle,
    time::Instant,
};

use lib_neural_network::{nlib::Layer, LayerTopology, Network};
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::grid::{CaseValue, Grid, GridProvider, VecProvider};

#[derive(Clone)]
pub struct Bot {
    /* layer_in: Layer<{ 5 * 6 }, { 5 * 6 }>,
    layer_hidden: Layer<{ 5 * 6 }, 1>, */
    net: Network,
}

impl Bot {
    pub fn new() -> Self {
        Self {
            net: Network::random(
                &mut rand::thread_rng(),
                &[LayerTopology(6 * 5), LayerTopology(6 * 5), LayerTopology(1)],
            ), /* layer_in: Layer::random(),
               layer_hidden: Layer::random(), */
        }
    }

    pub fn auto_save(&self) {
        let n = format!("saves/{}.json", std::fs::read_dir("saves").unwrap().count());
        std::fs::write(&n, self.net.save()).unwrap();
        println!("New save in `{}`", n);
    }

    pub fn load_save(n: usize) -> Self {
        Bot {
            net: Network::load(&std::fs::read_to_string(&format!("saves/{}.json", n)).unwrap()),
        }
    }

    fn build_handle(
        ol: Arc<AtomicBool>,
        var: Arc<Mutex<Option<Bot>>>,
        this: Arc<Bot>,
        mutation_ratio: f64,
    ) -> JoinHandle<()> {
        std::thread::spawn(move || {
            let i = Instant::now();
            let cscore = (0..100)
                .map(|_| crate::compare_random(&this).score())
                .sum::<isize>()
                / 100;
            loop {
                if ol.load(std::sync::atomic::Ordering::SeqCst) {
                    break;
                }
                let i1 = this.mutate(mutation_ratio);
                let p = this.other_win(&i1);
                if p == -1 {
                    continue;
                }
                let s = crate::compare_random(&i1).score();
                if s > cscore - 40 {
                    if ol.load(std::sync::atomic::Ordering::SeqCst) {
                        break;
                    }
                    let bs = (0..100)
                        .map(|_| crate::compare_random(&i1).score())
                        .sum::<isize>()
                        / 100;
                    if ol.load(std::sync::atomic::Ordering::SeqCst) {
                        break;
                    }
                    if bs > cscore {
                        ol.store(true, std::sync::atomic::Ordering::SeqCst);
                        i1.auto_save();
                        *var.lock().unwrap() = Some(i1);
                        println!("Improved in {:?} new score = {}", i.elapsed(), bs);
                        break;
                    }
                }
                if p > 1_000_000 {
                    println!("Learn timeout");
                    *var.lock().unwrap() = Some(this.as_ref().clone());
                    break;
                }
            }
        })
    }

    pub fn evolve_mt(self, mutation_ratio: f64) -> Self {
        let var: Arc<Mutex<Option<Bot>>> = Arc::new(Mutex::new(None));
        let this = Arc::new(self);
        let bo = Arc::new(AtomicBool::new(false));
        let k: Vec<JoinHandle<()>> = (0..8)
            .map(|_| Self::build_handle(bo.clone(), var.clone(), this.clone(), mutation_ratio))
            .collect();
        k.into_iter().for_each(|x| x.join().unwrap());
        let k = var.lock().unwrap();
        k.as_ref().unwrap().clone()
    }

    pub fn evolve(self, mutation_ratio: f64) -> Self {
        let i = Instant::now();
        let cscore = (0..100)
            .map(|_| crate::compare_random(&self).score())
            .sum::<isize>()
            / 100;
        loop {
            let i1 = self.mutate(mutation_ratio);
            let p = self.other_win(&i1);
            if p == -1 {
                continue;
            }
            let s = crate::compare_random(&i1).score();
            if s > cscore - 40 {
                let bs = (0..100)
                    .map(|_| crate::compare_random(&i1).score())
                    .sum::<isize>()
                    / 100;
                if bs > cscore {
                    data_record(mutation_ratio, i.elapsed().as_millis() as u64, bs);
                    println!("Improved in {:?} new score = {}", i.elapsed(), bs);
                    i1.auto_save();
                    return i1;
                }
            }
            if p > 1_000_000 {
                println!("Learn timeout");
                return self;
            }
        }
    }

    pub fn other_win(&self, s: &Self) -> isize {
        let mut grid = Grid::new(VecProvider::new(5, 6));
        grid.is_red_turn = true;
        loop {
            //println!("{}", grid);
            match grid.play(self.best_play(&grid, CaseValue::Red)) {
                crate::grid::PlayResult::InvalidPosition => unreachable!(),
                crate::grid::PlayResult::Played => (),
                crate::grid::PlayResult::RedWin => {
                    //println!("{}", grid);
                    //println!("Red wins 2");
                    return -1;
                }
                crate::grid::PlayResult::BlueWin => return 1,
                crate::grid::PlayResult::NobodyWin => {
                    println!("Nobody wins");
                    return 0;
                }
            }
            match grid.play(s.best_play(&grid, CaseValue::Blue)) {
                crate::grid::PlayResult::InvalidPosition => unreachable!(),
                crate::grid::PlayResult::Played => (),
                crate::grid::PlayResult::BlueWin => return -1,
                crate::grid::PlayResult::RedWin => {
                    //println!("{}", grid);
                    //println!("Red wins 1");
                    return 1;
                }
                crate::grid::PlayResult::NobodyWin => {
                    //println!("Nobody wins");
                    return 0;
                }
            }
        }
    }

    pub fn mutate(&self, mutation_ratio: f64) -> Self {
        Self {
            net: self.net.mutate(mutation_ratio), /* layer_in: self.layer_in.mutate(),
                                                  layer_hidden: self.layer_hidden.mutate(), */
        }
    }

    pub fn execute(&self, grid: &Grid<VecProvider>, color: CaseValue) -> f32 {
        let cases/*: [f32; { 6 * 5 }] */ = grid
            .cases
            .iter()
            .map(|x| match x {
                crate::grid::CaseValue::Red => {
                    if color == CaseValue::Red {
                        1.
                    } else {
                        0.
                    }
                }
                crate::grid::CaseValue::Blue => {
                    if color == CaseValue::Red {
                        0.
                    } else {
                        1.
                    }
                }
                crate::grid::CaseValue::Yellow => 0.5,
                crate::grid::CaseValue::White => 0.5,
                crate::grid::CaseValue::Black => 0.5,
            })
            .collect::<Vec<_>>();
        self.net.propagate(cases)[0]
        /*     .try_into()
        .unwrap(); */
        /* let cases = self.layer_in.execute(cases.into());
        let cases = self.layer_hidden.execute(cases);
        *cases.get(0).unwrap() */
    }

    pub fn best_play(&self, grid: &Grid<VecProvider>, color: CaseValue) -> usize {
        let mut p = grid.clone();
        let o = grid.get_yellows();
        let mut o = o.into_iter();
        let t = o.next().unwrap();
        p.set(t, color);
        let mut k = (self.execute(&p, color), t);
        p.set(t, CaseValue::Yellow);
        while let Some(t) = o.next() {
            p.set(t, color);
            let n = (self.execute(&p, color), t);
            p.set(t, CaseValue::Yellow);
            if n.0 < k.0 {
                k = n;
            }
        }
        k.1
    }
}

// 10.3 - 603
// 20.7 - 709

fn data_record(mut_ratio: f64, time: u64, score: isize) {
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open("data.csv")
        .unwrap();
    file.write_all(format!("{},{},{}\n", mut_ratio, time, score).as_bytes())
        .unwrap();
}
