use ttri::reexport::winit::{
	event_loop::EventLoopBuilder,
};

use vecdraw::gui::Gui;
use vecdraw::app::Vecdraw;
use psva4_model::rawmodel::Rawmodel;

fn main() {
	let el = EventLoopBuilder::<String>::with_user_event().build();
	let elp = el.create_proxy();
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
	std::thread::spawn(move || {
		use std::io::{BufRead, BufReader, stdin};
		let stdin = stdin().lock();
		let reader = BufReader::new(stdin);
		for line in reader.lines() {
			let line = line.unwrap();
			elp.send_event(line).unwrap();
		}
	});
	let vecdraw = Vecdraw::new(rawmo);
	let mut gui = Gui::new(vecdraw, &el);
	el.run(move |event, _, ctrl| gui.proc_event(event, ctrl));
}
