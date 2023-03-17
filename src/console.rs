//! This is a separate module so we can exclude it from WASM compilation

use crossterm::{
    style::{Attribute, Color, Print, SetAttribute, SetForegroundColor},
    QueueableCommand,
};

use std::io::{stdout, Stdout, Write};

use crate::render::{HORIZONTAL_PIPE, RIGHT_FORK, VERTICAL_PIPE};

struct Style {
    color: Color,
    attribute: Attribute,
}

impl Style {
    fn set_color(&mut self, stdout: &mut Stdout, color: Color) -> Result<(), std::io::Error> {
        if self.color != color {
            stdout.queue(SetForegroundColor(color))?;
            self.color = color;
        }
        Ok(())
    }

    fn set_attribute(
        &mut self,
        stdout: &mut Stdout,
        attribute: Attribute,
    ) -> Result<(), std::io::Error> {
        if self.attribute != attribute {
            stdout.queue(SetAttribute(attribute))?;
            self.attribute = attribute;
        }
        Ok(())
    }
}

impl Default for Style {
    fn default() -> Self {
        Self {
            color: Color::Reset,
            attribute: Attribute::Reset,
        }
    }
}

pub fn colorful(input: &str) -> Result<(), std::io::Error> {
    let mut stdout = stdout();
    stdout.queue(SetAttribute(Attribute::Bold))?;
    let mut style = Style::default();
    let mut chars = input.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '0'..='9' => {
                style.set_color(&mut stdout, Color::Magenta)?;
                style.set_attribute(&mut stdout, Attribute::Reset)?;
            }
            '+' | '-' | '\u{00D7}' | '=' | '>' => {
                style.set_color(&mut stdout, Color::DarkYellow)?;
                style.set_attribute(&mut stdout, Attribute::Reset)?;
            }
            'k' => {
                if let Some('0'..='9' | 'l') = chars.peek() {
                    style.set_color(&mut stdout, Color::Magenta)?;
                    style.set_attribute(&mut stdout, Attribute::Reset)?;
                }
            }
            'd' | 'l' => {
                if let Some('0'..='9') = chars.peek() {
                    style.set_color(&mut stdout, Color::Magenta)?;
                    style.set_attribute(&mut stdout, Attribute::Reset)?;
                }
            }
            'a'..='z' | 'A'..='Z' => {
                style.set_color(&mut stdout, Color::Green)?;
                style.set_attribute(&mut stdout, Attribute::Bold)?;
            }
            VERTICAL_PIPE | HORIZONTAL_PIPE | RIGHT_FORK => {
                style.set_color(&mut stdout, Color::Reset)?;
                style.set_attribute(&mut stdout, Attribute::Reset)?;
            }
            _ => {
                style.set_color(&mut stdout, Color::Reset)?;
                style.set_attribute(&mut stdout, Attribute::Reset)?;
            }
        }
        stdout.queue(Print(c))?;
    }
    stdout.flush()?;
    Ok(())
}
