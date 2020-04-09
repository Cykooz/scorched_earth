use gfx::{self, *};
use ggez;
use ggez::graphics;

gfx_defines! {
    constant GlowParams {
        glow_color: [f32; 3] = "glow_color",
        glow_intensity: f32 = "glow_intensity",
    }
}

pub type GlowShader = graphics::Shader<GlowParams>;

pub fn load_glow_shader(ctx: &mut ggez::Context) -> ggez::GameResult<GlowShader> {
    let params = GlowParams {
        glow_color: [1., 1., 1.],
        glow_intensity: 1.0,
    };
    graphics::Shader::new(
        ctx,
        "/shaders/basic_150.glslv",
        "/shaders/glow.glslf",
        params,
        "GlowParams",
        None,
    )
}

gfx_defines! {
    constant HueParams {
        hue_offset: f32 = "hue_offset",
    }
}

pub type HueShader = graphics::Shader<HueParams>;

pub fn load_hue_shader(ctx: &mut ggez::Context) -> ggez::GameResult<HueShader> {
    let params = HueParams { hue_offset: 0. };
    graphics::Shader::new(
        ctx,
        "/shaders/basic_150.glslv",
        "/shaders/hue.glslf",
        params,
        "HueParams",
        None,
    )
}

#[derive(Debug, Clone, Copy)]
pub struct HueOffset(f32);

impl HueOffset {
    #[inline]
    pub fn new<T: Into<f32>>(offset: T) -> Self {
        let mut offset = offset.into() % 360.;
        if offset < 0. {
            // Offset is always positive
            offset += 360.;
        }
        Self(offset)
    }
}

impl Into<HueParams> for HueOffset {
    #[inline]
    fn into(self) -> HueParams {
        HueParams {
            hue_offset: self.0 / 360.,
        }
    }
}
