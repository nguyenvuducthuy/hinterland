use cgmath::BaseFloat;
use gfx;
use std;
use std::{fmt::{Display, Formatter, Result}, ops::{Add, Sub}};

gfx_defines! {
  constant TileMapData {
    data: [f32; 4] = "data",
  }

  constant TilemapSettings {
    world_size: [f32; 2] = "u_WorldSize",
    tilesheet_size: [f32; 2] = "u_TilesheetSize",
  }

  vertex VertexData {
    pos: [f32; 2] = "a_Pos",
    uv: [f32; 2] = "a_BufPos",
  }

  constant CharacterSheet {
    x_div: f32 = "x_div",
    y_div: f32 = "y_div",
    row_idx: u32 = "a_row",
    index: f32 = "a_index",
  }

  constant Position {
    position: [f32; 2] = "a_position",
  }

  pipeline bullet_pipeline {
    vbuf: gfx::VertexBuffer<VertexData> = (),
    projection_cb: gfx::ConstantBuffer<Projection> = "b_VsLocals",
    position_cb: gfx::ConstantBuffer<Position> = "b_BulletPosition",
    out_color: gfx::RenderTarget<gfx::format::Rgba8> = "Target0",
    out_depth: gfx::DepthTarget<gfx::format::DepthStencil> = gfx::preset::depth::LESS_EQUAL_WRITE,
  }

  pipeline critter_pipeline {
    vbuf: gfx::VertexBuffer<VertexData> = (),
    projection_cb: gfx::ConstantBuffer<Projection> = "b_VsLocals",
    position_cb: gfx::ConstantBuffer<Position> = "b_CharacterPosition",
    character_sprite_cb: gfx::ConstantBuffer<CharacterSheet> = "b_CharacterSprite",
    charactersheet: gfx::TextureSampler<[f32; 4]> = "t_CharacterSheet",
    out_color: gfx::RenderTarget<gfx::format::Rgba8> = "Target0",
    out_depth: gfx::DepthTarget<gfx::format::DepthStencil> = gfx::preset::depth::LESS_EQUAL_WRITE,
  }

  pipeline tilemap_pipeline {
    vbuf: gfx::VertexBuffer<VertexData> = (),
    position_cb: gfx::ConstantBuffer<Position> = "b_TileMapPosition",
    projection_cb: gfx::ConstantBuffer<Projection> = "b_VsLocals",
    tilemap: gfx::ConstantBuffer<TileMapData> = "b_TileMap",
    tilemap_cb: gfx::ConstantBuffer<TilemapSettings> = "b_PsLocals",
    tilesheet: gfx::TextureSampler<[f32; 4]> = "t_TileSheet",
    out_color: gfx::RenderTarget<gfx::format::Rgba8> = "Target0",
    out_depth: gfx::DepthTarget<gfx::format::DepthStencil> = gfx::preset::depth::LESS_EQUAL_WRITE,
  }

  pipeline static_element_pipeline {
    vbuf: gfx::VertexBuffer<VertexData> = (),
    position_cb: gfx::ConstantBuffer<Position> = "b_StaticElementPosition",
    projection_cb: gfx::ConstantBuffer<Projection> = "b_VsLocals",
    static_element_sheet: gfx::TextureSampler<[f32; 4]> = "t_StaticElementSheet",
    out_color: gfx::RenderTarget<gfx::format::Rgba8> = "Target0",
    out_depth: gfx::DepthTarget<gfx::format::DepthStencil> = gfx::preset::depth::LESS_EQUAL_WRITE,
  }

  pipeline text_pipeline {
    vbuf: gfx::VertexBuffer<VertexData> = (),
    position_cb: gfx::ConstantBuffer<Position> = "b_TextPosition",
    text_sheet: gfx::TextureSampler<[f32; 4]> = "t_TextSheet",
    out_color: gfx::RenderTarget<gfx::format::Rgba8> = "Target0",
    out_depth: gfx::DepthTarget<gfx::format::DepthStencil> = gfx::preset::depth::LESS_EQUAL_WRITE,
  }

  constant Projection {
    model: [[f32; 4]; 4] = "u_Model",
    view: [[f32; 4]; 4] = "u_View",
    proj: [[f32; 4]; 4] = "u_Proj",
  }
}

impl VertexData {
  pub fn new(pos: [f32; 2], uv: [f32; 2]) -> VertexData {
    VertexData {
      pos,
      uv,
    }
  }
}

impl TileMapData {
  pub fn new_empty() -> TileMapData {
    TileMapData { data: [32.0, 32.0, 0.0, 0.0] }
  }

  pub fn new(data: [f32; 4]) -> TileMapData {
    TileMapData { data }
  }
}

impl Position {
  pub fn new<T: BaseFloat>(x: T, y: T) -> Position where f32: std::convert::From<T> {
    Position { position: [f32::from(x), f32::from(y)] }
  }

  pub fn origin() -> Position {
    Position { position: [0.0, 0.0] }
  }

  pub fn x(self) -> f32 {
    self.position[0]
  }

  pub fn y(self) -> f32 {
    self.position[1]
  }
}

impl Add for Position {
  type Output = Position;

  fn add(self, other: Position) -> Position {
    Position::new(self.x() + other.x(), self.y() + other.y())
  }
}

impl Sub for Position {
  type Output = Position;

  fn sub(self, other: Position) -> Position {
    Position::new(self.x() - other.x(), self.y() - other.y())
  }
}

impl Display for Position {
  fn fmt(&self, f: &mut Formatter) -> Result {
    write!(f, "{}, {}", self.x(), self.y())
  }
}
