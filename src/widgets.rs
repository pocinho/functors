#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct WidgetId(pub u64);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bounds {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rgba(pub u8, pub u8, pub u8, pub u8);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TypographyTokens {
    pub body_size: f32,
    pub menu_size: f32,
    pub emphasis_size: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpacingTokens {
    pub small: f32,
    pub medium: f32,
    pub large: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ColorTokens {
    pub background: Rgba,
    pub surface: Rgba,
    pub text: Rgba,
    pub accent: Rgba,
    pub error: Rgba,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BorderTokens {
    pub width: f32,
    pub radius: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Theme {
    pub typography: TypographyTokens,
    pub spacing: SpacingTokens,
    pub colors: ColorTokens,
    pub borders: BorderTokens,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BorderPaint {
    pub color: Rgba,
    pub width: f32,
    pub radius: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PaintDescription {
    Panel {
        bounds: Bounds,
        fill: Rgba,
        border: Option<BorderPaint>,
    },
    Menu {
        bounds: Bounds,
        fill: Rgba,
        text: Rgba,
    },
    CommandBar {
        bounds: Bounds,
        fill: Rgba,
        text: Rgba,
    },
    Scrollbar {
        track: Bounds,
        thumb: Bounds,
        track_color: Rgba,
        thumb_color: Rgba,
    },
    StatusSurface {
        bounds: Bounds,
        fill: Rgba,
        text: Rgba,
    },
}

impl Bounds {
    pub fn contains(self, x: f32, y: f32) -> bool {
        x.is_finite()
            && y.is_finite()
            && x >= self.x
            && y >= self.y
            && x <= self.x + self.width
            && y <= self.y + self.height
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            typography: TypographyTokens {
                body_size: 16.0,
                menu_size: 14.0,
                emphasis_size: 18.0,
            },
            spacing: SpacingTokens {
                small: 4.0,
                medium: 8.0,
                large: 16.0,
            },
            colors: ColorTokens {
                background: Rgba(30, 30, 30, 255),
                surface: Rgba(45, 45, 45, 255),
                text: Rgba(220, 220, 220, 255),
                accent: Rgba(97, 175, 239, 255),
                error: Rgba(220, 80, 80, 255),
            },
            borders: BorderTokens {
                width: 1.0,
                radius: 2.0,
            },
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WidgetState(u8);

impl WidgetState {
    const HOVERED: u8 = 1 << 0;
    const PRESSED: u8 = 1 << 1;
    const FOCUSED: u8 = 1 << 2;
    const DISABLED: u8 = 1 << 3;
    const SELECTED: u8 = 1 << 4;
    const ERROR: u8 = 1 << 5;

    pub const fn empty() -> Self {
        Self(0)
    }

    pub const fn hovered(self) -> bool {
        self.0 & Self::HOVERED != 0
    }

    pub const fn pressed(self) -> bool {
        self.0 & Self::PRESSED != 0
    }

    pub const fn focused(self) -> bool {
        self.0 & Self::FOCUSED != 0
    }

    pub const fn disabled(self) -> bool {
        self.0 & Self::DISABLED != 0
    }

    pub const fn selected(self) -> bool {
        self.0 & Self::SELECTED != 0
    }

    pub const fn error(self) -> bool {
        self.0 & Self::ERROR != 0
    }

    pub const fn with_hovered(self, value: bool) -> Self {
        self.with_flag(Self::HOVERED, value)
    }

    pub const fn with_pressed(self, value: bool) -> Self {
        self.with_flag(Self::PRESSED, value)
    }

    pub const fn with_focused(self, value: bool) -> Self {
        self.with_flag(Self::FOCUSED, value)
    }

    pub const fn with_disabled(self, value: bool) -> Self {
        self.with_flag(Self::DISABLED, value)
    }

    pub const fn with_selected(self, value: bool) -> Self {
        self.with_flag(Self::SELECTED, value)
    }

    pub const fn with_error(self, value: bool) -> Self {
        self.with_flag(Self::ERROR, value)
    }

    const fn with_flag(self, flag: u8, value: bool) -> Self {
        if value {
            Self(self.0 | flag)
        } else {
            Self(self.0 & !flag)
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct WidgetDescription {
    pub id: WidgetId,
    pub bounds: Bounds,
    pub state: WidgetState,
    pub z_index: i32,
    pub focus_order: Option<u32>,
    pub accessibility: AccessibilityInfo,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AccessibilityRole {
    Generic,
    Button,
    Menu,
    TextField,
    Scrollbar,
    Status,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AccessibilityInfo {
    pub role: AccessibilityRole,
    pub name: String,
    pub position_in_set: Option<(u32, u32)>,
}

impl Default for AccessibilityInfo {
    fn default() -> Self {
        Self {
            role: AccessibilityRole::Generic,
            name: String::new(),
            position_in_set: None,
        }
    }
}

impl WidgetDescription {
    pub fn is_focusable(&self) -> bool {
        self.focus_order.is_some() && !self.state.disabled()
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct WidgetTree {
    widgets: Vec<WidgetDescription>,
}

impl WidgetTree {
    pub fn push(&mut self, widget: WidgetDescription) {
        self.widgets.push(widget);
    }

    pub fn widgets(&self) -> &[WidgetDescription] {
        &self.widgets
    }

    pub fn hit_test(&self, x: f32, y: f32) -> Option<WidgetId> {
        self.widgets
            .iter()
            .enumerate()
            .filter(|widget| !widget.1.state.disabled() && widget.1.bounds.contains(x, y))
            .max_by_key(|(index, widget)| (widget.z_index, *index))
            .map(|(_, widget)| widget.id)
    }

    pub fn focus_order(&self) -> Vec<WidgetId> {
        let mut focusable = self
            .widgets
            .iter()
            .filter(|widget| widget.is_focusable())
            .collect::<Vec<_>>();
        focusable.sort_by(|left, right| {
            left.focus_order
                .cmp(&right.focus_order)
                .then_with(|| left.id.0.cmp(&right.id.0))
        });
        focusable.into_iter().map(|widget| widget.id).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        AccessibilityRole, Bounds, PaintDescription, Rgba, Theme, WidgetDescription, WidgetId,
        WidgetState, WidgetTree,
    };

    fn widget(id: u64, z_index: i32, focus_order: Option<u32>) -> WidgetDescription {
        WidgetDescription {
            id: WidgetId(id),
            bounds: Bounds {
                x: 10.0,
                y: 10.0,
                width: 20.0,
                height: 20.0,
            },
            state: WidgetState::empty(),
            z_index,
            focus_order,
            accessibility: super::AccessibilityInfo::default(),
        }
    }

    #[test]
    fn bounds_include_edges_but_reject_non_finite_points() {
        let bounds = Bounds {
            x: 4.0,
            y: 8.0,
            width: 12.0,
            height: 6.0,
        };

        assert!(bounds.contains(4.0, 8.0));
        assert!(bounds.contains(16.0, 14.0));
        assert!(!bounds.contains(f32::NAN, 10.0));
        assert!(!bounds.contains(20.0, 10.0));
    }

    #[test]
    fn hit_testing_prefers_topmost_insertion_order() {
        let mut tree = WidgetTree::default();
        tree.push(widget(1, 0, None));
        tree.push(widget(2, 1, None));

        assert_eq!(tree.hit_test(20.0, 20.0), Some(WidgetId(2)));
    }

    #[test]
    fn disabled_widgets_are_not_hit_or_focusable() {
        let mut disabled = widget(1, 2, Some(0));
        disabled.state = disabled.state.with_disabled(true);
        let mut tree = WidgetTree::default();
        tree.push(disabled);

        assert_eq!(tree.hit_test(20.0, 20.0), None);
        assert!(tree.focus_order().is_empty());
    }

    #[test]
    fn focus_order_is_stable_and_ignores_non_focusable_widgets() {
        let mut tree = WidgetTree::default();
        tree.push(widget(3, 0, Some(2)));
        tree.push(widget(1, 0, Some(1)));
        tree.push(widget(2, 0, None));

        assert_eq!(tree.focus_order(), vec![WidgetId(1), WidgetId(3)]);
    }

    #[test]
    fn theme_tokens_are_backend_neutral_and_paint_keeps_them_explicit() {
        let theme = Theme::default();
        let panel = PaintDescription::Panel {
            bounds: Bounds {
                x: 1.0,
                y: 2.0,
                width: 30.0,
                height: 40.0,
            },
            fill: theme.colors.surface,
            border: None,
        };

        assert_eq!(theme.spacing.medium, 8.0);
        assert_eq!(theme.colors.accent, Rgba(97, 175, 239, 255));
        assert!(matches!(panel, PaintDescription::Panel { .. }));
    }

    #[test]
    fn accessibility_metadata_preserves_role_name_and_set_position() {
        let mut widget = widget(7, 0, Some(2));
        widget.accessibility.role = AccessibilityRole::Menu;
        widget.accessibility.name = "View".into();
        widget.accessibility.position_in_set = Some((2, 3));

        assert_eq!(widget.accessibility.role, AccessibilityRole::Menu);
        assert_eq!(widget.accessibility.name, "View");
        assert_eq!(widget.accessibility.position_in_set, Some((2, 3)));
    }
}
