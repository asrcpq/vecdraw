use ttri::reexport::winit::{
	event_loop::{ControlFlow, EventLoopBuilder},
	event::{Event, WindowEvent, ElementState, MouseButton},
};

use psva4_model::rawmodel::Rawmodel;
use vecdraw::app::Vecdraw;
use vecdraw::V2;
use skey::{Skey, Sktype};
use skey::modtrack::ModifierTracker as Modtrack;
use skey::winit::{WinitConversion, WinitModifier};
use ttri::camcon::Camcon;
use ttri::renderer::Renderer;

fn main() {
	let el = EventLoopBuilder::<()>::with_user_event().build();
	let mut rdr = Renderer::new(&el);
	// let (tx, rx) = channel();
	let mut _model_handle = Vec::new();
	let mut camcon = Camcon::new([640, 480]);
	let mut button_on = false;
	let mut modtrack = Modtrack::default();
	// 0: draw mode
	// 1: select mode
	// 2: move mode
	let mut mode = 1;
	let args = std::env::args().collect::<Vec<_>>();
	let rawmo = if let Ok(x) = Rawmodel::load(&args[1]) {
		eprintln!("load json ok");
		x
	} else if let Ok(x) = Rawmodel::simple_load(&args[1]) {
		eprintln!("load simple ok");
		x
	} else {
		eprintln!("load fail, create");
		Default::default()
	};
	let mut vecdraw = Vecdraw::new(rawmo);
	camcon.fit_inner(V2::new(0.0, 0.0), V2::new(1.0, 1.0));
	el.run(move |event, _, ctrl| match event {
		Event::WindowEvent { event: e, .. } => {
			camcon.process_event(&e);
			match e {
				WindowEvent::CloseRequested => {
					vecdraw.save(&args[1]);
					*ctrl = ControlFlow::Exit;
				}
				WindowEvent::CursorMoved {
					position,
					..
				} => {
					let pos: [f64; 2] = position.into();
					let pos = V2::new(pos[0] as f32, pos[1] as f32);
					let wpos = camcon.s2w(pos);
					if modtrack.ctrl {
						vecdraw.snap_update(wpos);
					} else {
						vecdraw.snap_off();
					}
					if button_on {
						if mode == 2 {
							// tx.send(Msg::Lock(wpos)).unwrap();
						} else if mode == 1 {
							vecdraw.select_update(wpos);
						} else if mode == 0 {
							vecdraw.drawing_update(wpos);
						}
					}
				}
				WindowEvent::ModifiersChanged(ms) => {
					modtrack.update_state(ms);
				}
				WindowEvent::MouseInput {
					state,
					button,
					..
				} => {
					if button == MouseButton::Left {
						if state == ElementState::Pressed {
							button_on = true;
						} else {
							if mode == 2 {
								// tx.send(Msg::Unlock).unwrap();
							} else if mode == 1 {
								vecdraw.finish_select();
							} else {
								vecdraw.finish_draw();
							}
							button_on = false;
						}
					}
				}
				WindowEvent::KeyboardInput { input: wki, .. } => {
					if let Some(key) = Skey::from_wki(wki) {
						if key.down { match key.ty {
							Sktype::Ascii(b'a') => {
								mode = 0;
							}
							Sktype::Ascii(b'r') => {
								mode = 1;
							},
							Sktype::Ascii(b't') => {
								mode = 2;
							}
							_ => {},
						}}
					}
				}
				WindowEvent::Resized(_) => {
					rdr.damage();
					camcon.resize(rdr.get_size());
				}
				_ => {}
			}
		}
		Event::MainEventsCleared => {
			let model = vecdraw.render();
			_model_handle = vec![rdr.insert_model(&model)];
			rdr.render(camcon.get_camera());
			*ctrl = ControlFlow::Wait;
		}
		_ => {},
	})
}
