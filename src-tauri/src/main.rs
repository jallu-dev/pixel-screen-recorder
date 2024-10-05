#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
//final
use std::process::Stdio;
use std::sync::Arc;
use std::path::{PathBuf,Path};
use lazy_static::lazy_static;
use std::io::Write;
use chrono::Local;
use std::fs;
use tauri::{AppHandle,Manager,Window};
use tokio::process::Command; // Use Tokio's async Command
use tokio::process::Child;
use tokio::sync::Mutex;
use tokio::io::AsyncWriteExt;
use std::os::windows::process::CommandExt; // For `creation_flags
use tokio::io::{AsyncBufReadExt, BufReader};
use regex::Regex;
use tauri::Emitter;

lazy_static! {
    static ref VIDEO_RECORDING_PROCESS: Arc<Mutex<Option<Child>>> = Arc::new(Mutex::new(None));
    static ref AUDIO_RECORDING_PROCESS: Arc<Mutex<Option<Child>>> = Arc::new(Mutex::new(None));
    static ref TRIM_PROCESS: Arc<Mutex<Option<Child>>> = Arc::new(Mutex::new(None));
    static ref EDIT_PROCESS: Arc<Mutex<Option<Child>>> = Arc::new(Mutex::new(None));
    static ref COMPRESS_PROCESS: Arc<Mutex<Option<Child>>> = Arc::new(Mutex::new(None));
    static ref CURRENT_RECORDING_FILENAME: Mutex<String> = Mutex::new(String::new());
    static ref DEFAULT_RECORDING_PATH: Mutex<String> = Mutex::new(String::new());
}

const CREATE_NO_WINDOW: u32 = 0x08000000; // Hide the window

// async fn kill_processes() -> Result<(), String> {
//     let mut video_recording_state = VIDEO_RECORDING_PROCESS.lock().await;
//     let mut audio_recording_state = AUDIO_RECORDING_PROCESS.lock().await;
//     let mut trim_state = TRIM_PROCESS.lock().await;
//     let mut edit_state = EDIT_PROCESS.lock().await;
//     let mut compress_state = COMPRESS_PROCESS.lock().await;

//     if let Some(mut video_child) = video_recording_state.take() {
//         if let Err(e) = video_child.kill().await {
//             println!("Failed to kill video process: {:?}", e);
//             return Err("Failed to kill video process".into());
//         }
//         println!("Video process killed successfully.");
//     }

//     if let Some(mut audio_child) = audio_recording_state.take() {
//         if let Err(e) = audio_child.kill().await {
//             println!("Failed to kill audio process: {:?}", e);
//             return Err("Failed to kill audio process".into());
//         }
//         println!("Audio process killed successfully.");
//     }

//     if let Some(mut trim_child) = trim_state.take() {
//         if let Err(e) = trim_child.kill().await {
//             println!("Failed to kill trim process: {:?}", e);
//             return Err("Failed to kill trim process".into());
//         }
//         println!("trim process killed successfully.");
//     }

//     if let Some(mut edit_child) = edit_state.take() {
//         if let Err(e) = edit_child.kill().await {
//             println!("Failed to kill edit process: {:?}", e);
//             return Err("Failed to kill edit process".into());
//         }
//         println!("edit process killed successfully.");
//     }

//     if let Some(mut compress_child) = compress_state.take() {
//         if let Err(e) = compress_child.kill().await {
//             println!("Failed to kill compress process: {:?}", e);
//             return Err("Failed to kill compress process".into());
//         }
//         println!("compress process killed successfully.");
//     }

    

//     Ok(())
// }

// Function to convert duration in "HH:MM:SS" format to total seconds
fn parse_duration(duration: &str) -> Option<f32> {
    let parts: Vec<&str> = duration.split(':').collect();
    
    if parts.len() == 3 {
        let hours: f32 = parts[0].parse().unwrap_or(0.0);
        let minutes: f32 = parts[1].parse().unwrap_or(0.0);
        let seconds: f32 = parts[2].parse().unwrap_or(0.0);
        Some(hours * 3600.0 + minutes * 60.0 + seconds)
    } else {
        None
    }
}

fn extract_progress_from_line(line: &str) -> Option<f32> {
    let re = Regex::new(r"time=(\d+):(\d+):(\d+\.\d+)").unwrap();
    if let Some(caps) = re.captures(line) {
        let hours: f32 = caps[1].parse().unwrap_or(0.0);
        let minutes: f32 = caps[2].parse().unwrap_or(0.0);
        let seconds: f32 = caps[3].parse().unwrap_or(0.0);
        let total_seconds = hours * 3600.0 + minutes * 60.0 + seconds;
        return Some(total_seconds);
    }
    None
}

fn calculate_progress(current_time: f32, total_duration: f32) -> f32 {
    (current_time / total_duration) * 100.0
}

#[tauri::command]
async fn edit_and_compress(window: Window,file_location: String, mute: String, speed: String, trim_start_hr: String, trim_start_min: String,
    trim_start_sec: String, trim_end_hr: String, trim_end_min: String, trim_end_sec: String, crop_x: String, crop_y: String, crop_width: String, crop_height: String, duration_hr: String, duration_min: String, duration_sec: String, screen_width: String, screen_height: String) -> Result<String, String> {

    // Get the file name from the video path
    let path = Path::new(&file_location);
    let file_name = match path.file_name() {
        Some(name) => name.to_string_lossy().to_string(),
        None => return Err("Invalid video path".into()),
    };


    // Create the output file name with the 'Edited_' and 'Compressed_' 
    let temp_output_file_name = file_name.replace("Recording", "Temp");
    let temp_output_path = path.with_file_name(temp_output_file_name);

    let edited_output_file_name = file_name.replace("Recording", "Edited");
    let edited_output_path = path.with_file_name(edited_output_file_name);

    let compressed_output_file_name = file_name.replace("Recording", "Compressed");
    let compressed_output_path = path.with_file_name(compressed_output_file_name);

    let start_time = format!("{}:{}:{}", trim_start_hr, trim_start_min, trim_start_sec);
    let end_time = format!("{}:{}:{}", trim_end_hr, trim_end_min, trim_end_sec);
    let duration =  format!("{}:{}:{}", duration_hr, duration_min, duration_sec);
    let video_filter = format!("crop={}:{}:{}:{},setpts={}*PTS", crop_width, crop_height, crop_x, crop_y, speed);
    let video_filter_default = format!("crop=0:0:{}:{},setpts=1*PTS", screen_width, screen_height);

    let total_duration = parse_duration(&duration).unwrap_or(0.0);

    let filter = if video_filter == video_filter_default {
        println!("no crop and speed");
        format!("")
    } else if speed=="1" {
        println!("only crop");
        format!("crop={}:{}:{}:{}", crop_width, crop_height, crop_x, crop_y)
    } else if screen_width==crop_width&&screen_height==crop_height&&"0"==crop_x&&"0"==crop_y {
        println!("only speed");
        format!("setpts={}*PTS", speed)
    } else {
        println!("both filters");
        video_filter
    };

    if start_time != "0:0:0" || end_time != duration {
        let mut trim_args = vec![
            "-y","-progress", "pipe:1",
            "-i", &file_location,
            "-ss", &start_time,   
            "-to", &end_time,
            "-v", "error",
            temp_output_path.to_str().unwrap()    
        ];

        let mut trim_process_lock = TRIM_PROCESS.lock().await;

        let trim_ffmpeg_command = Command::new("ffmpeg")
            .args(&trim_args).stdout(Stdio::piped()) .creation_flags (CREATE_NO_WINDOW)
            .spawn();


        let mut trim_ffmpeg_child = match trim_ffmpeg_command {
            Ok(trim_ffmpeg_child) => {
                println!("FFmpeg Video trimming on progress");
                *trim_process_lock = Some(trim_ffmpeg_child); // Store the child process in the lock
                trim_process_lock.as_mut().unwrap() // Return the child to use for waiting
            },
            Err(e) => {
                println!("Failed to start video trimming: {:?}", e);
                return Err("Failed to start video triming".into());
            }
        };

        if let Some(stdout) = trim_ffmpeg_child.stdout.take() {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();
        
            // Process each line from stderr
            while let Some(line) = lines.next_line().await.expect("Failed to read line") {
                if line.contains("out_time=") {
                    if let Some(current_time) = extract_progress_from_line(&line) {
                        let progress = calculate_progress(current_time, total_duration);
                        // println!("Progress: {:.2}%", progress); // Notify user with percentage
                        // Emit progress update to frontend
                        window.emit("trim-progress", progress).expect("Failed to emit trim progress");
                    }
                }
            }
        }


        
        // Wait for the editing process to complete
        let trim_result: Result<PathBuf, String>  = match trim_ffmpeg_child.wait().await {
            Ok(status) => {
                if status.success() {
                    println!("Video trimed successfully: {}", temp_output_path.display());
                    Ok(temp_output_path.clone()) // Return the edited file path if successful
                } else {
                    return Err("Video triming failed.".into());
                }
            }
                    Err(err) => return Err(format!("Error while waiting for FFmpeg trim process: {}", err)),
        };

        let trim_output = trim_result?; // Use the result of the editing process (edited output path)

        let mut filter_args = if filter == "" {
            vec![
                "-y","-progress", "pipe:1",
                "-i", trim_output.to_str().unwrap(),
            ]
        } else {
            vec![
                "-y","-progress", "pipe:1",
                "-i", trim_output.to_str().unwrap(),
                "-filter:v", &filter, // Now using the variable instead of temporary value
            ]
        };
        

        // Handle audio speed adjustment based on the provided speed (0.5x, 1x, 2x, 4x, 10x)
        match speed.as_str() {
            "2" => {
                filter_args.push("-filter:a");
                filter_args.push("atempo=0.5");
            }
            "1" => {
                // No change to audio for 1x speed
            }
            "0.5" => {
                filter_args.push("-filter:a");
                filter_args.push("atempo=2.0");
            }
            "0.25" => {
                filter_args.push("-filter:a");
                filter_args.push("atempo=2.0,atempo=2.0"); // Chain two atempo filters for 4x speed
            }
            "0.1" => {
                filter_args.push("-filter:a");
                filter_args.push("atempo=2.0,atempo=2.0,atempo=2.5"); // Chain filters for 10x speed
            }
            _ => {
                return Err("Invalid speed provided".into());
            }
        }

        // Handle mute condition
        if mute == "-an" {
            filter_args.push("-an"); // Mute audio
        } else {
            filter_args.push("-c:a");
            filter_args.push("aac"); // Re-encode audio as AAC
        }

        filter_args.push(edited_output_path.to_str().unwrap());

        let mut edit_process_lock = EDIT_PROCESS.lock().await;

        let edit_ffmpeg_command = Command::new("ffmpeg")
            .args(&filter_args).stdout(Stdio::piped()).creation_flags (CREATE_NO_WINDOW)
            .spawn();

        let mut edit_ffmpeg_child = match edit_ffmpeg_command {
            Ok(edit_ffmpeg_child) => {
                println!("FFmpeg Video editing in progress");
                *edit_process_lock = Some(edit_ffmpeg_child); // Store the child process in the lock
                edit_process_lock.as_mut().unwrap() // Return the child to use for waiting
            },
            Err(e) => {
                println!("Failed to start video editing: {:?}", e);
                return Err("Failed to start video editing".into());
            }
        };
        

        let after_trim_duration = parse_duration(&end_time).unwrap_or(0.0)-parse_duration(&start_time).unwrap_or(0.0);

        if let Some(stdout) = edit_ffmpeg_child.stdout.take() {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();
        
            // Process each line from stderr
            while let Some(line) = lines.next_line().await.expect("Failed to read line") {
                if line.contains("out_time=") {
                    if let Some(current_time) = extract_progress_from_line(&line) {
                        let progress = calculate_progress(current_time, after_trim_duration);
                        // println!("Progress: {:.2}%", progress); // Notify user with percentage
                        // Emit progress update to frontend
                        window.emit("edit-progress", progress).expect("Failed to emit edit progress");
                    }
                }
            }
        }


        // Wait for the editing process to complete
        let edit_result: Result<PathBuf, String>  =  match edit_ffmpeg_child.wait().await {
            Ok(status) => {
                if status.success() {
                    println!("Video edited successfully: {}", edited_output_path.display());
                    Ok(edited_output_path.clone()) // Return the edited file path if successful
                } else {
                    return Err("Video editing failed.".into());
                }
            }
            Err(err) => return Err(format!("Error while waiting for FFmpeg edit process: {}", err)),
        };
            

        fs::remove_file(&temp_output_path).map_err(|e| format!("Failed to delete temp file: {:?}", e))?;

        // 2. Once the editing is complete, proceed with compression
        let edit_output = edit_result?; // Use the result of the editing process (edited output path)

        let mut compress_process_lock = COMPRESS_PROCESS.lock().await;


        let compress_ffmpeg_command = Command::new("ffmpeg")
            .args(&[
                "-y","-progress", "pipe:1",
                "-i", edit_output.to_str().unwrap(), // Use the edited file path as input
                "-vcodec", "libx264", // Use a better compression codec
                "-crf", "28",         // Compression level (lower is better quality, higher is more compression)
                compressed_output_path.to_str().unwrap(),
            ]).stdout(Stdio::piped()).creation_flags (CREATE_NO_WINDOW)
            .spawn();

        let mut compress_ffmpeg_child = match compress_ffmpeg_command {
            Ok(compress_ffmpeg_child) => {
                println!("FFmpeg Video compressing on progress");
                *compress_process_lock = Some(compress_ffmpeg_child); // Store the child process in the lock
                compress_process_lock.as_mut().unwrap() // Return the child to use for waiting
            },
            Err(e) => {
                println!("Failed to start video compressing: {:?}", e);
                return Err("Failed to start video compressing".into());
            }
        };

        if let Some(stdout) = compress_ffmpeg_child.stdout.take() {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();
        
            // Process each line from stderr
            while let Some(line) = lines.next_line().await.expect("Failed to read line") {
                if line.contains("out_time=") {
                    if let Some(current_time) = extract_progress_from_line(&line) {
                        let progress = calculate_progress(current_time, after_trim_duration);
                        // println!("Progress: {:.2}%", progress); // Notify user with percentage
                        // Emit progress update to frontend
                        window.emit("compress-progress", progress).expect("Failed to emit compress progress");
                    }
                }
            }
        }
                
        // Wait for the compression process to complete

        match compress_ffmpeg_child.wait().await {
            Ok(status) => {
                if status.success() {
                    // Return both the edited and compressed file paths
                    return Ok(format!("{}|{}", edit_output.to_str().unwrap(), compressed_output_path.to_str().unwrap()));
                } else {
                    return Err("Video compression failed.".into());
                }
            }
            Err(err) => return Err(format!("Error while waiting for FFmpeg compress process: {}", err)),
        };
    } else {
        let mut filter_args = if filter == "" {
            vec![
                "-y","-progress", "pipe:1",
                "-i", &file_location,
            ]
        }  else {
            vec![
                "-y","-progress", "pipe:1",
                "-i", &file_location,
                "-filter:v", &filter, // Now using the variable instead of temporary value
            ]
        };

        // Handle audio speed adjustment based on the provided speed (0.5x, 1x, 2x, 4x, 10x)
        match speed.as_str() {
            "2" => {
                filter_args.push("-filter:a");
                filter_args.push("atempo=0.5");
            }
            "1" => {
                // No change to audio for 1x speed
            }
            "0.5" => {
                filter_args.push("-filter:a");
                filter_args.push("atempo=2.0");
            }
            "0.25" => {
                filter_args.push("-filter:a");
                filter_args.push("atempo=2.0,atempo=2.0"); // Chain two atempo filters for 4x speed
            }
            "0.1" => {
                filter_args.push("-filter:a");
                filter_args.push("atempo=2.0,atempo=2.0,atempo=2.5"); // Chain filters for 10x speed
            }
            _ => {
                return Err("Invalid speed provided".into());
            }
        }

        // Handle mute condition
        if mute == "-an" {
            filter_args.push("-an"); // Mute audio
        } else {
            filter_args.push("-c:a");
            filter_args.push("aac"); // Re-encode audio as AAC
        }

        filter_args.push(edited_output_path.to_str().unwrap());

        let mut edit_process_lock = EDIT_PROCESS.lock().await;

        let edit_ffmpeg_command = Command::new("ffmpeg")
            .args(&filter_args).stdout(Stdio::piped()).creation_flags (CREATE_NO_WINDOW)
            .spawn();

        let mut edit_ffmpeg_child = match edit_ffmpeg_command {
            Ok(edit_ffmpeg_child) => {
                println!("FFmpeg Video editing in progress");
                *edit_process_lock = Some(edit_ffmpeg_child); // Store the child process in the lock
                edit_process_lock.as_mut().unwrap() // Return the child to use for waiting
            },
            Err(e) => {
                println!("Failed to start video editing: {:?}", e);
                return Err("Failed to start video editing".into());
            }
        };

        if let Some(stdout) = edit_ffmpeg_child.stdout.take() {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();
        
            // Process each line from stderr
            while let Some(line) = lines.next_line().await.expect("Failed to read line") {
                if line.contains("out_time=") {
                    if let Some(current_time) = extract_progress_from_line(&line) {
                        let progress = calculate_progress(current_time, total_duration);
                        // println!("Progress: {:.2}%", progress); // Notify user with percentage
                        // Emit progress update to frontend
                        window.emit("edit-progress", progress).expect("Failed to emit edit progress");
                    }
                }
            }
        }


        // Wait for the editing process to complete
        let edit_result: Result<PathBuf, String>  =  match edit_ffmpeg_child.wait().await {
            Ok(status) => {
                if status.success() {
                    println!("Video edited successfully: {}", edited_output_path.display());
                    Ok(edited_output_path.clone()) // Return the edited file path if successful
                } else {
                    return Err("Video editing failed.".into());
                }
            }
            Err(err) => return Err(format!("Error while waiting for FFmpeg edit process: {}", err)),
        };
            

        // 2. Once the editing is complete, proceed with compression
        let edit_output = edit_result?; // Use the result of the editing process (edited output path)

        let mut compress_process_lock = COMPRESS_PROCESS.lock().await;


        let compress_ffmpeg_command = Command::new("ffmpeg")
            .args(&[
                "-y","-progress", "pipe:1",
                "-i", edit_output.to_str().unwrap(), // Use the edited file path as input
                "-vcodec", "libx264", // Use a better compression codec
                "-crf", "28",         // Compression level (lower is better quality, higher is more compression)
                compressed_output_path.to_str().unwrap(),
            ]).stdout(Stdio::piped()).creation_flags (CREATE_NO_WINDOW)
            .spawn();

        let mut compress_ffmpeg_child = match compress_ffmpeg_command {
            Ok(compress_ffmpeg_child) => {
                println!("FFmpeg Video compressing on progress");
                *compress_process_lock = Some(compress_ffmpeg_child); // Store the child process in the lock
                compress_process_lock.as_mut().unwrap() // Return the child to use for waiting
            },
            Err(e) => {
                println!("Failed to start video compressing: {:?}", e);
                return Err("Failed to start video compressing".into());
            }
        };

        if let Some(stdout) = compress_ffmpeg_child.stdout.take() {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();
        
            // Process each line from stderr
            while let Some(line) = lines.next_line().await.expect("Failed to read line") {
                if line.contains("out_time=") {
                    if let Some(current_time) = extract_progress_from_line(&line) {
                        let progress = calculate_progress(current_time, total_duration);
                        // println!("Progress: {:.2}%", progress); // Notify user with percentage
                        // Emit progress update to frontend
                        window.emit("compress-progress", progress).expect("Failed to emit compress progress");
                    }
                }
            }
        }

                
        // Wait for the compression process to complete

        match compress_ffmpeg_child.wait().await {
            Ok(status) => {
                if status.success() {
                    // Return both the edited and compressed file paths
                    return Ok(format!("{}|{}", edit_output.to_str().unwrap(), compressed_output_path.to_str().unwrap()));
                } else {
                    return Err("Video compression failed.".into());
                }
            }
            Err(err) => return Err(format!("Error while waiting for FFmpeg compress process: {}", err)),
        }
    };
        
}      


#[tauri::command]
fn open_default_video_directory(app: AppHandle) -> Result<(), String> {
    let output_path_pathbuf = match app.path().video_dir() {
        Ok(path_buf) => path_buf,
        Err(_) => return Err("Could not find default video directory".to_string()),
    };

    let result = std::process::Command::new("explorer")
        .args([output_path_pathbuf.to_str().unwrap()]).stderr(Stdio::null()).creation_flags (CREATE_NO_WINDOW)
        .status();

    if result.is_err() {
        return Err("Could not find default video directory".to_string());
    }

    Ok(())
}

// Function to get default output path based on the user's video directory
// Function to get default output path based on the user's video directory
async fn get_default_output_file(app: &AppHandle) -> Result<String, String> {
    let mut output_path_pathbuf = match app.path().video_dir() {
        Ok(path_buf) => path_buf,
        Err(_) => return Err("Could not find default video directory".to_string()),
    };    

    //let mut output_path_pathbuf: PathBuf = PathBuf::from(video_path);
    let mut default_path = DEFAULT_RECORDING_PATH.lock().await;
    *default_path = output_path_pathbuf.to_string_lossy().to_string();

    // Get current date and time
    let now = Local::now();
    let formatted_time = now.format("%Y-%m-%d_%H-%M-%S").to_string();
    
    // Create the file name with date and time
    let file_name = format!("Recording_{}.mp4", formatted_time);
    let mut current_file_name = CURRENT_RECORDING_FILENAME.lock().await;
    *current_file_name = file_name.clone();

    output_path_pathbuf.push(file_name);
    let output_path = output_path_pathbuf.to_string_lossy().to_string();
    Ok(output_path)
}


async fn merge_videos(file1: &str, file2: &str, app: &AppHandle) -> Result<PathBuf, String> {
    // Check if file1 and file2 are the same file
    if Path::new(file1) == Path::new(file2) {
        println!("File1 and File2 are the same. Skipping merge.");
        return Ok(Path::new(file1).to_path_buf()); // No need to merge

    }

    let file_list = "don't_delete.txt"; // Temporary file to store the file paths
    let merged_file = format!("{}.merged.mp4", file1); // Temporary name for the merged file

    // Create a list of the files to be merged
    let list_content = format!("file '{}'\nfile '{}'", file1, file2);

    let mut file_list_path = match app.path().video_dir() {
        Ok(path_buf) => path_buf,
        Err(_) => return Err("Could not find default video directory".to_string()),
    };  

    file_list_path.push(file_list);

    // Write the list of files to a text file
    let mut file = fs::File::create(&file_list_path).map_err(|e| format!("Failed to create file list: {:?}", e))?;
    file.write_all(list_content.as_bytes()).map_err(|e| format!("Failed to write to file list: {:?}", e))?;

    // Command to merge the videos using the file list
    let command = format!("ffmpeg -f concat -safe 0 -i {} -c copy {}", file_list_path.to_string_lossy(), merged_file);

    // Run the command using the correct shell for the OS
    let status = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(&["/C", &command]).stderr(Stdio::null()).creation_flags (CREATE_NO_WINDOW)
            .status().await
    } else {
        Command::new("sh")
            .arg("-c")
            .arg(&command).stderr(Stdio::null()).creation_flags (CREATE_NO_WINDOW)
            .status().await
    };

    // Handle the result of the command execution
    let status = status.map_err(|e| format!("Failed to execute merge command: {:?}", e))?;

    // Check if the merging was successful
    if status.success() {
        // Delete the original files after merging
        fs::remove_file(&file1).map_err(|e| format!("Failed to delete file1: {:?}", e))?;
        fs::remove_file(&file2).map_err(|e| format!("Failed to delete file2: {:?}", e))?;

        // Rename the merged file to file1's name
        fs::rename(&merged_file, &file1).map_err(|e| format!("Failed to rename merged file: {:?}", e))?;

        // Clean up the temporary file list
        fs::remove_file(&file_list_path).map_err(|e| format!("Failed to delete file list: {:?}", e))?;

        Ok(Path::new(&file1).to_path_buf())
    } else {
        Err(format!("Merge command failed with status: {:?}", status))
    }
}


async fn merge_video_and_audio(video: &str, audio: &str) -> Result<PathBuf, String> {
    let video_path = PathBuf::from(video).to_string_lossy().to_string(); // Copy video path
    let audio_path = PathBuf::from(audio).to_string_lossy().to_string(); // Copy audio pat
    let merged_file = format!("{}.merged_video_and_audio.mp4", video_path); // Temporary name for the merged file

    // Command to merge the videos using the file list
    let command = format!("ffmpeg -i {} -i {} -c:v copy -c:a aac -strict experimental {}", video_path, audio_path, merged_file);

    // Spawn a task to run the merging process
    let result = tokio::spawn(async move {
        // Run the command using the correct shell for the OS
        let status = if cfg!(target_os = "windows") {
            Command::new("cmd")
                .args(&["/C", &command]).stderr(Stdio::null()).creation_flags (CREATE_NO_WINDOW)
                .status().await
        } else {
            Command::new("sh")
                .arg("-c")
                .arg(&command).stderr(Stdio::null()).creation_flags (CREATE_NO_WINDOW)
                .status().await
        };

        // Handle the result of the command execution
        let status = status.map_err(|e| format!("Failed to execute merge command: {:?}", e))?;

        // Check if the merging was successful
        if status.success() {
            // Delete the original files after merging
            fs::remove_file(&video_path).map_err(|e| format!("Failed to delete video file: {:?}", e))?;
            fs::remove_file(&audio_path).map_err(|e| format!("Failed to delete audio file: {:?}", e))?;

            // Rename the merged file to file1's name
            fs::rename(&merged_file, &video_path).map_err(|e| format!("Failed to rename merged file: {:?}", e))?;

            Ok(Path::new(&video_path).to_path_buf())
        } else {
            Err(format!("Merge command failed with status: {:?}", status))
        }
    }).await;

    // Handle the result of the spawned task and flatten the nested Result
    result.map_err(|e| format!("Error merging video and audio: {:?}", e)).and_then(|result| result)
}


// Function to check if audio routing tools are installed (VB-Audio or BlackHole)
async fn check_audio_tools() -> Result<bool, String> {
    let platform = std::env::consts::OS;
    match platform {
        "windows" => {
            let output = Command::new("where")
                .arg("VirtualCable").stderr(Stdio::null())
                .output()
                .await.map_err(|e| format!("Failed to execute command: {}", e))?;


            if output.status.success() {
                Ok(true)
            } else {
                Err("VB-Audio Virtual Cable not found".into())
            }
        }
        "macos" => {
            let output = Command::new("brew")
                .args(&["list", "blackhole-16ch"]).stderr(Stdio::null()).creation_flags (CREATE_NO_WINDOW)
                .output()
                .await.map_err(|e| format!("Failed to execute command: {}", e))?;

            if output.status.success() {
                Ok(true)
            } else {
                Err("BlackHole not found".into())
            }
        }
        "linux" => {
            // On Linux, we can skip this check or include a similar check for PulseAudio configuration
            Ok(true)
        }
        _ => Err("Unsupported platform".into()),
    }
}

// Function to detect and get audio input devices for different platforms
async fn get_audio_input() -> Result<String, String> {
    let platform = std::env::consts::OS;
    match platform {
        "windows" => {
            // Windows: Use FFmpeg's dshow to list available devices
            let output = Command::new("ffmpeg")
                .args(&["-list_devices", "true", "-f", "dshow", "-i", "dummy"]).stderr(Stdio::null()).creation_flags (CREATE_NO_WINDOW)
                .output()
                .await.map_err(|e| format!("Failed to execute FFmpeg command: {}", e))?;

            let stdout = String::from_utf8_lossy(&output.stderr);
            let lines: Vec<&str> = stdout.split('\n').collect();
            for line in lines {
                if line.contains("audio") && line.contains("Microphone") {
                    let start_idx = line.find("\"").unwrap_or(0) + 1;
                    let end_idx = line.rfind("\"").unwrap_or(line.len());
                    let device = &line[start_idx..end_idx];
                    return Ok(device.to_string());
                }
            }
            Err("No microphone found".into())
        }
        "macos" => {
            // macOS: Use AVFoundation for device detection
            let output = Command::new("ffmpeg")
                .args(&["-f", "avfoundation", "-list_devices", "true", "-i", "dummy"]).stderr(Stdio::null()).creation_flags (CREATE_NO_WINDOW)
                .output()
                .await.map_err(|e| format!("Failed to execute FFmpeg command: {}", e))?;

            let stdout = String::from_utf8_lossy(&output.stderr);
            let lines: Vec<&str> = stdout.split('\n').collect();
            for line in lines {
                if line.contains("audio") {
                    let device_idx = line.split('[').nth(1).unwrap_or("0").split(']').next().unwrap_or("0");
                    return Ok(device_idx.to_string());
                }
            }
            Err("No audio input device found".into())
        }
        "linux" => {
            // Linux: Use PulseAudio to list available devices
            let output = Command::new("pactl")
                .arg("list")
                .arg("sources").stderr(Stdio::null())
                .output()
                .await.map_err(|e| format!("Failed to execute FFmpeg command: {}", e))?;

            let stdout = String::from_utf8_lossy(&output.stdout);
            let lines: Vec<&str> = stdout.split('\n').collect();
            for line in lines {
                if line.contains("Name:") && line.contains("microphone") {
                    let device = line.split_whitespace().nth(1).unwrap_or("default");
                    return Ok(device.to_string());
                }
            }
            Err("No microphone found".into())
        }
        _ => Err("Unsupported platform".into()),
    }
}

#[tauri::command]
async fn start_screen_recording(app: AppHandle, file_name:String) -> Result<String, String> {
    let platform = std::env::consts::OS;
    let video_output_path = get_default_output_file(&app).await?;
    let audio_output_path = video_output_path.replace(".mp4", "_audio.aac");

    // Check if the required audio tools are installed
    if let Err(tool_error) = check_audio_tools().await {
        println!("Audio routing tool not found: {}", tool_error);
    }

    let audio_input = match get_audio_input().await {
        Ok(input) => format!("audio={}", input),
        Err(_) => {
            println!("Using default microphone");
            "default".to_string()
        }
    };

    // Video recording command (without audio)
    let video_command = match platform {
        "windows" => vec![
            "ffmpeg".to_string(), "-f".to_string(), "gdigrab".to_string(),
            "-framerate".to_string(), "30".to_string(), "-i".to_string(), "desktop".to_string(),
            "-c:v".to_string(), "libx264".to_string(), "-preset".to_string(), "ultrafast".to_string(),
            "-crf".to_string(), "18".to_string(), "-pix_fmt".to_string(), "yuv420p".to_string(),
            video_output_path.clone()
        ],
        "macos" => vec![
            "ffmpeg".to_string(), "-f".to_string(), "avfoundation".to_string(),
            "-framerate".to_string(), "30".to_string(), "-i".to_string(), "1:0".to_string(),
            "-c:v".to_string(), "libx264".to_string(), "-preset".to_string(), "ultrafast".to_string(),
            "-crf".to_string(), "18".to_string(), "-pix_fmt".to_string(), "yuv420p".to_string(),
            video_output_path.clone()
        ],
        "linux" => vec![
            "ffmpeg".to_string(), "-f".to_string(), "x11grab".to_string(),
            "-framerate".to_string(), "30".to_string(), "-i".to_string(), ":0.0".to_string(),
            "-c:v".to_string(), "libx264".to_string(), "-preset".to_string(), "ultrafast".to_string(),
            "-crf".to_string(), "18".to_string(), "-pix_fmt".to_string(), "yuv420p".to_string(),
            video_output_path.clone()
        ],
        _ => return Err("Unsupported platform".into()),
    };

    // Audio recording command (audio-only)
    let audio_command = match platform {
        "windows" => vec![
            "ffmpeg".to_string(), "-f".to_string(), "dshow".to_string(),
            "-i".to_string(), audio_input,
            "-c:a".to_string(), "aac".to_string(), "-b:a".to_string(), "128k".to_string(),
            audio_output_path.clone()
        ],
        "macos" => vec![
            "ffmpeg".to_string(), "-f".to_string(), "avfoundation".to_string(),
            "-i".to_string(), "none:0".to_string(),
            "-c:a".to_string(), "aac".to_string(), "-b:a".to_string(), "128k".to_string(),
            audio_output_path.clone()
        ],
        "linux" => vec![
            "ffmpeg".to_string(), "-f".to_string(), "pulse".to_string(),
            "-i".to_string(), audio_input,
            "-c:a".to_string(), "aac".to_string(), "-b:a".to_string(), "128k".to_string(),
            audio_output_path.clone()
        ],
        _ => return Err("Unsupported platform".into()),
    };

    // Spawn the video recording process
    let video_command_clone = video_command.clone();
    tokio::spawn(async move {
        let mut process_lock = VIDEO_RECORDING_PROCESS.lock().await;

        let child = Command::new(&video_command_clone[0])
            .args(&video_command_clone[1..]).creation_flags (CREATE_NO_WINDOW)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn();

        *process_lock = match child {
            Ok(mut child) => {
                println!("FFmpeg Video Recording on progress");
                Some(child)
            },
            Err(e) => {
                println!("Failed to start video recording: {:?}", e);
                None
            }
        };
    });

    // Spawn the audio recording process
    let audio_command_clone = audio_command.clone();
    tokio::spawn(async move {
        let mut process_lock = AUDIO_RECORDING_PROCESS.lock().await;

        let child = Command::new(&audio_command_clone[0])
            .args(&audio_command_clone[1..]).creation_flags (CREATE_NO_WINDOW)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn();

        *process_lock = match child {
            Ok(mut child) => {
                println!("FFmpeg Audio Recording on progress");
                Some(child)
            },
            Err(e) => {
                println!("Failed to start audio recording: {:?}", e);
                None
            }
        };
    });


    let cur_recording_file = CURRENT_RECORDING_FILENAME.lock().await;
    Ok(cur_recording_file.clone())
}



#[tauri::command]
async fn stop_screen_recording(app: AppHandle, file_name: &str) -> Result<String, String> {
    let mut video_process_lock = VIDEO_RECORDING_PROCESS.lock().await;
    let mut audio_process_lock = AUDIO_RECORDING_PROCESS.lock().await;

    // Check and stop the video recording process
    if let Some(mut video_child) = video_process_lock.take() {
        if let Some(ref mut stdin) = video_child.stdin {
            println!("Stopping video recording (sending 'q')...");
            if let Err(e) = stdin.write_all(b"q\n").await {
                eprintln!("Failed to stop video recording: {:?}", e);
                return Err("Failed to stop video recording".into());
            }
        }

        // Wait for the video process to finish
        match video_child.wait().await {
            Ok(status) => {
                if status.success() {
                    println!("FFmpeg video process exited successfully.");
                } else {
                    return Err(format!("FFmpeg video process exited with error: {:?}", status));
                }
            }
            Err(e) => return Err(format!("Error waiting for FFmpeg video process: {:?}", e)),
        }
    } else {
        return Err("No video recording process found.".into());
    }

    // Check and stop the audio recording process
    if let Some(mut audio_child) = audio_process_lock.take() {
        if let Some(ref mut stdin) = audio_child.stdin {
            println!("Stopping audio recording (sending 'q')...");
            if let Err(e) = stdin.write_all(b"q\n").await {
                eprintln!("Failed to stop audio recording: {:?}", e);
                return Err("Failed to stop audio recording".into());
            }
        }

        // Wait for the audio process to finish
        match audio_child.wait().await {
            Ok(status) => {
                if status.success() {
                    println!("FFmpeg audio process exited successfully.");
                } else {
                    return Err(format!("FFmpeg audio process exited with error: {:?}", status));
                }
            }
            Err(e) => return Err(format!("Error waiting for FFmpeg audio process: {:?}", e)),
        }
    } else {
        return Err("No audio recording process found.".into());
    }

    // After stopping both video and audio, merge them
    let file_path = DEFAULT_RECORDING_PATH.lock().await;
    let cur_file_name = CURRENT_RECORDING_FILENAME.lock().await;

    // Path for the current recording video and the new file
    let video_output_path = format!("{}\\{}", file_path.clone(), cur_file_name);
    let audio_output_path = format!("{}\\{}", file_path.clone(), cur_file_name.clone().replace(".mp4", "_audio.aac"));

    println!("Merging videos and audio");
    println!("File 1: {}", video_output_path);
    println!("File 2: {}", audio_output_path);

    // Merge videos and return the path to the merged video
    let v_a_merged_path = match merge_video_and_audio(video_output_path.as_str(), audio_output_path.as_str()).await {
        Ok(path_buf) => path_buf.to_string_lossy().to_string(),
        Err(e) => {
            println!("Error merging video and audio: {}", e);
                String::new() // Return an empty string in case of error
        }
    };

    println!("file name 0 {}",file_name);

    // After both video and audio have stopped, merge the files
    if file_name != "" {
        println!("file name .0 {}",file_name);


        // let file_path = DEFAULT_RECORDING_PATH.lock().await;

        println!("file name 1.1 {}",file_name);

        // Path for the current recording video and the new file
        let file1 = format!("{}\\{}", file_path.clone(), file_name);
        let file2 = format!("{}", v_a_merged_path);
        println!("file name 1.2 {}",file_name);

        println!("Merging videos:");
        println!("File 1: {}", file1);
        println!("File 2: {}", file2);

        // Merge videos and return the path to the merged video
        let recorded_path = match merge_videos(file1.as_str(), file2.as_str(), &app).await {
            Ok(path_buf) => path_buf.to_string_lossy().to_string(),
            Err(e) => {
                println!("Error merging videos: {}", e);
                String::new() // Return an empty string in case of error
            }
        };
        println!("file name 1.3 {}",file_name);

        return Ok(recorded_path); // Return the merged video path
    } else {
        println!("file name 2 {}",file_name);

    }
    println!("file name 3 {}",file_name);

    Ok(String::new()) // Return Ok if no merge is needed or if the process exits successfully
}

#[tauri::command]
async fn is_recording() -> bool {
    let process_lock = VIDEO_RECORDING_PROCESS.lock().await;
    process_lock.is_some()
}

#[tokio::main]
async fn main() {
    tauri::Builder::default()        
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            start_screen_recording,
            stop_screen_recording,
            is_recording,
            open_default_video_directory,
            edit_and_compress
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

