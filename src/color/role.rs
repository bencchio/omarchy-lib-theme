//! The roles applications ask for, instead of reaching for a theme key by name.

/// A slot in the interface, resolved to a concrete color for the active theme.
///
/// Every role also has a content color, the one that stays legible on top of it — see
/// [`Roles::on`](super::Roles::on).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Role {
    /// The canvas the interface sits on.
    Background,
    /// A surface raised off the canvas, such as a card or a panel.
    Elevated,
    /// A recessed surface, such as an input field or a well.
    Recessed,
    /// A line that marks the interface apart from its canvas, such as a border or a divider.
    Border,
    /// The color that carries the theme's identity, used for emphasis.
    Primary,
    /// The ground behind selected content.
    Selection,
    /// De-emphasized content, such as secondary labels.
    Muted,
    /// Something went wrong.
    Error,
    /// The element holding keyboard focus.
    Focus,
    /// An element that cannot be interacted with.
    Disabled,
}

impl Role {
    pub const ALL: [Self; 10] = [
        Self::Background,
        Self::Elevated,
        Self::Recessed,
        Self::Border,
        Self::Primary,
        Self::Selection,
        Self::Muted,
        Self::Error,
        Self::Focus,
        Self::Disabled,
    ];
}
