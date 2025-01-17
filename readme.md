# Readme

## Features

- Define draw space at startup by cli parameter
- Type at current cursor position
- Draw a line of the same character between two points
- Draw a box between two points, using ASCII box characters like ╔
- Paste over multiple rows and columns
- Draw to multiple layers 0-9

## Modes

- M: Move Mode
- V: Select Mode
- Y: Yank Selection
- C: Cut Selection
- P: Paste Selection
- Arrows: Move around
- Q: Box Tool
- W: Pencil Tool
- E: Line Tool
- +: Next Layer
- -: Previous Layer
- H: Toggle Layer Visibility

## File Functions

- Ctrl-S: Save (output_layer_N.txt)

## Move Mode
Pressing "M" at any time activates Move mode.

- In this mode, the arrow keys move the cursor around the draw buffer area.
- Cursor arrows will wrap back to the start/end column of the same row, or top/bottom row of the same column

## Select Mode
Pressing "V" at any time activates Select mode.

- In this mode, selection will begin at the cursor position
- Moving with arrow keys will begin selection in block mode
- Selection inits with the present character of the layer selected, so just a 1x1 selection buffer
- Unlike Move mode, cursor arrows at the edge row/column will not wrap to the other side of the buffer.

## Yank Selection
Pressing "Y" will copy the current block selection from the current layer into the copy buffer.

- This only functions in Select mode

## Cut Selection
// Pressing "C" wil cut the current block selection from the current layer into the copy buffer.

- This only function in Select mode
- The contents in the layer will be cleared to spaces.

## Paste Selection
Pressing "P" will paste the current block in the copy buffer to the current cursor position.

- This only functions in Move mode
- This only functions if a copy buffer is present
- If the copy block extends the draw buffer (e.g. pasting a block with 10 rows overflows at the right edge of the buffer and overflows by 5), the overflow in rows and columns will be quietly ignored.
- Pasting will not clear the copy buffer; it continues to persist.

## Arrows:

- In Move and Select mode, arrow keys move the cursor around the draw buffer.
- In Move mode, cursor will wrap to the other side of the buffer.
- In Select mode, the cursor will not wrap to the other side of the buffer.

## Box Tool

Pressing "B" at any time activates Box Draw mode

- Activated from Move mode
- Begin drawing a box at the current cursor position.
- Only a box >= 2x2 is valid. Before this, no box is drawn.
- Draw a box between the starting cursor and current cursor with ╔-like symbols
- Pressing "B" again finalized the current box, and returns to Move mode
- Pressing Esc exists box mode, returns to Move mode without drawing

## Pencil Mode

Pressing "W" enters Pencil Draw mode

- Activated from Move mode
- Begins at the current cursor position
- Any key typed is drawn to the current layer
- The cursor moves to the next column and loops, allowing the user to keep typing
- Pressing Esc exits pencil mode, returns to Move mode

## Line Mode

Pressing "E" enters Line Draw mode

- The line begins at the current position
- The user moves the cursor, indicating the final position of the line
- Lines can be drawn over multiple rows/columns, but do not wrap
- Pressing any key draws that key between the start and final cursor position (e.g. draw a "-" character between two points)
- Draws line to the current layer
- Pressing "ESC" cancels line draw mode and returns to Move mode

## Layers

- The program has layers that can be drawn to
- When commiting a draw operation (paste, line, pencil, box), writing is done to the active layer
- In Move mode, +/- keys change the active layer. This will not wrap (e.g. + after layer 9 has no effect)
- There are 10 layers
- Pressing H toggles visibility of the current layer
- Layers are saved to their own buffer of appropriate size created at init
- Layers are initialized to all spaces

## Saving

- Pressing Ctrl->S saves all layers with non-space content to files
- Files are numbered ./output_layer_0.txt, ./output_layer_1.txt

## Drawing

- On every edit, the display is redrawn
- Layers are drawn in order, 0-10
- Each layer can overwrite the prior layer with non-space content

