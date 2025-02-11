use crate::brush::RoundBrush;
use crate::brush::Brush;
use crate::document::Document;

pub struct Editor {
	document: Document,
	brush: Brush<RoundBrush>,
}

impl Default for Editor {
	fn default() -> Self {
		Editor {
			document: Document::new(),
			brush: Brush::new("Round Brush".to_owned(), RoundBrush::new()),
		}
	}
}

impl Editor {
	pub fn add(&mut self, x: f32, y: f32) {
		self.brush.add(&mut self.document, x, y);
	}

	pub fn remove(&mut self, x: f32, y: f32) {
		self.brush.remove(&mut self.document, x, y);
	}
}
