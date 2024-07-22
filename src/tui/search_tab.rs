#[derive(Debug)]
pub struct SearchTab {
    input: String,
    character_index: usize,
    input_mode: InputMode,
}

#[derive(Debug)]
enum InputMode {
    Normal,
    Editing,
}

impl SearchTab {
    pub fn new() -> Self {
        Self {
            input: String::new(),
            character_index: 0,
            input_mode: InputMode::Normal,
        }
    }

    /// Move the cursor to the left.
    fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.character_index.saturating_sub(1);
        self.character_index = self.clamp_cursor(cursor_moved_left);
    }

    /// Move the cursor to the right.
    fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.character_index.saturating_add(1);
        self.character_index = self.clamp_cursor(cursor_moved_right);
    }

    fn enter_char(&mut self, new_char: char) {
        let index = self.byte_index();
        self.input.insert(index, new_char);
        self.move_cursor_right();
    }

    /// Returns the byte index of the current cursor position.
    ///
    /// Since each character in a string can contains multiple bytes, it's
    /// necessary to calculate the byte index of the cursor position.
    fn byte_index(&self) -> usize {
        self.input
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.character_index)
            .unwrap_or(self.input.len())
    }

    fn delete_char(&mut self) {
        let is_not_cursor_leftmost = self.character_index != 0;
        if is_not_cursor_leftmost {
            let current_index = self.character_index;
            let from_left_to_current_index = current_index - 1;

            let before_char_to_delete = self.input.chars().take(from_left_to_current_index);
            let after_char_to_delete = self.input.chars().skip(current_index);

            self.input = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }

    /// Clamp cursor position to be within the bounds of the input string.
    ///
    /// This function is used to ensure that the cursor position is always within the bounds of the input string.
    ///
    /// # Arguments
    /// + `new_cursor_pos` - The new cursor position.
    ///
    /// # Returns
    /// + The clamped cursor position.
    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input.chars().count())
    }
}
