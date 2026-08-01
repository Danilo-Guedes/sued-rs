pub const APP_TITLE: &str = " ☠  SueD — O Oráculo  ☠ ";

/// The terminal size the info screen tells people to use.
///
/// It lives here rather than inside the three translations because it is a fact
/// about the app, not a piece of language — three copies of a number are three
/// chances to drift, and the string it replaced had drifted from reality
/// entirely (it recommended `80×24`, inherited from the VT100 default and never
/// measured; §J.7 found the app actually breaks below ~92×40).
///
/// This is the *comfortable* size — measured as the one where everything
/// renders correctly, and the size the mockups were drawn at. The hard floor is
/// lower (~92×40); G3 may yet compact the design or band the layout, and when it
/// does, this is the one line that changes.
pub const RECOMMENDED_TERMINAL_SIZE: &str = "132×41";
