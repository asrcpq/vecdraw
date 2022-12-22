use ttri::reexport::winit::{
	event_loop::{ControlFlow, EventLoop},
	event::{Event, WindowEvent, ElementState, MouseButton},
	window::CursorIcon,
};

use crate::app::Vecdraw;
use crate::{M2, V2};
use skey::{Skey, Sktype};
use skey::modtrack::ModifierTracker as Modtrack;
use skey::winit::{WinitConversion, WinitModifier};
use ttri_model::draw::Pen;
use ttri_model::cmodel::{Face, Model as Ttrimo};
use ttri::teximg::Teximg;
use ttri::cam::camcon2::Camcon;
use ttri::renderer::Renderer;

pub struct Gui {
	rdr: Renderer,
	camcon: Camcon,
	button_on: bool,
	modtrack: Modtrack,
	mode: u8,
	vecdraw: Vecdraw,
	tex_layer: i32,
}

impl Gui {
	pub fn new(mut vecdraw: Vecdraw, el: &EventLoop<String>) -> Self {
		vecdraw.set_dcv(1e-6);
		let mut rdr = Renderer::new(el);
		let args = std::env::args().collect::<Vec<_>>();
		let tex_layer = if args.len() >= 3 {
			let teximg = Teximg::load(&args[2], true);
			rdr.upload_tex(teximg, 0);
			0
		} else {
			-2
		};
		let mut camcon = Camcon::new([640, 480]);
		camcon.fit_inner(V2::new(0.0, 0.0), V2::new(1.0, 1.0));
		let button_on = false;
		let modtrack = Modtrack::default();
		// 0: draw mode
		// 1: select mode
		// 2: move mode
		Self {
			rdr,
			camcon,
			button_on,
			modtrack,
			mode: 1,
			tex_layer,
			vecdraw,
		}
	}

	fn modeswitch(&mut self, mode: u8) {
		self.mode = mode;
		let window = self.rdr.get_window();
		match mode {
			0 => window.set_cursor_icon(CursorIcon::Crosshair),
			1 => window.set_cursor_icon(CursorIcon::Default),
			2 => window.set_cursor_icon(CursorIcon::Grab),
			_ => {},
		}
	}

	pub fn proc_event(&mut self, e: Event<String>, ctrl: &mut ControlFlow) { match e {
		Event::WindowEvent { event: e, .. } => {
			self.camcon.process_event(&e);
			match e {
				WindowEvent::CloseRequested => {
					let args = std::env::args().collect::<Vec<_>>();
					self.vecdraw.save(&args[1]);
					*ctrl = ControlFlow::Exit;
				}
				WindowEvent::CursorMoved {
					position,
					..
				} => {
					let pos: [f64; 2] = position.into();
					let pos = V2::new(pos[0] as f32, pos[1] as f32);
					let wpos = self.camcon.s2w(pos);
					if self.modtrack.ctrl {
						self.vecdraw.snap_update(wpos);
					} else {
						self.vecdraw.snap_off();
					}
					if self.button_on {
						if self.mode == 2 {
							self.vecdraw.move_select(wpos);
						} else if self.mode == 1 {
							if self.modtrack.ctrl {
								self.vecdraw.exact_select();
								self.button_on = false;
							} else {
								self.vecdraw.select_update(wpos);
							}
						} else if self.mode == 0 {
							self.vecdraw.drawing_update(wpos);
						}
					}
				}
				WindowEvent::ModifiersChanged(ms) => {
					self.modtrack.update_state(ms);
				}
				WindowEvent::MouseInput {
					state,
					button,
					..
				} => {
					if button == MouseButton::Left {
						if state == ElementState::Pressed {
							self.button_on = true;
						} else {
							// button_on could be manually changed by cancel operation
							if self.button_on {
								if self.mode == 2 {
									self.vecdraw.move_end();
								} else if self.mode == 1 {
									self.vecdraw.finish_select();
								} else if self.mode == 0 {
									self.vecdraw.finish_draw();
								}
							}
							self.button_on = false;
						}
					}
				}
				WindowEvent::KeyboardInput { input: wki, .. } => {
					if let Some(key) = Skey::from_wki(wki) {
						if key.down { match key.ty {
							Sktype::Ascii(b'a') => {
								self.vecdraw.unselect();
								self.modeswitch(0);
							}
							Sktype::Ascii(b'r') => {
								self.modeswitch(1);
							},
							Sktype::Ascii(b't') => {
								self.modeswitch(2);
							}
							Sktype::Ascii(b'x') => {
								self.vecdraw.delete_select();
							}
							_ => {},
						}}
					}
				}
				WindowEvent::Resized(_) => {
					self.rdr.damage();
					self.camcon.resize(self.rdr.get_size());
				}
				_ => {}
			}
		}
		Event::MainEventsCleared => {
			let mut model = if self.tex_layer < 0 {
				let mut model = Ttrimo::default();
				let pen = Pen {width: 0f32, color: [0f32, 0f32, 0f32, 1f32], z: 0.9f32};
				pen.draw_rect(&mut model, V2::new(0.0, 0.0), V2::new(1.0, 1.0));
				model
			} else {
				let vs = vec![
					[0.0, 0.0, 0.9, 1.0],
					[0.0, 1.0, 0.9, 1.0],
					[1.0, 0.0, 0.9, 1.0],
					[1.0, 1.0, 0.9, 1.0],
				];
				let uvs = vec![
					[0.0, 0.0],
					[0.0, 1.0],
					[1.0, 0.0],
					[1.0, 1.0],
				];
				let faces = vec![
					Face {
						vid: [0, 1, 2],
						uvid: [0, 1, 2],
						color: [0f32; 4],
						layer: self.tex_layer,
					},
					Face {
						vid: [3, 1, 2],
						uvid: [3, 1, 2],
						color: [0f32; 4],
						layer: self.tex_layer,
					},
				];
				Ttrimo {vs, uvs, faces}
			};
			let pen = Pen {width: 0f32, color: [0.1, 0.1, 0.1, 1f32], z: 0.95f32};
			pen.draw_rect(&mut model, V2::new(-10.0, -10.0), V2::new(9.0, 9.0));
			let _m1 = vec![self.rdr.insert_model(&model)];
			let model = self.vecdraw.render();
			let _m2 = vec![self.rdr.insert_model(&model)];
			self.rdr.render(self.camcon.get_camera());
			*ctrl = ControlFlow::Wait;
		}
		Event::UserEvent(cmd) => {
			let split: Vec<_> = cmd.split_whitespace().collect();
			if split.len() == 0 { return }
			match split[0] {
				"build" => {
					self.vecdraw.build();
				},
				"dcv" => {
					if let Some(f) = split.get(1)
						.map(|x| x.parse::<f32>().ok())
						.flatten()
					{
						self.vecdraw.set_dcv(f);
					}
				},
				"dcs" => {
					self.vecdraw.select_apply_dcv();
				}
				"name" => {
					if let Some(f) = split.get(1) {
						self.vecdraw.name_select(f.to_string());
					}
				}
				// transform
				"t" => {
					if split.len() >= 5 {
						let m: Vec<_> = (1..5).map(|idx|
							split[idx].parse::<f32>().unwrap()
						).collect();
						let m = M2::new(m[0], m[1], m[2], m[3]);
						self.vecdraw.transform(m);
					}
				}
				"selmode" => {
					self.vecdraw.toggle_selmode();
				}
				"asc" => {
					if split.len() == 1 {
						return
					}
					self.vecdraw.asc(split[1]);
				}
				_ => {},
			}
		}
		_ => {},
	}}
}
