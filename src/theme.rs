use ratatui::style::Color;

pub fn parse_hex_color(hex: &str) -> Color {
    let hex = hex.trim_start_matches('#');
    
    if hex.len() != 6 {
        return Color::White; // Fallback
    }
    
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(255);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(255);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(255);
    
    Color::Rgb(r, g, b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hex_color() {
        assert_eq!(parse_hex_color("#ff0000"), Color::Rgb(255, 0, 0));
        assert_eq!(parse_hex_color("00ff00"), Color::Rgb(0, 255, 0));
        assert_eq!(parse_hex_color("#0000ff"), Color::Rgb(0, 0, 255));
    }
}