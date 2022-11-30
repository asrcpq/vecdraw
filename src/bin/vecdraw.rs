use std::collections::HashMap;
use std::sync::mpsc::channel;
use ttri::reexport::winit::{
	event_loop::{ControlFlow, EventLoopBuilder},
	event::{Event, WindowEvent, ElementState, MouseButton},
};

use skey::{Skey, Sktype};
use skey::winit::WinitConversion;
use ttri::teximg::Teximg;
use ttri::camcon::Camcon;
use ttri::renderer::Renderer;
use ttri_model::cmodel::{Model, Face};
use ttri_model::draw::Pen;
use psva4_model::rawmodel::Rawmodel;

type V2 = rust_stddep::nalgebra::Vector2<f32>;

fn main() {
	let el = EventLoopBuilder::<()>::with_user_event().build();
	let mut rdr = Renderer::new(&el);
	let elp = el.create_proxy();
	// let (tx, rx) = channel();
	let mut model_handle = Vec::new();
	let mut camcon = Camcon::new([640, 480]);
	let mut button_on = false;
	// 0: draw mode
	// 1: select mode
	// 2: move mode
	let mut mode = 1;
	let mut select_range = Vec::new();
	let args = std::env::args().collect::<Vec<_>>();
	let mut rawmo = Rawmodel::load(&args[1]).unwrap_or_default();
	camcon.fit_inner(V2::new(0.0, 0.0), V2::new(1.0, 1.0));
	el.run(move |event, _, ctrl| match event {
		Event::WindowEvent { event: e, .. } => {
			camcon.process_event(&e);
			match e {
				WindowEvent::CloseRequested => {
					*ctrl = ControlFlow::Exit;
				}
				WindowEvent::CursorMoved {
					position,
					..
				} => {
					if button_on {
						let pos: [f64; 2] = position.into();
						let pos = V2::new(pos[0] as f32, pos[1] as f32);
						let wpos = camcon.s2w(pos);
						if mode == 2 {
							// tx.send(Msg::Lock(wpos)).unwrap();
						} else if mode == 1 {
							if select_range.len() < 2 {
								select_range.push(wpos);
							} else {
								select_range[1] = wpos;
							}
						}
					}
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
							} else {
								if select_range.len() == 2 {
									eprintln!("range {:?}", select_range);
									// tx.send(Msg::Select(select_range[0], select_range[1])).unwrap();
								}
								mode = 2;
								select_range.clear();
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
								if mode != 1 {
									mode = 1;
								}
							},
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
			let mut model = Model::default();
			let pen = Pen {width: 0f32, color: [0.1, 0.1, 0.1, 1f32], z: 0.9f32};
			pen.draw_rect(&mut model, V2::new(-10.0, -10.0), V2::new(9.0, 9.0));
			let pen = Pen {width: 0f32, color: [0f32, 0f32, 0f32, 1f32], z: 0.8f32};
			pen.draw_rect(&mut model, V2::new(0.0, 0.0), V2::new(1.0, 1.0));
			for v in rawmo.vs.values() {
			}
			model_handle = vec![rdr.insert_model(&model)];
			rdr.render(camcon.get_camera());
			*ctrl = ControlFlow::Wait;
		}
		_ => {},
	})
}
