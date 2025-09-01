use bevy::{
    color::palettes::{css::WHITE, tailwind::{ GRAY_200, PURPLE_300, PURPLE_500, PURPLE_600}}, 
    input::{common_conditions::input_just_released, mouse::AccumulatedMouseMotion},
    prelude::*, 
    window::{CursorGrabMode, PrimaryWindow, WindowFocused},
    ui::{AlignItems, JustifyContent, FlexDirection, UiRect, Val}
};

pub mod game;

use game::Game;
use game::RuleSet;

// for simple experimenting bind all to easily findable constants - default values
const PROB: f64 = 0.05;
const SIZE: usize = 50; // above 64 laggy
const RULE: RuleSet = RuleSet::Sparse;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Game of life 3d".to_string(),
            ..default()
        }),
        ..default()
    }));

    app.init_state::<AppState>();
    app.insert_resource(SelectedRule(RULE));
    
    // menu tweak
    app.add_systems(OnEnter(AppState::Menu), setup_menu);
    app.add_systems(Update, ( 
        rule_buttons_interactions.run_if(in_state(AppState::Menu)) 
    ));
    // despawn menu camera/UI when leaving Menu
    app.add_systems(OnExit(AppState::Menu), despawn_menu);

    // when entering the game spawn game camera etc
    app.add_systems(OnEnter(AppState::InGame), (
        spawn_camera,
        spawn_light,
        setup_game.before(spawn_camera),
        setup_timer
    ));
    // despawn game camera/UI when leaving InGame (optional but clean)
    app.add_systems(OnExit(AppState::InGame), despawn_game_camera);

    app.add_systems(Update, (
        camera_look,
        spawn_cube,
        focus_events,
        toggle_grab.run_if(input_just_released(KeyCode::Escape)),
        place_cubes.before(spawn_cube),
        game_step.before(spawn_cube)
    ).run_if(in_state(AppState::InGame)));

    app.add_event::<CubeSpawn>();
    app.init_resource::<CubeData>();
    app.insert_resource(ClearColor(Color::srgb(0.82353, 0.66667, 0.94902))); //210., 170., 242.
    app.add_observer(apply_grab);
    app.run();
}

// Camera switch from UI -> Simul.
#[derive(States, Clone, Eq, PartialEq, Debug, Hash, Default)]
enum AppState {
    #[default]
    Menu,
    InGame,
}

#[derive(Component)]
struct MenuCamera;

#[derive(Component)]
struct MenuUI;

#[derive(Component)]
struct GameCamera;

fn despawn_menu(mut commands: Commands, cameras: Query<Entity, With<MenuCamera>>, uis: Query<Entity, With<MenuUI>>) {
    for cam in cameras.iter() {
        commands.entity(cam).despawn();
    }
    for ui in uis.iter() {
        commands.entity(ui).despawn();
    }
}

fn despawn_game_camera(mut commands: Commands, cams: Query<Entity, With<GameCamera>>) {
    for cam in cams.iter() {
        commands.entity(cam).despawn();
    }
}


// Cube spawning logic and pipeline of placing, generating cubes and setting all their properties

// generator for both reusing resource for spawning the cubes
// assigns semi-random color to it and is made in a resuable way
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
    mut commands: Commands,
    mut spawner: EventWriter<CubeSpawn>,
    mut game: ResMut<Game>,
    keys: Res<ButtonInput<KeyCode>>,
){
    // intial render after which game_step function takes over
    if game.first_disp {     

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

    // reset game
    if keys.just_pressed(KeyCode::KeyR) {
        game.reset();
    }

    // generation counter - needs to be rerendered per generation and/or on reset
    commands.spawn((
        Text::new(format!("\n  Generation: {}", game.generation)),
        TextFont {
            font_size: 20.0,
            ..default()
        },
        
        TextColor(WHITE.into()),
        LifeCube
    ));
}

// spawner func, each rendered cube is tagged 
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
        GameCamera
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
    // change to use 100. divided by min width and hight, this will make the looking around feel same on different resolutions
    let sensitivity = 75. / window.width().min(window.height());

    // get angles as euler angles because they are more natural then Quats
    let (mut yaw, mut pitch, _) = observer.rotation.to_euler(EulerRot::YXZ);
    // subtract y movement for pitch - up/down
    pitch -= mouse_movement.delta.y * time.delta_secs() * sensitivity;

    // subtract x movement for yaw - left/right
    yaw -= mouse_movement.delta.x * time.delta_secs() * sensitivity;

    // stops looking past straight up, it will start flickering as the value becomes negative
    pitch = pitch.clamp(-1.57, 1.57);

    // recalculate the Quat from the yaw and pitch, yaw first or end up with unintended role
    observer.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, 0.);
}


// Timer and Game state transition logic

#[derive(Resource)]
struct StepTimer(Timer);

fn setup_timer(mut commands: Commands) {
    commands.insert_resource(StepTimer(Timer::from_seconds(0.7, TimerMode::Repeating)));
}

fn setup_game(mut commands: Commands, selected: Res<SelectedRule>) {
    let game = Game::new(SIZE, PROB, selected.0);
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

// UI Text, Button Select and state transition config

fn setup_menu(
    mut commands: Commands, 
    selected: Option<Res<SelectedRule>>
) {
    commands.spawn((Camera2d::default(), MenuCamera));

    // Root menu node 
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::SpaceEvenly,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,

                ..default()
            },
            BackgroundColor(Color::srgb(0.82353, 0.66667, 0.94902)),
            MenuUI,
        ))
        .with_children(|parent| {
        // add Title + Descriptions
            parent.spawn((
                Text::new("Choose Simulation Mode"),
                TextFont {
                    font_size: 40.0,
                    ..default()
                },
                TextColor(WHITE.into()),
                
            ));
                let current = selected.map(|r| r.0).unwrap_or(RULE);
                parent
                    .spawn(Node {
                        margin: UiRect::top(Val::Px(10.0)),
                        width: Val::Percent(300.0),
                        height: Val::Px(150.0),
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(8.0),
                        ..default()
                    })
                    .with_children(|row| {
                        let options = [
                            (RuleSet::Balanced, "Balanced"),
                            (RuleSet::Dense, "Dense"),
                            (RuleSet::Sparse, "Sparse"),
                            (RuleSet::Chaotic, "Chaotic"),
                            (RuleSet::NoDeath, "No Death"),
                        ];
    
                        for (rule_variant, label) in options {
                            // choose initial background depending on selected
                            let bg = if rule_variant == current {
                                PURPLE_300.into()
                            } else {
                                PURPLE_600.into()
                            };
    
                            row.spawn((
                                Button,
                                Node {
                                    padding: UiRect::all(Val::Px(8.0)),
                                    ..default()
                                },
                                BackgroundColor(bg),
                                RuleButton { rule: rule_variant },
                            ))
                            .with_children(|btn| {
                                btn.spawn((
                                    Text::new(label),
                                    TextFont {
                                        font_size: 30.0,
                                        ..default()
                                    },
                                    TextColor(WHITE.into()),
                                ));
                            });
                        }
                    });

            parent.spawn((
                Text::new(format!("Will start automatically after choosing Mode\nSpawn probability at {}, \nSize per dimension {}\nconfigurable by altering constants at the top of the file\n\nPress: \n'Esc' to (un)focus\n'R' to reset", {PROB}, {SIZE})),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(GRAY_200.into()),
            ));
        });
}

#[derive(Resource, Clone, Copy, Debug)]
struct SelectedRule(pub RuleSet);

#[derive(Component)]
struct RuleButton {
    rule: RuleSet,
}

fn rule_buttons_interactions(
    mut interactions: Query<
        (&Interaction, &mut BackgroundColor, &RuleButton),
        (Changed<Interaction>, With<Button>)
    >,
    mut selected: ResMut<SelectedRule>,
    mut next_state: ResMut<NextState<AppState>>
) {

    let sel_col: BackgroundColor = BackgroundColor(PURPLE_300.into());
    let hov_col: BackgroundColor = BackgroundColor(PURPLE_500.into());
    let def_col: BackgroundColor = BackgroundColor(PURPLE_600.into());

    for (interaction, mut bg, rule_btn) in &mut interactions {
        match *interaction {
            Interaction::Pressed => {
                // update selected rule enum
                selected.0 = rule_btn.rule;
                *bg = sel_col.clone();
                next_state.set(AppState::InGame);
            }
            Interaction::Hovered => {
                *bg = hov_col.clone();
            }
            Interaction::None => {
                *bg = def_col.clone();
            }
        }
    }
}

