#![cfg_attr(docsrs, feature(doc_auto_cfg))]

//! `bevy_anyhow_alert` provides an extension trait enabling Bevy systems that return `Result`
//! types to opt-in to a simple `Alert`-based error UI.
//!
//! The main benefit: your systems can return `Result<T, E>` (or even `Result<T, Vec<E>>`)
//! with one chain call: `system.anyhow_alert()` (or its counterpart, `system.anyhow_alerts`).
//!
//! ## Examples
//!
//! This example shows a system that returns some `Result`:
//!
//! ```
//! use bevy::prelude::*;
//! use bevy_anyhow_alert::{AlertsPlugin, AnyhowAlertExt, anyhow::Result};
//!
//! fn main() {
//!     let mut app = App::new();
//!     app.add_plugins(MinimalPlugins);
//!     app.add_plugins(AlertsPlugin::new());
//!     app.add_systems(Update, fallible_system.anyhow_alert());
//!     // app.run();
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
//! use bevy_anyhow_alert::{AnyhowAlertExt, ResultVec};
//! use bevy_anyhow_alert::anyhow::{Error, Result};
//!
//! #[derive(Component)]
//! struct MyComponent;
//!
//! fn fallible_system(my_query: Query<&MyComponent>) -> ResultVec<(), Error> {
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
//!
//! Furthermore, this does not allow for any actual error maangement beyond displaying them.
//! For errors that should be handled in more meaningful ways, consider using `system.pipe`
//! directly or using `.pipe_err` from the `bevy_try_mod_system` crate.

use bevy_ecs::prelude::*;

pub use anyhow;
pub use bevy_ui_mod_alerts::AlertsPlugin;

pub type ResultVec<T, E> = std::result::Result<T, Vec<E>>;

/// Defines the `anyhow_alert` method which pipes system output to an Alert UI if the output
/// is an error.
///
/// This trait is implemented for all `IntoSystem` that return `Result<(), Err>`.
pub trait AnyhowAlertExt<In, Err, Marker>
where
    Err: std::fmt::Display + Send + Sync + 'static,
{
    /// Pipes system output to an alert UI if the Result is Err.
    fn anyhow_alert(self) -> impl System<In = In, Out = ()>;
}

impl<F, In, Err, Marker> AnyhowAlertExt<In, Err, Marker> for F
where
    F: IntoSystem<In, Result<(), Err>, Marker>,
    Err: std::fmt::Debug + std::fmt::Display + Send + Sync + 'static,
    Marker: Send + Sync + 'static,
{
    fn anyhow_alert(self) -> impl System<In = In, Out = ()> {
        self.pipe(anyhow_alert_system)
    }
}

/// The inner PipeableSystem used by [`AnyhowAlertExt`].
///
/// Use this by piping a system that outputs a `Result<(), Err>` into this system.
///
/// ```
/// use bevy::prelude::*;
/// use bevy_anyhow_alert::*;
/// fn my_system() -> anyhow::Result<()> { /* ... */ Ok(()) }
/// // ...
/// my_system.pipe(anyhow_alert_system);
/// ```
pub fn anyhow_alert_system<Err>(In(input): In<Result<(), Err>>, commands: Commands)
where
    Err: std::fmt::Display,
{
    if let Err(error) = input {
        AlertsPlugin::alert(In(vec![format!("{error}")]), commands)
    }
}

/// Defines the `anyhow_alert` method which pipes system output to an Alert UI if the output
/// `Vec<MyError>` is non-empty.
///
/// This trait is implemented for all `IntoSystem` that return `Result<(), Vec<Err>>`.
pub trait AnyhowAlertsExt<In, Err, Marker>
where
    Err: std::fmt::Debug + Send + Sync + 'static,
{
    /// Pipes system output to an alert UI if the Result is Err.
    fn anyhow_alerts(self) -> impl System<In = In, Out = ()>;
}

impl<F, In, Err, Marker> AnyhowAlertsExt<In, Vec<Err>, Marker> for F
where
    F: IntoSystem<In, Result<(), Vec<Err>>, Marker>,
    Err: std::error::Error + Send + Sync + 'static,
    Marker: Send + Sync + 'static,
{
    fn anyhow_alerts(self) -> impl System<In = In, Out = ()> {
        self.pipe(anyhow_alerts_system)
    }
}

/// The inner PipeableSystem used by [`AnyhowAlertsExt`].
///
/// Use this by piping a system that outputs a `Result<(), Vec<Err>>` into this system.
///
/// ```
/// use bevy::prelude::*;
/// use bevy_anyhow_alert::*;
/// fn my_system() -> Result<(), Vec<anyhow::Error>> { /* ... */ Ok(()) }
/// // ...
/// my_system.pipe(anyhow_alerts_system);
/// ```
pub fn anyhow_alerts_system<Err>(In(input): In<Result<(), Vec<Err>>>, commands: Commands)
where
    Err: std::fmt::Display,
{
    if let Err(errors) = input {
        let messages = errors
            .into_iter()
            .map(|error| format!("{error}"))
            .collect::<Vec<_>>();
        AlertsPlugin::alert(In(messages), commands)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use bevy_ui_mod_alerts::Alert;
    use thiserror::Error;

    #[derive(Debug, Error)]
    #[error("testing!")]
    struct TestError;

    fn alternate_output(mut counter: Local<usize>) -> Result<(), TestError> {
        *counter += 1;
        if *counter % 2 == 1 {
            Ok(())
        } else {
            Err(TestError)
        }
    }

    fn alternate_output_many_errors(mut counter: Local<usize>) -> ResultVec<(), TestError> {
        *counter += 1;
        if *counter % 2 == 1 {
            Ok(())
        } else {
            Err(vec![TestError])
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
        app.add_systems(Update, alternate_output_many_errors.anyhow_alerts());
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
