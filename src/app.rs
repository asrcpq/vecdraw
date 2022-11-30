use ttri_model::cmodel::Model as Ttrimo;
use ttri_model::draw::Pen;
use psva4_model::rawmodel::{Rawmodel, RawVertex, Vid};
use crate::V2;

#[derive(Default)]
pub struct Vecdraw {
	rawmo: Rawmodel,
	drawing: Vec<Point>,
	select_range: Vec<V2>,
	snap_highlight: Option<Vid>,
}

enum Point {
	Exist(Vid),
	New(V2),
}

impl Vecdraw {
	pub fn new(rawmo: Rawmodel) -> Self {
		Self {
			rawmo,
			..Default::default()
		}
	}

	fn point_pos(&self, p: &Point) -> V2 {
		match p {
			Point::New(p) => *p,
			Point::Exist(x) => self.rawmo.vs.get(x).unwrap().pos,
		}
	}

	pub fn save(&self, path: &str) {
		self.rawmo.save(path).unwrap();
	}

	pub fn finish_draw(&mut self) {
		if self.drawing.len() == 2 {
			let id1 = match self.drawing[0] {
				Point::New(_) => self.rawmo.alloc_id(),
				Point::Exist(id) => id,
			};
			let id2 = match self.drawing[1] {
				Point::New(_) => self.rawmo.alloc_id(),
				Point::Exist(id) => id,
			};
			let mut flag = true;
			if let Some(v1) = self.rawmo.neigh.get(&id1) {
				if v1.iter().any(|&x| x == id2) {
					flag = false;
				}
			}
			if let Some(v2) = self.rawmo.neigh.get(&id2) {
				if v2.iter().any(|&x| x == id1) {
					flag = false;
				}
			}
			if flag {
				let p0 = self.point_pos(&self.drawing[0]);
				self.rawmo.vs.insert(id1, RawVertex {
					pos: p0,
					tex: p0,
					im: 1f32,
				});
				let p1 = self.point_pos(&self.drawing[1]);
				self.rawmo.vs.insert(id2, RawVertex {
					pos: p1,
					tex: p1,
					im: 1f32,
				});
				let e1 = self.rawmo.neigh.entry(id1).or_insert_with(Default::default);
				e1.push(id2);
				let e2 = self.rawmo.neigh.entry(id2).or_insert_with(Default::default);
				e2.push(id1);
			} else {
				eprintln!("dup edge!");
			}
		}
		self.drawing.clear();
	}
	
	pub fn finish_select(&mut self) {
		if self.select_range.len() == 2 {
			// do select
		}
		self.select_range.clear();
	}

	pub fn select_update(&mut self, wpos: V2) {
		if self.select_range.len() < 2 {
			self.select_range.push(wpos);
		} else {
			self.select_range[1] = wpos;
		}
	}

	pub fn snap_update(&mut self, wpos: V2) {
		self.snap_highlight.take();
		let mut min_dist = 0.2f32;
		for (k, v) in self.rawmo.vs.iter() {
			let dist = (v.pos - wpos).norm();
			if dist < min_dist {
				min_dist = dist;
				self.snap_highlight = Some(*k);
			}
		}
	}

	pub fn snap_off(&mut self) {
		self.snap_highlight = None;
	}

	pub fn drawing_update(&mut self, wpos: V2) {
		let p = if let Some(x) = self.snap_highlight {
			Point::Exist(x)
		} else {
			Point::New(wpos)
		};
		if self.drawing.len() < 2 {
			self.drawing.push(p);
		} else {
			self.drawing[1] = p;
		}
	}

	pub fn render(&self) -> Ttrimo {
		let mut model = Ttrimo::default();
		let pen = Pen {width: 0f32, color: [0.1, 0.1, 0.1, 1f32], z: 0.9f32};
		pen.draw_rect(&mut model, V2::new(-10.0, -10.0), V2::new(9.0, 9.0));
		let pen = Pen {width: 0f32, color: [0f32, 0f32, 0f32, 1f32], z: 0.8f32};
		pen.draw_rect(&mut model, V2::new(0.0, 0.0), V2::new(1.0, 1.0));
		let pen = Pen {width: 0.002f32, color: [1f32; 4], z: 0.6f32};
		for (k, v) in self.rawmo.neigh.iter() {
			let v1 = self.rawmo.vs.get(k).unwrap();
			for vv in v.iter() {
				let v2 = self.rawmo.vs.get(vv).unwrap();
				pen.draw_line(&mut model, [v1.pos, v2.pos]);
			}
		}
		let pen = Pen {width: 0.003f32, color: [0f32, 1f32, 1f32, 1f32], z: 0.5f32};
		if self.drawing.len() == 2 {
			pen.draw_line(&mut model, [
				self.point_pos(&self.drawing[0]),
				self.point_pos(&self.drawing[1]),
			]);
		}
		let pen = Pen {width: 0.0f32, color: [0.3f32; 4], z: 0.6f32};
		if self.select_range.len() == 2 {
			pen.draw_rect(&mut model, self.select_range[0], self.select_range[1]);
		}
		let pen = Pen {width: 0.01f32, color: [1f32, 1f32, 0f32, 0.5f32], z: 0.6f32};
		if let Some(x) = self.snap_highlight {
			pen.draw_dot(&mut model, self.rawmo.vs.get(&x).unwrap().pos);
		}
		model
	}
}
