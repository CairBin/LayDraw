use crate::i18n::{en_us::EnUs, zh_cn_simple::ZhCnSimple};

pub mod en_us;
pub mod zh_cn_simple;

#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub enum LanguageText {
    Ready,
    NoToolActive,
    NewCanvas,
    CanvasResized,
    UndoDone,
    RedoDone,
    Opened,
    ImportedImage,
    OpenFailed,
    SaveFailedInvalidBuffer,
    Saved,
    SaveFailed,
    Drawing,
    Erasing,
    FilledStatus,
    PickedColor,
    PreviewShape,
    ShapeDrawn,
    StrokeDone,
    AppTitle,
    New,
    Open,
    RecentFiles,
    ImportImage,
    Save,
    SaveAs,
    Undo,
    Redo,
    Handle,
    Tools,
    Select,
    Shapes,
    PrimaryColor,
    SecondaryColor,
    Swap,
    Size,
    Zoom,
    Fit,
    Grid,
    StatusBar,
    Pixel,
    Layer,
    Layers,
    BackgroundLayer,
    AddLayer,
    DeleteLayer,
    MoveLayerUp,
    MoveLayerDown,
    MergeLayerDown,
    MergeVisibleLayers,
    LayerOpacity,
    LayerBlendMode,
    BlendNormal,
    BlendMultiply,
    BlendScreen,
    LayerDragHint,
    LayersMerged,
    LayerCleared,
    LayerDeleted,
    LayerMoved,
    LayerMergedDown,
    ClearLayer,
    ConfirmClearLayerTitle,
    ConfirmClearLayerMessage,
    Canvas,
    ApplySize,
    Unsaved,
    SavedState,
    Untitled,
    DiscardChangesTitle,
    DiscardChangesMessage,
    Language,
    FileTab,
    HomeTab,
    ViewTab,
    PluginsTab,
    PluginApplied,
    BuiltInPlugins,
    Components,
    PluginActive,
    EnableAllComponents,
    Panels,
    Hooks,
    ImageGroup,
    RotateLeft,
    RotateRight,
    FlipHorizontal,
    FlipVertical,
    Brushes,
    Colors,
    EditColors,
    BasicColors,
    RecentColors,
    CustomColors,
    AddCustomColor,
    Confirm,
    Cancel,
    Red,
    Green,
    Blue,
    Rulers,
    TextTool,
    TextValue,
    Magnifier,
    SelectionStatus,
    TextPlaced,
    TextEditing,
    Font,
    FontSize,
    TextBackgroundFill,
    CommitText,
    CancelText,
    SelectionMove,
    SelectionCopy,
    SelectionCut,
    SelectionPaste,
    SelectionCrop,
    SelectionDelete,
    TransparentSelection,
    BrushKindBrush,
    BrushKindPencil,
    BrushKindCalligraphy,
    BrushKindSpray,
    BrushKindOil,
    BrushKindCrayon,
    BrushKindMarker,
    BrushKindNaturalPencil,
    BrushKindWatercolor,
    ImageFilter,
    PngFilter,
    JpegFilter,
    BmpFilter,
    Pencil,
    Brush,
    Eraser,
    Fill,
    Picker,
    Shape,
    Line,
    Curve,
    Oval,
    Rectangle,
    RoundedRectangle,
    Polygon,
    Triangle,
    RightTriangle,
    Diamond,
    Pentagon,
    Hexagon,
    RightArrow,
    LeftArrow,
    UpArrow,
    DownArrow,
    FourPointStar,
    FivePointStar,
    SixPointStar,
    RectCallout,
    OvalCallout,
    CloudCallout,
    Heart,
    Lightning,
    Outline,
    Filled,
    FilledOutline,

    // 作为扩展
    Extra(String),
}

pub trait I18n {
    fn get_text(&self, text: LanguageText) -> String;

    fn get_name(&self) -> &str;
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub enum Language {
    // 简体中文
    ZhCnSimple(ZhCnSimple),

    // 英文（美国）
    EnUs(EnUs),

    // 作为扩展
    Extra(String),
}

impl Default for Language {
    fn default() -> Self {
        Self::EnUs(EnUs)
    }
}

impl Language {
    pub fn as_zh_cn_simple(&self) -> Option<&ZhCnSimple> {
        match self {
            Self::ZhCnSimple(language) => Some(language),
            _ => None,
        }
    }

    pub fn as_en_us(&self) -> Option<&EnUs> {
        match self {
            Self::EnUs(language) => Some(language),
            _ => None,
        }
    }

    pub fn extra_name(&self) -> Option<&str> {
        match self {
            Self::Extra(name) => Some(name.as_str()),
            _ => None,
        }
    }

    pub fn plugin_locale_key(&self) -> &str {
        match self {
            Self::ZhCnSimple(_) => "zh-CN",
            Self::EnUs(_) => "en-US",
            Self::Extra(name) => name.as_str(),
        }
    }

    pub fn plugin_text(&self, text: impl Into<String>) -> String {
        self.get_text(LanguageText::Extra(text.into()))
    }

    pub fn get_text(&self, text: LanguageText) -> String {
        match self {
            Self::ZhCnSimple(lang) => lang.get_text(text),
            Self::EnUs(lang) => lang.get_text(text),
            Self::Extra(_) => match text {
                LanguageText::Extra(text) => text,
                other => ZhCnSimple.get_text(other),
            },
        }
    }

    pub fn get_name(&self) -> &str {
        match self {
            Self::ZhCnSimple(lang) => lang.get_name(),
            Self::EnUs(lang) => lang.get_name(),
            Self::Extra(name) => name.as_str(),
        }
    }
}
