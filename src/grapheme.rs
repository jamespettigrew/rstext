use crate::text_buffer::line::Line;
use crate::window::Window;

#[derive(Debug, Eq, PartialEq)]
pub struct Grapheme {
    pub content: String,
    pub is_escaped: bool,
}

pub fn visible_in_window(graphemes: &Vec<Grapheme>, window: &Window) -> Vec<Grapheme> {
    if graphemes.is_empty() {
        return vec![];
    }

    let mut visible_graphemes = Vec::new();
    let mut total_width_count = 0usize;
    let mut width_count_at_first = 0usize;
    for grapheme in graphemes.iter() {
        if grapheme.len() + total_width_count < window.horizontal_offset {
            total_width_count += grapheme.len();
            continue;
        }

        if total_width_count >= window.right() {
            break;
        }

        if visible_graphemes.is_empty() {
            width_count_at_first = total_width_count;
        }

        total_width_count += grapheme.len();
        visible_graphemes.push(Grapheme {
            content: grapheme.content.clone(),
            is_escaped: grapheme.is_escaped,
        });
    }

    let first_grapheme = match visible_graphemes.first_mut() {
        Some(g) => g,
        None => return visible_graphemes,
    };
    let trim_count = window.horizontal_offset - width_count_at_first;
    // Trim characters off front of first grapheme if horizontal offset sits within
    let trimmed_content = first_grapheme
        .content
        .chars()
        .skip(trim_count)
        .collect::<String>();
    first_grapheme.content = trimmed_content;
    if first_grapheme.content.is_empty() {
        visible_graphemes.remove(0);
    }

    // Truncate last grapheme if window ends within
    let width_of_visible = total_width_count - width_count_at_first - trim_count;
    if width_of_visible > window.width as usize {
        let last_grapheme = match visible_graphemes.last_mut() {
            Some(g) => g,
            None => return visible_graphemes,
        };

        let trim_count = width_of_visible - window.width as usize;
        let trimmed_content = last_grapheme
            .content
            .chars()
            .take(last_grapheme.content.chars().count() - trim_count)
            .collect::<String>();
        last_grapheme.content = trimmed_content;

        if last_grapheme.content.is_empty() {
            visible_graphemes.pop();
        }
    }

    visible_graphemes
}

impl Grapheme {
    // No robust way that I know of to determine the visual width of a grapheme (cluster).
    // Instead, any unicode characters beyond latin-1 set will be escaped to angle bracket form.
    pub fn from(ch: char) -> Grapheme {
        match ch {
            ch if ch < 'ǿ' => Grapheme {
                content: ch.to_string(),
                is_escaped: false,
            },
            _ => {
                let unicode = ch
                    .escape_unicode()
                    .skip(3)
                    .take_while(|c| *c != '}')
                    .collect::<String>();
                let formatted = format!("<{}>", unicode);

                Grapheme {
                    content: formatted,
                    is_escaped: true,
                }
            }
        }
    }

    pub fn from_line(line: &Line) -> Vec<Grapheme> {
        let mut graphemes = vec![];
        for ch in line.characters.iter() {
            graphemes.push(Grapheme::from(*ch));
        }

        graphemes
    }

    pub fn len(&self) -> usize {
        self.content.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_visible_in_spacious_window() {
        let graphemes = &vec![
            Grapheme {
                content: String::from("a"),
                is_escaped: false,
            },
            Grapheme {
                content: String::from("b"),
                is_escaped: false,
            },
            Grapheme {
                content: String::from("c"),
                is_escaped: false,
            },
            Grapheme {
                content: String::from("d"),
                is_escaped: false,
            },
            Grapheme {
                content: String::from("e"),
                is_escaped: false,
            },
            Grapheme {
                content: String::from("f"),
                is_escaped: false,
            },
            Grapheme {
                content: String::from("g"),
                is_escaped: false,
            },
        ];

        let window = &mut Window {
            height: 5,
            width: 9,
            horizontal_offset: 0,
            vertical_offset: 0,
        };

        let visible_graphemes = super::visible_in_window(graphemes, window);
        let expected_visible_graphemes = vec![
            Grapheme {
                content: String::from("a"),
                is_escaped: false,
            },
            Grapheme {
                content: String::from("b"),
                is_escaped: false,
            },
            Grapheme {
                content: String::from("c"),
                is_escaped: false,
            },
            Grapheme {
                content: String::from("d"),
                is_escaped: false,
            },
            Grapheme {
                content: String::from("e"),
                is_escaped: false,
            },
            Grapheme {
                content: String::from("f"),
                is_escaped: false,
            },
            Grapheme {
                content: String::from("g"),
                is_escaped: false,
            },
        ];
        assert_eq!(visible_graphemes, expected_visible_graphemes);
    }

    #[test]
    fn end_trimmed_when_window_narrow_width() {
        let graphemes = &vec![
            Grapheme {
                content: String::from("a"),
                is_escaped: false,
            },
            Grapheme {
                content: String::from("b"),
                is_escaped: false,
            },
            Grapheme {
                content: String::from("c"),
                is_escaped: false,
            },
            Grapheme {
                content: String::from("d"),
                is_escaped: false,
            },
            Grapheme {
                content: String::from("e"),
                is_escaped: false,
            },
            Grapheme {
                content: String::from("f"),
                is_escaped: false,
            },
            Grapheme {
                content: String::from("g"),
                is_escaped: false,
            },
        ];

        let window = &mut Window {
            height: 5,
            width: 9,
            horizontal_offset: 0,
            vertical_offset: 0,
        };

        window.width = 6;
        let visible_graphemes = super::visible_in_window(graphemes, window);
        let expected_visible_graphemes = vec![
            Grapheme {
                content: String::from("a"),
                is_escaped: false,
            },
            Grapheme {
                content: String::from("b"),
                is_escaped: false,
            },
            Grapheme {
                content: String::from("c"),
                is_escaped: false,
            },
            Grapheme {
                content: String::from("d"),
                is_escaped: false,
            },
            Grapheme {
                content: String::from("e"),
                is_escaped: false,
            },
            Grapheme {
                content: String::from("f"),
                is_escaped: false,
            },
        ];
        assert_eq!(visible_graphemes, expected_visible_graphemes);
    }

    #[test]
    fn start_trimmed_when_horizontal_offset() {
        let graphemes = &vec![
            Grapheme {
                content: String::from("a"),
                is_escaped: false,
            },
            Grapheme {
                content: String::from("b"),
                is_escaped: false,
            },
            Grapheme {
                content: String::from("c"),
                is_escaped: false,
            },
            Grapheme {
                content: String::from("d"),
                is_escaped: false,
            },
            Grapheme {
                content: String::from("e"),
                is_escaped: false,
            },
            Grapheme {
                content: String::from("f"),
                is_escaped: false,
            },
            Grapheme {
                content: String::from("g"),
                is_escaped: false,
            },
        ];

        let window = &mut Window {
            height: 5,
            width: 7,
            horizontal_offset: 3,
            vertical_offset: 0,
        };

        window.width = 9;
        let visible_graphemes = super::visible_in_window(graphemes, window);
        let expected_visible_graphemes = vec![
            Grapheme {
                content: String::from("d"),
                is_escaped: false,
            },
            Grapheme {
                content: String::from("e"),
                is_escaped: false,
            },
            Grapheme {
                content: String::from("f"),
                is_escaped: false,
            },
            Grapheme {
                content: String::from("g"),
                is_escaped: false,
            },
        ];
        assert_eq!(visible_graphemes, expected_visible_graphemes);
    }

    #[test]
    fn none_visible_when_large_horizontal_offset() {
        let graphemes = &vec![
            Grapheme {
                content: String::from("a"),
                is_escaped: false,
            },
            Grapheme {
                content: String::from("b"),
                is_escaped: false,
            },
            Grapheme {
                content: String::from("c"),
                is_escaped: false,
            },
            Grapheme {
                content: String::from("d"),
                is_escaped: false,
            },
            Grapheme {
                content: String::from("e"),
                is_escaped: false,
            },
            Grapheme {
                content: String::from("f"),
                is_escaped: false,
            },
            Grapheme {
                content: String::from("g"),
                is_escaped: false,
            },
        ];

        let window = &mut Window {
            height: 5,
            width: 9,
            horizontal_offset: 10,
            vertical_offset: 0,
        };

        window.width = 9;
        let visible_graphemes = super::visible_in_window(graphemes, window);
        let expected_visible_graphemes = vec![];
        assert_eq!(visible_graphemes, expected_visible_graphemes);
    }

    #[test]
    fn trimmed_escaped_graphemes() {
        let window = &mut Window {
            height: 5,
            width: 6,
            horizontal_offset: 0,
            vertical_offset: 0,
        };

        let graphemes = &String::from("👨‍👩‍👧 ")
            .chars()
            .map(|c| Grapheme::from(c))
            .collect::<Vec<Grapheme>>();

        window.horizontal_offset = 0;
        let visible_graphemes = super::visible_in_window(&graphemes, window);
        let expected_visible_graphemes = vec![Grapheme {
            content: String::from("<1f468"),
            is_escaped: true,
        }];
        assert_eq!(visible_graphemes, expected_visible_graphemes);

        window.width = 9;
        window.horizontal_offset = 3;
        let visible_graphemes = super::visible_in_window(&graphemes, window);
        let expected_visible_graphemes = vec![
            Grapheme {
                content: String::from("468>"),
                is_escaped: true,
            },
            Grapheme {
                content: String::from("<200d"),
                is_escaped: true,
            },
        ];
        assert_eq!(visible_graphemes, expected_visible_graphemes);
    }

    #[test]
    fn trimmed_when_window_narrow_width_and_horizontal_offset() {
        let graphemes = &vec![
            Grapheme {
                content: String::from("a"),
                is_escaped: false,
            },
            Grapheme {
                content: String::from("b"),
                is_escaped: false,
            },
            Grapheme {
                content: String::from("c"),
                is_escaped: false,
            },
            Grapheme {
                content: String::from("d"),
                is_escaped: false,
            },
            Grapheme {
                content: String::from("e"),
                is_escaped: false,
            },
            Grapheme {
                content: String::from("f"),
                is_escaped: false,
            },
            Grapheme {
                content: String::from("g"),
                is_escaped: false,
            },
        ];

        let window = &mut Window {
            height: 5,
            width: 2,
            horizontal_offset: 3,
            vertical_offset: 0,
        };

        let visible_graphemes = super::visible_in_window(graphemes, window);
        let expected_visible_graphemes = vec![
            Grapheme {
                content: String::from("d"),
                is_escaped: false,
            },
            Grapheme {
                content: String::from("e"),
                is_escaped: false,
            },
        ];
        assert_eq!(visible_graphemes, expected_visible_graphemes);
    }
}
