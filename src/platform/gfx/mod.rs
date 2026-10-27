//! Amiga graphics library: planar bitmaps, rastports, drawing, text, palette, views.
//!
//! Bitmaps stay planar (one byte array per bitplane, 1 bit per pixel) exactly like the original,
//! because the game code reads and writes them directly. Only the final display step
//! (display.rs) converts them to RGB.

mod area;

mod bitmap;

mod blit;

mod draw;

mod text;

mod view;

pub use self::area::{
    area_draw, area_end, area_move, create_upfront_layer, delete_layer, dispose_layer_info,
    init_area, init_tmp_ras, new_layer_info,
};
pub use self::bitmap::{alloc_raster, free_raster, init_bit_map, make_bit_map, unmake_bit_map};
pub use self::blit::{blt_bit_map, scroll_raster};
pub use self::draw::{
    draw_line, init_rast_port, move_pen, rect_fill, set_a_pen, set_b_pen, set_draw_mode, set_rast,
};
pub use self::text::{close_font, load_seg, open_font, set_font, text_out, unload_seg};
pub use self::view::{
    change_sprite, disown_blitter, free_color_map, free_cpr_list, free_sprite,
    free_vport_cop_lists, get_color_map, get_sprite, init_view, init_vport, load_rgb4, load_view,
    make_vport, merge_copper, own_blitter, set_rgb4, wait_blit, wait_bovp,
};
