use std::io::stdout;
use std::process::exit;
use std::thread::sleep;
use std::time::Duration;
use termimad::crossterm::{cursor, ExecutableCommand};
use transprompt::utils::printing::IncrementalMarkdownPrinter;

const MARKDOWN_TEXT: &str = r#"
# Hello

This is inline code `print("hello")`.

A super long line to "tttttttttttttttttttttttttttttttttttttttttttttttttttttttttttttttttest" the hard wrapping.

```python
print("hello")
```

Here ends it."#;


fn main() {
    ctrlc::set_handler(move || {
        // to avoid missing cursor when Ctrl-C is pressed
        stdout().execute(cursor::Show).unwrap();
        exit(0);
    }).expect("Error setting Ctrl-C handler");
    let mut printer = IncrementalMarkdownPrinter::default();
    printer.activate(true);
    let mut generator = MARKDOWN_TEXT.chars();
    while let Some(chunk) = generator.next() {
        printer.push_and_print(&chunk.to_string());
        sleep(Duration::from_millis(50));
    }
    printer.deactivate();
}