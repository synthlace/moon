use console::{style, Attribute, Style, Term};
use miette::IntoDiagnostic;
use starbase_styles::color::{self, Color};
use std::fmt::Display;

pub enum Label {
    Default,
    Brand,
    Failure,
    // Success,
}

pub type TermWriteResult = miette::Result<()>;

// Extend `Term` with our own methods

pub trait ExtendedTerm {
    fn format(&self, value: &impl Display) -> String;
    fn format_label<V: AsRef<str>>(&self, kind: Label, message: V) -> String;
    fn line<V: AsRef<str>>(&self, message: V) -> TermWriteResult;
    fn flush_lines(&self) -> TermWriteResult;

    // RENDERERS

    fn render_entry<K: AsRef<str>, V: AsRef<str>>(&self, key: K, value: V) -> TermWriteResult;
    fn render_entry_bool<K: AsRef<str>>(&self, key: K, value: bool) -> TermWriteResult;
    fn render_entry_list<K: AsRef<str>, V: AsRef<[String]>>(
        &self,
        key: K,
        values: V,
    ) -> TermWriteResult;
    fn render_label<V: AsRef<str>>(&self, kind: Label, message: V) -> TermWriteResult;
    fn render_list<V: AsRef<[String]>>(&self, values: V) -> TermWriteResult;
}

impl ExtendedTerm for Term {
    fn format(&self, value: &impl Display) -> String {
        format!("{value}")
    }

    fn format_label<V: AsRef<str>>(&self, kind: Label, message: V) -> String {
        let mut style = Style::new()
            .attr(Attribute::Bold)
            .color256(Color::Black as u8);

        match kind {
            Label::Brand => {
                style = style.on_color256(Color::Purple as u8);
            }
            Label::Default => {
                style = style.on_color256(Color::White as u8);
            }
            Label::Failure => {
                style = style
                    .color256(Color::White as u8)
                    .on_color256(Color::Red as u8);
            } // Label::Success => {
              //     style = style.on_color256(Color::Green as u8);
              // }
        }

        style
            .apply_to(format!(" {} ", message.as_ref()).to_uppercase())
            .to_string()
    }

    fn line<V: AsRef<str>>(&self, message: V) -> TermWriteResult {
        self.write_line(message.as_ref()).into_diagnostic()
    }

    fn flush_lines(&self) -> TermWriteResult {
        self.flush().into_diagnostic()
    }

    fn render_entry<K: AsRef<str>, V: AsRef<str>>(&self, key: K, value: V) -> TermWriteResult {
        let label = color::muted_light(format!("{}:", style(key.as_ref()).bold()));

        self.line(format!("{} {}", label, value.as_ref()))
    }

    fn render_entry_bool<K: AsRef<str>>(&self, key: K, value: bool) -> TermWriteResult {
        self.render_entry(key, if value { "Yes" } else { "No" })
    }

    fn render_entry_list<K: AsRef<str>, V: AsRef<[String]>>(
        &self,
        key: K,
        values: V,
    ) -> TermWriteResult {
        let label = color::muted_light(format!("{}:", style(key.as_ref()).bold()));

        self.line(label)?;
        self.render_list(values)?;

        Ok(())
    }

    fn render_label<V: AsRef<str>>(&self, kind: Label, message: V) -> TermWriteResult {
        self.line(self.format_label(kind, message.as_ref()))?;
        self.line("")?;

        Ok(())
    }

    fn render_list<V: AsRef<[String]>>(&self, values: V) -> TermWriteResult {
        let mut values = values.as_ref().to_owned();
        values.sort();

        for value in values {
            self.line(format!(" {} {}", color::muted("-"), value))?;
        }

        Ok(())
    }
}
