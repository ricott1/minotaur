use image::{Rgb, Rgba, RgbaImage};
use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
};

pub fn img_to_lines<'a>(
    img: &RgbaImage,
    image_char_overrides: (Vec<(u32, u32)>, char),
) -> Vec<Line<'a>> {
    let mut lines: Vec<Line> = vec![];
    let width = img.width();
    let height = img.height();

    for y in (0..height - 1).step_by(2) {
        let mut line: Vec<Span> = vec![];

        'pixels: for x in 0..width {
            let &top_pixel = img.get_pixel(x, y);
            let &btm_pixel = img.get_pixel(x, y + 1);

            // check for overrides.
            // Maze valid, visible, not occupied positions (alpha < 255)
            let override_positions = &image_char_overrides.0;
            let override_char = image_char_overrides.1;
            if override_positions.contains(&(x, y)) && override_positions.contains(&(x, y + 1)) {
                let [_, _, _, ta] = top_pixel.0;

                let [_, _, _, ba] = btm_pixel.0;
                if ta < 255 && ba < 255 {
                    // Convert rgba to rgb
                    line.push(Span::styled(
                        override_char.to_string(),
                        Style::default().fg(Color::Rgb(ta, ta, ta)),
                    ));
                    continue 'pixels;
                }
            }

            // both pixels are transparent
            if top_pixel[3] == 0 && btm_pixel[3] == 0 {
                line.push(Span::raw(" "));
                continue;
            }

            // render top pixel
            if top_pixel[3] > 0 && btm_pixel[3] == 0 {
                let color = top_pixel.to_color();
                line.push(Span::styled("▀", Style::default().fg(color)));
                continue;
            }

            // render bottom pixel
            if top_pixel[3] == 0 && btm_pixel[3] > 0 {
                let color = btm_pixel.to_color();
                line.push(Span::styled("▄", Style::default().fg(color)));
                continue;
            }

            // render both pixels
            let fg_color = top_pixel.to_color();
            let bg_color = btm_pixel.to_color();
            line.push(Span::styled(
                "▀",
                Style::default().fg(fg_color).bg(bg_color),
            ));
        }
        lines.push(Line::from(line));
    }
    // append last line if height is odd
    if height % 2 == 1 {
        let mut line: Vec<Span> = vec![];
        for x in 0..width {
            let top_pixel = img.get_pixel(x, height - 1);
            if top_pixel[3] == 0 {
                line.push(Span::raw(" "));
                continue;
            }
            let [r, g, b, _] = top_pixel.0;
            let color = Color::Rgb(r, g, b);
            line.push(Span::styled("▀", Style::default().fg(color)));
        }
        lines.push(Line::from(line));
    }

    lines
}

pub trait RataColor {
    fn to_color(&self) -> Color;
}

impl RataColor for Rgb<u8> {
    fn to_color(&self) -> Color {
        let [r, g, b] = self.0;
        Color::Rgb(r, g, b)
    }
}

impl RataColor for Rgba<u8> {
    fn to_color(&self) -> Color {
        let [r, g, b, a] = self.0;
        let alpha = (a as f32 / 255.0) as f32;
        let r = (r as f32 * alpha) as u8;
        let g = (g as f32 * alpha) as u8;
        let b = (b as f32 * alpha) as u8;

        Color::Rgb(r, g, b)
    }
}
