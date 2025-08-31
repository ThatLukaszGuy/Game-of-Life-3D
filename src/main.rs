use bevy::{
    input::{common_conditions::input_just_released, mouse::AccumulatedMouseMotion},
    prelude::*,
    window::{CursorGrabMode, PrimaryWindow, WindowFocused},
};
use rand::Rng;

pub mod game;

const PROB: f64 = 0.05;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_systems(Startup, (
        spawn_camera,
        spawn_light,
        setup_game,
        setup_timer
    ));
    app.add_systems(Update, (
        camera_look,
        spawn_cube,
        place_cubes.before(spawn_cube),
        game_step.before(spawn_cube)
    ));
    app.add_event::<CubeSpawn>();
    app.init_resource::<CubeData>();
    app.run();
}

// cube spawning logic
#[derive(Resource)]
struct CubeData {
    mesh: Handle<Mesh>,
    materials: Vec<Handle<StandardMaterial>>,
    rng: std::sync::Mutex<rand::rngs::StdRng>,
}

impl CubeData {
    fn mesh(&self) -> Handle<Mesh> {
        self.mesh.clone()
    }
    fn material(&self) -> Handle<StandardMaterial> {
        use rand::seq::SliceRandom;
        let mut rng = self.rng.lock().unwrap();
        self.materials.choose(&mut *rng).unwrap().clone()
    }
}

impl FromWorld for CubeData {
    fn from_world(world: &mut World) -> Self {
        // makes the cubes spawn with a random color
        use rand::SeedableRng;
        let mesh = world.resource_mut::<Assets<Mesh>>().add(Cuboid::from_length(1.)); 
        let mut materials = Vec::new();
        let mut material_assets = world.resource_mut::<Assets<StandardMaterial>>();
        for i in 0..36 {
            let color = Color::hsl((i * 10) as f32, 1., 0.5);
            materials.push(material_assets.add(StandardMaterial {
                base_color: color,
                ..Default::default()
            }));
        }
        let seed = *b"GameOfLifeRandomSimulationColor1";
        CubeData {
            mesh,
            materials,
            rng: std::sync::Mutex::new(rand::rngs::StdRng::from_seed(seed)),
        }
    }
}
#[derive(Component)]
struct LifeCube;


#[derive(Event)]
struct CubeSpawn {
    position: Vec3,
}


fn place_cubes(
    inputs: Res<ButtonInput<MouseButton>>,
    mut spawner: EventWriter<CubeSpawn>,
    window: Single<&Window, With<PrimaryWindow>>,
    time: Res<Time>,
    mut game: ResMut<Game>,
    
) {
    if inputs.just_pressed(MouseButton::Left) && game.first_disp {     

        for x in 0..game.grid.len() {
            for y in 0..game.grid.len() {
                for z in 0..game.grid.len() {
                    if game.grid[x][y][z] {
                        spawner.write(CubeSpawn {
                            position: Vec3::new(x as f32, y as f32, z as f32 * -1.),
                        });
                    }
                }
            }
        }
        game.first_disp = false;
    }


}


fn spawn_cube(
    mut events: EventReader<CubeSpawn>,
    mut commands: Commands,
    cube_data: Res<CubeData>,
) {
    for spawn in events.read() {
        commands.spawn((
            Transform::from_translation(spawn.position),
            Mesh3d(cube_data.mesh()),
            MeshMaterial3d(cube_data.material()),
            LifeCube,  
        ));
    }
}



// observer
#[derive(Component)]
struct Observer;

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(10.0, 10.0, 20.0)  
            .looking_at(Vec3::ZERO, Vec3::Y), 
        Observer,
    ));
}

fn spawn_light(mut commands: Commands) {
        
        commands.spawn((
            DirectionalLight {
                shadows_enabled: true,
                illuminance: 10000.0,
                ..default()
            },
            // Tilt light
            Transform::from_rotation(
                Quat::from_euler(EulerRot::XYZ, -std::f32::consts::FRAC_PI_4, std::f32::consts::FRAC_PI_4, 0.0)
            )
        ));
    
        commands.insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 200.0,
            affects_lightmapped_meshes: true
        });
}

fn camera_look(    
    mut observer: Single<&mut Transform, With<Observer>>,
    mouse_movement: Res<AccumulatedMouseMotion>,
    time: Res<Time>,
    window: Single<&Window, With<PrimaryWindow>>,
) {
    // change to use 100. divided by min width and hight, this will make the game feel the same even on different resolutions
    let sensitivity = 100. / window.width().min(window.height());

    // get angles as euler angles because they are more natural then Quats, don't need role
    let (mut yaw, mut pitch, _) = observer.rotation.to_euler(EulerRot::YXZ);
    // subtract y movement for pitch - up/down
    pitch -= mouse_movement.delta.y * time.delta_secs() * sensitivity;

    // subtract x movement for yaw - left/right
    yaw -= mouse_movement.delta.x * time.delta_secs() * sensitivity;

    // stops you looking past straight up, it will flickering as the value becomes negative
    pitch = pitch.clamp(-1.57, 1.57);

    // recalculate the Quat from the yaw and pitch, yaw first or we end up with unintended role
    observer.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, 0.);
}

#[derive(Resource)]
struct StepTimer(Timer);

fn setup_timer(mut commands: Commands) {
    commands.insert_resource(StepTimer(Timer::from_seconds(0.5, TimerMode::Repeating)));
}

fn game_step(
    time: Res<Time>,
    mut timer: ResMut<StepTimer>,
    mut game: ResMut<Game>,
    mut spawner: EventWriter<CubeSpawn>,
    query: Query<Entity, With<LifeCube>>,
    mut commands: Commands
) {
    if timer.0.tick(time.delta()).just_finished() {
        
        // only run steps after the initial display
        if !game.first_disp {
            // advance state
            game.advance_state();

            // despawn previous visual cubes
            for entity in query.iter() {
                commands.entity(entity).despawn();
            }

            // emit spawn events for the new generation (spawn_cube will run after this system)
            for x in 0..game.grid.len() {
                for y in 0..game.grid.len() {
                    for z in 0..game.grid.len() {
                        if game.grid[x][y][z] {
                            spawner.write(CubeSpawn {
                                position: Vec3::new(x as f32, y as f32, z as f32 * -1.),
                            });
                        }
                    }
                }
            }
        }


        
    }
}

// game 
pub enum RuleSet {
    Balanced,
    Dense,
    Sparse,
    Chaotic,
    NoDeath
}

#[derive(Resource)]
struct Game {
    grid: Vec<Vec<Vec<bool>>>,
    generation: usize,
    first_disp: bool,
    cell_count: usize ,// per dim i.e. for cc = 16 => 16x16x16
    prob: f64,
    rule: RuleSet
}

fn setup_game(mut commands: Commands) {
    let game = Game::new(40, PROB, RuleSet::Balanced);
    commands.insert_resource(game);
}


impl Game {

    fn new(size: usize, prob:f64, rule: RuleSet) -> Game {
        let mut cell_count = 0;
        let mut rng = rand::thread_rng();

        if size < 16 { cell_count = 16; } 
        else { cell_count = size; } 
        let mut grid:Vec<Vec<Vec<bool>>> = (0..cell_count).map(|_| {
            (0..cell_count).map(|_| {
                (0..cell_count).map(|_| rng.gen_bool(prob)).collect() // Prob% chance true/false as alive cells should be rarer initially.collect()
            }).collect()   
        }).collect();

        Game {
            grid,
            generation: 0,
            first_disp: true,
            cell_count,
            prob,
            rule
        }
    }

    fn reset(&mut self) {
        let mut rng = rand::thread_rng();

        let mut grid:Vec<Vec<Vec<bool>>> = (0..self.cell_count).map(|_| {
            (0..self.cell_count).map(|_| {
                (0..self.cell_count).map(|_| rng.gen_bool(self.prob)).collect() 
            }).collect()   
        }).collect();

        self.grid = grid;
        self.generation = 0;
        self.first_disp = true;
    }

    fn advance_state(&mut self) {
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
                    // balanced impl for now
                    if self.grid[x][y][z] {
                        if count >= 5 && count <= 7 { new_grid[x][y][z] = true }
                    } else {
                        if count == 6 || count == 5 { new_grid[x][y][z] = true; }
                    }

                }
            }
        };

        self.grid = new_grid;
        self.generation +=1;
        //match self.rule {
        //    RuleSet::Balanced => todo!(),
        //    RuleSet::Sparse => todo!(),
        //    RuleSet::Dense => todo!(),
        //    RuleSet::Chaotic => todo!(),
        //    RuleSet::NoDeath => todo!()
        //}
    }

    fn count_neighbors(&mut self, x:usize,y:usize,z:usize ) -> usize {
    
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

