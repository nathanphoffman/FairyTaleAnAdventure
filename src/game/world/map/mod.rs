//! The map routines that were 68000 assembly in fsubs.asm (from csrc/asm_map.c). Behaviour,
//! including 16 bit wrap-around, follows the assembly. See docs/data-formats.md "Map data layout".
//!
//! World coordinates are 15 bit. A tile ("image") is 16 pixels wide and 32 tall; a sector is
//! 16 x 8 tiles; the region map is 128 x 32 sectors, one byte each.
//!
//! Looking up the ground under a pixel goes: pixel -> tile column/row -> which sector (from the
//! region map `map_mem`) -> which tile (from that sector in `sector_mem`) -> the terrain table
//! `terrain_mem` for that tile. The odd wrap-around tricks (`wrap_secx`, `clamp_secy`) copy what the
//! assembly did when the position is outside the loaded region.
//!
//! Drawing: the screen shows 19 x 6 tiles. `fill_minimap` fills `minimap` with the tile numbers for the
//! visible area, and `draw_map`, `draw_map_column`, `draw_map_row` copy the tile graphics to the screen.
//! `scroll_map` shifts the picture by one tile so only a new edge has to be drawn.

mod camera;

mod drawing;

mod lookup;

mod overview;

mod steps;

pub use self::camera::{move_camera_to, page_turn_inset};
pub use self::drawing::{draw_map, draw_map_column, draw_map_row, scroll_map};
pub use self::lookup::{fill_minimap, probe_collision, terrain_at, tile_index};
pub use self::overview::draw_overview_map;
pub use self::steps::{step_x, step_y, wrap_coordinate};
