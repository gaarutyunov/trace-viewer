/// ANSI escape code parser for terminal output
/// Converts ANSI escape sequences to HTML with appropriate styling

#[derive(Debug, Clone, PartialEq)]
pub enum AnsiStyle {
    Bold,
    Dim,
    FgRed,
    FgGreen,
    FgYellow,
    FgBlue,
    FgMagenta,
    FgCyan,
}

impl AnsiStyle {
    fn to_css_class(&self) -> &'static str {
        match self {
            AnsiStyle::Bold => "ansi-bold",
            AnsiStyle::Dim => "ansi-dim",
            AnsiStyle::FgRed => "ansi-red",
            AnsiStyle::FgGreen => "ansi-green",
            AnsiStyle::FgYellow => "ansi-yellow",
            AnsiStyle::FgBlue => "ansi-blue",
            AnsiStyle::FgMagenta => "ansi-magenta",
            AnsiStyle::FgCyan => "ansi-cyan",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AnsiSegment {
    pub text: String,
    pub styles: Vec<AnsiStyle>,
}

impl AnsiSegment {
    pub fn new(text: String, styles: Vec<AnsiStyle>) -> Self {
        Self { text, styles }
    }

    pub fn css_classes(&self) -> String {
        self.styles
            .iter()
            .map(|style| style.to_css_class())
            .collect::<Vec<_>>()
            .join(" ")
    }
}

/// Parse ANSI escape codes from a string
pub fn parse_ansi(input: &str) -> Vec<AnsiSegment> {
    let mut segments = Vec::new();
    let mut current_text = String::new();
    let mut current_styles = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\x1b' || ch == '[' {
            // Check for ANSI escape sequence
            if ch == '\x1b' && chars.peek() == Some(&'[') {
                chars.next(); // consume '['
            }

            // Try to parse the escape sequence
            let mut code = String::new();
            while let Some(&next_ch) = chars.peek() {
                if next_ch.is_ascii_digit() {
                    code.push(next_ch);
                    chars.next();
                } else if next_ch == 'm' {
                    chars.next(); // consume 'm'
                    break;
                } else if next_ch == ';' {
                    chars.next(); // consume separator
                    code.push(';');
                } else {
                    // Not a valid ANSI sequence
                    break;
                }
            }

            // Save current segment if we have text
            if !current_text.is_empty() {
                segments.push(AnsiSegment::new(
                    current_text.clone(),
                    current_styles.clone(),
                ));
                current_text.clear();
            }

            // Parse the code
            if !code.is_empty() {
                for code_part in code.split(';') {
                    if let Ok(num) = code_part.parse::<u32>() {
                        match num {
                            0 => current_styles.clear(), // Reset
                            1 => current_styles.push(AnsiStyle::Bold),
                            2 => current_styles.push(AnsiStyle::Dim),
                            22 => {
                                // Normal intensity - remove bold and dim
                                current_styles
                                    .retain(|s| !matches!(s, AnsiStyle::Bold | AnsiStyle::Dim));
                            }
                            31 => current_styles.push(AnsiStyle::FgRed),
                            32 => current_styles.push(AnsiStyle::FgGreen),
                            33 => current_styles.push(AnsiStyle::FgYellow),
                            34 => current_styles.push(AnsiStyle::FgBlue),
                            35 => current_styles.push(AnsiStyle::FgMagenta),
                            36 => current_styles.push(AnsiStyle::FgCyan),
                            39 => {
                                // Default foreground - remove color styles
                                current_styles.retain(|s| {
                                    !matches!(
                                        s,
                                        AnsiStyle::FgRed
                                            | AnsiStyle::FgGreen
                                            | AnsiStyle::FgYellow
                                            | AnsiStyle::FgBlue
                                            | AnsiStyle::FgMagenta
                                            | AnsiStyle::FgCyan
                                    )
                                });
                            }
                            _ => {} // Ignore unsupported codes
                        }
                    }
                }
            }
        } else {
            current_text.push(ch);
        }
    }

    // Add final segment
    if !current_text.is_empty() {
        segments.push(AnsiSegment::new(current_text, current_styles));
    }

    segments
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_text() {
        let input = "Hello, world!";
        let segments = parse_ansi(input);
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].text, "Hello, world!");
        assert_eq!(segments[0].styles.len(), 0);
    }

    #[test]
    fn test_parse_red_text() {
        let input = "\x1b[31mRed text\x1b[39m";
        let segments = parse_ansi(input);
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].text, "Red text");
        assert!(segments[0].styles.contains(&AnsiStyle::FgRed));
    }

    #[test]
    fn test_parse_bracket_format() {
        // Test format like [31m instead of \x1b[31m
        let input = "[31mRed[39m";
        let segments = parse_ansi(input);
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].text, "Red");
        assert!(segments[0].styles.contains(&AnsiStyle::FgRed));
    }

    #[test]
    fn test_parse_mixed_styles() {
        let input = "[2mexpect([22m[31mreceived[39m[2m).[22m";
        let segments = parse_ansi(input);

        // Should have multiple segments with different styles
        assert!(!segments.is_empty());
    }

    #[test]
    fn test_css_classes() {
        let segment = AnsiSegment::new("test".to_string(), vec![AnsiStyle::Bold, AnsiStyle::FgRed]);
        let classes = segment.css_classes();
        assert!(classes.contains("ansi-bold"));
        assert!(classes.contains("ansi-red"));
    }
}
