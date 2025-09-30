use std::str::FromStr;

/// Typed representation of common `mako` configuration options.
///
/// This module provides a conservative, extensible Rust model of the
/// configuration keys found in mako's man page. Fields are optional
/// so the struct can be used to hold a partial configuration and
/// round-trip values to/from key/value strings.
#[derive(Debug, Clone, Default)]
pub struct MakoConfig {
    pub sort: Option<String>, // e.g. "-time" or "+priority" (kept as string)
    pub layer: Option<Layer>,
    pub background_color: Option<String>,
    pub width: Option<u32>,   // pixels
    pub height: Option<u32>,  // pixels
    pub border_size: Option<u32>,
    pub border_color: Option<String>,
    pub border_radius: Option<u32>,
    pub icons: Option<bool>,
    pub max_icon_size: Option<u32>,
    pub default_timeout: Option<u32>, // milliseconds
    pub ignore_timeout: Option<bool>,
    pub font: Option<String>,
    pub outer_margin: Option<u32>,
    pub padding: Option<u32>,
    pub markup: Option<bool>,
    pub progress_color: Option<String>,
    pub progress_background_color: Option<String>,
    pub icon_path: Option<String>,
    pub icon_location: Option<IconLocation>,
    pub icon_border_radius: Option<u32>,
    pub group_by: Option<String>,
    pub layout: Option<LayoutKind>,
    pub text_align: Option<TextAlign>,
}

/// Where to draw the notifications (common mako values).
#[derive(Clone)]
pub enum Layer {
    Overlay,
    Bottom,
    Top,
    Normal,
}

impl FromStr for Layer {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "overlay" => Ok(Layer::Overlay),
            "bottom" => Ok(Layer::Bottom),
            "top" => Ok(Layer::Top),
            "normal" => Ok(Layer::Normal),
            _ => Err(()),
        }
    }
}

/// Simple layout hint (mako supports several named layouts).
#[derive(Clone)]
pub enum LayoutKind {
    Normal,
    Overlay,
    Center,
    Other(String),
}

impl FromStr for LayoutKind {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "normal" => Ok(LayoutKind::Normal),
            "overlay" => Ok(LayoutKind::Overlay),
            "center" => Ok(LayoutKind::Center),
            other => Ok(LayoutKind::Other(other.to_string())),
        }
    }
}

/// Where the icon is placed relative to the notification box.
#[derive(Clone)]
pub enum IconLocation {
    Left,
    Right,
    Top,
    Bottom,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Center,
    Other(String),
}

impl FromStr for IconLocation {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "left" => Ok(IconLocation::Left),
            "right" => Ok(IconLocation::Right),
            "top" => Ok(IconLocation::Top),
            "bottom" => Ok(IconLocation::Bottom),
            "top-left" | "topleft" => Ok(IconLocation::TopLeft),
            "top-right" | "topright" => Ok(IconLocation::TopRight),
            "bottom-right" | "bottomright" => Ok(IconLocation::BottomRight),
            "center" => Ok(IconLocation::Center),
            other => Ok(IconLocation::Other(other.to_string())),
        }
    }
}
#[derive(Clone)]
pub enum TextAlign {
    Left,
    Center,
    Right,
}

impl FromStr for TextAlign {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "left" => Ok(TextAlign::Left),
            "center" => Ok(TextAlign::Center),
            "right" => Ok(TextAlign::Right),
            _ => Err(()),
        }
    }
}

impl MakoConfig {
    /// Create an empty config.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set a known key from a string key/value pair. Unknown keys are ignored.
    ///
    /// This is a forgiving parser: it accepts values like "100", "100px",
    /// boolean strings like "1", "0", "true", "false", and preserves
    /// color/font strings as-is.
    pub fn set_from_kv(&mut self, key: &str, value: &str) {
        match key.trim() {
            "sort" => self.sort = Some(value.trim().to_string()),
            "layer" => match Layer::from_str(value) { Ok(l) => self.layer = Some(l), Err(_) => {} },
            "background-color" => self.background_color = Some(value.trim().to_string()),
            "width" => self.width = parse_px(value),
            "height" => self.height = parse_px(value),
            "border-size" => self.border_size = parse_px(value),
            "border-color" => self.border_color = Some(value.trim().to_string()),
            "border-radius" => self.border_radius = parse_px(value),
            "icons" => self.icons = parse_bool(value),
            "max-icon-size" => self.max_icon_size = parse_px(value),
            "default-timeout" => self.default_timeout = parse_u32(value),
            "ignore-timeout" => self.ignore_timeout = parse_bool(value),
            "font" => self.font = Some(value.trim().to_string()),
            "outer-margin" => self.outer_margin = parse_px(value),
            "padding" => self.padding = parse_px(value),
            "markup" => self.markup = parse_bool(value),
            "progress-color" => self.progress_color = Some(value.trim().to_string()),
            "progress-background-color" => self.progress_background_color = Some(value.trim().to_string()),
            "icon-path" => self.icon_path = Some(value.trim().to_string()),
            "icon-location" => match IconLocation::from_str(value) { Ok(v) => self.icon_location = Some(v), Err(_) => {} },
            "icon-border-radius" => self.icon_border_radius = parse_px(value),
            "group-by" => self.group_by = Some(value.trim().to_string()),
            "layout" => match LayoutKind::from_str(value) { Ok(l) => self.layout = Some(l), Err(_) => {} },
            "text-align" => match TextAlign::from_str(value) { Ok(a) => self.text_align = Some(a), Err(_) => {} },
            _ => {
                // unknown key — keep it ignored for now
            }
        }
    }

    /// Convert the typed config back into key/value string pairs. Only
    /// populated fields are emitted.
    pub fn to_kv_pairs(&self) -> Vec<(String, String)> {
        let mut out = Vec::new();
        if let Some(v) = &self.sort { out.push(("sort".to_string(), v.clone())); }
        if let Some(v) = &self.layer { out.push(("layer".to_string(), format!("{:?}", v).to_lowercase())); }
        if let Some(v) = &self.background_color { out.push(("background-color".to_string(), v.clone())); }
        if let Some(v) = &self.width { out.push(("width".to_string(), format!("{}", v))); }
        if let Some(v) = &self.height { out.push(("height".to_string(), format!("{}", v))); }
        if let Some(v) = &self.border_size { out.push(("border-size".to_string(), format!("{}", v))); }
        if let Some(v) = &self.border_color { out.push(("border-color".to_string(), v.clone())); }
        if let Some(v) = &self.border_radius { out.push(("border-radius".to_string(), format!("{}", v))); }
        if let Some(v) = &self.icons { out.push(("icons".to_string(), bool_to_str(*v))); }
        if let Some(v) = &self.max_icon_size { out.push(("max-icon-size".to_string(), format!("{}", v))); }
        if let Some(v) = &self.default_timeout { out.push(("default-timeout".to_string(), format!("{}", v))); }
        if let Some(v) = &self.ignore_timeout { out.push(("ignore-timeout".to_string(), bool_to_str(*v))); }
        if let Some(v) = &self.font { out.push(("font".to_string(), v.clone())); }
        if let Some(v) = &self.outer_margin { out.push(("outer-margin".to_string(), format!("{}", v))); }
        if let Some(v) = &self.padding { out.push(("padding".to_string(), format!("{}", v))); }
        if let Some(v) = &self.markup { out.push(("markup".to_string(), bool_to_str(*v))); }
        if let Some(v) = &self.progress_color { out.push(("progress-color".to_string(), v.clone())); }
        if let Some(v) = &self.progress_background_color { out.push(("progress-background-color".to_string(), v.clone())); }
        if let Some(v) = &self.icon_path { out.push(("icon-path".to_string(), v.clone())); }
        if let Some(v) = &self.icon_location { out.push(("icon-location".to_string(), format!("{:?}", v).to_lowercase())); }
        if let Some(v) = &self.icon_border_radius { out.push(("icon-border-radius".to_string(), format!("{}", v))); }
        if let Some(v) = &self.group_by { out.push(("group-by".to_string(), v.clone())); }
        if let Some(v) = &self.layout { out.push(("layout".to_string(), format!("{:?}", v).to_lowercase())); }
        if let Some(v) = &self.text_align { out.push(("text-align".to_string(), format!("{:?}", v).to_lowercase())); }
        out
    }
}

/// Return a list of known mako configuration keys with a short description.
pub fn known_keys() -> Vec<(&'static str, &'static str)> {
    vec![
        ("sort", "Sort order expression, e.g. -time"),
        ("layer", "Window layer: overlay, normal, top, bottom"),
        ("background-color", "Background color (#rrggbb or named)") ,
        ("width", "Notification width in pixels"),
        ("height", "Notification height in pixels"),
        ("border-size", "Border width in pixels"),
        ("border-color", "Border color (#rrggbb)") ,
        ("border-radius", "Corner radius in pixels"),
        ("icons", "Show icons: 1 or 0"),
        ("max-icon-size", "Maximum icon size in pixels"),
        ("default-timeout", "Default timeout in milliseconds"),
        ("ignore-timeout", "Ignore per-notification timeout: 1 or 0"),
        ("font", "Font description, e.g. 'monospace 10'"),
        ("outer-margin", "Outer margin in pixels"),
        ("padding", "Padding in pixels"),
        ("markup", "Enable markup rendering: 1 or 0"),
        ("progress-color", "Progress bar color"),
        ("progress-background-color", "Progress background color"),
        ("icon-path", "Search paths for icons (colon separated)") ,
        ("icon-location", "Icon position: left, right, top, bottom, top-left, ..."),
    ("anchor", "Anchor position: top-right, top-center, top-left, bottom-right, bottom-center, bottom-left, center-right, center-left, center"),
    ("anchor-point", "Alias for anchor; same values as anchor"),
    ("<custom>", "Create a custom key name (type after selecting this)"),
        ("icon-border-radius", "Icon corner radius in pixels"),
        ("group-by", "Group notifications by this property (e.g. category)"),
        ("layout", "Layout hint: normal, overlay, center"),
        ("text-align", "Text alignment: left, center, right"),
    ]
}

/// For a given key, return a small set of allowed values when applicable.
pub fn allowed_values(key: &str) -> Option<Vec<&'static str>> {
    match key {
        "layer" => Some(vec!["overlay", "normal", "top", "bottom"]),
        "icons" => Some(vec!["1", "0", "true", "false"]),
        "ignore-timeout" => Some(vec!["1", "0", "true", "false"]),
        "markup" => Some(vec!["1", "0", "true", "false"]),
        "icon-location" => Some(vec!["left", "right", "top", "bottom", "top-left", "top-right", "bottom-left", "bottom-right", "center"]),
        "text-align" => Some(vec!["left", "center", "right"]),
        "layout" => Some(vec!["normal", "overlay", "center"]),
    "anchor" => Some(vec!["top-right","top-center","top-left","bottom-right","bottom-center","bottom-left","center-right","center-left","center","bottom","top","left","right","top","bottom"]),
    "anchor-point" => Some(vec!["top-right","top-center","top-left","bottom-right","bottom-center","bottom-left","center-right","center-left","center","bottom","top","left","right","top","bottom"]),
        _ => None,
    }
}

fn parse_px(s: &str) -> Option<u32> {
    let s = s.trim();
    let s = s.strip_suffix("px").unwrap_or(s);
    s.parse::<u32>().ok()
}

fn parse_u32(s: &str) -> Option<u32> {
    s.trim().parse::<u32>().ok()
}

fn parse_bool(s: &str) -> Option<bool> {
    match s.trim().to_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    }
}

fn bool_to_str(b: bool) -> String { if b { "1".to_string() } else { "0".to_string() } }

// Debug helpers to make enum formatting stable when converting to strings
impl std::fmt::Debug for Layer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Layer::Overlay => write!(f, "overlay"),
            Layer::Bottom => write!(f, "bottom"),
            Layer::Top => write!(f, "top"),
            Layer::Normal => write!(f, "normal"),
        }
    }
}

impl std::fmt::Debug for LayoutKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LayoutKind::Normal => write!(f, "normal"),
            LayoutKind::Overlay => write!(f, "overlay"),
            LayoutKind::Center => write!(f, "center"),
            LayoutKind::Other(s) => write!(f, "{}", s),
        }
    }
}

impl std::fmt::Debug for IconLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IconLocation::Left => write!(f, "left"),
            IconLocation::Right => write!(f, "right"),
            IconLocation::Top => write!(f, "top"),
            IconLocation::Bottom => write!(f, "bottom"),
            IconLocation::TopLeft => write!(f, "top-left"),
            IconLocation::TopRight => write!(f, "top-right"),
            IconLocation::BottomLeft => write!(f, "bottom-left"),
            IconLocation::BottomRight => write!(f, "bottom-right"),
            IconLocation::Center => write!(f, "center"),
            IconLocation::Other(s) => write!(f, "{}", s),
        }
    }
}

impl std::fmt::Debug for TextAlign {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TextAlign::Left => write!(f, "left"),
            TextAlign::Center => write!(f, "center"),
            TextAlign::Right => write!(f, "right"),
        }
    }
}
