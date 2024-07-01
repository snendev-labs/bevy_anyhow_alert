# bevy_anyhow_alert

Simple application-level error management. Mildly configurable; this is in early stages. The main benefit: your systems can return `anyhow::Result` (or even `Result<T, Vec<anyhow::Error>>`)!

## How To Use

When writing your systems, return one of the two accepted types:

- `bevy_anyhow_alert::Result<T>`: a re-export of `anyhow::Result<T>`
- `bevy_anyhow_alert::ResultVec<T>`: an alias for `Result<T, Vec<anyhow::Error>>`

Then call `my_system.anyhow_alert()`! When it errors, you'll see toasts fire (don't forget a camera).

```rust
let mut app = App::new();
// ...
app.add_system(fire_error.anyhow_alert());
// ..
app.run();
```

Feel free to define whatever types of errors your want throughout your application. When returning errors, the error is wrapped in an `anyhow::Error` with `anyhow::Error::new(MyError)`. This is especially easy if you derive `thiserror::Error` on your Error type.

```rust
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
`
```
