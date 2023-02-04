use std::sync::{Arc, Mutex};
use std::time::Duration;

use windows_volume_mixer::events::EventCallbacks;
use windows_volume_mixer::{AudioSessionManager};

fn main() -> windows::core::Result<()> {
  let name = "Spotify.exe";
	let audio_session = Arc::new(Mutex::new(None));
	
  let mut manager = AudioSessionManager::new()?;
	manager.on_session_created(move |session| {
		if let Ok(process_name) = session.process_name() {
			println!("Session: {process_name}");
			
			if process_name != name {
				return;
			}

			session.register_session_notification(
				EventCallbacks::new()
					.on_volume_changed(|volume, _mute, _| {
						println!("volume is: {volume}");
					})
					.build(),
			).unwrap();

			println!("{:?}", session.volume_control().get_volume());
			session.volume_control().set_volume(0.75).unwrap();
			println!("{:?}", session.volume_control().get_volume());

			*audio_session.lock().unwrap() = Some(session);
		}
	})?;

	loop {		
		std::thread::sleep(Duration::from_secs(1));
	}
}