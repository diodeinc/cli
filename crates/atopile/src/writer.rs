use std::io::{self, Write};

pub struct AtopileWriter<W: Write> {
    writer: W,
    indent_level: usize,
    indent_str: String,
    last_line_empty: bool,
}

#[allow(dead_code)]
impl<W: Write> AtopileWriter<W> {
    pub fn new(writer: W) -> Self {
        AtopileWriter {
            writer,
            indent_level: 0,
            indent_str: "    ".to_string(), // Default to 4 spaces
            last_line_empty: true,
        }
    }

    pub fn indent(&mut self) {
        self.indent_level += 1;
    }

    pub fn dedent(&mut self) {
        if self.indent_level > 0 {
            self.indent_level -= 1;
        }
    }

    pub fn write_line(&mut self, line: &str) -> io::Result<()> {
        if line.trim().is_empty() {
            self.last_line_empty = true;
        } else {
            self.last_line_empty = false;
            self.write_indentation()?;
        }

        writeln!(self.writer, "{}", line)
    }

    pub fn write_raw(&mut self, text: &str) -> io::Result<()> {
        write!(self.writer, "{}", text)
    }

    pub fn ensure_break(&mut self) -> io::Result<()> {
        if !self.last_line_empty {
            self.write_line("")?;
        }
        self.last_line_empty = true;
        Ok(())
    }

    pub fn start_block(&mut self, line: &str) -> io::Result<()> {
        self.write_line(line)?;
        self.indent();
        Ok(())
    }

    pub fn end_block(&mut self) -> io::Result<()> {
        self.dedent();
        Ok(())
    }

    fn write_indentation(&mut self) -> io::Result<()> {
        for _ in 0..self.indent_level {
            write!(self.writer, "{}", self.indent_str)?;
        }
        Ok(())
    }
}
