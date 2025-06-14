use {
    crate::{
        crossterm::{
            style::{
                Attribute,
                Attributes,
                Color,
                ContentStyle,
                PrintStyledContent,
                SetBackgroundColor,
                SetForegroundColor,
                StyledContent,
            },
            terminal::{
                Clear,
                ClearType,
            },
            QueueableCommand,
        },
        errors::Result,
        styled_char::StyledChar,
    },
    std::fmt::{
        self,
        Display,
    },
};

/// The attributes which are often supported
pub static ATTRIBUTES: &[Attribute] = &[
    Attribute::Bold,
    Attribute::Dim,
    Attribute::Italic,
    Attribute::Underlined,
    Attribute::Reverse,
    Attribute::CrossedOut,
    Attribute::Encircled,
    Attribute::OverLined,
];

/// A style which may be applied to a compound
#[derive(Default, Clone, Debug, PartialEq)]
pub struct CompoundStyle {
    pub object_style: ContentStyle, // a crossterm content style
}

impl From<ContentStyle> for CompoundStyle {
    fn from(object_style: ContentStyle) -> Self {
        Self { object_style }
    }
}

impl CompoundStyle {
    /// Apply an `StyledContent` to the passed displayable object.
    pub fn apply_to<D>(&self, val: D) -> StyledContent<D>
    where
        D: Clone + Display,
    {
        self.object_style.apply(val)
    }

    /// Get an new instance of `CompoundStyle`
    pub const fn new(
        foreground_color: Option<Color>,
        background_color: Option<Color>,
        attributes: Attributes,
    ) -> Self {
        Self {
            object_style: ContentStyle {
                foreground_color,
                background_color,
                underline_color: None,
                attributes,
            },
        }
    }

    /// Blend the foreground and background colors (if any) into the given dest color,
    /// with a weight in `[0..1]`.
    ///
    /// The `dest` color can be for example a [crossterm] color or a [coolor] one.
    pub fn blend_with<C: Into<coolor::Color>>(&mut self, dest: C, weight: f32) {
        debug_assert!((0.0..=1.0).contains(&weight));
        let dest: coolor::Color = dest.into();
        if let Some(fg) = self.object_style.foreground_color.as_mut() {
            let src: coolor::Color = (*fg).into();
            *fg = coolor::Color::blend(src, 1.0 - weight, dest, weight).into();
        }
        if let Some(bg) = self.object_style.foreground_color.as_mut() {
            let src: coolor::Color = (*bg).into();
            *bg = coolor::Color::blend(src, 1.0 - weight, dest, weight).into();
        }
    }

    /// Get an new instance of `CompoundStyle`
    pub fn with_fgbg(fg: Color, bg: Color) -> Self {
        Self::new(Some(fg), Some(bg), Attributes::default())
    }

    /// Get an new instance of `CompoundStyle`
    pub fn with_fg(fg: Color) -> Self {
        Self::new(Some(fg), None, Attributes::default())
    }

    /// Get an new instance of `CompoundStyle`
    pub fn with_bg(bg: Color) -> Self {
        Self::new(None, Some(bg), Attributes::default())
    }

    /// Get an new instance of `CompoundStyle`
    pub fn with_attr(attr: Attribute) -> Self {
        let mut cp = Self::default();
        cp.add_attr(attr);
        cp
    }

    /// Set the foreground color to the passed color.
    pub fn set_fg(&mut self, color: Color) {
        self.object_style.foreground_color = Some(color);
    }

    /// Set the background color to the passed color.
    pub fn set_bg(&mut self, color: Color) {
        self.object_style.background_color = Some(color);
    }

    /// Set the colors to the passed ones
    pub fn set_fgbg(&mut self, fg: Color, bg: Color) {
        self.object_style.foreground_color = Some(fg);
        self.object_style.background_color = Some(bg);
    }

    /// Add an `Attribute`. Like italic, underlined or bold.
    pub fn add_attr(&mut self, attr: Attribute) {
        self.object_style.attributes.set(attr);
    }

    /// Check whether the style contains the attribute
    pub fn has_attr(&self, attr: Attribute) -> bool {
        self.object_style.attributes.has(attr)
    }

    /// Remove an `Attribute`. Like italic, underlined or bold.
    pub fn remove_attr(&mut self, attr: Attribute) {
        self.object_style.attributes.unset(attr);
    }

    /// Add the defined characteristics of `other` to self, overwriting
    ///  its own one when defined
    pub fn overwrite_with(&mut self, other: &Self) {
        self.object_style.foreground_color = other
            .object_style
            .foreground_color
            .or(self.object_style.foreground_color);
        self.object_style.background_color = other
            .object_style
            .background_color
            .or(self.object_style.background_color);
        self.object_style
            .attributes
            .extend(other.object_style.attributes);
    }

    #[inline(always)]
    pub const fn get_fg(&self) -> Option<Color> {
        self.object_style.foreground_color
    }

    #[inline(always)]
    pub const fn get_bg(&self) -> Option<Color> {
        self.object_style.background_color
    }

    /// Write a char several times with the line compound style
    #[inline(always)]
    pub fn repeat_char(&self, f: &mut fmt::Formatter<'_>, c: char, count: usize) -> fmt::Result {
        if count > 0 {
            let s = std::iter::repeat(c).take(count).collect::<String>();
            write!(f, "{}", self.apply_to(s))?;
        }
        Ok(())
    }

    /// Write a string several times with the line compound style
    ///
    /// Implementation Note: performances here are critical
    #[inline(always)]
    pub fn repeat_string(&self, f: &mut fmt::Formatter<'_>, s: &str, count: usize) -> fmt::Result {
        if count > 0 {
            write!(f, "{}", self.apply_to(s.repeat(count)))
        } else {
            Ok(())
        }
    }

    /// Write 0 or more spaces with the line's compound style
    #[inline(always)]
    pub fn repeat_space(&self, f: &mut fmt::Formatter<'_>, count: usize) -> fmt::Result {
        self.repeat_string(f, " ", count)
    }

    /// write the value with this style on the given
    /// writer
    pub fn queue<W, D>(&self, w: &mut W, val: D) -> Result<()>
    where
        D: Clone + Display,
        W: std::io::Write,
    {
        w.queue(PrintStyledContent(self.apply_to(val)))?;
        Ok(())
    }

    /// write the string with this style on the given
    /// writer
    pub fn queue_str<W, S: Into<String>>(&self, w: &mut W, s: S) -> Result<()>
    where
        W: std::io::Write,
    {
        self.queue(w, s.into())
    }

    pub fn queue_fg<W>(&self, w: &mut W) -> Result<()>
    where
        W: std::io::Write,
    {
        if let Some(fg) = self.object_style.foreground_color {
            w.queue(SetForegroundColor(fg))?;
        }
        Ok(())
    }

    pub fn queue_bg<W>(&self, w: &mut W) -> Result<()>
    where
        W: std::io::Write,
    {
        if let Some(bg) = self.object_style.background_color {
            w.queue(SetBackgroundColor(bg))?;
        }
        Ok(())
    }

    /// Clear with the compound_style's background.
    ///
    /// ```
    /// # use termimad::*;
    /// # use termimad::crossterm::terminal::ClearType;
    /// # let skin = MadSkin::default();
    /// let mut w = std::io::stderr();
    /// skin.paragraph.compound_style.clear(&mut w, ClearType::UntilNewLine).unwrap();
    /// ```
    pub fn clear<W>(&self, w: &mut W, clear_type: ClearType) -> Result<()>
    where
        W: std::io::Write,
    {
        self.queue_bg(w)?;
        w.queue(Clear(clear_type))?;
        Ok(())
    }

    pub fn style_char(&self, nude_char: char) -> StyledChar {
        StyledChar::new(self.clone(), nude_char)
    }

    pub fn attrs(&self) -> Attributes {
        self.object_style.attributes
    }
}
