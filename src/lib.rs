use xbmediathek_provider_macro::define_providers;

define_providers!();

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ProviderPicker {
    selected: usize,
}

impl ProviderPicker {
    pub fn selected(&self) -> Provider {
        Provider::ALL[self.selected]
    }

    pub fn move_left(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    pub fn move_right(&mut self) {
        self.selected = (self.selected + 1).min(Provider::ALL.len() - 1);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum View {
    Providers,
    Web(Provider),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BackAction {
    GoBack,
    ShowProviders,
    None,
}

pub fn back_action(view: View, can_go_back: bool) -> BackAction {
    match (view, can_go_back) {
        (View::Providers, _) => BackAction::None,
        (View::Web(_), true) => BackAction::GoBack,
        (View::Web(_), false) => BackAction::ShowProviders,
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub left: f64,
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

pub fn next_focus(current: Rect, candidates: &[Rect], direction: Direction) -> Option<usize> {
    let center = |rect: Rect| {
        (
            (rect.left + rect.right) / 2.0,
            (rect.top + rect.bottom) / 2.0,
        )
    };
    let (current_x, current_y) = center(current);

    candidates
        .iter()
        .enumerate()
        .filter_map(|(index, candidate)| {
            let (candidate_x, candidate_y) = center(*candidate);
            let (forward, sideways) = match direction {
                Direction::Up => (current_y - candidate_y, (candidate_x - current_x).abs()),
                Direction::Down => (candidate_y - current_y, (candidate_x - current_x).abs()),
                Direction::Left => (current_x - candidate_x, (candidate_y - current_y).abs()),
                Direction::Right => (candidate_x - current_x, (candidate_y - current_y).abs()),
            };
            (forward > 0.0 && forward.is_finite() && sideways.is_finite())
                .then_some((index, forward + sideways * 2.0))
        })
        .min_by(|left, right| left.1.total_cmp(&right.1))
        .map(|(index, _)| index)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn picker_moves_between_providers_without_wrapping() {
        let mut picker = ProviderPicker::default();
        picker.move_left();
        assert_eq!(picker.selected(), Provider::ALL[0]);
        for provider in Provider::ALL.iter().skip(1) {
            picker.move_right();
            assert_eq!(picker.selected(), *provider);
        }
        picker.move_right();
        assert_eq!(picker.selected(), *Provider::ALL.last().unwrap());
    }

    #[test]
    fn providers_match_manifest_in_order() {
        let manifest: toml::Value = include_str!("../Cargo.toml").parse().unwrap();
        let providers = manifest["package"]["metadata"]["providers"]
            .as_table()
            .unwrap();
        assert_eq!(Provider::ALL.len(), providers.len());
        for (provider, (_, metadata)) in Provider::ALL.iter().zip(providers) {
            assert_eq!(Some(provider.name()), metadata["name"].as_str());
            assert_eq!(Some(provider.url()), metadata["url"].as_str());
            assert_eq!(Some(provider.logo()), metadata["logo"].as_str());
        }
    }

    #[test]
    fn providers_have_https_homepages() {
        for provider in Provider::ALL {
            assert!(provider.url().starts_with("https://"));
        }
    }

    #[test]
    fn back_prefers_history_then_provider_selection() {
        assert_eq!(
            back_action(View::Web(Provider::ALL[0]), true),
            BackAction::GoBack
        );
        assert_eq!(
            back_action(View::Web(Provider::ALL[0]), false),
            BackAction::ShowProviders
        );
        assert_eq!(back_action(View::Providers, true), BackAction::None);
    }

    fn rect(x: f64, y: f64) -> Rect {
        Rect {
            left: x,
            top: y,
            right: x + 20.0,
            bottom: y + 20.0,
        }
    }

    #[test]
    fn chooses_nearby_aligned_candidate() {
        let candidates = [rect(30.0, 60.0), rect(50.0, 0.0), rect(-30.0, 0.0)];
        assert_eq!(
            next_focus(rect(0.0, 0.0), &candidates, Direction::Right),
            Some(1)
        );
    }

    #[test]
    fn rejects_candidates_behind_or_at_current_position() {
        let candidates = [rect(0.0, 0.0), rect(0.0, 20.0)];
        assert_eq!(next_focus(rect(0.0, 0.0), &candidates, Direction::Up), None);
        assert_eq!(
            next_focus(rect(0.0, 0.0), &candidates, Direction::Down),
            Some(1)
        );
    }

    #[test]
    fn ignores_non_finite_coordinates() {
        let candidates = [rect(f64::NAN, 0.0), rect(20.0, 0.0)];
        assert_eq!(
            next_focus(rect(0.0, 0.0), &candidates, Direction::Right),
            Some(1)
        );
    }
}
