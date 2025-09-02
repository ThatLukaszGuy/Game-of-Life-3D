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
const SIZE: usize = 64; // above 64 laggy
const RULE: RuleSet = RuleSet::Sparse;
const SPEED:f32 = 2.;

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
        setup_counter,
        setup_pause,
        spawn_camera,
        spawn_all_cells.after(setup_game),
        spawn_light,
        setup_game.before(spawn_camera),
        setup_timer
    ));

    // despawn game camera/UI when leaving InGame (optional but clean)
    app.add_systems(OnExit(AppState::InGame), despawn_game_camera);

    app.add_systems(Update, (
        update_generation_counter,
        camera_look,
        camera_move.after(camera_look),
        focus_events,
        toggle_grab.run_if(input_just_released(KeyCode::Escape)),
        place_cubes,
        game_step
    ).run_if(in_state(AppState::InGame)));

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

// Generation counter

#[derive(Component)]
struct GenerationText;

fn setup_counter(mut commands: Commands) {
    commands.spawn((
        Text::new("\n  Generation: 0"),
        TextFont {
            font_size: 20.0,
            ..default()
        },
        TextColor(WHITE.into()),
        GenerationText,
    ));
}

fn update_generation_counter(
    game: Res<Game>,
    mut query: Query<&mut Text, With<GenerationText>>,
) {
    // skip if generation hasn't advanced
    if !game.is_changed() {
        return; 
    }

    for mut text in &mut query {
        *text = Text::new(format!("\n  Generation: {}", game.generation));
    }
}


// Cube spawning logic and pipeline of placing, generating cubes and setting all their properties 
// Now using visibility flags instead of re/de - spawning each cube per tick

// new component for mapping a spawned entity to a grid cell 
#[derive(Component)]
struct CubeCell {
    x: usize,
    y: usize,
    z: usize,
}

// Resource that stores the flat-indexed list of pre-spawned entities
#[derive(Resource)]
struct CellEntities {
    entities: Vec<Entity>,
    size: usize,
}

// Initial render of the cubes - more expensive in gen. 0 - never run after 
fn spawn_all_cells(
    mut commands: Commands,
    cube_data: Res<CubeData>,
    maybe_cells: Option<Res<CellEntities>>, // look if it already exists
) {
    // If already spawned  - e.g. re-entering => skip
    if maybe_cells.is_some() {
        return;
    }

    let mut entities: Vec<Entity> = Vec::with_capacity(SIZE*SIZE*SIZE);

    for x in 0..SIZE {
        for y in 0..SIZE {
            for z in 0..SIZE {
                let pos = Vec3::new(x as f32, y as f32, -(z as f32));
                let ent = commands
                    .spawn((
                        Transform::from_translation(pos),
                        Mesh3d(cube_data.mesh()),
                        MeshMaterial3d(cube_data.material()),
                        LifeCube,
                        CubeCell { x, y, z },
                        Visibility::Hidden,
                    ))
                    .id();
                entities.push(ent);
            }
        }
    }

    commands.insert_resource(CellEntities {
        entities,
        size: SIZE,
    });

}

// helper to compute linear index = x * n*n + y * n + z
#[inline]
fn linear_index(x: usize, y: usize, z: usize, n: usize) -> usize {
    x * n * n + y * n + z
}



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

fn place_cubes(
    mut commands: Commands,
    mut game: ResMut<Game>,
    keys: Res<ButtonInput<KeyCode>>,
    mut paused: ResMut<Paused>,
    mut observer: Single<&mut Transform, With<Observer>>,
    cell_entities: Option<Res<CellEntities>>,
){

    // intial render after which game_step function takes over
    if game.first_disp {
        if let Some(cells) = cell_entities.as_ref() {
            let n = cells.size;
            for x in 0..n {
                for y in 0..n {
                    for z in 0..n {
                        // new update system to toggle visibility - i.e. intial render is heavier as it generates
                        // SIZE^3 cubes but subsequent ticks/generations are cheaper as they're only flag triggers

                        if game.grid[x][y][z] {
                            let idx = linear_index(x, y, z, n);
                            let ent = cells.entities[idx];
                            // show alive cells
                            commands.entity(ent).insert(Visibility::Visible);
                        } else {
                            // ensure dead cells are hidden - they were created hidden but safe to set
                            let idx = linear_index(x, y, z, n);
                            let ent = cells.entities[idx];
                            commands.entity(ent).insert(Visibility::Hidden);
                        }
                    }
                }
            }

            game.first_disp = false;
        } 

    }

    // reset game
    if keys.just_pressed(KeyCode::KeyR) {
        let mid = game.grid.len() as f32 /2.;
        game.reset();
        **observer = Transform::from_xyz(mid, mid, 3.0 * mid)
        .looking_at(Vec3::new(mid, mid, mid), Vec3::Y);


        // hide/show entities according to the reset state
        if let Some(cells) = cell_entities.as_ref() {
            let n = cells.size;
            for x in 0..n {
                for y in 0..n {
                    for z in 0..n {
                        let idx = linear_index(x, y, z, n);
                        let ent = cells.entities[idx];
                        if game.grid[x][y][z] {
                            commands.entity(ent).insert(Visibility::Visible);
                        } else {
                            commands.entity(ent).insert(Visibility::Hidden);
                        }
                    }
                }
            }
        }

    }

    // listen for pause
    if keys.just_pressed(KeyCode::Space) {
        paused.0 = !paused.0;
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
    game: Res<Game>
) {
    // also lock during first/initial load which will prevent camera sometimes spasming/lagging to the side
    if !window.focused || game.first_disp  {
        return;
    }

    // change to use x. divided by min width and hight, this will make the looking around feel same on different resolutions
    let sensitivity = 50. / window.width().min(window.height());

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

// move 3d camera if state is paused - move speed should depend on size of the simulation
// as original placement of camera is also dependent on size
fn camera_move(
    mut observer: Single<&mut Transform, With<Observer>>,
    inputs: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    paused: Res<Paused>,
) {
    if !paused.0 {
        return; // early exit if NOT paused
    }

    let cam_speed = SIZE as f32 / 3.;
    let mut delta = Vec3::ZERO;
    if inputs.pressed(KeyCode::KeyA) {
        delta.x -= cam_speed;
    }
    if inputs.pressed(KeyCode::KeyD) {
        delta.x += cam_speed;
    }
    if inputs.pressed(KeyCode::KeyW) {
        delta.z += cam_speed;
    }
    if inputs.pressed(KeyCode::KeyS) {
        delta.z -= cam_speed;
    }

    let forward = observer.forward().as_vec3() * delta.z;
    let left = observer.right().as_vec3() * delta.x;
    let to_move = forward + left;
    observer.translation += to_move * time.delta_secs() * SPEED;
}


// Timer and Game state transition logic

#[derive(Resource)]
struct StepTimer(Timer);

fn setup_timer(mut commands: Commands) {
    commands.insert_resource(StepTimer(Timer::from_seconds(1. / SPEED, TimerMode::Repeating)));
}

// pause logic
#[derive(Resource)]
struct Paused(bool);

fn setup_pause(mut commands: Commands) {
    commands.insert_resource(Paused(false));
}

fn setup_game(mut commands: Commands, selected: Res<SelectedRule>) {
    let game = Game::new(SIZE, PROB, selected.0);
    commands.insert_resource(game);
}

fn game_step(
    time: Res<Time>,
    mut timer: ResMut<StepTimer>,
    mut game: ResMut<Game>,
    mut commands: Commands,
    paused: Res<Paused>,
    cell_entities: Option<Res<CellEntities>>
) {

    if paused.0 {
        return; // early exit if paused
    }

    if timer.0.tick(time.delta()).just_finished() {
        

        // only run steps after the initial display
        if !game.first_disp {
            game.advance_state();

            // update visuals using pre-spawned entities - just flag switch
            if let Some(cells) = cell_entities.as_ref() {
                let n = cells.size;
                for x in 0..n {
                    for y in 0..n {
                        for z in 0..n {
                            let idx = linear_index(x, y, z, n);
                            let ent = cells.entities[idx];
                            if game.grid[x][y][z] {
                                commands.entity(ent).insert(Visibility::Visible);
                            } else {
                                commands.entity(ent).insert(Visibility::Hidden);
                            }
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
                Text::new(format!("Will start automatically after choosing Mode\nSpawn probability at {}, \nSize per dimension {}\nTick Speed per Generation at {}\nAll configurable by altering constants at the top of the main.rs file\n\nPress: \n'Esc' to (un)focus\n'R' to reset\n'Space' to pause simulation\nWhen Paused - can move around with WASD keys", {PROB}, {SIZE}, {SPEED})),
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

