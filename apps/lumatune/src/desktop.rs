use lumatune_core::{FilterParams, Ramp};
use lumatune_win::display::Display;
use lumatune_win::settings::{DesktopPreset, Settings};
use lumatune_win::{Session, state};

pub struct Screen {
    pub display: Display,
    pub desktop: Option<DesktopPreset>,
    session: Option<Session>,
    rendered: Option<Option<Ramp>>,
    reported_failure: bool,
}

impl Screen {
    fn open(display: Display, settings: &Settings) -> Self {
        let desktop = display
            .identity
            .as_ref()
            .and_then(|id| settings.desktop.get(id))
            .cloned();
        let mut screen = Self {
            display,
            desktop,
            session: None,
            rendered: Some(None),
            reported_failure: false,
        };
        screen.retry_open();
        screen
    }

    fn retry_open(&mut self) -> bool {
        if self.ready() || self.display.hdr() || self.display.identity.is_none() {
            return false;
        }
        state::heal(&self.display);
        self.session = Session::open(&self.display);
        self.ready()
    }

    pub fn ready(&self) -> bool {
        self.session.is_some()
    }

    pub fn desired_ramp(&self, foreground: Option<(usize, &FilterParams)>) -> Option<Ramp> {
        if let Some((monitor, params)) = foreground
            && monitor == self.display.handle
        {
            return Some(params.to_ramp());
        }
        let params = &self.desktop.as_ref()?.params;
        (!params.is_neutral()).then(|| params.to_ramp())
    }

    pub fn needs_retry(&self) -> bool {
        self.ready() && self.rendered.is_none()
    }

    pub fn render(&mut self, desired: Option<Ramp>) -> bool {
        let Some(session) = &mut self.session else {
            return true;
        };
        if self.rendered.as_ref() == Some(&desired) {
            return true;
        }
        let success = match &desired {
            Some(ramp) => session.apply(ramp),
            None => session.restore(),
        };
        if success {
            self.rendered = Some(desired);
            self.reported_failure = false;
        } else {
            self.rendered = None;
        }
        success
    }

    pub fn first_failure(&mut self) -> bool {
        !std::mem::replace(&mut self.reported_failure, true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn screen(handle: usize, name: &str, params: FilterParams) -> Screen {
        Screen {
            display: Display {
                device: name.into(),
                handle,
                identity: Some(name.into()),
                name: name.into(),
                adapter: String::new(),
                width: 1920,
                height: 1080,
                primary: false,
                color_space: Some(0),
                bits_per_color: Some(8),
            },
            desktop: Some(DesktopPreset {
                preset_id: name.into(),
                params,
            }),
            session: None,
            rendered: Some(None),
            reported_failure: false,
        }
    }

    #[test]
    fn moving_the_foreground_app_restores_the_other_screens_desktop_preset() {
        let main = FilterParams {
            temperature: 6000,
            ..FilterParams::default()
        };
        let secondary = FilterParams {
            temperature: 5200,
            ..FilterParams::default()
        };
        let game = FilterParams {
            gamma: 1.6,
            ..FilterParams::default()
        };
        let left = screen(7, "main", main.clone());
        let right = screen(9, "secondary", secondary.clone());

        assert!(left.desired_ramp(Some((7, &game))) == Some(game.to_ramp()));
        assert!(right.desired_ramp(Some((7, &game))) == Some(secondary.to_ramp()));
        assert!(left.desired_ramp(Some((9, &game))) == Some(main.to_ramp()));
        assert!(right.desired_ramp(Some((9, &game))) == Some(game.to_ramp()));
        assert!(right.desired_ramp(None) == Some(secondary.to_ramp()));
    }
}

fn same_output(a: &Display, b: &Display) -> bool {
    a.handle == b.handle && a.device == b.device && a.identity == b.identity && a.hdr() == b.hdr()
}

pub fn reconcile(screens: &mut Vec<Screen>, displays: Vec<Display>, settings: &Settings) -> bool {
    if screens
        .iter()
        .map(|screen| &screen.display)
        .eq(displays.iter())
    {
        let mut changed = false;
        for screen in screens {
            changed |= screen.retry_open();
        }
        return changed;
    }
    let mut previous = std::mem::take(screens);
    for screen in &mut previous {
        if !displays
            .iter()
            .any(|display| same_output(display, &screen.display))
            && let Some(session) = screen.session.take()
        {
            // A GDI name can now point to another monitor. Recover by monitor identity instead.
            session.detach();
        }
    }
    *screens = displays
        .into_iter()
        .map(|display| {
            if let Some(index) = previous
                .iter()
                .position(|screen| same_output(&screen.display, &display))
            {
                let mut screen = previous.swap_remove(index);
                screen.display = display;
                screen.rendered = None;
                screen.retry_open();
                screen
            } else {
                Screen::open(display, settings)
            }
        })
        .collect();
    true
}
