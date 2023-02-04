use events::{AudioSessionNotification, AudioSessionNotificationCallbacks};
use sysinfo::{Pid, ProcessExt, System, SystemExt, PidExt};
use windows::Win32::Foundation::RPC_E_CHANGED_MODE;
use windows::core::{Interface, GUID, HRESULT};
use windows::Win32::Media::Audio::{
  eMultimedia,
  eRender,
  IAudioSessionControl2,
  IAudioSessionEvents,
  IAudioSessionManager2,
  IMMDeviceEnumerator,
  ISimpleAudioVolume,
  MMDeviceEnumerator, IAudioSessionNotification,
};
use windows::Win32::System::Com::{CoCreateInstance, CLSCTX_ALL, CoUninitialize, CoInitialize};

use self::events::{AudioSessionEvents, EventCallbacks};

pub mod events;

pub struct AudioSessionControl {
  control: IAudioSessionControl2,
  volume_control: AudioVolumeControl,
}

unsafe impl Send for AudioSessionControl {}

impl AudioSessionControl {
  pub fn new(control: IAudioSessionControl2) -> Self {
    Self {
      volume_control: AudioVolumeControl::new(&control),
      control,
    }
  }

  pub fn process_name(&self) -> Result<String, String> {
		let mut sys = System::new_all();
		sys.refresh_all();

		match sys.process(Pid::from(self.process_id().unwrap() as usize)) {
			Some(app_path) => return Ok(app_path.name().to_string()),
			None => Err("Error getting process name".into()),
		}
	}

  pub fn process_id(&self) -> windows::core::Result<u32> { unsafe { self.control.GetProcessId() } }

  pub fn volume_control(&self) -> &AudioVolumeControl { &self.volume_control }

  pub fn register_session_notification(
    &self,
    callbacks: EventCallbacks,
  ) -> windows::core::Result<()> {
    unsafe {
      let events: IAudioSessionEvents = AudioSessionEvents::new(callbacks).into();
      self.control.RegisterAudioSessionNotification(&events)
    }
  }
}

pub struct AudioVolumeControl {
  control: ISimpleAudioVolume,
}

impl AudioVolumeControl {
  pub fn new(session_control: &IAudioSessionControl2) -> Self {
    // TODO: handle the error
    Self {
      control: session_control.cast().unwrap(),
    }
  }

  pub fn get_volume(&self) -> f32 {
    unsafe { self.control.GetMasterVolume().unwrap_or(-1.0) }
  }

  pub fn set_volume(&self, volume: f32) -> windows::core::Result<()> {
    unsafe { self.control.SetMasterVolume(volume, &GUID::zeroed()) }
  }

  pub fn set_mute(&self, mute: bool) -> windows::core::Result<()> {
    unsafe { self.control.SetMute(mute, &GUID::zeroed()) }
  }
}

pub struct AudioSessionManager {
	manager: IAudioSessionManager2,
	notifications: Option<IAudioSessionNotification>
}

unsafe impl Send for AudioSessionManager {}

impl AudioSessionManager {
	pub fn new() -> Result<Self, windows::core::Error> {
		println!("AudioSessionManager::New");
		unsafe {
			match CoInitialize(None)
				.map_err(|e| e.code()) {
					Ok(_) | Err(RPC_E_CHANGED_MODE) => println!("ERROR: {}", windows::core::Error::from(RPC_E_CHANGED_MODE)),
					Err(e) => panic!("Failed to initialize COM: {}", std::io::Error::from_raw_os_error(e.0))
				};
			
			let manager =
				CoCreateInstance::<_, IMMDeviceEnumerator>(&MMDeviceEnumerator, None, CLSCTX_ALL)?
					.GetDefaultAudioEndpoint(eRender, eMultimedia)?
					.Activate::<IAudioSessionManager2>(CLSCTX_ALL, None)?;

			Ok(Self { manager, notifications: None })
		}
	}

	pub fn on_session_created<F>(&mut self, callback: F) -> Result<(), windows::core::Error>
	where F: Fn(AudioSessionControl) + 'static {
		unsafe {
			// Run callback on all currently active sessions
			let sessions = self.manager.GetSessionEnumerator()?;
			for i in 0..sessions.GetCount()? {
				callback(AudioSessionControl::new(sessions.GetSession(i)?.cast()?));
			}

			let callbacks = AudioSessionNotificationCallbacks::new()
				.on_session_created(callback)
				.build();

			let notifications = AudioSessionNotification::new(callbacks).into();
			self.manager.RegisterSessionNotification(&notifications)?;
			self.notifications = Some(notifications);
		}
		Ok(())
	}

	pub fn find_active_session(&self, name: &str) -> Result<AudioSessionControl, windows::core::Error> {
		let mut sys = System::new_all();
		sys.refresh_all();

		unsafe {
			let sessions = self.manager.GetSessionEnumerator()?;

			for i in 0..sessions.GetCount()? {
				let session_control = AudioSessionControl::new(sessions.GetSession(i)?.cast()?);
				let id = session_control.process_id()?;
				
				let Some(app_path) = sys.process(Pid::from_u32(id)) else {
					return Err(windows::core::Error::new(HRESULT(0), "".into()))
				};
				
				if app_path.name() != name {
					continue
				}
				
				return Ok(session_control);
			}

			Err(windows::core::Error::new(HRESULT(0), "".into()))
		}
	}
}

impl Drop for AudioSessionManager {
	fn drop(&mut self) {
		println!("AudioSessionManager::Drop");

		unsafe {
			CoUninitialize();

			if let Some(notifications) = &self.notifications {
				self.manager.UnregisterSessionNotification(notifications).unwrap();
			}
		}
	}
}