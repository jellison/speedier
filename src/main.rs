mod app;
mod calc;
mod prefs;
mod syntax;
mod theme;

use app::SpeedierApp;
use gpui::{
    size, App, AppContext, Application, Bounds, KeyBinding, TitlebarOptions, WindowBounds,
    WindowOptions,
};
use gpui_component::input::InputState;
use gpui_component::Root;
use gpui_component_assets::Assets;
use prefs::Preferences;

gpui::actions!(speedier, [Quit]);

fn main() {
    Application::new().with_assets(Assets).run(|cx: &mut App| {
        gpui_component::init(cx);

        let prefs = Preferences::load().unwrap_or_default();
        let window_size = prefs.window_size().unwrap_or((640.0, 720.0));

        let bounds = Bounds::centered(
            None,
            size(gpui::px(window_size.0), gpui::px(window_size.1)),
            cx,
        );

        let options = WindowOptions {
            titlebar: Some(TitlebarOptions {
                title: Some("Speedier".into()),
                ..Default::default()
            }),
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            ..Default::default()
        };

        let _ = cx.open_window(options, move |window, cx| {
            let input = cx.new(|cx| InputState::new(window, cx).placeholder("Type an expression"));
            let view = cx.new(|cx| SpeedierApp::new(input, prefs.clone(), window, cx));
            cx.new(|cx| Root::new(view, window, cx))
        });

        cx.activate(true);
        cx.on_action(|_: &Quit, cx| cx.quit());
        cx.bind_keys([KeyBinding::new("cmd-q", Quit, None)]);
        let _ = cx.on_window_closed(|cx| {
            if cx.windows().is_empty() {
                cx.quit();
            }
        });
    });
}
