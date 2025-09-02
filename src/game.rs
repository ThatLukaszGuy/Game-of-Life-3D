use bevy::prelude::*;
use rand::Rng;

#[derive(Clone, Copy, PartialEq, PartialOrd,Debug)]
pub enum RuleSet {
    Balanced,
    Dense,
    Sparse,
    Chaotic,
    NoDeath,
}

#[derive(Resource)]
pub struct Game {
    pub grid: Vec<Vec<Vec<bool>>>,
    pub generation: usize,
    pub first_disp: bool,
    pub cell_count: usize ,// per dim i.e. for cc = 16 => 16x16x16
    pub prob: f64,
    pub rule: RuleSet
}
// todo : add "custom config" variations that allow spawning in specific structures rather than random generation
impl Game {

    pub fn new(size: usize, prob:f64, rule: RuleSet) -> Game {
        let mut cell_count = 0;

        if size < 16 { cell_count = 16; } 
        else { cell_count = size; } 


        // if rule matches specific structure - i.e. specific generated pattern - else randomize
        let grid:Vec<Vec<Vec<bool>>> = match rule {
            _ => Game::randomize(prob, cell_count)
        };

        Game {
            grid,
            generation: 0,
            first_disp: true,
            cell_count,
            prob,
            rule
        }
    }

    pub fn randomize(prob:f64,cell_count:usize) -> Vec<Vec<Vec<bool>>> {
        let mut rng = rand::thread_rng();
        let grid:Vec<Vec<Vec<bool>>> = (0..cell_count).map(|_| {
            (0..cell_count).map(|_| {
                (0..cell_count).map(|_| rng.gen_bool(prob)).collect() // Prob% chance true/false as alive cells should be rarer initially.collect()
            }).collect()   
        }).collect();
        grid
    }

    pub fn reset(&mut self) {
        let mut rng = rand::thread_rng();

        let grid:Vec<Vec<Vec<bool>>> = (0..self.cell_count).map(|_| {
            (0..self.cell_count).map(|_| {
                (0..self.cell_count).map(|_| rng.gen_bool(self.prob)).collect() 
            }).collect()   
        }).collect();

        self.grid = grid;
        self.generation = 0;
        self.first_disp = true;
    }

    pub fn advance_state(&mut self) {
        self.first_disp = false;

        let mut new_grid:Vec<Vec<Vec<bool>>> = (0..self.cell_count).map(|_| {
            (0..self.cell_count).map(|_| {
                (0..self.cell_count).map(|_| false).collect() // just init empty vec 3d vec of false
            }).collect()   
        }).collect();

        for x in 0..self.grid.len() {
            for y in 0..self.grid.len() {
                for z in 0..self.grid.len() {
                    // to get alive neighbors for each cell
                    let count = self.count_neighbors(x, y, z);
                    
                    match self.rule {
                        RuleSet::Balanced => self.balanced(count, &mut new_grid, x,y,z),
                        RuleSet::Sparse => self.sparse(count, &mut new_grid, x,y,z),
                        RuleSet::Dense => self.dense(count, &mut new_grid, x,y,z),
                        RuleSet::Chaotic => self.chaotic(count, &mut new_grid, x,y,z),
                        RuleSet::NoDeath => self.no_death(count, &mut new_grid, x,y,z),
                    }
                    
                }
            }
        };

        self.grid = new_grid;
        self.generation +=1;
    }


    fn balanced(&self, count: usize, g: &mut Vec<Vec<Vec<bool>>>, x:usize,y:usize,z:usize) {
        if self.grid[x][y][z] {
            if count >= 5 && count <= 7 { g[x][y][z] = true }
        } else {
            if count == 6 || count == 5 {g[x][y][z] = true; }
        }
    }

    fn sparse(&self,count: usize, g: &mut Vec<Vec<Vec<bool>>>, x:usize,y:usize,z:usize) {
        if self.grid[x][y][z] {
            if count >= 3 && count <= 5 { g[x][y][z] = true }
        } else {
            if  count == 5 {g[x][y][z] = true; }
        }
    }

    fn dense(&self,count: usize, g: &mut Vec<Vec<Vec<bool>>>, x:usize,y:usize,z:usize) {
        if self.grid[x][y][z] {
            if count >= 4 && count <= 9 { g[x][y][z] = true }
        } else {
            if count == 6 || count == 5 {g[x][y][z] = true; }
        }
    }
    
    // todo: find better params
    fn chaotic(&self,count: usize, g: &mut Vec<Vec<Vec<bool>>>, x:usize,y:usize,z:usize) {
        if self.grid[x][y][z] {
            if count >= 5 && count <= 8 { g[x][y][z] = true }
        } else {
            if count == 6 || count == 5 {g[x][y][z] = true; }
        }
    }

    fn no_death(&self,count: usize, g: &mut Vec<Vec<Vec<bool>>>, x:usize,y:usize,z:usize) {
        if self.grid[x][y][z] {
            g[x][y][z] = true // a born cell never dies
        } else {
            if count == 5 {g[x][y][z] = true; }
        }
    }
    
    // create more mode/rules for interesting structures/distributions
    // these will not generate the grid randomly but rather place patterns into it 
    // and overwrite/insert custom rules

    // Rule Set - recommended 96x96x96





    pub fn count_neighbors(&mut self, x:usize,y:usize,z:usize ) -> usize {
    
        // enumarate all possible neighbor combinations i.e. 
        // for cell a at (0,0,0) relative its neighbors 
        // are in b in {-1,0,1} (all combos => 27-1 = 26)
        let size = self.grid.len() as isize;
        let mut count = 0;
    
        for dz in -1..=1 {
            for dy in -1..=1 {
                for dx in -1..=1 {
                    if dx == 0 && dy == 0 && dz == 0 {
                        continue; // skip self
                    }
    
                    let nx = x as isize + dx;
                    let ny = y as isize + dy;
                    let nz = z as isize + dz;
    
                    if nx >= 0 && nx < size && ny >= 0 && ny < size && nz >= 0 && nz < size {
                        if self.grid[nx as usize][ny as usize][nz as usize] {
                            count += 1;
                        }
                    }
                }
            }
        }
        count
    }
    
}


