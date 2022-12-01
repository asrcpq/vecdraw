use std::collections::HashSet;

use ttri_model::cmodel::{Face, Model as Ttrimo};
use ttri_model::draw::{Pen, v2p4};
use psva4_model::rawmodel::{Rawmodel, RawVertex, Vid};
use crate::V2;

#[derive(Default)]
pub struct Vecdraw {
	rawmo: Rawmodel,
	drawing: Vec<Point>,
	select_range: Vec<V2>,
	snap_highlight: Option<Vid>,
	selected: HashSet<Vid>,
	move_pps: Option<V2>,
	dc: f32,
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

	pub fn save(&mut self, path: &str) {
		self.rawmo.fix();
		self.rawmo.save(path).unwrap();
	}

	pub fn set_dcv(&mut self, dc: f32) {
		eprintln!("set dc to {}", dc);
		self.dc = dc;
	}

	pub fn select_apply_dcv(&mut self) {
		for ([id1, id2], v) in self.rawmo.dcs.iter_mut() {
			if self.selected.contains(id1) && self.selected.contains(id2) {
				eprint!("[{}, {}]", id1, id2);
				*v = self.dc;
			}
		}
		eprintln!();
		eprintln!("dc set to {}", self.dc);
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
		// dirty
		self.rawmo.fs.clear();
		self.rawmo.border.clear();
		for vsel in self.selected.iter() {
			self.rawmo.neigh.remove(vsel);
			self.rawmo.vs.remove(vsel);
			for (_, v) in self.rawmo.neigh.iter_mut() {
				if let Some(x) = v.iter().position(|x| x == vsel) {
					v.remove(x);
				}
			}
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
					self.selected.insert(*k);
				}
			}
		}
		eprintln!("select {:?}", self.selected);
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
			self.selected.clear();
			self.selected.insert(id);
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

	pub fn build(&mut self) {
		self.rawmo.build_topo2();
	}

	pub fn render(&self) -> Ttrimo {
		let mut model = Ttrimo::default();
		for ids in self.rawmo.fs.iter() {
			let vlen = model.vs.len();
			let vspos: [V2; 3] = core::array::from_fn(
				|idx| self.rawmo.vs.get(&ids[idx]).unwrap().pos
			);
			model.vs.push(v2p4(vspos[0], 0.79));
			model.vs.push(v2p4(vspos[1], 0.79));
			model.vs.push(v2p4(vspos[2], 0.79));
			let color = [0f32, 1.0, 0.5, 0.1];
			model.faces.push(Face::solid([vlen, vlen + 1, vlen + 2], color));
		}

		let peni = Pen {width: 0.005f32, color: [1f32, 0.5, 0.0, 0.5], z: 0.75f32};
		let penb = Pen {width: 0.005f32, color: [0f32, 1f32, 0f32, 1f32], z: 0.75f32};
		let mut pen2 = Pen {width: 0.001f32, color: [1f32, 0f32, 0f32, 1f32], z: 0.77f32};
		let mut drawed = HashSet::new();
		for (k, v) in self.rawmo.neigh.iter() {
			let v1 = self.rawmo.vs.get(k).unwrap();
			if self.rawmo.border.contains(k) {
				penb.draw_dot(&mut model, v1.pos);
			} else {
				peni.draw_dot(&mut model, v1.pos);
			}
			for vv in v.iter() {
				let v2 = self.rawmo.vs.get(vv).unwrap();
				let mut ids = [*k, *vv];
				ids.sort_unstable();
				if drawed.insert(ids) {
					pen2.color = if let Some(dc) = self.rawmo.dcs.get(&ids) {
						let lg = dc.log10();
						if lg < 0f32 {
							[1f32, -lg / 10f32, 0f32, 1f32]
						} else {
							[1f32, 0f32, 1f32, 1f32]
						}
					} else {
						[1f32, 0f32, 1f32, 1f32]
					};
					pen2.draw_line(&mut model, [v1.pos, v2.pos]);
				}
			}
		}
		let pen = Pen {width: 0.003f32, color: [0f32, 1f32, 1f32, 1f32], z: 0.5f32};
		if self.drawing.len() == 2 {
			pen.draw_line(&mut model, [
				self.point_pos(&self.drawing[0]),
				self.point_pos(&self.drawing[1]),
			]);
		}
		let pen = Pen {width: 0.0f32, color: [0.3f32; 4], z: 0.78f32};
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
