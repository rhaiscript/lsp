#![allow(dead_code)]
use crate::{ring::RingBuffer, Options};
use std::{
    cmp,
    collections::VecDeque,
    io::{self, Write},
    iter,
};

const MIN_SPACE: isize = 60;

#[derive(Clone, Copy, PartialEq)]
pub enum Breaks {
    Consistent,
    Inconsistent,
}

#[derive(Clone, Copy, Default)]
pub struct BreakToken {
    pub offset: isize,
    pub blank_space: usize,
    pub pre_break: Option<char>,
    pub post_break: Option<char>,
    pub no_break: Option<char>,
    pub if_nonempty: bool,
    pub never_break: bool,
}

#[derive(Clone, Copy)]
pub struct BeginToken {
    pub offset: isize,
    pub breaks: Breaks,
}

#[derive(Clone)]
pub enum Token {
    String(&'static str),
    Break(BreakToken),
    Begin(BeginToken),
    End,
}

#[derive(Copy, Clone)]
enum PrintFrame {
    Fits(Breaks),
    Broken(usize, Breaks),
}

pub const SIZE_INFINITY: isize = 0xffff;

pub struct Formatter<W: Write> {
    pub(crate) options: Options,
    pub(crate) out: W,
    // Number of spaces left on line
    space: isize,
    // Ring-buffer of tokens and calculated sizes
    buf: RingBuffer<BufEntry>,
    // Total size of tokens already printed
    left_total: isize,
    // Total size of tokens enqueued, including printed and not yet printed
    right_total: isize,
    // Holds the ring-buffer index of the Begin that started the current block,
    // possibly with the most recent Break after that Begin (if there is any) on
    // top of it. Values are pushed and popped on the back of the queue using it
    // like stack, and elsewhere old values are popped from the front of the
    // queue as they become irrelevant due to the primary ring-buffer advancing.
    scan_stack: VecDeque<usize>,
    // Stack of blocks-in-progress being flushed by print
    print_stack: Vec<PrintFrame>,
    // Level of indentation of current line
    indent: usize,
    // Buffered indentation to avoid writing trailing whitespace
    pending_indentation: usize,
    // Spaces are separate from indentation,
    // as indentation can be tabs or any number of
    // spaces.
    pending_spaces: usize,
}

#[derive(Clone)]
struct BufEntry {
    token: Token,
    size: isize,
}

impl<W: Write> Formatter<W> {
    pub fn new(out: W) -> Self {
        Self::new_with_options(out, Options::default())
    }

    pub fn new_with_options(out: W, options: Options) -> Self {
        Formatter {
            out,
            space: options.max_width as _,
            options,
            buf: RingBuffer::new(),
            left_total: 0,
            right_total: 0,
            scan_stack: VecDeque::new(),
            print_stack: Vec::new(),
            indent: 0,
            pending_indentation: 0,
            pending_spaces: 0,
        }
    }

    pub(crate) fn eof(mut self) -> io::Result<()> {
        if !self.scan_stack.is_empty() {
            self.check_stack(0);
            self.advance_left()?;
        }

        Ok(())
    }

    pub(crate) fn scan_begin(&mut self, token: BeginToken) {
        if self.scan_stack.is_empty() {
            self.left_total = 1;
            self.right_total = 1;
            self.buf.clear();
        }
        let right = self.buf.push(BufEntry {
            token: Token::Begin(token),
            size: -self.right_total,
        });
        self.scan_stack.push_back(right);
    }

    pub(crate) fn scan_end(&mut self) {
        if self.scan_stack.is_empty() {
            self.print_end();
        } else {
            if !self.buf.is_empty() {
                if let Token::Break(break_token) = self.buf.last().token {
                    if self.buf.len() >= 2 {
                        if let Token::Begin(_) = self.buf.second_last().token {
                            self.buf.pop_last();
                            self.buf.pop_last();
                            self.scan_stack.pop_back();
                            self.scan_stack.pop_back();
                            self.right_total -= break_token.blank_space as isize;
                            return;
                        }
                    }
                    if break_token.if_nonempty {
                        self.buf.pop_last();
                        self.scan_stack.pop_back();
                        self.right_total -= break_token.blank_space as isize;
                    }
                }
            }
            let right = self.buf.push(BufEntry {
                token: Token::End,
                size: -1,
            });
            self.scan_stack.push_back(right);
        }
    }

    pub(crate) fn scan_break(&mut self, token: BreakToken) {
        if self.scan_stack.is_empty() {
            self.left_total = 1;
            self.right_total = 1;
            self.buf.clear();
        } else {
            self.check_stack(0);
        }
        let right = self.buf.push(BufEntry {
            token: Token::Break(token),
            size: -self.right_total,
        });
        self.scan_stack.push_back(right);
        self.right_total += token.blank_space as isize;
    }

    pub(crate) fn scan_string(&mut self, string: &'static str) -> io::Result<()> {
        if self.scan_stack.is_empty() {
            self.print_string(string)?;
        } else {
            let len = string.len() as isize;
            self.buf.push(BufEntry {
                token: Token::String(string),
                size: len,
            });
            self.right_total += len;
            self.check_stream()?;
        }

        Ok(())
    }

    pub(crate) fn offset(&mut self, offset: isize) {
        match &mut self.buf.last_mut().token {
            Token::Break(token) => token.offset += offset,
            Token::Begin(_) => {}
            Token::String(_) | Token::End => unreachable!(),
        }
    }

    pub(crate) fn end_with_max_width(&mut self, max: isize) {
        let mut depth = 1;
        for &index in self.scan_stack.iter().rev() {
            let entry = &self.buf[index];
            match entry.token {
                Token::Begin(_) => {
                    depth -= 1;
                    if depth == 0 {
                        if entry.size < 0 {
                            let actual_width = entry.size + self.right_total;
                            if actual_width > max {
                                self.buf.push(BufEntry {
                                    token: Token::String(""),
                                    size: SIZE_INFINITY,
                                });
                                self.right_total += SIZE_INFINITY;
                            }
                        }
                        break;
                    }
                }
                Token::End => depth += 1,
                Token::Break(_) => {}
                Token::String(_) => unreachable!(),
            }
        }
        self.scan_end();
    }

    fn check_stream(&mut self) -> io::Result<()> {
        while self.right_total - self.left_total > self.space {
            if *self.scan_stack.front().unwrap() == self.buf.index_of_first() {
                self.scan_stack.pop_front().unwrap();
                self.buf.first_mut().size = SIZE_INFINITY;
            }

            self.advance_left()?;

            if self.buf.is_empty() {
                break;
            }
        }

        Ok(())
    }

    fn advance_left(&mut self) -> io::Result<()> {
        while self.buf.first().size >= 0 {
            let left = self.buf.pop_first();

            match left.token {
                Token::String(string) => {
                    self.left_total += left.size;
                    self.print_string(string)?;
                }
                Token::Break(token) => {
                    self.left_total += token.blank_space as isize;
                    self.print_break(token, left.size)?;
                }
                Token::Begin(token) => self.print_begin(token, left.size),
                Token::End => self.print_end(),
            }

            if self.buf.is_empty() {
                break;
            }
        }

        Ok(())
    }

    fn check_stack(&mut self, mut depth: usize) {
        while let Some(&index) = self.scan_stack.back() {
            let mut entry = &mut self.buf[index];
            match entry.token {
                Token::Begin(_) => {
                    if depth == 0 {
                        break;
                    }
                    self.scan_stack.pop_back().unwrap();
                    entry.size += self.right_total;
                    depth -= 1;
                }
                Token::End => {
                    self.scan_stack.pop_back().unwrap();
                    entry.size = 1;
                    depth += 1;
                }
                Token::Break(_) => {
                    self.scan_stack.pop_back().unwrap();
                    entry.size += self.right_total;
                    if depth == 0 {
                        break;
                    }
                }
                Token::String(_) => unreachable!(),
            }
        }
    }

    fn get_top(&self) -> PrintFrame {
        const OUTER: PrintFrame = PrintFrame::Broken(0, Breaks::Inconsistent);
        self.print_stack.last().map_or(OUTER, PrintFrame::clone)
    }

    fn print_begin(&mut self, token: BeginToken, size: isize) {
        if size > self.space {
            self.print_stack
                .push(PrintFrame::Broken(self.indent, token.breaks));
            self.indent = usize::try_from(self.indent as isize + token.offset).unwrap();
        } else {
            self.print_stack.push(PrintFrame::Fits(token.breaks));
        }
    }

    fn print_end(&mut self) {
        match self.print_stack.pop().unwrap() {
            PrintFrame::Broken(indent, breaks) => {
                self.indent = indent;
                breaks
            }
            PrintFrame::Fits(breaks) => breaks,
        };
    }

    fn print_break(&mut self, token: BreakToken, size: isize) -> io::Result<()> {
        let fits = token.never_break
            || match self.get_top() {
                PrintFrame::Fits(..) => true,
                PrintFrame::Broken(.., Breaks::Consistent) => false,
                PrintFrame::Broken(.., Breaks::Inconsistent) => size <= self.space,
            };
        if fits {
            self.pending_spaces += token.blank_space;
            self.space -= token.blank_space as isize;
            if let Some(no_break) = token.no_break {
                self.out.write_all(&[no_break as _])?;
                self.space -= no_break.len_utf8() as isize;
            }
        } else {
            if let Some(pre_break) = token.pre_break {
                self.print_indent()?;
                self.out.write_all(&[pre_break as _])?;
            }
            self.out.write_all(b"\n")?;
            let indent = self.indent as isize + token.offset;
            self.pending_indentation = usize::try_from(indent).unwrap();
            self.space = cmp::max(self.options.max_width as isize - indent, MIN_SPACE);
            if let Some(post_break) = token.post_break {
                self.print_indent()?;
                self.out.write_all(&[post_break as _])?;
                self.space -= post_break.len_utf8() as isize;
            }
        }

        Ok(())
    }

    fn print_string(&mut self, string: &'static str) -> io::Result<()> {
        self.print_indent()?;
        self.out.write_all(string.as_bytes())?;
        self.space -= string.len() as isize;
        Ok(())
    }

    fn print_indent(&mut self) -> io::Result<()> {
        for _ in 0..self.pending_indentation {
            self.out.write_all(self.options.indent_string.as_bytes())?;
        }

        for sp in iter::repeat(' ').take(self.pending_spaces) {
            self.out.write_all(&[sp as _])?;
        }

        self.pending_indentation = 0;
        self.pending_spaces = 0;
        Ok(())
    }
}
