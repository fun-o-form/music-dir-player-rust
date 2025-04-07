use std::io;
use rand::Rng;
use crossbeam_channel::{unbounded, Sender, Receiver};
use std::sync::{Arc, Mutex, mpsc};

use crate::music_player::{MusicPlayer, PlaybackControls, PlaybackStatus};
use crate::file_utils::file_utils;
use crate::settings_changed::SettingsChanged;

/// Tracks the status of the currently playing song and enqueues the next one once the current finishes
struct MonSongThread {
    _song_ctrl: Arc<Mutex<SongControlThread>>,
    _thread: Option<std::thread::JoinHandle<()>>,
}
impl MonSongThread {
    pub fn init(song_ctrl: Arc<Mutex<SongControlThread>>, song_over_l: mpsc::Receiver<PlaybackStatus>) -> MonSongThread {
        let _thread = std::thread::spawn({
            let song_ctrl = Arc::clone(&song_ctrl);
            move || {
                loop {
                    match song_over_l.recv() {
                        Ok(PlaybackStatus::PlaybackComplete(_)) => {
                            println!("Song finished playing.");
                            song_ctrl.lock().unwrap().play_next_song();
                        }
                        Ok(PlaybackStatus::PlaybackPercentage(elapsed, total)) => {
                            // println!("Song playback at {}/{}", elapsed, total);
                            song_ctrl.lock().unwrap()._cur_settings.song_time = (elapsed, total);
                        }
                        Err(e) => {
                            eprintln!("Error receiving song over notification: {}", e);
                            break;
                        }
                    }
                    // Notify listeners of the current settings
                    song_ctrl.lock().unwrap().send_settings();
                }
            }
        });

        MonSongThread {
            _song_ctrl: song_ctrl,
            _thread: Some(_thread),
        }
    }
}

/// Allows outside classes to affect the songs that are played
struct SongControlThread {
    _queued_music_files: Vec<std::path::PathBuf>,
    _cur_playing_index: i64,
    _player: MusicPlayer,
    _cur_settings: SettingsChanged,
    _settings_changed_sender: Sender<SettingsChanged>,
    _settings_changed_receiver: Receiver<SettingsChanged>,
    _playback_controls_sender: std::sync::mpsc::Sender<PlaybackControls>,
}

// Implement Send and Sync for SongControlThread
unsafe impl Send for SongControlThread {}
unsafe impl Sync for SongControlThread {}
impl SongControlThread {
    pub fn init(starting_dir: String, mut player: MusicPlayer) -> SongControlThread {
        // Define all our default settings
        let _cur_settings = SettingsChanged {
            recursive: false,
            repeat: false,
            random: true,
            playing_dir: String::new(),
            browsing_dir: starting_dir.clone(),
            song_playing: String::new(),
            song_time: (0, 0),
        };

        // create the crossbeam letting the single Controller notify as many listeners that care
        // about changes in the controller state, such as settings changing or playback duration
        let (_settings_changed_sender, _settings_changed_receiver) = unbounded::<SettingsChanged>();

        let pb_controls = player.get_playback_controls();

        SongControlThread {
            _queued_music_files: Vec::new(),
            _cur_playing_index: -1,
            _player: player,
            _cur_settings,
            _settings_changed_sender,
            _settings_changed_receiver,
            _playback_controls_sender: pb_controls,
        }
    }

    pub fn set_random(&mut self, is_random: bool) {
        self._cur_settings.random = is_random;
        SongControlThread::send_settings(self);
    }

    pub fn set_repeat_all(&mut self, is_repeat_all: bool) {
        self._cur_settings.repeat = is_repeat_all;
        SongControlThread::send_settings(self);
    }

    pub fn set_recursive(&mut self, is_recursive: bool) {
        self._cur_settings.recursive = is_recursive;
        SongControlThread::send_settings(self);
    }

    pub fn pause(&mut self) {
        self._playback_controls_sender.send(PlaybackControls::Pause(true)).unwrap();
    }

    pub fn stop(&mut self) {
        self._playback_controls_sender.send(PlaybackControls::Stop(true)).unwrap();
    }

    pub fn play(&mut self) {
        self._playback_controls_sender.send(PlaybackControls::Play(true)).unwrap();
    }

    fn send_settings(&mut self) {
        self._settings_changed_sender.send(self._cur_settings.clone()).unwrap();
    }

    pub fn register_settings_listener(&mut self) -> Receiver<SettingsChanged> {
        self._settings_changed_receiver.clone()
    }

    pub fn get_browsing_dir(&self) -> String {
        self._cur_settings.browsing_dir.clone()
    }

    pub fn play_browsing_dir(&mut self) -> io::Result<()> {
        self._cur_settings.playing_dir = self._cur_settings.browsing_dir.clone();
        match file_utils::list_music_files(&self._cur_settings.playing_dir, self._cur_settings.recursive) {
            Err(e) => {
                eprintln!("No music files found: {}", e);
                return Err(e);
            },
            Ok(files) => {
                println!("Successfully read {} music files from {}", files.len(), self._cur_settings.playing_dir);
                self._queued_music_files = files;
            }
        };
        self.play_next_song();
        Ok(())
    }


    fn play_next_song(&mut self) {
        if self._cur_settings.random {
            // play a random song
            let mut rng = rand::thread_rng();
            let random_index = rng.gen_range(0..self._queued_music_files.len());
            self._cur_playing_index = random_index.try_into().unwrap();
            println!("Playing next random song at index {}: {}", self._cur_playing_index, self._queued_music_files[self._cur_playing_index as usize].display());
        } else {
            // play the next song in the queue

            // figure out the index of the next song to play
            let next_index: usize;
            if -1 == self._cur_playing_index{
                // -1 indicates we aren't playing a song yet
                next_index = 0;
            } else {
                next_index = self._cur_playing_index as usize + 1;
            }
            self._cur_playing_index = next_index.try_into().unwrap();

            if next_index >= self._queued_music_files.len() {
                if self._cur_settings.repeat {
                    self._cur_playing_index = 0;
                    println!("Last song played. Starting over with: {}", self._queued_music_files[0].display());
                } else {
                    println!("End of playlist. No more songs to play.");
                    return;
                }
            }
            else {
                println!("Playing next song: {}", self._queued_music_files[next_index].display());
            }
        }
        // actually play the song, regardless of whether it was randomly or sequentially chosen
        let song_to_play = self._queued_music_files[self._cur_playing_index as usize].clone();
        self.play_song(&song_to_play);
    }


    pub fn play_song(&mut self, song: &std::path::PathBuf) {
        self._cur_settings.song_playing = song.to_str().unwrap().to_string();
        self._cur_settings.song_time = (0, 0);

        match self._player.play_music_file(song) {
            Err(e) => {
                eprintln!("Failed to play file: {}", e);
            }
            Ok(_) => {}
        };
    }
}

pub struct Controller {
    _mon_song_thread: MonSongThread,
    _song_ctrl_thread: Arc<Mutex<SongControlThread>>,
}

impl Controller {
    pub fn init(starting_dir: String) -> Controller {
        let (notifier, listener) = std::sync::mpsc::channel::<PlaybackStatus>();        
        let player: MusicPlayer = MusicPlayer::init(notifier);

        let sct: SongControlThread = SongControlThread::init(starting_dir.clone(), player);

        let _song_ctrl_thread: Arc<Mutex<SongControlThread>> = Arc::new(Mutex::new(sct));
        let _mon_song_thread: MonSongThread = MonSongThread::init(Arc::clone(&_song_ctrl_thread), listener);

        Controller {
            _mon_song_thread,
            _song_ctrl_thread,
        }
    }

    pub fn set_random(&mut self, is_random: bool) {
        self._song_ctrl_thread.lock().unwrap().set_random(is_random);
    }

    pub fn set_repeat_all(&mut self, is_repeat_all: bool) {
        self._song_ctrl_thread.lock().unwrap().set_repeat_all(is_repeat_all);
    }

    pub fn set_recursive(&mut self, is_recursive: bool) {
        self._song_ctrl_thread.lock().unwrap().set_recursive(is_recursive);
    }

    pub fn register_settings_listener(&mut self) -> Receiver<SettingsChanged> {
        println!("Controller: About to register");
        self._song_ctrl_thread.lock().unwrap().register_settings_listener()
    }

    pub fn get_available_dirs(&self) -> io::Result<Vec<String>> {
        let sub_dirs_res: Result<Vec<String>, std::io::Error> = file_utils::sub_directories(&self._song_ctrl_thread.lock().unwrap().get_browsing_dir());
        match sub_dirs_res {
            Ok(sub_dirs) => Ok(sub_dirs),
            Err(e) => Err(e),
        }
    }

    pub fn play_browsing_dir(&mut self) -> io::Result<()> {
        self._song_ctrl_thread.lock().unwrap().play_browsing_dir()
    }

    pub fn play_song(&mut self, song: &std::path::PathBuf) {
        self._song_ctrl_thread.lock().unwrap().play_song(song)
    }

    pub fn play(&mut self) {
        self._song_ctrl_thread.lock().unwrap().play();
    }

    pub fn pause(&mut self) {
        self._song_ctrl_thread.lock().unwrap().pause();
    }

    pub fn stop(&mut self) {
        self._song_ctrl_thread.lock().unwrap().stop();
    }

    pub fn next(&mut self) {
        self._song_ctrl_thread.lock().unwrap().play_next_song();
    }
}