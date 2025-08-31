use bevy::{
    input::{common_conditions::input_just_released, mouse::AccumulatedMouseMotion},
    prelude::*,
    window::{CursorGrabMode, PrimaryWindow, WindowFocused},
};

pub mod game;

use game::Game;
use game::RuleSet;

const PROB: f64 = 0.05;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_systems(Startup, (
        spawn_camera,
        spawn_light,
        setup_game.before(spawn_camera),
        setup_timer
    ));
    app.add_systems(Update, (
        camera_look,
        spawn_cube,
        focus_events,
        toggle_grab.run_if(input_just_released(KeyCode::Escape)),
        place_cubes.before(spawn_cube),
        game_step.before(spawn_cube)
    ));
    app.add_event::<CubeSpawn>();
    app.init_resource::<CubeData>();
    app.add_observer(apply_grab);
    app.run();
}

// Cube spawning logic and pipeline of placing, generating cubes and setting all their properties

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

// used as "tagging" so it can be despawned later
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


// Observer/Camera logic and positioning

#[derive(Component)]
struct Observer;

fn spawn_camera(mut commands: Commands, game: Res<Game>) {

    // have camera look roughly at middle of cube structure

    let mid = game.grid.len() as f32 /2.;
    println!("{}",mid);
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(mid, mid, 3.*mid)  
            .looking_at(Vec3::from_array([mid,mid,mid]), Vec3::Y), 
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
    if !window.focused {
        return;
    }
    // change to use 100. divided by min width and hight, this will make the game feel the same even on different resolutions
    let sensitivity = 75. / window.width().min(window.height());

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


// Timer and Game state transition logic

#[derive(Resource)]
struct StepTimer(Timer);

fn setup_timer(mut commands: Commands) {
    commands.insert_resource(StepTimer(Timer::from_seconds(0.5, TimerMode::Repeating)));
}

fn setup_game(mut commands: Commands) {
    let game = Game::new(32, PROB, RuleSet::Balanced);
    commands.insert_resource(game);
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

// Grab & Focus on Esc key s.t. u can only look iff ur focused

#[derive(Event, Deref)]
struct GrabEvent(bool);

fn apply_grab(
    // tells bevy what event to watch for with this observer
    grab: Trigger<GrabEvent>,
    mut window: Single<&mut Window, With<PrimaryWindow>>,
) {
    if **grab {
        window.cursor_options.visible = false;
        window.cursor_options.grab_mode = CursorGrabMode::Locked
    } else {
        window.cursor_options.visible = true;
        window.cursor_options.grab_mode = CursorGrabMode::None;
    }
}

fn focus_events(mut events: EventReader<WindowFocused>, mut commands: Commands) {
    if let Some(event) = events.read().last() {
        commands.trigger(GrabEvent(event.focused));
    }
}

fn toggle_grab(mut window: Single<&mut Window, With<PrimaryWindow>>, mut commands: Commands) {
    window.focused = !window.focused;
    commands.trigger(GrabEvent(window.focused));
}