#[derive(Debug)]
pub struct SettingsChanged {
    pub recursive: bool,
    pub repeat: bool,
    pub random: bool,
    pub playing_dir: String,
    pub browsing_dir: String,
    pub song_playing: String,
    // The pair is u32 elasped seconds, u32 total seconds
    pub song_time: (u32, u32)
}

impl Clone for SettingsChanged {
    fn clone(&self) -> Self {
        SettingsChanged {
            recursive: self.recursive,
            repeat: self.repeat,
            random: self.random,
            playing_dir: self.playing_dir.clone(),
            browsing_dir: self.browsing_dir.clone(),
            song_playing: self.song_playing.clone(),
            song_time: self.song_time
        }
    }
}