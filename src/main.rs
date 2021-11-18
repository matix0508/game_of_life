use bevy::prelude::*;

const ARENA_WIDTH: u32 = 30;
const ARENA_HEIGHT: u32 = 30;

const SCREEN_WIDTH: f32 = 700.0;
const SCREEN_HEIGHT: f32 = 700.0;



fn setup(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.insert_resource(Materials {
        alive_material: materials.add(Color::rgb(1., 1., 1.).into()),
        dead_material: materials.add(Color::rgb(0.1, 0.1, 0.1).into())
    });
    commands.insert_resource(Grid::default());

}

fn main() {
   App::build()
        .insert_resource(WindowDescriptor {
            title: "Game Of Life".to_string(),
            width: SCREEN_WIDTH,
            height: SCREEN_HEIGHT,
            ..Default::default()
        })
        .add_startup_system(setup.system())
        .add_startup_stage("initializing_grid", SystemStage::single(initialize_grid.system()))
        .add_startup_stage("game_setup", SystemStage::single(spawn_grid.system()))
        .add_system_set_to_stage(
            CoreStage::PostUpdate,
            SystemSet::new()
                .with_system(position_translation.system())
                .with_system(size_scaling.system())
        )
        .add_plugins(DefaultPlugins)
        .run();

}

enum State {
    Dead,
    Alive
}


struct Materials {
    alive_material: Handle<ColorMaterial>,
    dead_material: Handle<ColorMaterial>,
}

struct Position {
    x: i32,
    y: i32,
}

struct Size {
    width: f32,
    height: f32
}

impl Size {
    pub fn square(x: f32) -> Self {
        Self {
            width: x,
            height: x,
        }
    }
}

struct Tile;
struct SingleTile {
    state: State,
    position: Position
}


#[derive(Default)]
struct Grid(Vec<SingleTile>);

fn initialize_grid(
    mut grid: ResMut<Grid>
) {
    grid.0 = vec![];
    for i in 0..ARENA_WIDTH {
        for j in 0..ARENA_HEIGHT {
            grid.0.push(SingleTile{
                state: State::Dead,
                position: Position {x: i as i32, y: j as i32} 
            });
        }
    }
}

fn read_coords(
    abs_x: u16,
    abs_y: u16,
) -> Position {

    Position {x: 0, y: 0}
}

fn change_state(
    windows: Res<Windows>,
    buttons: Res<Input<MouseButton>>,
    mut grid: ResMut<Grid>,
) {
    let window = windows.get_primary().unwrap();

    if let Some(_position) = window.cursor_position() {

    }
}


fn spawn_grid(
    mut commands: Commands,
    materials: Res<Materials>,
    grid: Res<Grid>,
) {
    for tile in grid.0.iter() {
        let SingleTile{state: state, position: Position {x: x, y: y}} = tile;
        let m = match state {
            State::Alive => materials.alive_material.clone(),
            State::Dead => materials.dead_material.clone(),
        };
        commands
        .spawn_bundle(SpriteBundle {
            material: m,
            ..Default::default()
        })
        .insert(Tile)
        .insert(Position {x: *x, y: *y})
        .insert(Size::square(0.65))
        .id();
    }
    
    
    
}

fn size_scaling(windows: Res<Windows>, mut q: Query<(&Size, &mut Sprite)>) {
    let window = windows.get_primary().unwrap();
    for (sprite_size, mut sprite) in q.iter_mut() {
        sprite.size = Vec2::new(
            sprite_size.width / ARENA_WIDTH as f32 * window.width() as f32,
            sprite_size.height / ARENA_HEIGHT as f32 * window.height() as f32,
        );
    }
}

fn position_translation(windows: Res<Windows>, mut q: Query<(&Position, &mut Transform)>) {
    fn convert(pos: f32, bound_window: f32, bound_game: f32) -> f32 {
        let tile_size = bound_window / bound_game;
        pos / bound_game * bound_window - (bound_window / 2.) + (tile_size / 2.0)
    }
    let window = windows.get_primary().unwrap();
    for (pos, mut transform) in q.iter_mut() {
        transform.translation = Vec3::new(
            convert(pos.x as f32, window.width() as f32, ARENA_WIDTH as f32),
            convert(pos.y as f32, window.height() as f32, ARENA_HEIGHT as f32),
            0.0,
        )
    }
}