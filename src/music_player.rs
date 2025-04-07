use std::fs;
use std::io::{self, BufReader};
use std::os::unix::thread;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread::{sleep, JoinHandle, Thread};
use rodio::{Decoder, OutputStream, Sink, Source};

struct CurSong {
    _file_name: String,
    _duration: u32,
    // _sink: Sink,
    _stream: OutputStream,
    _stream_handle: rodio::OutputStreamHandle,
    _thread: JoinHandle<()>,
}

// Struct responsible for playing MP3 files
pub struct MusicPlayer {
    end_of_song_notifier: std::sync::mpsc::Sender<PlaybackStatus>,
    _playback_controls_sender: std::sync::mpsc::Sender<PlaybackControls>,
    _playback_controls_receiver: std::sync::mpsc::Receiver<PlaybackControls>,
    _cur_song: Option<Arc<Mutex<CurSong>>>,
}

pub enum PlaybackControls {
    Pause(bool),
    Play(bool),
    Stop(bool),
}

pub enum PlaybackStatus {
    PlaybackComplete(bool),
    // The pair is u32 elasped seconds, u32 total seconds
    PlaybackPercentage(u32, u32)
}

impl MusicPlayer {

    pub fn init(song_over_notifier: std::sync::mpsc::Sender<PlaybackStatus>) -> MusicPlayer {
        let (playback_controls_sender, playback_controls_receiver) = std::sync::mpsc::channel::<PlaybackControls>();
        MusicPlayer {
            end_of_song_notifier: song_over_notifier,
            _playback_controls_sender: playback_controls_sender,
            _playback_controls_receiver: playback_controls_receiver,
            _cur_song: None,
        }
    }

    pub fn get_playback_controls(&mut self) -> std::sync::mpsc::Sender<PlaybackControls> {
        self._playback_controls_sender.clone()
    }

    pub fn play_music_file(&mut self, file_path: &PathBuf) -> io::Result<()> {

        // TODO: Kill thread if one is already playing. Will require making _sink available to curSon


        println!("Playing {}", file_path.to_str().unwrap().to_string());
        let cur_playing_file_name = Some(file_path.to_str().unwrap().to_string());

        // Open the MP3 file and decode it for playback
        let file = fs::File::open(file_path)?;
        let source = Decoder::new(BufReader::new(file)).expect("Failed to decode music file");

        let song_duration: u32 = source.total_duration().unwrap().as_secs().try_into().unwrap();

        let (stream, stream_handle) = OutputStream::try_default().expect("Failed to open audio output stream");
        let sink = Sink::try_new(&stream_handle).expect("Failed to create sink");
        sink.append(source);
        sink.play();

        let eosn = self.end_of_song_notifier.clone();
        let thread = std::thread::spawn(move || {
            while !sink.empty() {
                let cur_pos: u32 = sink.get_pos().as_secs().try_into().unwrap();
                match eosn.send(PlaybackStatus::PlaybackPercentage(cur_pos, song_duration)) {
                    Ok(_) => (),
                    Err(e) => eprintln!("Failed to send playback percentage: {}", e),
                }
                // TODO: read _playback_controls_receiver to see if the song is supposed to play/pause/stop
                sleep(std::time::Duration::from_millis(1000));
            }
        });

        self._cur_song = Some(Arc::new(Mutex::new(CurSong {
            _file_name: cur_playing_file_name.unwrap(),
            _duration: song_duration,
            // _sink: sink,
            _stream: stream,
            _stream_handle: stream_handle,
            _thread: thread,
        })));

        Ok(())
    }
}