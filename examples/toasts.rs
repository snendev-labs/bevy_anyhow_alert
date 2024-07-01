use thiserror::Error;

use bevy::prelude::*;

use bevy_anyhow_alert::{AlertsPlugin, AnyhowAlertExt};

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_plugins(AlertsPlugin::new());

    app.add_systems(Startup, init);
    app.add_systems(Update, fire_error.anyhow_alert().in_set(MySystems));

    app.run();
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, SystemSet)]
pub struct MySystems;

fn init(mut commands: Commands) {
    commands.spawn((Camera2dBundle::default(), IsDefaultUiCamera));
    commands
        .spawn((
            Name::new("Banner"),
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..Default::default()
                },
                background_color: Color::ANTIQUE_WHITE.into(),
                ..Default::default()
            },
        ))
        .with_children(|builder| {
            builder.spawn(TextBundle::from_section(
                "Press Space to fire error toast",
                TextStyle {
                    font_size: 48.,
                    color: Color::BLACK,
                    ..Default::default()
                },
            ));
        });
}

#[derive(Debug, Error)]
pub struct MyError;

impl std::fmt::Display for MyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "My Error")
    }
}

fn fire_error(inputs: Res<ButtonInput<KeyCode>>) -> anyhow::Result<()> {
    if inputs.just_pressed(KeyCode::Space) {
        Err(anyhow::Error::new(MyError))
    } else {
        Ok(())
    }
}
