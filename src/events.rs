use windows::core::{implement, GUID, Interface};
use windows::Win32::Foundation::BOOL;
use windows::Win32::Media::Audio::{IAudioSessionEvents, IAudioSessionEvents_Impl, IAudioSessionNotification, IAudioSessionNotification_Impl};

use crate::AudioSessionControl;

type OptionBox<T> = Option<Box<T>>;

pub struct EventCallbacks {
  simple_volume: OptionBox<dyn Fn(f32, bool, GUID)>,
}

impl Default for EventCallbacks {
  fn default() -> Self { Self::new() }
}

impl EventCallbacks {
  pub fn new() -> Self {
    Self {
      simple_volume: None,
    }
  }

  pub fn on_volume_changed<F>(mut self, callback: F) -> Self
  where
    F: Fn(f32, bool, GUID) + 'static,
  {
    self.simple_volume = Some(Box::new(callback));
    self
  }

  pub fn build(self) -> Self { self }
}

#[implement(IAudioSessionEvents)]
pub struct AudioSessionEvents {
  callbacks: EventCallbacks,
}

impl AudioSessionEvents {
  pub fn new(callbacks: EventCallbacks) -> Self { Self { callbacks } }
}

impl IAudioSessionEvents_Impl for AudioSessionEvents {
  fn OnSimpleVolumeChanged(
    &self,
    newvolume: f32,
    newmute: BOOL,
    eventcontext: *const GUID,
  ) -> windows::core::Result<()> {
    if let Some(callback) = &self.callbacks.simple_volume {
      unsafe {
        let context = eventcontext;
        callback(newvolume, bool::from(newmute), *context);
      }
    }

    Ok(())
  }

  fn OnDisplayNameChanged(
    &self,
    _newdisplayname: &windows::core::PCWSTR,
    _eventcontext: *const windows::core::GUID,
  ) -> windows::core::Result<()> {
    Ok(())
  }

  fn OnIconPathChanged(
    &self,
    _newiconpath: &windows::core::PCWSTR,
    _eventcontext: *const windows::core::GUID,
  ) -> windows::core::Result<()> {
    Ok(())
  }

  fn OnChannelVolumeChanged(
    &self,
    _channelcount: u32,
    _newchannelvolumearray: *const f32,
    _changedchannel: u32,
    _eventcontext: *const windows::core::GUID,
  ) -> windows::core::Result<()> {
    Ok(())
  }

  fn OnGroupingParamChanged(
    &self,
    _newgroupingparam: *const windows::core::GUID,
    _eventcontext: *const windows::core::GUID,
  ) -> windows::core::Result<()> {
    Ok(())
  }

  fn OnStateChanged(
    &self,
    _newstate: windows::Win32::Media::Audio::AudioSessionState,
  ) -> windows::core::Result<()> {
    Ok(())
  }

  fn OnSessionDisconnected(
    &self,
    _disconnectreason: windows::Win32::Media::Audio::AudioSessionDisconnectReason,
  ) -> windows::core::Result<()> {
    Ok(())
  }
}

// TODO: Might replace this with my own wrapper later
pub struct AudioSessionNotificationCallbacks {
  session_created: OptionBox<dyn Fn(AudioSessionControl)>,
}

impl Default for AudioSessionNotificationCallbacks {
  fn default() -> Self { Self::new() }
}

impl AudioSessionNotificationCallbacks {
  pub fn new() -> Self {
    Self {
      session_created: None,
    }
  }

  pub fn on_session_created<F>(mut self, callback: F) -> Self
  where
    F: Fn(AudioSessionControl) + 'static,
  {
    self.session_created = Some(Box::new(callback));
		self
  }

  pub fn build(self) -> Self { self }
}

#[implement(IAudioSessionNotification)]
pub struct AudioSessionNotification {
  callbacks: AudioSessionNotificationCallbacks,
}

impl AudioSessionNotification {
  pub fn new(callbacks: AudioSessionNotificationCallbacks) -> Self { Self { callbacks } }
}

impl IAudioSessionNotification_Impl for AudioSessionNotification {
    fn OnSessionCreated(
			&self,
			newsession: &core::option::Option<windows::Win32::Media::Audio::IAudioSessionControl>
		) -> windows::core::Result<()> {
			if let Some(callback) = &self.callbacks.session_created {
				if let Some(session) = newsession {
					let session = AudioSessionControl::new(session.cast()?);
					callback(session);
				}
			}

			Ok(())
    }
}
