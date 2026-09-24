mod text;

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: text::WINDOW_TITLE.into(),
                resolution: (960, 600).into(),
                #[cfg(target_arch = "wasm32")]
                canvas: Some("#bredboard".into()),
                #[cfg(target_arch = "wasm32")]
                fit_canvas_to_parent: true,
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands.spawn((
        Sprite::from_color(Color::srgb(0.12, 0.18, 0.24), Vec2::new(760.0, 340.0)),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
    commands.spawn((
        Text2d::new(text::HEADING),
        TextFont::from_font_size(56.0),
        TextColor(Color::srgb(0.95, 0.97, 0.99)),
        Transform::from_xyz(0.0, 65.0, 1.0),
    ));
    commands.spawn((
        Text2d::new(text::SUBHEADING),
        TextFont::from_font_size(26.0),
        TextColor(Color::srgb(0.65, 0.83, 0.94)),
        Transform::from_xyz(0.0, -15.0, 1.0),
    ));
    commands.spawn((
        Text2d::new(format!("{} {}", text::CORE_LABEL, bredboard_core::VERSION)),
        TextFont::from_font_size(19.0),
        TextColor(Color::srgb(0.65, 0.83, 0.94)),
        Transform::from_xyz(0.0, -78.0, 1.0),
    ));
}
