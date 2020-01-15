use gfx::{self, *};
use ggez;
use ggez::graphics;

// Define the input struct for our shader.
gfx_defines! {
    constant GlowParams {
        glow_color: [f32; 3] = "glow_color",
        glow_intensity: f32 = "glow_intensity",
    }
}

pub type GlowShader = graphics::Shader<GlowParams>;

pub fn load_glow_shader(ctx: &mut ggez::Context) -> ggez::GameResult<GlowShader> {
    let glow_params = GlowParams {
        glow_color: [1., 1., 1.],
        glow_intensity: 1.0,
    };
    graphics::Shader::new(
        ctx,
        "/shaders/basic_150.glslv",
        "/shaders/glow.glslf",
        glow_params,
        "GlowParams",
        None,
    )
}
