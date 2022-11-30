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
	selected: Vec<Vid>,
	move_pps: Option<V2>,
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

	pub fn delete_select(&mut self) {
		for vsel in self.selected.iter() {
			self.rawmo.neigh.remove(vsel);
			self.rawmo.vs.remove(vsel);
			for (_, v) in self.rawmo.neigh.iter_mut() {
				if let Some(x) = v.iter().position(|x| x == vsel) {
					v.remove(x);
				}
			}
			// vecdraw does not care about faces
			// for f in std::mem::take(&mut self.rawmo.fs)
			// 	.into_iter()
			// {
			// 	if !f.contains(vsel) {
			// 		self.rawmo.fs.push(f);
			// 	}
			// }
		}
		self.unselect();
	}

	pub fn finish_select(&mut self) {
		self.selected.clear();
		if self.select_range.len() == 2 {
			let s1 = self.select_range[0];
			let s2 = self.select_range[1];
			let minx = s1[0].min(s2[0]);
			let maxx = s1[0].max(s2[0]);
			let miny = s1[1].min(s2[1]);
			let maxy = s1[1].max(s2[1]);
			for (k, v) in self.rawmo.vs.iter() {
				if v.pos[0] > minx && v.pos[0] < maxx &&
					v.pos[1] > miny && v.pos[1] < maxy
				{
					self.selected.push(*k);
				}
			}
		}
		self.select_range.clear();
	}

	pub fn unselect(&mut self) {
		self.selected.clear();
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

	pub fn move_end(&mut self) {
		self.move_pps = None;
	}

	pub fn exact_select(&mut self) {
		if let Some(id) = self.snap_highlight {
			self.selected = vec![id];
		}
	}

	pub fn move_select(&mut self, o: V2) {
		if let Some(oo) = self.move_pps {
			for vsel in self.selected.iter() {
				let v = self.rawmo.vs.get_mut(vsel).unwrap();
				v.pos += o - oo;
			}
		}
		self.move_pps = Some(o);
	}

	pub fn render(&self) -> Ttrimo {
		let mut model = Ttrimo::default();
		let pen = Pen {width: 0f32, color: [0.1, 0.1, 0.1, 1f32], z: 0.9f32};
		pen.draw_rect(&mut model, V2::new(-10.0, -10.0), V2::new(9.0, 9.0));
		let pen = Pen {width: 0f32, color: [0f32, 0f32, 0f32, 1f32], z: 0.8f32};
		pen.draw_rect(&mut model, V2::new(0.0, 0.0), V2::new(1.0, 1.0));
		let pen = Pen {width: 0.001f32, color: [1f32; 4], z: 0.6f32};
		let pen2 = Pen {width: 0.005f32, color: [1f32, 0.5, 0.0, 0.5], z: 0.55f32};
		for (k, v) in self.rawmo.neigh.iter() {
			let v1 = self.rawmo.vs.get(k).unwrap();
			pen2.draw_dot(&mut model, v1.pos);
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
		let pen = Pen {width: 0.01f32, color: [0f32, 1f32, 0f32, 0.3f32], z: 0.6f32};
		for vsel in self.selected.iter() {
			let p = self.rawmo.vs.get(vsel).unwrap().pos;
			pen.draw_dot(&mut model, p);
		}
		model
	}
}
