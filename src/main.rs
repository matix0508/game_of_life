use bevy::prelude::*;
use bevy::core::FixedTimestep;


const ARENA_WIDTH: u32 = 30;
const ARENA_HEIGHT: u32 = 30;

const SCREEN_WIDTH: f32 = 700.0;
const SCREEN_HEIGHT: f32 = 700.0;

/// Main function and app builder
/// Here the windows is defined and system are called
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
         .add_system(space_hit.system()) 
         .add_system_set(
             SystemSet::new()
                 .with_run_criteria(FixedTimestep::step(1.0))
                 .with_system(next_turn.system())
         )
         .add_system(change_state.system())
         
 
         .add_plugins(DefaultPlugins)
         .run();
 
 }


/// Setup function
/// Here all the resources are initialized
fn setup(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.insert_resource(Materials {
        alive_material: materials.add(Color::rgb(1., 1., 1.).into()),
        dead_material: materials.add(Color::rgb(0.1, 0.1, 0.1).into())
    });
    commands.insert_resource(Grid::default());
    commands.insert_resource(Indexes::default());
    commands.insert_resource(Run::default());

}





/// Schrodinger Enum
enum State {
    Dead,
    Alive
}



/// Color Keeper
struct Materials {
    alive_material: Handle<ColorMaterial>,
    dead_material: Handle<ColorMaterial>,
}

/// Position struct
struct Position {
    x: i32,
    y: i32,
}

/// Size struct xd
struct Size {
    width: f32,
    height: f32
}

/// Static method to generate square
impl Size {
    pub fn square(x: f32) -> Self {
        Self {
            width: x,
            height: x,
        }
    }
}


/// Sprite Struct used to spawn cells
struct Tile;

/// Struct used to hold information of an individual cell
struct SingleTile {
    state: State,
    position: Position
}
/// Method to change state from alive to dead and vice versa
impl SingleTile {
    pub fn change_state(&mut self) {
        self.state = match self.state {
            State::Alive => State::Dead,
            State::Dead => State::Alive,
        }
    }
}



/// Struct that holds information of pausing the game
#[derive(Default)]
struct Run(bool);


/// Struct that holds info with information and state of each cell
#[derive(Default)]
struct Grid(Vec<SingleTile>);


/// Struct that holds info with Entity ID's of each cell
#[derive(Default)]
struct Indexes(Vec<Entity>);


// GRID

/// Initialize grid with all the dead cells
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

/// Gets ArrayIndex based on Position Struct
fn get_index(pos: Position) -> usize {
    
    (pos.x as u32 * ARENA_WIDTH + pos.y as u32) as usize
}


/// Converts real mouse input into game coordinates
fn read_coords(
    abs_x: f32,
    abs_y: f32,
) -> Position {
    let x = abs_x * ARENA_WIDTH as f32 / SCREEN_WIDTH;
    let y = abs_y * ARENA_HEIGHT as f32 / SCREEN_HEIGHT;

    Position {x: x as i32, y: y as i32}
}


/// Originally spawning the grid
fn spawn_grid(
    mut commands: Commands,
    materials: Res<Materials>,
    grid: ResMut<Grid>,
    mut idx: ResMut<Indexes>,
) {
    for tile in grid.0.iter() {
        let SingleTile{state, position: Position {x, y}} = tile;
        let m = match state {
            State::Alive => materials.alive_material.clone(),
            State::Dead => materials.dead_material.clone(),
        };
        let id = commands
        .spawn_bundle(SpriteBundle {
            material: m,
            ..Default::default()
        })
        .insert(Tile)
        .insert(Position {x: *x, y: *y})
        .insert(Size::square(0.65))
        .id();
        idx.0.push(id);
    }
    
    
}

/// Kills grid and respawns it
fn respawn(
    mut commands: Commands,
    mut indexes: ResMut<Indexes>,
    materials: Res<Materials>,
    grid: ResMut<Grid>,

) {
    for idx in indexes.0.iter() {
        commands.entity(*idx).despawn()
    }
    indexes.0 = vec![];

    spawn_grid(commands, materials, grid, indexes)


}

/// Changes size of sprites
fn size_scaling(windows: Res<Windows>, mut q: Query<(&Size, &mut Sprite)>) {
    let window = windows.get_primary().unwrap();
    for (sprite_size, mut sprite) in q.iter_mut() {
        sprite.size = Vec2::new(
            sprite_size.width / ARENA_WIDTH as f32 * window.width() as f32,
            sprite_size.height / ARENA_HEIGHT as f32 * window.height() as f32,
        );
    }
}

/// Converts absolute coordinates into game coordinates (row, col)
fn convert(pos: f32, bound_window: f32, bound_game: f32) -> f32 {
    let tile_size = bound_window / bound_game;
    pos / bound_game * bound_window - (bound_window / 2.) + (tile_size / 2.0)
}

/// Translats positions into 3D Vector
fn position_translation(windows: Res<Windows>, mut q: Query<(&Position, &mut Transform)>) {
    
    let window = windows.get_primary().unwrap();
    for (pos, mut transform) in q.iter_mut() {
        transform.translation = Vec3::new(
            convert(pos.x as f32, window.width() as f32, ARENA_WIDTH as f32),
            convert(pos.y as f32, window.height() as f32, ARENA_HEIGHT as f32),
            0.0,
        )
    }
}


/// Function that changes state with a mouse click
/// We're basically playing Gods now.
fn change_state(
    commands: Commands,
    windows: Res<Windows>,
    buttons: Res<Input<MouseButton>>,
    mut grid: ResMut<Grid>,
    index: ResMut<Indexes>,
    materials: Res<Materials>,
) {
    let window = windows.get_primary().unwrap();

    if let Some(abs_position) = window.cursor_position() {
        let pos = read_coords(abs_position.x, abs_position.y);
        if buttons.just_pressed(MouseButton::Left) {
            let idx = get_index(pos);
            grid.0[idx].change_state();
            respawn(commands, index, materials, grid)
            
        }

    }
}



// LOGIC


/// Function that is called every one second that calculates all the future states and implements them
/// (If game is not paused)
fn next_turn(
    run: Res<Run>,
    mut grid: ResMut<Grid>,
    commands: Commands,
    indexes: ResMut<Indexes>,
    materials: Res<Materials>,


) {
    if !run.0 {
        return
    }
    let neighbours = living_neighbours(&grid);
    for tile in grid.0.iter_mut() {
        let Position {x, y} = tile.position;
        match tile.state {
            State::Alive => {
                tile.state = match neighbours[x as usize][y as usize] {
                    0 | 1 => State::Dead,
                    2 | 3 => State::Alive,
                    3.. => State::Dead,

            }
        },
            State::Dead => {
                if neighbours[x as usize][y as usize] == 3 {
                    tile.state = State::Alive;
                }
            }
        }
    }
    respawn(commands, indexes, materials, grid);
}


/// Returns 2D Array with info of all the living neighbour cells
fn living_neighbours(
    grid: &ResMut<Grid>
) ->  [[u8 ; ARENA_WIDTH as usize] ; ARENA_HEIGHT as usize]{
    let mut output = [[0 ; ARENA_WIDTH as usize] ; ARENA_HEIGHT as usize];
    let mut table = [[0 as u8; ARENA_HEIGHT as usize] ; ARENA_WIDTH as usize];
    for tile in grid.0.iter() {
        let SingleTile{state: s, position: Position {x, y}} = tile;
        table[*x as usize][*y as usize] = match s {
            State::Alive => 1,
            State::Dead => 0,
        };
    }
    for i in 0..ARENA_WIDTH {
        for j in 0..ARENA_HEIGHT {
            if i != 0 && j != 0 && i != ARENA_WIDTH - 1 && j != ARENA_HEIGHT - 1 {
                let x = i as usize;
                let y = j as usize;
                output[x][y] = table[x][y+1] + table[x][y-1] + table[x+1][y] + table[x-1][y] + table[x+1][y+1]+table[x+1][y-1] + table[x-1][y-1]+table[x-1][y+1];
            }
        }
    }
    return output
}


/// Pauses and Repauses Game
/// run is a parameter witch indicates the current state 
fn space_hit(
    mut run: ResMut<Run>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    if keyboard_input.pressed(KeyCode::Space) {
        run.0 = !run.0;
    }
    
}