#[derive(Clone, Copy, PartialEq, PartialOrd,Debug)]
pub enum RuleSet {
    // basic rules found w little experimentation/testing
    Balanced,
    Dense,
    Sparse,
    Chaotic,
    NoDeath,
    
    // research paper patterns
    Tunnel,

    ClosestTranslation,
    ClosestTranslationFixed,
    
    Life4555,
    Life4555Fixed,
}

// These initial "basic" rules were just made by me from basic experimentation & trial at error
// tested mainly at around 5% probability
// these usually either decay either into small round oscillating structures or depending on
// starting prob. grow boundlessly over time
    
// balanced growth that causes sphere like objects 
pub fn balanced(grid: &Vec<Vec<Vec<bool>>>, count: usize, g: &mut Vec<Vec<Vec<bool>>>, x:usize,y:usize,z:usize) {
    if grid[x][y][z] {
        if count >= 5 && count <= 7 { g[x][y][z] = true }
    } else {
        if count == 6 || count == 5 {g[x][y][z] = true; }
    }
}

// at 5% prob. starts by deteriorating fast and then over time becoming larger and denser
pub fn sparse(grid: &Vec<Vec<Vec<bool>>>,count: usize, g: &mut Vec<Vec<Vec<bool>>>, x:usize,y:usize,z:usize) {
    if grid[x][y][z] {
        if count >= 3 && count <= 5 { g[x][y][z] = true }
    } else {
        if  count == 5 {g[x][y][z] = true; }
    }
}

// very dense with boundless growth - filled cube with very little "air" or space inside
pub fn dense(grid: &Vec<Vec<Vec<bool>>>,count: usize, g: &mut Vec<Vec<Vec<bool>>>, x:usize,y:usize,z:usize) {
    if grid[x][y][z] {
        if count >= 4 && count <= 9 { g[x][y][z] = true }
    } else {
        if count == 6 || count == 5 {g[x][y][z] = true; }
    }
}
    
// supposed to cause very rapid chaotic growth
pub fn chaotic(grid: &Vec<Vec<Vec<bool>>>,count: usize, g: &mut Vec<Vec<Vec<bool>>>, x:usize,y:usize,z:usize) {
    if grid[x][y][z] {
        if count >= 5 && count <= 8 { g[x][y][z] = true }
    } else {
        if count == 6 || count == 5 {g[x][y][z] = true; }
    }
}

// simple experimental condition that a cell thats born never dies - boundless growth  
pub fn no_death(grid: &Vec<Vec<Vec<bool>>>,count: usize, g: &mut Vec<Vec<Vec<bool>>>, x:usize,y:usize,z:usize) {
    if grid[x][y][z] {
        g[x][y][z] = true // a born cell never dies
    } else {
        if count == 5 {g[x][y][z] = true; }
    }
}

// experimental to keep initial state - literally doesnt change any cell status ever
pub fn no_change(grid: &Vec<Vec<Vec<bool>>>,count: usize, g: &mut Vec<Vec<Vec<bool>>>, x:usize,y:usize,z:usize) {
    if grid[x][y][z] { g[x][y][z] = true } 
    else { g[x][y][z] = false }
}
    
// The rules below are taken from the observations and conclusions made by the paper Carter Bays - referenced in the
// README.md file of this project - for clarity the same notation as in the paper is utilized to describe which rules
// are implemented and how the results are reproduced

// R = (10,21,10,21) - page 4/28 is supposed to create long tunnel like structures at high initial
// probabilities
// Simul. Results: (64^3 grid): 
// prob < 25% => cells die out completely within first 5 generations
// 25% <= prob <= 26% => cells forms short tunnel like structures in some distance from each other without 
//                       boundless growth
// prob >= 27% => cells first approach long tunnel like structures and then grow boundlessly and fill
//                space or immediately (prob >= 40%) start growing boundlessly
pub fn tunnel_like(grid: &Vec<Vec<Vec<bool>>>,count: usize, g: &mut Vec<Vec<Vec<bool>>>, x:usize,y:usize,z:usize) {
    if grid[x][y][z] {
        if count >= 10 && count <= 21 { g[x][y][z] = true }
    } else {
        if count >= 10 && count <= 21  {g[x][y][z] = true; }
    }
}

// R = (5,7,6,6) - its the closest to an analog translation of the original 2d ruleset of the game 
// (Conway's R'=(2,3,3,3)), to paraphrase: [the rule] leads to stable oscillating forms that are similar
// in many ways to Conway's and a key characteristic that the rule allows for a large number of small stable
// asymetric objects
// Simul. Results: at 25% spawn probability after certain n of rounds the structures stabilize exactly as expected
// result can be fully reproduced - example screenshot also in README.md
    
// for page 14 deploy fixed structures - Section 2.3 Fig. 7
pub fn closest_translation(grid: &Vec<Vec<Vec<bool>>>,count: usize, g: &mut Vec<Vec<Vec<bool>>>, x:usize,y:usize,z:usize) {
    if grid[x][y][z] {
        if count >= 5 && count <= 7 { g[x][y][z] = true }
    } else {
        if count == 6  {g[x][y][z] = true; }
    }
}


// R = (4,5,5,5) - Completely different Universe to Conway with no analog object existing
// Reference page 17/28 - in the README.md image p6.jpg structures like A. , B. , D. and some others
// can be reproduced naturally
// Simul. Results: at 25% spawn probability after certain 70-100 of rounds (to stabilize) the structures become symmetric
// and occasionally oscillating between what seem to be fixed symmetric objects 
    
// for page 19 - 20 deploy fixed structures - Section 2.3.3
pub fn life_4555(grid: &Vec<Vec<Vec<bool>>>,count: usize, g: &mut Vec<Vec<Vec<bool>>>, x:usize,y:usize,z:usize) {
    if grid[x][y][z] {
        if count >= 4 && count <= 5 { g[x][y][z] = true }
    } else {
        if count == 5  {g[x][y][z] = true; }
    }
}