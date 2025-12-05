//! Undo/redo history management using snapshot-based approach.
//!
//! Stores complete layout snapshots for simple and reliable undo/redo.

use crate::model::LayoutDocument;

/// Maximum number of states to keep in history.
const MAX_HISTORY_SIZE: usize = 50;

/// Manages undo/redo history for layout changes.
#[derive(Debug, Clone)]
pub struct History {
    /// Stack of previous states (for undo).
    undo_stack: Vec<LayoutDocument>,
    /// Stack of future states (for redo).
    redo_stack: Vec<LayoutDocument>,
}

impl Default for History {
    fn default() -> Self {
        Self::new()
    }
}

impl History {
    /// Create a new empty history.
    pub fn new() -> Self {
        Self {
            undo_stack: Vec::with_capacity(MAX_HISTORY_SIZE),
            redo_stack: Vec::with_capacity(MAX_HISTORY_SIZE),
        }
    }

    /// Push a snapshot before making a change.
    /// This clears the redo stack.
    pub fn push(&mut self, snapshot: LayoutDocument) {
        // Clear redo stack when new changes are made
        self.redo_stack.clear();

        // Add to undo stack
        self.undo_stack.push(snapshot);

        // Trim to max size
        if self.undo_stack.len() > MAX_HISTORY_SIZE {
            self.undo_stack.remove(0);
        }
    }

    /// Undo the last change.
    /// Returns the previous state, or None if no undo available.
    /// The caller should pass in the current state to save for redo.
    pub fn undo(&mut self, current: LayoutDocument) -> Option<LayoutDocument> {
        let previous = self.undo_stack.pop()?;
        self.redo_stack.push(current);
        Some(previous)
    }

    /// Redo a previously undone change.
    /// Returns the next state, or None if no redo available.
    /// The caller should pass in the current state to save for undo.
    pub fn redo(&mut self, current: LayoutDocument) -> Option<LayoutDocument> {
        let next = self.redo_stack.pop()?;
        self.undo_stack.push(current);
        Some(next)
    }

    /// Check if undo is available.
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Check if redo is available.
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Clear all history.
    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }

    /// Get the number of undo steps available.
    pub fn undo_count(&self) -> usize {
        self.undo_stack.len()
    }

    /// Get the number of redo steps available.
    pub fn redo_count(&self) -> usize {
        self.redo_stack.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_doc(name: &str) -> LayoutDocument {
        let mut doc = LayoutDocument::default();
        doc.name = name.to_string();
        doc
    }

    #[test]
    fn test_push_and_undo() {
        let mut history = History::new();

        let state1 = make_doc("State 1");
        let state2 = make_doc("State 2");
        let current = make_doc("Current");

        history.push(state1.clone());
        history.push(state2.clone());

        assert!(history.can_undo());
        assert!(!history.can_redo());

        let restored = history.undo(current.clone()).unwrap();
        assert_eq!(restored.name, "State 2");
        assert!(history.can_redo());
    }

    #[test]
    fn test_redo() {
        let mut history = History::new();

        let state1 = make_doc("State 1");
        let current = make_doc("Current");

        history.push(state1.clone());

        let restored = history.undo(current.clone()).unwrap();
        assert_eq!(restored.name, "State 1");

        let redone = history.redo(restored).unwrap();
        assert_eq!(redone.name, "Current");
    }

    #[test]
    fn test_push_clears_redo() {
        let mut history = History::new();

        let state1 = make_doc("State 1");
        let state2 = make_doc("State 2");
        let current = make_doc("Current");

        history.push(state1);
        history.undo(current);

        assert!(history.can_redo());

        history.push(state2);
        assert!(!history.can_redo());
    }

    #[test]
    fn test_clear() {
        let mut history = History::new();
        history.push(make_doc("State 1"));
        history.push(make_doc("State 2"));
        
        assert!(history.can_undo());
        
        history.clear();
        
        assert!(!history.can_undo());
        assert!(!history.can_redo());
        assert_eq!(history.undo_count(), 0);
        assert_eq!(history.redo_count(), 0);
    }

    #[test]
    fn test_undo_count_redo_count() {
        let mut history = History::new();
        
        assert_eq!(history.undo_count(), 0);
        assert_eq!(history.redo_count(), 0);
        
        history.push(make_doc("State 1"));
        history.push(make_doc("State 2"));
        
        assert_eq!(history.undo_count(), 2);
        assert_eq!(history.redo_count(), 0);
        
        history.undo(make_doc("Current"));
        
        assert_eq!(history.undo_count(), 1);
        assert_eq!(history.redo_count(), 1);
    }

    #[test]
    fn test_max_undo_limit() {
        let mut history = History::new();
        
        // Push more than MAX_UNDO_STACK (50) states
        for i in 0..60 {
            history.push(make_doc(&format!("State {}", i)));
        }
        
        // Should be capped at 50
        assert_eq!(history.undo_count(), 50);
    }

    #[test]
    fn test_undo_empty_returns_none() {
        let mut history = History::new();
        let result = history.undo(make_doc("Current"));
        assert!(result.is_none());
    }

    #[test]
    fn test_redo_empty_returns_none() {
        let mut history = History::new();
        let result = history.redo(make_doc("Current"));
        assert!(result.is_none());
    }

    #[test]
    fn test_multiple_undo_redo_cycles() {
        let mut history = History::new();
        
        history.push(make_doc("A"));
        history.push(make_doc("B"));
        history.push(make_doc("C"));
        
        // Undo all
        let c = history.undo(make_doc("D")).unwrap();
        assert_eq!(c.name, "C");
        let b = history.undo(c).unwrap();
        assert_eq!(b.name, "B");
        let a = history.undo(b).unwrap();
        assert_eq!(a.name, "A");
        
        // Redo all
        let b2 = history.redo(a).unwrap();
        assert_eq!(b2.name, "B");
        let c2 = history.redo(b2).unwrap();
        assert_eq!(c2.name, "C");
        let d = history.redo(c2).unwrap();
        assert_eq!(d.name, "D");
    }
}
