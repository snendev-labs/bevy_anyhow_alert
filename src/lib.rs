use bevy::prelude::*;

pub use bevy_mod_try_system::*;
pub use bevy_ui_mod_alerts::AlertsPlugin;

pub type ErrorVec = Vec<anyhow::Error>;
pub type ResultVec<T> = std::result::Result<T, ErrorVec>;
pub type Result<T> = anyhow::Result<T>;

pub trait AnyhowAlertExt<In, Err, Marker>: TrySystemExt<In, (), Err, Marker>
where
    Err: std::fmt::Debug + Send + Sync + 'static,
{
    fn anyhow_alert(self) -> impl System<In = In, Out = ()>;
}

impl<F, In, Marker> AnyhowAlertExt<In, Vec<anyhow::Error>, Marker> for F
where
    F: TrySystemExt<In, (), Vec<anyhow::Error>, Marker> + IntoSystem<In, ResultVec<()>, Marker>,
    Marker: Send + Sync + 'static,
{
    fn anyhow_alert(self) -> impl System<In = In, Out = ()> {
        self.map(|result: ResultVec<()>| {
            result.map_err(|errors| {
                errors
                    .into_iter()
                    .map(|error| format!("{error}"))
                    .collect::<Vec<_>>()
            })
        })
        .pipe_err(AlertsPlugin::alert)
    }
}

impl<F, In, Marker> AnyhowAlertExt<In, anyhow::Error, Marker> for F
where
    F: TrySystemExt<In, (), anyhow::Error, Marker> + IntoSystem<In, Result<()>, Marker>,
    Marker: Send + Sync + 'static,
{
    fn anyhow_alert(self) -> impl System<In = In, Out = ()> {
        self.map(|result: Result<()>| result.map_err(|error| vec![format!("{error}")]))
            .pipe_err(AlertsPlugin::alert)
    }
}

#[cfg(test)]
mod tests {
    use bevy_ui_mod_alerts::Alert;
    use thiserror::Error;

    use super::*;

    #[derive(Debug, Error)]
    struct TestError;

    impl std::fmt::Display for TestError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Test error reached!")
        }
    }

    fn alternate_output(mut counter: Local<usize>) -> Result<()> {
        *counter += 1;
        if *counter % 2 == 1 {
            Ok(())
        } else {
            Err(anyhow::Error::new(TestError))
        }
    }

    fn alternate_output_many_errors(mut counter: Local<usize>) -> ResultVec<()> {
        *counter += 1;
        if *counter % 2 == 1 {
            Ok(())
        } else {
            Err(vec![anyhow::Error::new(TestError)])
        }
    }

    fn app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(AlertsPlugin::new());
        app
    }

    #[test]
    fn test_one_error_system() {
        let mut app = app();
        app.add_systems(Update, alternate_output.anyhow_alert());
        let mut query = app.world.query::<&Alert>();
        assert_eq!(query.iter(&app.world).count(), 0);
        app.update();
        assert_eq!(query.iter(&app.world).count(), 0);
        app.update();
        assert_eq!(query.iter(&app.world).count(), 1);
        app.update();
        assert_eq!(query.iter(&app.world).count(), 1);
        app.update();
        assert_eq!(query.iter(&app.world).count(), 2);
    }

    #[test]
    fn test_error_collecting_system() {
        let mut app = app();
        app.add_systems(Update, alternate_output_many_errors.anyhow_alert());
        let mut query = app.world.query::<&Alert>();
        assert_eq!(query.iter(&app.world).count(), 0);
        app.update();
        assert_eq!(query.iter(&app.world).count(), 0);
        app.update();
        assert_eq!(query.iter(&app.world).count(), 1);
        app.update();
        assert_eq!(query.iter(&app.world).count(), 1);
        app.update();
        assert_eq!(query.iter(&app.world).count(), 2);
    }
}
