use bevy::prelude::*;
use rand::Rng;

use super::RuleSet;
use super::rules::*;

#[derive(Resource)]
pub struct Game {
    pub grid: Vec<Vec<Vec<bool>>>,
    pub generation: usize,
    pub first_disp: bool,
    pub cell_count: usize ,// per dim i.e. for cc = 16 => 16x16x16
    pub prob: f32,
    pub rule: RuleSet,
    pub speed: f32
}

impl Game {

    pub fn new(size: usize, prob:f32, rule: RuleSet, speed: f32) -> Game {
        let mut cell_count = 0;

        // additional check - even though buttons should keep 16 as minimum
        if size < 16 { cell_count = 16; } 
        else { cell_count = size; } 


        // if rule matches specific structure - i.e. specific generated pattern - else randomize
        let grid:Vec<Vec<Vec<bool>>> = match rule {
            RuleSet::ClosestTranslationFixed => {
                cell_count = 96;
                Game::closest_translation_fixed(cell_count)
            },
            // fixed as it just demonstrates carefully crafted stable forms
            RuleSet::Life4555Fixed => {
                if size < 48 {cell_count = 48;} // fixed bottom size , upwards will work just look more spaced
                Game::life_4555_fixed(cell_count)
            },
            _ => Game::randomize(prob, cell_count)
        };

        Game {
            grid,
            generation: 0,
            first_disp: true,
            cell_count,
            prob,
            rule,
            speed
        }
    }

    fn randomize(prob:f32,cell_count:usize) -> Vec<Vec<Vec<bool>>> {
        let mut rng = rand::thread_rng();
        let grid:Vec<Vec<Vec<bool>>> = (0..cell_count).map(|_| {
            (0..cell_count).map(|_| {
                (0..cell_count).map(|_| rng.gen_bool(prob as f64)).collect() // Prob% chance true/false as alive cells should be rarer initially.collect()
            }).collect()   
        }).collect();
        grid
    }

    pub fn reset(&mut self) {
        
        let grid:Vec<Vec<Vec<bool>>> = match self.rule {
            RuleSet::ClosestTranslationFixed => {
                Game::closest_translation_fixed(96)
            },
            RuleSet::Life4555Fixed => Game::life_4555_fixed(self.cell_count),
            _ => Game::randomize(self.prob, self.cell_count)
        };

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
                        RuleSet::Balanced => balanced(&self.grid, count, &mut new_grid, x,y,z),
                        RuleSet::Sparse => sparse(&self.grid,count, &mut new_grid, x,y,z),
                        RuleSet::Dense => dense(&self.grid,count, &mut new_grid, x,y,z),
                        RuleSet::Chaotic => chaotic(&self.grid,count, &mut new_grid, x,y,z),
                        RuleSet::NoDeath => no_death(&self.grid,count, &mut new_grid, x,y,z),
                        RuleSet::Tunnel => tunnel_like(&self.grid,count, &mut new_grid, x,y,z),
                        RuleSet::ClosestTranslation => closest_translation(&self.grid,count, &mut new_grid, x,y,z),
                        RuleSet::ClosestTranslationFixed => {
                            // recompute neighbors for different indexing
                            let count = self.count_neighbors(z, y, x);
                            // Additionally need to limit / skip life spawning one cell away from the barriers
                            // if x is depth - which defines slices - we keep track which values are actual barriers
                            // and for x+1 && x-1 we skip them (i.e. rule never applied => cell state untouched)

                            // If this cell is DEAD and is face-adjacent (6-neighborhood) to a barrier slice
                            // (as defined by the same pattern used when building the fixed layout),
                            // block births by skipping rule application (leave it false in new_grid).
                            if !self.grid[z][y][x] {
                                let block = Game::verify_barrier_margin(self.cell_count,z,y,x);
                                if block {
                                    // keep dead; do not apply the rule
                                    continue;
                                }
                            }

                            closest_translation(&self.grid,count, &mut new_grid, z,y,x)
                        },
                        RuleSet::Life4555 => life_4555(&self.grid,count, &mut new_grid, x,y,z),
                        RuleSet::Life4555Fixed => life_4555(&self.grid,count, &mut new_grid, x,y,z),
                    }
                    
                }
            }
        };

        self.grid = new_grid;
        self.generation +=1;
    }

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

    // generate fixed grids - reference page 14/28
    // we construct a sort of Barrier and inbetween it thin "more classic" conway universes
    // can exist
    fn closest_translation_fixed(cell_count:usize) -> Vec<Vec<Vec<bool>>> {
        let mut grid:Vec<Vec<Vec<bool>>> =vec![vec![vec![false; cell_count]; cell_count]; cell_count];

        // first construct parallel barriers 
        
        // -3 is precise as with these offsets 4..cell_count - 3 with creating holes modulo 3
        // we have both at the top and at the bottom the same starting point for fixed size of 96
        let boundary = cell_count -3; 
        
        let slice_spacing = 5; // 1 filled + 4 empty
        // depth
        for x in (4..(boundary)).step_by(slice_spacing) {
            // each "slice"
            for z in 4..(boundary)  {
                for y in 4..(boundary) {
                    grid[z][y][x] = true; // vertical slice at this x

                    // due to the holes in the barrier the centre and most of the plane will be stable
                    // under Rule R=(5,7,6,6) but its corners will deteriorate - eventually compromising
                    // entire structure - unless the corners are stabilized by small structures next to them
                    // I'll denote them stabilizers (originally unnamed in paper)

                    // now cut out in each slice three cubes in each corner
                    
                    // bottom left cut & stabilizer
                    if (z == 4 && y == 4) || (z == 4 && y == 5) || (z == 5 && y == 4) {
                        grid[z][y][x] = false;

                        // stabilizer - analog in other corners
                        if z!=5 && y !=4 {
                            grid[z][y][x+1] = true; 
                            grid[z][y][x+2] = true;
                            grid[z + 1][y - 1 ][x+1] = true;
                            grid[z + 1][y - 1][x+2] = true;

                            grid[z + 1][y][x+3] = true;// corner fill
                        }
                        
                        grid[z][y][x+3] = true; // corner piece
                        
                    }
                    // top left cut & stabilizer
                    if (z == 4 && y == boundary -1) || (z == 4 && y == boundary -2) || (z == 5 && y==  boundary -1) {
                        grid[z][y][x] = false;

                        if z!=5 && y !=boundary -1 { 
                            grid[z][y][x+1] = true;
                            grid[z][y][x+2] = true;
                            grid[z + 1][y + 1 ][x+1] = true;
                            grid[z + 1][y + 1][x+2] = true;

                            grid[z + 1][y][x+3] = true;// corner fill
                        }
                        
                        grid[z][y][x+3] = true; // corner piece

                    }

                    // bottom right cut & stabilizer
                    if (z == boundary -1 && y == 4) || (z == boundary -1 && y == 5) || (z == boundary -2 && y == 4) {
                        grid[z][y][x] = false;
                    
                        if z == boundary -2 && y == 4 {
                            grid[z][y][x+1] = true;
                            grid[z][y][x+2] = true;
                            grid[z + 1][y + 1 ][x+1] = true;
                            grid[z + 1][y + 1][x+2] = true;

                            grid[z][y + 1][x+3] = true;// corner fill
                        }

                        grid[z][y][x+3] = true; // corner piece
                    }

                    // top right cut & stabilizer
                    if (z == boundary -1 && y == boundary -1) || (z == boundary -1 && y == boundary -2) || (z == boundary -2 && y == boundary -1) {
                        grid[z][y][x] = false;
                        
                        if z == boundary -2 && y == boundary -1 {
                            grid[z][y][x+1] = true;
                            grid[z][y][x+2] = true;
                            grid[z + 1][y - 1 ][x+1] = true;
                            grid[z + 1][y - 1][x+2] = true;

                            grid[z][y - 1][x+3] = true;// corner fill
                        }
                        grid[z][y][x+3] = true; // corner piece
                    }

                    // insert the holes , mod 3 - we want every third cube (horizontally & vertically) to be dead

                    if z % 3 == 0 && y % 3 == 0 {
                        grid[z][y][x] = false;
                    }

                    // this generates in an even tighter intervall small conway universes that can exist in stable
                    // manner between the parallel barriers
                    if (z >= 15 && z <= boundary - 15) && (y >= 15 && y <= boundary - 15) {
                        let mut rng = rand::thread_rng();
                        // i.e. since the iterations run in steps of 4
                        // x => 6 , 10, 14 , ..
                        grid[z][y][x + 2] = rng.gen_bool(0.25);
                        grid[z][y][x + 3] = rng.gen_bool(0.25);
                    }

                }
            }
        }

        grid
    }

    // barriers need 1 block padding/margin where dead cells cant reanimate
    // this functions computes boolean for which we check whether to disable
    // rules i.e. whether to skip rule applying which prevents dead cell reanimation
    fn verify_barrier_margin(cells: usize, z:usize,y:usize,x:usize) -> bool {
        let cc = cells;
        let boundary = cc - 3;
        let slice_spacing = 5;

        // Inline checker - check if the position itself a base barrier cell
        // Matches generator's base plane + holes + corner cuts. (z,y,x order)
        let is_barrier_base = |zz: usize, yy: usize, xx: usize| -> bool {
            if xx < 4 || xx >= boundary { return false; }
            if zz < 4 || zz >= boundary { return false; }
            if yy < 4 || yy >= boundary { return false; }
            if (xx - 4) % slice_spacing != 0 { return false; }

            // holes
            if zz % 3 == 0 && yy % 3 == 0 { return false; }

            // corner cuts (same triples cut in the barrier generator)
            if (zz == 4 && (yy == 4 || yy == 5)) || (zz == 5 && yy == 4) { return false; }                               // bottom-left
            if (zz == 4 && (yy == boundary - 1 || yy == boundary - 2)) || (zz == 5 && yy == boundary - 1) { return false; } // top-left
            if (zz == boundary - 1 && (yy == 4 || yy == 5)) || (zz == boundary - 2 && yy == 4) { return false; }           // bottom-right
            if (zz == boundary - 1 && (yy == boundary - 1 || yy == boundary - 2)) || (zz == boundary - 2 && yy == boundary - 1) { return false; } // top-right

            true
        };

        let s = cc as isize;
        // face neighbors in (dz, dy, dx) with z y x indexing
        let dirs: [(isize, isize, isize); 6] = [(1,0,0), (-1,0,0), (0,1,0), (0,-1,0), (0,0,1), (0,0,-1)];

        let mut block = false;
        for (dz, dy, dx) in dirs.iter() {
            let nz = z as isize + dz;
            let ny = y as isize + dy;
            let nx = x as isize + dx;
            if nz < 0 || ny < 0 || nx < 0 || nz >= s || ny >= s || nx >= s { continue; }
            let (nzu, nyu, nxu) = (nz as usize, ny as usize, nx as usize);
            if is_barrier_base(nzu, nyu, nxu) {
                block = true;
                break;
            }
        }

        block
    }
   
    // reference page 20/28 in paper 
    // carefully construct each of the 4 forms in each of the 4
    // corners of the area
    fn life_4555_fixed(cell_count:usize) -> Vec<Vec<Vec<bool>>> {
        let mut grid:Vec<Vec<Vec<bool>>> =vec![vec![vec![false; cell_count]; cell_count]; cell_count];
        // compared to the more dynamic function for R = (5,7,6,6) this is purely static
        // to demonstrate the structures and how they remain unchanged
        
        // like before if we iterate through x ( z ( y ) ) -> grid[z][y][x]
        // x is the depth - slices , z left/right, y height
        
        // ----------------------------------------------------------------------------------
        // top left - bow looking structure
        // center
        grid[11][cell_count - 10][cell_count - 20] = true;
        grid[12][cell_count - 10][cell_count - 20] = true;
        grid[11][cell_count - 9][cell_count - 20] = true;
        grid[12][cell_count - 9][cell_count - 20] = true;

        grid[10][cell_count - 10][cell_count - 21] = true;
        grid[13][cell_count - 10][cell_count - 21] = true;
        grid[10][cell_count - 9][cell_count - 21] = true;
        grid[13][cell_count - 9][cell_count - 21] = true;
        // left wing
        grid[9][cell_count - 8][cell_count - 20] = true;
        grid[9][cell_count - 8][cell_count - 21] = true;

        grid[9][cell_count - 11][cell_count - 20] = true;
        grid[9][cell_count - 11][cell_count - 21] = true;
        
        grid[8][cell_count - 7][cell_count - 20] = true;
        grid[8][cell_count - 7][cell_count - 21] = true;

        grid[8][cell_count - 12][cell_count - 20] = true;
        grid[8][cell_count - 12][cell_count - 21] = true;

        grid[7][cell_count - 11][cell_count - 20] = true;
        grid[7][cell_count - 11][cell_count - 21] = true;
        grid[7][cell_count - 10][cell_count - 20] = true;
        grid[7][cell_count - 10][cell_count - 21] = true;
        grid[7][cell_count - 9][cell_count - 20] = true;
        grid[7][cell_count - 9][cell_count - 21] = true;
        grid[7][cell_count - 8][cell_count - 20] = true;
        grid[7][cell_count - 8][cell_count - 21] = true;
        // right wing
        grid[14][cell_count - 8][cell_count - 20] = true;
        grid[14][cell_count - 8][cell_count - 21] = true;

        grid[14][cell_count - 11][cell_count - 20] = true;
        grid[14][cell_count - 11][cell_count - 21] = true;
        
        grid[15][cell_count - 7][cell_count - 20] = true;
        grid[15][cell_count - 7][cell_count - 21] = true;

        grid[15][cell_count - 12][cell_count - 20] = true;
        grid[15][cell_count - 12][cell_count - 21] = true;

        grid[16][cell_count - 11][cell_count - 20] = true;
        grid[16][cell_count - 11][cell_count - 21] = true;
        grid[16][cell_count - 10][cell_count - 20] = true;
        grid[16][cell_count - 10][cell_count - 21] = true;
        grid[16][cell_count - 9][cell_count - 20] = true;
        grid[16][cell_count - 9][cell_count - 21] = true;
        grid[16][cell_count - 8][cell_count - 20] = true;
        grid[16][cell_count - 8][cell_count - 21] = true;
        
        // ----------------------------------------------------------------------------------
        // top right - circular spiral structure

        // inner ring - from backside to front wrap around
        grid[cell_count - 12][cell_count - 17][cell_count - 19] = true;
        grid[cell_count - 12][cell_count - 17][cell_count - 18] = true; 

        grid[cell_count - 13][cell_count - 16][cell_count - 19] = true;
        grid[cell_count - 13][cell_count - 16][cell_count - 20] = true; 
        grid[cell_count - 14][cell_count - 15][cell_count - 19] = true;
        grid[cell_count - 14][cell_count - 15][cell_count - 20] = true; 
        grid[cell_count - 13][cell_count - 14][cell_count - 19] = true;
        grid[cell_count - 13][cell_count - 14][cell_count - 20] = true; 

        grid[cell_count - 12][cell_count - 13][cell_count - 20] = true;
        grid[cell_count - 12][cell_count - 13][cell_count - 21] = true;    
        grid[cell_count - 11][cell_count - 12][cell_count - 20] = true;
        grid[cell_count - 11][cell_count - 12][cell_count - 21] = true;    
        grid[cell_count - 10][cell_count - 13][cell_count - 20] = true;
        grid[cell_count - 10][cell_count - 13][cell_count - 21] = true;      

        grid[cell_count - 9][cell_count - 14][cell_count - 21] = true;
        grid[cell_count - 9][cell_count - 14][cell_count - 22] = true;
        grid[cell_count - 8][cell_count - 15][cell_count - 21] = true;
        grid[cell_count - 8][cell_count - 15][cell_count - 22] = true;
        grid[cell_count - 9][cell_count - 16][cell_count - 21] = true;
        grid[cell_count - 9][cell_count - 16][cell_count - 22] = true;

        grid[cell_count - 10][cell_count - 17][cell_count - 23] = true;
        grid[cell_count - 10][cell_count - 17][cell_count - 22] = true; 
        grid[cell_count - 11][cell_count - 18][cell_count - 23] = true;
        grid[cell_count - 11][cell_count - 18][cell_count - 22] = true; 
        grid[cell_count - 12][cell_count - 17][cell_count - 23] = true;
        grid[cell_count - 12][cell_count - 17][cell_count - 22] = true; 

        grid[cell_count - 13][cell_count - 16][cell_count - 24] = true;
        grid[cell_count - 13][cell_count - 16][cell_count - 23] = true; 
        grid[cell_count - 14][cell_count - 15][cell_count - 24] = true;
        grid[cell_count - 14][cell_count - 15][cell_count - 23] = true; 
        grid[cell_count - 13][cell_count - 14][cell_count - 24] = true;
        grid[cell_count - 13][cell_count - 14][cell_count - 23] = true; 

        grid[cell_count - 12][cell_count - 13][cell_count - 24] = true;
        grid[cell_count - 12][cell_count - 13][cell_count - 25] = true;    
        grid[cell_count - 11][cell_count - 12][cell_count - 24] = true;
        grid[cell_count - 11][cell_count - 12][cell_count - 25] = true;    
        grid[cell_count - 10][cell_count - 13][cell_count - 24] = true;
        grid[cell_count - 10][cell_count - 13][cell_count - 25] = true;       

        grid[cell_count - 9][cell_count - 14][cell_count - 25] = true;
        grid[cell_count - 9][cell_count - 14][cell_count - 26] = true;
        grid[cell_count - 8][cell_count - 15][cell_count - 25] = true;
        grid[cell_count - 8][cell_count - 15][cell_count - 26] = true;
        grid[cell_count - 9][cell_count - 16][cell_count - 25] = true;
        grid[cell_count - 9][cell_count - 16][cell_count - 26] = true;

        grid[cell_count - 10][cell_count - 17][cell_count - 27] = true;
        grid[cell_count - 10][cell_count - 17][cell_count - 26] = true;
        
        // outer ring - center pillars
        grid[cell_count - 11][cell_count - 18][cell_count - 27] = true;
        grid[cell_count - 11][cell_count - 18][cell_count - 26] = true;
        grid[cell_count - 11][cell_count - 19][cell_count - 26] = true;
        grid[cell_count - 11][cell_count - 19][cell_count - 27] = true;
        grid[cell_count - 11][cell_count - 20][cell_count - 27] = true;
        grid[cell_count - 11][cell_count - 20][cell_count - 26] = true;

        grid[cell_count - 11][cell_count - 18][cell_count - 19] = true;
        grid[cell_count - 11][cell_count - 18][cell_count - 18] = true;
        grid[cell_count - 11][cell_count - 19][cell_count - 19] = true;
        grid[cell_count - 11][cell_count - 19][cell_count - 18] = true;
        grid[cell_count - 11][cell_count - 20][cell_count - 19] = true;
        grid[cell_count - 11][cell_count - 20][cell_count - 18] = true;
        // outer ring - bottom left + center bottom
        grid[cell_count - 10][cell_count - 21][cell_count - 19] = true;
        grid[cell_count - 10][cell_count - 21][cell_count - 18] = true;
        grid[cell_count - 9][cell_count - 21][cell_count - 19] = true;
        grid[cell_count - 9][cell_count - 21][cell_count - 18] = true;

        grid[cell_count - 12][cell_count - 21][cell_count - 27] = true;
        grid[cell_count - 12][cell_count - 21][cell_count - 26] = true;
        grid[cell_count - 13][cell_count - 21][cell_count - 27] = true;
        grid[cell_count - 13][cell_count - 21][cell_count - 26] = true;

        grid[cell_count - 14][cell_count - 20][cell_count - 27] = true;
        grid[cell_count - 14][cell_count - 20][cell_count - 26] = true;
        grid[cell_count - 15][cell_count - 19][cell_count - 26] = true;
        grid[cell_count - 15][cell_count - 19][cell_count - 25] = true;
        grid[cell_count - 16][cell_count - 18][cell_count - 25] = true;
        grid[cell_count - 16][cell_count - 18][cell_count - 24] = true;
        // outer ring - center left
        grid[cell_count - 17][cell_count - 17][cell_count - 25] = true;
        grid[cell_count - 17][cell_count - 17][cell_count - 24] = true;
        grid[cell_count - 17][cell_count - 16][cell_count - 25] = true;
        grid[cell_count - 17][cell_count - 16][cell_count - 24] = true;
        grid[cell_count - 17][cell_count - 15][cell_count - 25] = true;
        grid[cell_count - 17][cell_count - 15][cell_count - 24] = true;
        grid[cell_count - 17][cell_count - 14][cell_count - 25] = true;
        grid[cell_count - 17][cell_count - 14][cell_count - 24] = true;
        grid[cell_count - 17][cell_count - 13][cell_count - 25] = true;
        grid[cell_count - 17][cell_count - 13][cell_count - 24] = true;
        // outer ring - top left
        grid[cell_count - 16][cell_count - 12][cell_count - 25] = true;
        grid[cell_count - 16][cell_count - 12][cell_count - 24] = true;
        grid[cell_count - 15][cell_count - 11][cell_count - 24] = true;
        grid[cell_count - 15][cell_count - 11][cell_count - 23] = true;
        grid[cell_count - 14][cell_count - 10][cell_count - 22] = true;
        grid[cell_count - 14][cell_count - 10][cell_count - 23] = true;
        // outer ring - top center
        grid[cell_count - 13][cell_count - 9][cell_count - 22] = true;
        grid[cell_count - 13][cell_count - 9][cell_count - 23] = true;
        grid[cell_count - 12][cell_count - 9][cell_count - 22] = true;
        grid[cell_count - 12][cell_count - 9][cell_count - 23] = true;
        grid[cell_count - 11][cell_count - 9][cell_count - 22] = true;
        grid[cell_count - 11][cell_count - 9][cell_count - 23] = true;
        grid[cell_count - 10][cell_count - 9][cell_count - 22] = true;
        grid[cell_count - 10][cell_count - 9][cell_count - 23] = true;
        grid[cell_count - 9][cell_count - 9][cell_count - 22] = true;
        grid[cell_count - 9][cell_count - 9][cell_count - 23] = true;
        // outer ring - top right
        grid[cell_count - 6][cell_count - 12][cell_count - 21] = true;
        grid[cell_count - 6][cell_count - 12][cell_count - 20] = true;

        grid[cell_count - 7][cell_count - 11][cell_count - 21] = true;
        grid[cell_count - 7][cell_count - 11][cell_count - 22] = true;
        
        grid[cell_count - 8][cell_count - 10][cell_count - 22] = true;
        grid[cell_count - 8][cell_count - 10][cell_count - 23] = true;
        // outer ring - center right
        grid[cell_count - 5][cell_count - 13][cell_count - 21] = true;
        grid[cell_count - 5][cell_count - 13][cell_count - 20] = true;
        grid[cell_count - 5][cell_count - 14][cell_count - 21] = true;
        grid[cell_count - 5][cell_count - 14][cell_count - 20] = true;
        grid[cell_count - 5][cell_count - 15][cell_count - 21] = true;
        grid[cell_count - 5][cell_count - 15][cell_count - 20] = true;
        grid[cell_count - 5][cell_count - 16][cell_count - 21] = true;
        grid[cell_count - 5][cell_count - 16][cell_count - 20] = true;
        grid[cell_count - 5][cell_count - 17][cell_count - 21] = true;
        grid[cell_count - 5][cell_count - 17][cell_count - 20] = true;
        // outer ring - bottom right + centre bottom
        grid[cell_count - 6][cell_count - 18][cell_count - 21] = true;
        grid[cell_count - 6][cell_count - 18][cell_count - 20] = true;

        grid[cell_count - 7][cell_count - 19][cell_count - 20] = true;
        grid[cell_count - 7][cell_count - 19][cell_count - 19] = true;
        
        grid[cell_count - 8][cell_count - 20][cell_count - 18] = true;
        grid[cell_count - 8][cell_count - 20][cell_count - 19] = true;
        
        // ----------------------------------------------------------------------------------
        // bottom left - long structure
        // left key part
        grid[10][5][5] = true;
        grid[11][5][5] = true;
        grid[12][5][5] = true;
        grid[13][5][5] = true;

        grid[10][7][5] = true;
        grid[11][7][5] = true;
        grid[12][7][5] = true;
        grid[13][7][5] = true;

        grid[9][6][5] = true;
        grid[10][6][4] = true;
        grid[10][6][6] = true;
        // right key part
        grid[13][6][4] = true;
        grid[14][6][4] = true;
        grid[15][6][4] = true;
        grid[16][6][4] = true;

        grid[13][6][6] = true;
        grid[14][6][6] = true;
        grid[15][6][6] = true;
        grid[16][6][6] = true;

        grid[16][7][5] = true;
        grid[16][5][5] = true;
        grid[17][6][5] = true;
        
        // ----------------------------------------------------------------------------------
        // bottom right - immobilized gliders
        // lower glider
        grid[cell_count - 10][5][5] = true;
        grid[cell_count - 9][5][5] = true;
        grid[cell_count - 10][5][4] = true;
        grid[cell_count - 9][5][4] = true;        

        grid[cell_count - 10][6][6] = true;
        grid[cell_count - 9][6][6] = true;
        grid[cell_count - 10][6][3] = true;
        grid[cell_count - 9][6][3] = true;

        grid[cell_count - 10][7][5] = true;
        grid[cell_count - 10][7][4] = true;
        grid[cell_count - 11][7][5] = true;
        // upper glider
        grid[cell_count - 12][8][6] = true;
        grid[cell_count - 13][8][6] = true;
        grid[cell_count - 13][8][7] = true;

        grid[cell_count - 13][9][8] = true;
        grid[cell_count - 14][9][8] = true;
        grid[cell_count - 13][9][5] = true;
        grid[cell_count - 14][9][5] = true; 

        grid[cell_count - 13][10][6] = true;
        grid[cell_count - 14][10][6] = true;
        grid[cell_count - 13][10][7] = true;
        grid[cell_count - 14][10][7] = true;      

        // all structures spawned 

        grid
    }
}


