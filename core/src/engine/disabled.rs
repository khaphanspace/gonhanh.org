//! IME switched off: Vietnamese typing is skipped but shortcuts (btw → by the way, -> → →) keep working.

use super::autocap::break_char;
use super::out::Out;
use super::Engine;
use crate::data::keys;
use crate::utils;

impl Engine {
    pub(super) fn disabled_key(&mut self, key: u16, caps: bool, shift: bool) -> Out {
        self.word.clear();
        self.screen = crate::compose::Display::empty();
        self.history.clear();
        self.spaces_after_commit = 0;
        let im = self.input_method();

        if key == keys::SPACE || key == keys::RETURN || key == keys::ENTER {
            if !self.shortcut_prefix.is_empty() {
                if let Some(m) =
                    self.shortcuts
                        .try_match_for_method(&self.shortcut_prefix, None, true, im)
                {
                    let mut output: Vec<char> = m.output.chars().collect();
                    let backspace = m.backspace_count as u8;
                    self.shortcut_prefix.clear();
                    if key == keys::SPACE {
                        output.push(' ');
                    }
                    return Out::send(backspace, &output);
                }
            }
            self.shortcut_prefix.clear();
            return Out::none();
        }
        if key == keys::DELETE {
            self.shortcut_prefix.pop();
            return Out::none();
        }
        if keys::is_break_ext(key, shift) {
            if let Some(ch) = break_char(key, shift) {
                if !self.shortcut_prefix.is_empty() {
                    if let Some(m) =
                        self.shortcuts
                            .try_match_for_method(&self.shortcut_prefix, None, true, im)
                    {
                        let output: Vec<char> = m.output.chars().collect();
                        let backspace = m.backspace_count as u8;
                        self.shortcut_prefix.clear();
                        return Out::send(backspace, &output);
                    }
                }
                self.shortcut_prefix.push(ch);
                if let Some(m) =
                    self.shortcuts
                        .try_match_for_method(&self.shortcut_prefix, None, false, im)
                {
                    let output: Vec<char> = m.output.chars().collect();
                    let backspace = (m.backspace_count as u8).saturating_sub(1);
                    self.shortcut_prefix.clear();
                    return Out::send_consumed(backspace, &output);
                }
                return Out::none();
            }
            self.shortcut_prefix.clear();
            return Out::none();
        }
        if let Some(ch) = utils::key_to_char(key, caps) {
            self.shortcut_prefix.push(ch);
            return Out::none();
        }
        self.shortcut_prefix.clear();
        Out::none()
    }
}
