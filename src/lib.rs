use bevy::prelude::*;

pub use bevy_mod_try_system::*;
pub use bevy_toasts_ui::ToastPlugin;

pub type ErrorVec = Vec<anyhow::Error>;
pub type ResultVec<T> = std::result::Result<T, ErrorVec>;
pub type Result<T> = anyhow::Result<T>;

pub trait AnyToastsExt<In, Marker, E>: TrySystemExt<In, Marker, (), E>
where
    E: std::fmt::Debug + Send + Sync + 'static,
{
    fn anyhow(self) -> impl System<In = In, Out = ()>;
}

impl<F, In, Marker> AnyToastsExt<In, Marker, Vec<anyhow::Error>> for F
where
    F: TrySystemExt<In, Marker, (), Vec<anyhow::Error>> + IntoSystem<In, ResultVec<()>, Marker>,
{
    fn anyhow(self) -> impl System<In = In, Out = ()> {
        self.map(|result: ResultVec<()>| {
            result.map_err(|errors| {
                errors
                    .into_iter()
                    .map(|error| format!("{error}"))
                    .collect::<Vec<_>>()
            })
        })
        .pipe_err(ToastPlugin::toast)
    }
}

impl<F, In, Marker> AnyToastsExt<In, Marker, anyhow::Error> for F
where
    F: TrySystemExt<In, Marker, (), anyhow::Error> + IntoSystem<In, Result<()>, Marker>,
{
    fn anyhow(self) -> impl System<In = In, Out = ()> {
        self.map(|result: Result<()>| result.map_err(|error| vec![format!("{error}")]))
            .pipe_err(ToastPlugin::toast)
    }
}

#[cfg(test)]
mod tests {
    use bevy_toasts_ui::Toast;
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
        app.add_plugins(ToastPlugin::new());
        app
    }

    #[test]
    fn test_one_error_system() {
        let mut app = app();
        app.add_systems(Update, alternate_output.anyhow());
        let mut query = app.world.query::<&Toast>();
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
        app.add_systems(Update, alternate_output_many_errors.anyhow());
        let mut query = app.world.query::<&Toast>();
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
