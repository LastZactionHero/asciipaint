use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{read, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    style::{Print, SetAttribute},
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
};
use std::fs::File;
use std::io::{stdout, Result, Write}; // Use std::io::Result here

// Constants for box drawing characters
const BOX_TOP_LEFT: char = '╔';
const BOX_TOP_RIGHT: char = '╗';
const BOX_BOTTOM_LEFT: char = '╚';
const BOX_BOTTOM_RIGHT: char = '╝';
const BOX_HORIZONTAL: char = '═';
const BOX_VERTICAL: char = '║';

// Modes
#[derive(PartialEq, Clone, Copy, Debug)] // Add Debug here
enum Mode {
    Move,
    Select,
    BoxDraw,
    PencilDraw,
    LineDraw,
}

// Layer data structure
struct Layer {
    data: Vec<Vec<char>>,
    visible: bool,
}

// App state
struct AppState {
    width: usize,
    height: usize,
    cursor_x: usize,
    cursor_y: usize,
    mode: Mode,
    layers: Vec<Layer>,
    active_layer: usize,
    copy_buffer: Option<Vec<Vec<char>>>,
    select_start_x: usize,
    select_start_y: usize,
    line_start_x: usize,
    line_start_y: usize,
}

impl AppState {
    fn new(width: usize, height: usize) -> Self {
        let layers = (0..10)
            .map(|_| Layer {
                data: vec![vec![' '; width]; height],
                visible: true,
            })
            .collect();

        AppState {
            width,
            height,
            cursor_x: 0,
            cursor_y: 0,
            mode: Mode::Move,
            layers,
            active_layer: 0,
            copy_buffer: None,
            select_start_x: 0,
            select_start_y: 0,
            line_start_x: 0,
            line_start_y: 0,
        }
    }

    // Function to draw a box on the active layer
    fn draw_box(&mut self, x1: usize, y1: usize, x2: usize, y2: usize) {
        if x1 == x2 || y1 == y2 {
            return; // Invalid box
        }

        let (x1, x2) = (x1.min(x2), x1.max(x2));
        let (y1, y2) = (y1.min(y2), y1.max(y2));

        let layer = &mut self.layers[self.active_layer];

        for x in (x1 + 1)..x2 {
            layer.data[y1][x] = BOX_HORIZONTAL;
            layer.data[y2][x] = BOX_HORIZONTAL;
        }
        for y in (y1 + 1)..y2 {
            layer.data[y][x1] = BOX_VERTICAL;
            layer.data[y][x2] = BOX_VERTICAL;
        }
        layer.data[y1][x1] = BOX_TOP_LEFT;
        layer.data[y1][x2] = BOX_TOP_RIGHT;
        layer.data[y2][x1] = BOX_BOTTOM_LEFT;
        layer.data[y2][x2] = BOX_BOTTOM_RIGHT;
    }

    // Function to draw a line on the active layer
    fn draw_line(&mut self, x1: usize, y1: usize, x2: usize, y2: usize, ch: char) {
        let dx = (x2 as isize - x1 as isize).abs();
        let dy = (y2 as isize - y1 as isize).abs();
        let sx = if x1 < x2 { 1 } else { -1 };
        let sy = if y1 < y2 { 1 } else { -1 };
        let mut err = dx - dy;

        let mut x = x1 as isize;
        let mut y = y1 as isize;

        while x != x2 as isize || y != y2 as isize {
            if x >= 0 && x < self.width as isize && y >= 0 && y < self.height as isize {
                self.layers[self.active_layer].data[y as usize][x as usize] = ch;
            }

            let e2 = 2 * err;
            if e2 > -dy {
                err -= dy;
                x += sx;
            }
            if e2 < dx {
                err += dx;
                y += sy;
            }
        }
        if x >= 0 && x < self.width as isize && y >= 0 && y < self.height as isize {
            self.layers[self.active_layer].data[y as usize][x as usize] = ch;
        }
    }

    // Function to redraw the entire screen
    fn redraw(&self, stdout: &mut std::io::Stdout) -> Result<()> {
        execute!(stdout, Hide, Clear(ClearType::All), MoveTo(0, 0))?;

        let mut combined_layer = vec![vec![' '; self.width]; self.height];

        for (i, layer) in self.layers.iter().enumerate() {
            if layer.visible {
                for y in 0..self.height {
                    for x in 0..self.width {
                        if layer.data[y][x] != ' ' {
                            combined_layer[y][x] = layer.data[y][x];
                        }
                    }
                }
            }
        }

        for y in 0..self.height {
            for x in 0..self.width {
                if self.mode == Mode::Select
                    && x >= self.select_start_x.min(self.cursor_x)
                    && x <= self.select_start_x.max(self.cursor_x)
                    && y >= self.select_start_y.min(self.cursor_y)
                    && y <= self.select_start_y.max(self.cursor_y)
                {
                    // Invert colors for selection
                    execute!(
                        stdout,
                        SetAttribute(crossterm::style::Attribute::Reverse),
                        Print(combined_layer[y][x]),
                        SetAttribute(crossterm::style::Attribute::Reset),
                    )?;
                } else {
                    execute!(stdout, Print(combined_layer[y][x]))?;
                }
            }
            execute!(stdout, Print("\r\n"))?;
        }

        // Display status bar
        execute!(
            stdout,
            MoveTo(0, (self.height) as u16),
            Print(format!(
                "Mode: {:?} | Layer: {}/10 | Cursor: ({}, {})",
                self.mode,
                self.active_layer + 1,
                self.cursor_x,
                self.cursor_y
            )),
        )?;

        execute!(
            stdout,
            MoveTo(self.cursor_x as u16, self.cursor_y as u16),
            Show
        )?;
        stdout.flush()?;
        Ok(())
    }

    fn save_layers(&self) -> Result<()> {
        for (i, layer) in self.layers.iter().enumerate() {
            // Check if the layer has non-space content
            let has_content = layer.data.iter().any(|row| row.iter().any(|&c| c != ' '));

            if has_content {
                let filename = format!("./output_layer_{}.txt", i);
                let mut file = File::create(filename)?;
                for row in &layer.data {
                    for &ch in row {
                        write!(file, "{}", ch)?;
                    }
                    writeln!(file)?;
                }
            }
        }
        Ok(())
    }

    fn move_cursor(&mut self, dx: isize, dy: isize) {
        if self.mode == Mode::Move {
            self.cursor_x = (self.cursor_x as isize + dx).rem_euclid(self.width as isize) as usize;
            self.cursor_y = (self.cursor_y as isize + dy).rem_euclid(self.height as isize) as usize;
        } else if self.mode == Mode::Select {
            let new_x = (self.cursor_x as isize + dx)
                .max(0)
                .min(self.width as isize - 1) as usize;
            let new_y = (self.cursor_y as isize + dy)
                .max(0)
                .min(self.height as isize - 1) as usize;
            self.cursor_x = new_x;
            self.cursor_y = new_y;
        } else if self.mode == Mode::BoxDraw || self.mode == Mode::LineDraw {
            // Only update cursor position, no wrapping or selection logic
            self.cursor_x = (self.cursor_x as isize + dx)
                .max(0)
                .min(self.width as isize - 1) as usize;
            self.cursor_y = (self.cursor_y as isize + dy)
                .max(0)
                .min(self.height as isize - 1) as usize;
        } else if self.mode == Mode::PencilDraw {
            // Advance to next column, wrap to next line if needed
            self.cursor_x += 1;
            if self.cursor_x >= self.width {
                self.cursor_x = 0;
                self.cursor_y += 1;
                if self.cursor_y >= self.height {
                    self.cursor_y = 0; // Wrap to top
                }
            }
        }
    }

    fn handle_key_event(&mut self, key: KeyEvent, stdout: &mut std::io::Stdout) -> Result<()> {
        match self.mode {
            Mode::Move => match key.code {
                KeyCode::Char('m') | KeyCode::Char('M') => {
                    self.mode = Mode::Move;
                }
                KeyCode::Char('v') | KeyCode::Char('V') => {
                    self.mode = Mode::Select;
                    self.select_start_x = self.cursor_x;
                    self.select_start_y = self.cursor_y;
                }
                KeyCode::Char('q') | KeyCode::Char('Q') => {
                    self.mode = Mode::BoxDraw;
                    self.select_start_x = self.cursor_x;
                    self.select_start_y = self.cursor_y;
                }
                KeyCode::Char('w') | KeyCode::Char('W') => {
                    self.mode = Mode::PencilDraw;
                }
                KeyCode::Char('e') | KeyCode::Char('E') => {
                    self.mode = Mode::LineDraw;
                    self.line_start_x = self.cursor_x;
                    self.line_start_y = self.cursor_y;
                }
                KeyCode::Char('p') | KeyCode::Char('P') => {
                    if let Some(buffer) = &self.copy_buffer {
                        let buffer_height = buffer.len();
                        let buffer_width = if buffer_height > 0 {
                            buffer[0].len()
                        } else {
                            0
                        };

                        for dy in 0..buffer_height {
                            for dx in 0..buffer_width {
                                let x = self.cursor_x + dx;
                                let y = self.cursor_y + dy;
                                if x < self.width && y < self.height {
                                    self.layers[self.active_layer].data[y][x] = buffer[dy][dx];
                                }
                            }
                        }
                    }
                }
                KeyCode::Char('+') => {
                    if self.active_layer < 9 {
                        self.active_layer += 1;
                    }
                }
                KeyCode::Char('-') => {
                    if self.active_layer > 0 {
                        self.active_layer -= 1;
                    }
                }
                KeyCode::Char('h') | KeyCode::Char('H') => {
                    self.layers[self.active_layer].visible =
                        !self.layers[self.active_layer].visible;
                }
                KeyCode::Up => self.move_cursor(0, -1),
                KeyCode::Down => self.move_cursor(0, 1),
                KeyCode::Left => self.move_cursor(-1, 0),
                KeyCode::Right => self.move_cursor(1, 0),
                _ => {}
            },
            Mode::Select => match key.code {
                KeyCode::Char('y') | KeyCode::Char('Y') => {
                    let x1 = self.select_start_x.min(self.cursor_x);
                    let y1 = self.select_start_y.min(self.cursor_y);
                    let x2 = self.select_start_x.max(self.cursor_x);
                    let y2 = self.select_start_y.max(self.cursor_y);

                    let mut yanked_data = Vec::new();
                    for y in y1..=y2 {
                        let row = self.layers[self.active_layer].data[y][x1..=x2].to_vec();
                        yanked_data.push(row);
                    }
                    self.copy_buffer = Some(yanked_data);
                    self.mode = Mode::Move;
                }
                KeyCode::Char('c') | KeyCode::Char('C') => {
                    let x1 = self.select_start_x.min(self.cursor_x);
                    let y1 = self.select_start_y.min(self.cursor_y);
                    let x2 = self.select_start_x.max(self.cursor_x);
                    let y2 = self.select_start_y.max(self.cursor_y);

                    let mut yanked_data = Vec::new();
                    for y in y1..=y2 {
                        let mut row = self.layers[self.active_layer].data[y][x1..=x2].to_vec();
                        yanked_data.push(row.clone());
                        // Clear the selected area in the current layer
                        for x in 0..row.len() {
                            row[x] = ' ';
                        }
                        self.layers[self.active_layer].data[y][x1..=x2].copy_from_slice(&row);
                    }
                    self.copy_buffer = Some(yanked_data);
                    self.mode = Mode::Move;
                }
                KeyCode::Esc => {
                    self.mode = Mode::Move;
                }
                KeyCode::Up => self.move_cursor(0, -1),
                KeyCode::Down => self.move_cursor(0, 1),
                KeyCode::Left => self.move_cursor(-1, 0),
                KeyCode::Right => self.move_cursor(1, 0),
                _ => {}
            },
            Mode::BoxDraw => match key.code {
                KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Enter => {
                    self.draw_box(
                        self.select_start_x,
                        self.select_start_y,
                        self.cursor_x,
                        self.cursor_y,
                    );
                    self.mode = Mode::Move;
                }
                KeyCode::Esc => {
                    self.mode = Mode::Move;
                }
                KeyCode::Up => self.move_cursor(0, -1),
                KeyCode::Down => self.move_cursor(0, 1),
                KeyCode::Left => self.move_cursor(-1, 0),
                KeyCode::Right => self.move_cursor(1, 0),
                _ => {}
            },
            Mode::PencilDraw => match key.code {
                KeyCode::Esc => {
                    self.mode = Mode::Move;
                }
                KeyCode::Char(c) => {
                    self.layers[self.active_layer].data[self.cursor_y][self.cursor_x] = c;
                    self.move_cursor(1, 0);
                    self.redraw(stdout)?; // Redraw after moving the cursor
                }
                _ => {}
            },
            Mode::LineDraw => match key.code {
                KeyCode::Esc => {
                    self.mode = Mode::Move;
                }
                KeyCode::Char(c) => {
                    self.draw_line(
                        self.line_start_x,
                        self.line_start_y,
                        self.cursor_x,
                        self.cursor_y,
                        c,
                    );
                    self.mode = Mode::Move;
                }
                KeyCode::Up => self.move_cursor(0, -1),
                KeyCode::Down => self.move_cursor(0, 1),
                KeyCode::Left => self.move_cursor(-1, 0),
                KeyCode::Right => self.move_cursor(1, 0),
                _ => {}
            },
        }

        // Handle Ctrl+S for saving
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('s') {
            self.save_layers()?;
        }

        self.redraw(stdout)?;
        Ok(())
    }

    fn load_layers(&mut self) -> Result<()> {
        for i in 0..10 {
            let filename = format!("./output_layer_{}.txt", i);
            if let Ok(lines) = std::fs::read_to_string(&filename) {
                let layer_data: Vec<Vec<char>> =
                    lines.lines().map(|line| line.chars().collect()).collect();

                // Validate layer data dimensions
                if layer_data.len() > self.height
                    || layer_data.iter().any(|row| row.len() > self.width)
                {
                    eprintln!(
                        "Warning: Layer {} data dimensions exceed canvas size. Skipping.",
                        i
                    );
                    continue;
                }

                // Copy data to layer
                for (y, row) in layer_data.iter().enumerate() {
                    for (x, &ch) in row.iter().enumerate() {
                        self.layers[i].data[y][x] = ch;
                    }
                }
            }
        }
        Ok(())
    }
}

fn main() -> Result<()> {
    // Handle command line arguments for width and height
    let args: Vec<String> = std::env::args().collect();
    let width = if args.len() > 1 {
        args[1].parse().unwrap_or(80)
    } else {
        80
    };
    let height = if args.len() > 2 {
        args[2].parse().unwrap_or(24)
    } else {
        24
    };

    enable_raw_mode()?;
    let mut stdout = stdout();
    let mut app_state = AppState::new(width, height);
    app_state.load_layers()?;

    app_state.redraw(&mut stdout)?;

    loop {
        if let Event::Key(key_event) = read()? {
            app_state.handle_key_event(key_event, &mut stdout)?;

            if key_event.modifiers.contains(KeyModifiers::CONTROL)
                && key_event.code == KeyCode::Char('c')
            {
                break;
            }
        }
    }

    disable_raw_mode()?;
    execute!(stdout, Show)?;

    Ok(())
}
