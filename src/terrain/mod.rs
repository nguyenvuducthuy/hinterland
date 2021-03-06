use cgmath::Point2;
use genmesh::{generators::{IndexedPolygon, Plane, SharedVertex}, Triangulate, Vertices};
use gfx;
use specs;
use specs::prelude::{Read, ReadStorage, WriteStorage};

use crate::character::controls::CharacterInputState;
use crate::game::constants::{ASPECT_RATIO, TILE_SIZE, TILES_PCS_H, TILES_PCS_W, VIEW_DISTANCE};
use crate::gfx_app::{ColorFormat, DepthFormat};
use crate::graphics::{camera::CameraInputState, can_move_to_tile, coords_to_tile, dimensions::{Dimensions, get_projection, get_view_matrix}};
use crate::graphics::mesh::TexturedMesh;
use crate::graphics::texture::{load_texture, Texture};
use crate::shaders::{Position, Projection, tilemap_pipeline, TilemapSettings, Time, VertexData};

pub mod path_finding;
pub mod tile_map;

fn cartesian_to_isometric(point_x: f32, point_y: f32) -> (f32, f32) {
  ((point_x - point_y), (point_x + point_y) / (16.0 / 9.0))
}

pub struct TerrainDrawable {
  projection: Projection,
  pub position: Position,
  pub tile_position: Point2<i32>,
}

impl TerrainDrawable {
  pub fn new() -> TerrainDrawable {
    let view = get_view_matrix(VIEW_DISTANCE);
    let projection = get_projection(view, ASPECT_RATIO);
    TerrainDrawable {
      projection,
      position: Position::origin(),
      tile_position: coords_to_tile(Position::origin()),
    }
  }

  pub fn update(&mut self, world_to_clip: &Projection, ci: &mut CharacterInputState) {
    self.projection = *world_to_clip;
    if can_move_to_tile(ci.movement) {
      ci.is_colliding = false;
      self.position = ci.movement;
      self.tile_position = coords_to_tile(self.position);
    } else {
      ci.is_colliding = true;
    }
  }
}

impl specs::prelude::Component for TerrainDrawable {
  type Storage = specs::storage::HashMapStorage<TerrainDrawable>;
}

const SHADER_VERT: &[u8] = include_bytes!("../shaders/terrain.v.glsl");
const SHADER_FRAG: &[u8] = include_bytes!("../shaders/terrain.f.glsl");

pub struct TerrainDrawSystem<R: gfx::Resources> {
  bundle: gfx::pso::bundle::Bundle<R, tilemap_pipeline::Data<R>>,
  is_tile_map_dirty: bool,
}

impl<R: gfx::Resources> TerrainDrawSystem<R> {
  pub fn new<F>(factory: &mut F,
                rtv: gfx::handle::RenderTargetView<R, ColorFormat>,
                dsv: gfx::handle::DepthStencilView<R, DepthFormat>)
                -> TerrainDrawSystem<R>
    where F: gfx::Factory<R> {
    use gfx::traits::FactoryExt;

    let plane = Plane::subdivide(TILES_PCS_W, TILES_PCS_H);
    let vertex_data: Vec<VertexData> =
      plane.shared_vertex_iter()
        .map(|vertex| {
          let tile_x = TILES_PCS_W as f32;
          let tile_y = TILES_PCS_H as f32;
          let (raw_x, raw_y) = cartesian_to_isometric(vertex.pos.x, vertex.pos.y);
          let vertex_x = (TILE_SIZE * (tile_x as f32) / 1.5) * raw_x;
          let vertex_y = (TILE_SIZE * (tile_y as f32) / 1.666) * raw_y;

          let (u_pos, v_pos) = ((raw_x / 4.0 - raw_y / 2.25) + 0.5, (raw_x / 4.0 + raw_y / 2.25) + 0.5);
          let tile_map_x = u_pos * tile_x as f32;
          let tile_map_y = v_pos * tile_y as f32;

          VertexData::new([vertex_x, vertex_y], [tile_map_x, tile_map_y])
        })
        .collect();

    let index_data =
      plane.indexed_polygon_iter()
        .triangulate()
        .vertices()
        .map(|i| i as u16)
        .collect::<Vec<u16>>();

    let tile_sheet_bytes = &include_bytes!("../../assets/maps/terrain.png")[..];
    let tile_texture = load_texture(factory, tile_sheet_bytes);

    let mesh = TexturedMesh::new(factory, &vertex_data.as_slice(), index_data.as_slice(), Texture::new(tile_texture, None));

    let pso = factory.create_pipeline_simple(SHADER_VERT, SHADER_FRAG, tilemap_pipeline::new())
      .expect("Terrain shader loading error");

    let terrain = tile_map::Terrain::new();

    let pipeline_data = tilemap_pipeline::Data {
      vbuf: mesh.vertex_buffer,
      position_cb: factory.create_constant_buffer(1),
      time_passed_cb: factory.create_constant_buffer(1),
      projection_cb: factory.create_constant_buffer(1),
      tilemap: factory.create_buffer_immutable(&terrain.tiles.as_slice(),
                                               gfx::buffer::Role::Constant,
                                               gfx::memory::Bind::empty()).unwrap(),
      tilemap_cb: factory.create_constant_buffer(1),
      tilesheet: (mesh.texture.raw, factory.create_sampler_linear()),
      out_color: rtv,
      out_depth: dsv,
    };

    TerrainDrawSystem {
      bundle: gfx::Bundle::new(mesh.slice, pso, pipeline_data),
      is_tile_map_dirty: true,
    }
  }

  pub fn draw<C>(&mut self,
                 drawable: &TerrainDrawable,
                 time_passed: u64,
                 encoder: &mut gfx::Encoder<R, C>)
    where C: gfx::CommandBuffer<R> {
    encoder.update_constant_buffer(&self.bundle.data.projection_cb, &drawable.projection);
    encoder.update_constant_buffer(&self.bundle.data.position_cb, &drawable.position);
    encoder.update_constant_buffer(&self.bundle.data.time_passed_cb, &Time::new(time_passed));

    if self.is_tile_map_dirty {
      encoder.update_constant_buffer(&self.bundle.data.tilemap_cb, &TilemapSettings {
        world_size: [TILES_PCS_W as f32, TILES_PCS_H as f32],
        tilesheet_size: [32.0, 32.0],
      });
      self.is_tile_map_dirty = false
    }

    self.bundle.encode(encoder);
  }
}

pub struct PreDrawSystem;

impl<'a> specs::prelude::System<'a> for PreDrawSystem {
  type SystemData = (WriteStorage<'a, TerrainDrawable>,
                     ReadStorage<'a, CameraInputState>,
                     WriteStorage<'a, CharacterInputState>,
                     Read<'a, Dimensions>);

  fn run(&mut self, (mut terrain, camera_input, mut character_input, dim): Self::SystemData) {
    use specs::join::Join;

    for (t, camera, ci) in (&mut terrain, &camera_input, &mut character_input).join() {
      let world_to_clip = dim.world_to_projection(camera);
      t.update(&world_to_clip, ci);
    }
  }
}
