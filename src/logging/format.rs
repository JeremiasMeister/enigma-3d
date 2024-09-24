#[macro_export]
macro_rules! smart_format {
    ($fmt:expr) => {{
        $fmt.to_string()
    }};
    ($fmt:expr, $($arg:expr),*) => {{
        use std::fmt::Write;
        let mut result = String::new();
        let mut fmt_parts = $fmt.split('{');

        // Handle the first part (before any format specifiers)
        if let Some(part) = fmt_parts.next() {
            result.push_str(part);
        }

        $(
            if let Some(part) = fmt_parts.next() {
                if let Some(end_brace) = part.find('}') {
                    let (format_spec, rest) = part.split_at(end_brace);
                    let format_spec = format_spec.trim();

                    match format_spec {
                        ":?" => write!(result, "{:?}", $arg),
                        ":#?" => write!(result, "{:#?}", $arg),
                        "" => write!(result, "{:?}", $arg),  // Use debug formatting by default
                        _ => write!(result, "{}", format_spec),  // For unsupported format specifiers, just write them as-is
                    }.unwrap();

                    result.push_str(&rest[1..]);  // Skip the closing brace
                } else {
                    result.push('{');
                    result.push_str(part);
                }
            }
        )*

        // Handle any remaining parts of the format string
        for part in fmt_parts {
            result.push('{');
            result.push_str(part);
        }

        result
    }};
}

// Example usage
#[cfg(test)]
mod tests {
    use uuid::Uuid;

    #[test]
    fn test_smart_format() {
        let id = Uuid::new_v4();
        let num = 42;
        let formatted = smart_format!("ID: {}, Number: {}", id, num);
        assert!(formatted.starts_with("ID: "));
        assert!(formatted.contains("Number: 42"));

        let text = "Hello";
        let formatted = smart_format!("{}, world!", text);
        assert_eq!(formatted, "\"Hello\", world!");  // Note the quotes due to debug formatting

        let vec = vec![Uuid::new_v4(), Uuid::new_v4()];
        let formatted = smart_format!("UUIDs: {:?}", vec);
        assert!(formatted.starts_with("UUIDs: ["));

        // Test with no arguments
        let formatted = smart_format!("No arguments");
        assert_eq!(formatted, "No arguments");
    }
}