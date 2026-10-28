//! What the Displays page knows: the displays the compositor lists, with the modes each can run.
//! Parsed once where it enters.

/// A display's connector name (`DP-1`): what names a display.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct OutputId(pub String);

/// A size in pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PixelSize {
    /// Width.
    pub width: u32,
    /// Height.
    pub height: u32,
}

/// A refresh rate in thousandths of a hertz (60000 is 60 Hz), as the compositor says it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RefreshMilliHz(pub u32);

/// A scale in thousandths (1500 is 1.5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ScaleMilli(pub u32);

/// A place in the compositor's space, in logical pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DisplayPosition {
    /// Rightwards.
    pub x: i32,
    /// Downwards.
    pub y: i32,
}

/// A physical size in millimetres.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Millimetres {
    /// Width.
    pub width: u32,
    /// Height.
    pub height: u32,
}

/// How a display is turned.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DisplayRotation {
    /// As it stands.
    Standard,
    /// A quarter turn.
    Quarter,
    /// Upside down.
    Half,
    /// Three quarters of a turn.
    ThreeQuarters,
}

impl DisplayRotation {
    /// Every rotation, in the order the page lists them.
    pub const ALL: [DisplayRotation; 4] = [
        DisplayRotation::Standard,
        DisplayRotation::Quarter,
        DisplayRotation::Half,
        DisplayRotation::ThreeQuarters,
    ];

    /// Whether width and height trade places.
    pub fn is_sideways(self) -> bool {
        matches!(
            self,
            DisplayRotation::Quarter | DisplayRotation::ThreeQuarters
        )
    }
}

/// A mode a display can run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DisplayMode {
    /// Its size in pixels.
    pub size: PixelSize,
    /// Its refresh rate.
    pub refresh: RefreshMilliHz,
    /// Whether the display prefers it.
    pub preferred: bool,
}

/// One display.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Monitor {
    /// Its connector.
    pub id: OutputId,
    /// What a person calls it: the maker and model, else the connector.
    pub name: String,
    /// Every mode it can run.
    pub modes: Vec<DisplayMode>,
    /// The mode it runs now.
    pub current: Option<DisplayMode>,
    /// Its scale now.
    pub scale: ScaleMilli,
    /// Its rotation now.
    pub rotation: DisplayRotation,
    /// Its place now.
    pub position: DisplayPosition,
    /// Its physical size, when it says.
    pub size_mm: Option<Millimetres>,
}

/// The displays in the order the compositor lists them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisplaysStatus {
    /// The displays.
    pub displays: Vec<Monitor>,
}

impl DisplaysStatus {
    /// The display called `id`.
    pub fn display(&self, id: &OutputId) -> Option<&Monitor> {
        self.displays.iter().find(|d| &d.id == id)
    }
}

impl Monitor {
    /// The size the display fills in logical pixels now: its mode over its scale, turned.
    pub fn logical_size(&self) -> Option<PixelSize> {
        let mode = self.current?;
        let scaled = |pixels: u32| {
            u32::try_from(u64::from(pixels) * 1000 / u64::from(self.scale.0.max(1)))
                .unwrap_or(u32::MAX)
        };
        let (width, height) = (scaled(mode.size.width), scaled(mode.size.height));
        Some(if self.rotation.is_sideways() {
            PixelSize {
                width: height,
                height: width,
            }
        } else {
            PixelSize { width, height }
        })
    }
}
