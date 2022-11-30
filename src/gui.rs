use ttri::reexport::winit::{
	event_loop::{ControlFlow, EventLoop},
	event::{Event, WindowEvent, ElementState, MouseButton},
	window::CursorIcon,
};

use crate::app::Vecdraw;
use crate::V2;
use skey::{Skey, Sktype};
use skey::modtrack::ModifierTracker as Modtrack;
use skey::winit::{WinitConversion, WinitModifier};
use ttri::camcon::Camcon;
use ttri::renderer::Renderer;

pub struct Gui {
	rdr: Renderer,
	camcon: Camcon,
	button_on: bool,
	modtrack: Modtrack,
	mode: u8,
	vecdraw: Vecdraw,
}

impl Gui {
	pub fn new(vecdraw: Vecdraw, el: &EventLoop<()>) -> Self {
		let rdr = Renderer::new(el);
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

	pub fn proc_event(&mut self, e: Event<()>, ctrl: &mut ControlFlow) { match e {
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
			let model = self.vecdraw.render();
			let _model_handle = vec![self.rdr.insert_model(&model)];
			self.rdr.render(self.camcon.get_camera());
			*ctrl = ControlFlow::Wait;
		}
		_ => {},
	}}
}
