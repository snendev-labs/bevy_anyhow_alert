#![cfg_attr(docsrs, feature(doc_auto_cfg))]

//! `bevy_anyhow_alert` provides an extension trait enabling Bevy systems that return `Result`
//! types to opt-in to a simple `Alert`-based error UI.
//!
//! The main benefit: your systems can return `anyhow::Result` (or even
//! `Result<T, Vec<anyhow::Error>>`) with one (chainable) method: `system.anyhow_alert()`
//!
//! ## Examples
//!
//! This example shows a system that returns some `Result`:
//!
//! ```
//! use bevy::prelude::*;
//! use bevy_anyhow_alert::{AlertsPlugin, AnyhowAlertExt, Result};
//!
//! fn main() {
//!     let mut app = App::new();
//!     app.add_plugins(MinimalPlugins);
//!     app.add_plugins(AlertsPlugin::new());
//!     app.add_systems(Update, fallible_system.anyhow_alert());
//! }
//!
//! #[derive(Component)]
//! struct MyComponent;
//!
//! fn fallible_system(my_query: Query<&MyComponent>) -> Result<()> {
//!     for my_value in my_query.iter() {
//!         // we can use the `?` operator!
//!         get_result()?;
//!     }
//!     Ok(())
//! }
//!
//! fn get_result() -> Result<()> {
//!     Ok(())
//! }
//! ```
//!
//! Alternatively, the system can collect errors without interrupting the iteration and return
//! a vector of `Result`s:
//!
//! ```
//! use bevy::prelude::*;
//! use bevy_anyhow_alert::{AnyhowAlertExt, Result, ResultVec};
//!
//! #[derive(Component)]
//! struct MyComponent;
//!
//! fn fallible_system(my_query: Query<&MyComponent>) -> ResultVec<()> {
//!     let mut errors = vec![];
//!     for my_value in my_query.iter() {
//!         if let Err(error) = get_result() {
//!            errors.push(error);
//!         }
//!     }
//!     if errors.is_empty() {
//!         Ok(())
//!     } else {
//!         Err(errors)
//!     }
//! }
//!
//! fn get_result() -> Result<()> {
//!     Ok(())
//! }
//! ```
//!
//! The resulting UI is somewhat restylable but may not fit every application.

use bevy::prelude::*;

pub use bevy_mod_try_system::*;
pub use bevy_ui_mod_alerts::AlertsPlugin;

pub type ErrorVec = Vec<anyhow::Error>;
pub type ResultVec<T> = std::result::Result<T, ErrorVec>;
pub type Result<T> = anyhow::Result<T>;

/// Defines the `anyhow_alert` method which pipes system output to an Alert UI if the output
/// is an error.
///
/// This trait is implemented for all `IntoSystem` that return `anyhow::Result` or
/// `Result<(), Vec<anyhow::Error>>`.
pub trait AnyhowAlertExt<In, Err, Marker>: TrySystemExt<In, (), Err, Marker>
where
    Err: std::fmt::Debug + Send + Sync + 'static,
{
    /// Pipes system output to an alert UI if the Result is Err. The `Ok` variant can be piped
    /// into subsequent systems.
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
    #[error("testing!")]
    struct TestError;

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
