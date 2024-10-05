import React, { useState, useEffect, useRef } from "react";
import { invoke, convertFileSrc } from "@tauri-apps/api/core";
import { BaseDirectory, stat } from "@tauri-apps/plugin-fs";
import "./App.css";
import VideoContainer from "./components/VideoContainer";
import SoundEdit from "./components/SoundEdit";
import SpeedEdit from "./components/SpeedEdit";
import TrimEdit from "./components/TrimEdit";
import CropEdit from "./components/CropEdit";
import Controllers from "./components/Controllers";
import { listen } from "@tauri-apps/api/event";

function App() {
  const [isRecording, setIsRecording] = useState(false);
  const [isPaused, setIsPaused] = useState(false);
  const [fileName, setFileName] = useState("");
  const [videoSrc, setVideoSrc] = useState("");
  const [fileLocation, setfileLocation] = useState("");
  const [editedVideoSrc, setEditedVideoSrc] = useState("");
  const [editedfileLocation, setEditedfileLocation] = useState("");
  const [compressedVideoSrc, setCompressedVideoSrc] = useState("");
  const [compressedfileLocation, setCompressedfileLocation] = useState("");
  const [previewState, setPreviewState] = useState("o");
  const [screenSize, setScreenSize] = useState({ width: 0, height: 0 });
  const [ratio, setRatio] = useState(0);
  const [isSaving, setIsSaving] = useState(false);
  const [isControllersActive, setIsControllersActive] = useState(true);
  const [progressRate, setProgressRate] = useState(0);
  const [currentEditProgress, setCurrentEditProgress] = useState("");
  const [fileSizes, setFileSizes] = useState({
    original: "",
    edited: "",
    compressed: "",
  });
  const [realCropArea, setRealCropArea] = useState({
    x: 0,
    y: 0,
    width: 0,
    height: 0,
  });
  const [fakeCropArea, setFakeCropArea] = useState({
    x: 0,
    y: 0,
    width: 0,
    height: 0,
  });
  const [fakingValue, setFakingValue] = useState(null);
  const [trimTime, setTrimTime] = useState({
    startHr: 0,
    startMin: 0,
    startSec: 0,
    endHr: 0,
    endMin: 0,
    endSec: 0,
  });
  const [duration, setDuration] = useState({ hr: 0, min: 0, sec: 0 });
  const videoRef = useRef(null);
  const [editDetails, setEditDetails] = useState({
    mute: "", //unmute
    speed: 1,
    trim: {
      startHr: 0,
      startMin: 0,
      startSec: 0,
      endHr: 0,
      endMin: 0,
      endSec: 0,
    },
    crop: {
      x: 0,
      y: 0,
      width: 0,
      height: 0,
    },
  });

  //useEffects
  useEffect(() => {
    // Check if recording is happening when the component mounts
    checkRecordingState();
  }, []);

  useEffect(() => {
    const editListen = async () => {
      const trimProgressListen = await listen("trim-progress", (event) => {
        let rate = `${Math.round(+event.payload)}%`;
        if (rate != progressRate) {
          setProgressRate(rate);
        }
        setCurrentEditProgress("trimming");
      });

      const editProgressListen = await listen("edit-progress", (event) => {
        let rate = `${Math.round(+event.payload)}%`;
        if (rate != progressRate) {
          setProgressRate(rate);
        }
        setCurrentEditProgress("editing");
      });

      const compressProgressListen = await listen(
        "compress-progress",
        (event) => {
          let rate = `${Math.round(+event.payload)}%`;
          if (rate != progressRate) {
            setProgressRate(rate);
          }
          setCurrentEditProgress("compressing");
        }
      );
    };

    editListen();

    return () => editListen();
  }, []);

  useEffect(() => {
    const handleResize = () => {
      setScreenSize({
        width: screen.width,
        height: screen.height,
      });
    };

    window.addEventListener("resize", handleResize);
    handleResize();

    return () => {
      window.removeEventListener("resize", handleResize);
    };
  }, []);

  useEffect(() => {
    const videoElement = videoRef.current;

    const handleLoadedMetadata = () => {
      let sec = Math.floor(videoElement.duration);
      let min = Math.floor(sec / 60);
      sec = sec % 60;
      let hr = Math.floor(min / 60);
      min = min % 60;

      setDuration({ hr, min, sec });
      console.log({ hr, min, sec });
    };

    // Set up the event listener
    videoElement?.addEventListener("loadedmetadata", handleLoadedMetadata);

    // Cleanup the event listener when the component unmounts
    return () => {
      videoElement?.removeEventListener("loadedmetadata", handleLoadedMetadata);
    };
  }, [isRecording, isPaused]);

  useEffect(() => {
    setTrimTime((obj) => {
      return {
        ...obj,
        endHr: duration.hr,
        endMin: duration.min,
        endSec: duration.sec,
      };
    });
  }, [duration]);

  useEffect(() => {
    let rat = screenSize.width / screenSize.height;
    setRatio(rat);

    setRealCropArea((obj) => {
      return { ...obj, width: screenSize.width, height: screenSize.height };
    });

    setFakeCropArea((obj) => {
      return { ...obj, width: rat * 100, height: 100 };
    });

    setFakingValue((rat * 100) / screenSize.width);
  }, [screenSize]);

  useEffect(() => {
    setFakeCropArea({
      x: realCropArea.x * fakingValue,
      y: realCropArea.y * fakingValue,
      width: realCropArea.width * fakingValue,
      height: realCropArea.height * fakingValue,
    });
  }, [realCropArea]);

  useEffect(() => {
    setEditDetails((obj) => {
      return {
        ...obj,
        crop: realCropArea,
      };
    });
  }, [realCropArea]);

  useEffect(() => {
    setEditDetails((obj) => {
      return {
        ...obj,
        trim: trimTime,
      };
    });
  }, [trimTime]);

  useEffect(() => {
    const findSizes = async () => {
      const calc = (byts) => {
        let size = byts / 1052085.44;
        if (Math.floor(size) <= 0) {
          return Math.round(byts / 1026.05208) + "KB";
        }
        size = (byts / 1052085.44).toFixed(2) + "MB";

        return size;
      };

      let originalStat = fileLocation
        ? await stat(fileLocation.slice(fileLocation.search("Reco")), {
            baseDir: BaseDirectory.Video,
          })
        : "";
      let editedStat = editedfileLocation
        ? await stat(
            editedfileLocation.slice(editedfileLocation.search("Edit")),
            { baseDir: BaseDirectory.Video }
          )
        : "";
      let compressedStat = compressedfileLocation
        ? await stat(
            compressedfileLocation.slice(compressedfileLocation.search("Comp")),
            { baseDir: BaseDirectory.Video }
          )
        : "";

      setFileSizes({
        original: videoSrc && originalStat ? calc(originalStat.size) : "",
        edited: editedVideoSrc && editedStat ? calc(editedStat.size) : "",
        compressed:
          compressedVideoSrc && compressedStat ? calc(compressedStat.size) : "",
      });
    };

    findSizes();
  }, [videoSrc, compressedVideoSrc, editedVideoSrc]);

  //Methods
  const onRealValuesChange = (e) => {
    setRealCropArea((obj) => {
      return { ...obj, [e.target.id]: e.target.value };
    });
  };

  const enforceMinMax = (el) => {
    if (el.target.value != "") {
      if (parseInt(el.target.value) < parseInt(el.target.min)) {
        el.target.value = el.target.min;
      }
      if (parseInt(el.target.value) > parseInt(el.target.max)) {
        el.target.value = el.target.max;
      }
    }
  };

  const onPreviewClicked = (e) => {
    setPreviewState(e.target.id);
  };

  const onSoundClick = (e) => {
    setEditDetails((obj) => {
      return {
        ...obj,
        mute: e.target.id,
      };
    });
  };

  const onSpeedClick = (e) => {
    setEditDetails((obj) => {
      return {
        ...obj,
        speed: e.target.id,
      };
    });
  };

  const onSaveClicked = async (e) => {
    console.log(editDetails);
    setIsSaving(true);
    try {
      const paths = await invoke("edit_and_compress", {
        mute: editDetails.mute.toString(),
        speed: editDetails.speed.toString(),
        trimStartHr: editDetails.trim.startHr.toString(),
        trimStartMin: editDetails.trim.startMin.toString(),
        trimStartSec: editDetails.trim.startSec.toString(),
        trimEndHr: editDetails.trim.endHr.toString(),
        trimEndMin: editDetails.trim.endMin.toString(),
        trimEndSec: editDetails.trim.endSec.toString(),
        cropX: editDetails.crop.x.toString(),
        cropY: editDetails.crop.y.toString(),
        cropWidth: editDetails.crop.width.toString(),
        cropHeight: editDetails.crop.height.toString(),
        durationHr: duration.hr.toString(),
        durationMin: duration.min.toString(),
        durationSec: duration.sec.toString(),
        screenWidth: screenSize.width.toString(),
        screenHeight: screenSize.height.toString(),
        fileLocation,
      });

      const [editedPath, compressedPath] = paths.split("|");
      console.log(editedPath, compressedPath);
      setEditedfileLocation(editedPath);
      const editedVideoUrl = convertFileSrc(editedPath);
      const editedVideoUrlWithCacheBuster = `${editedVideoUrl}?t=${new Date().getTime()}`;
      setEditedVideoSrc(editedVideoUrlWithCacheBuster);

      setCompressedfileLocation(compressedPath);
      const compressedVideoUrl = convertFileSrc(compressedPath);
      const compressedVideoUrlWithCacheBuster = `${compressedVideoUrl}?t=${new Date().getTime()}`;
      setCompressedVideoSrc(compressedVideoUrlWithCacheBuster);
      setIsSaving(false);
      setProgressRate("");
      setCurrentEditProgress("");
      setPreviewState("e");
    } catch (error) {
      console.log(error);
    }
  };

  useEffect(() => {
    if (trimTime.startHr >= trimTime.endHr) {
      if (trimTime.startMin > trimTime.endMin) {
        setTrimTime((obj) => {
          console.log(obj);
          return { ...obj, startMin: trimTime.endMin };
        });
      }
    }
  }, [trimTime]);

  useEffect(() => {
    if (trimTime.startHr >= trimTime.endHr) {
      if (trimTime.startMin >= trimTime.endMin) {
        if (trimTime.startSec > trimTime.endSec) {
          setTrimTime((obj) => {
            console.log(obj);
            return { ...obj, startSec: trimTime.endSec };
          });
        }
      }
    }
  }, [trimTime]);

  const onTrimTimeChanged = (e) => {
    setTrimTime((obj) => {
      console.log(obj);
      return { ...obj, [e.target.id]: e.target.value };
    });
  };

  const checkRecordingState = async () => {
    const recording = await invoke("is_recording");
    setIsRecording(recording);
  };

  //Controllers
  const startRecording = async () => {
    setIsControllersActive(false);
    try {
      const file = await invoke("start_screen_recording", { fileName });
      if (!fileName) {
        setFileName(file);
        setVideoSrc("");
        setEditedVideoSrc("");
        setCompressedVideoSrc("");
        setfileLocation("");
        setEditedfileLocation("");
        setCompressedfileLocation("");
        setIsSaving(false);
        setEditDetails((obj) => {
          return { ...obj, mute: "", speed: "1" };
        });
        setTrimTime((obj) => {
          return { ...obj, startHr: "0", startMin: "0", startSec: "0" };
        });
        setRealCropArea({
          x: "0",
          y: "0",
          width: screenSize.width,
          height: screenSize.height,
        });
      }
      console.log(file);
      setIsRecording(true);
      setIsPaused(false);
    } catch (error) {
      console.error("Error starting recording:", error);

      // If the error is related to missing audio tools, display a prompt
      if (error.message.includes("No audio tool found")) {
        if (
          window.confirm(
            "No audio tool found. Please install VB-CABLE or BlackHole. Do you want to download it?"
          )
        ) {
          // Redirect to the appropriate download link
          const platform = window.navigator.platform;
          if (platform.includes("Win")) {
            window.open("https://vb-audio.com/Cable/index.htm", "_blank");
          } else if (platform.includes("Mac")) {
            window.open("https://existential.audio/blackhole/", "_blank");
          }
        }
      }
    } finally {
      setIsControllersActive(true);
    }
  };

  const pausedRecording = async () => {
    setIsControllersActive(false);
    try {
      const last_recorder_video = await invoke("stop_screen_recording", {
        fileName,
      });
      console.log("Recording stopped, saved at:", last_recorder_video);
      setIsRecording(true);
      setIsPaused(true);
      if (last_recorder_video != "") {
        setfileLocation(last_recorder_video);
        const videoUrl = convertFileSrc(last_recorder_video);
        const videoUrlWithCacheBuster = `${videoUrl}?t=${new Date().getTime()}`;

        setVideoSrc(videoUrlWithCacheBuster);
      }
    } catch (error) {
      console.error("Error pausing recording:", error);
    } finally {
      setIsControllersActive(true);
    }
  };

  const stopRecording = async () => {
    setIsControllersActive(false);

    try {
      const last_recorder_video = await invoke("stop_screen_recording", {
        fileName,
      });
      console.log("Recording stopped, saved at:", last_recorder_video);
      if (last_recorder_video != "") {
        setfileLocation(last_recorder_video);
        const videoUrl = convertFileSrc(last_recorder_video);
        const videoUrlWithCacheBuster = `${videoUrl}?t=${new Date().getTime()}`;
        setVideoSrc(videoUrlWithCacheBuster);
      }
    } catch (error) {
      console.log("Error stopping recording:", error);
    } finally {
      setIsControllersActive(true);
      setIsRecording(false);
      setIsPaused(false);
      setFileName("");
      setEditedVideoSrc("");
      setCompressedVideoSrc("");
      setPreviewState("o");
    }
  };

  return (
    <div className="bg-zinc-900 h-full w-full">
      <Controllers
        isPaused={isPaused}
        isRecording={isRecording}
        startRecording={startRecording}
        pausedRecording={pausedRecording}
        stopRecording={stopRecording}
        isControllersActive={isControllersActive}
      />
      {videoSrc && (
        <div className="flex justify-around my-10 p-3 gap-2">
          <div className="w-[40%] shrink-0 rounded-md overflow-hidden shadow-2xl shadow-black bg-black">
            {fileLocation.slice(fileLocation.search("Reco")) != fileName && (
              <div className="text-center p-2  uppercase">
                <p className="text-xl text-zinc-200 ">Preview</p>
              </div>
            )}
            {fileLocation.slice(fileLocation.search("Reco")) != fileName && (
              <div className="flex justify-between p-3">
                <button
                  id="o"
                  onClick={(e) => onPreviewClicked(e)}
                  className={`${
                    previewState == "o" ? "bg-indigo-600" : "bg-zinc-600"
                  } hover:bg-opacity-80 text-zinc-100 px-4 py-2 mx-1 rounded-md uppercase`}
                >
                  Original
                </button>
                <button
                  id="e"
                  onClick={(e) => onPreviewClicked(e)}
                  className={`${
                    editedVideoSrc
                      ? "opacity-100 hover:bg-opacity-80"
                      : "opacity-50"
                  } ${
                    previewState == "e" ? "bg-indigo-600" : "bg-zinc-600"
                  } text-zinc-100 px-4 py-2 mx-1 rounded-md uppercase`}
                  disabled={editedVideoSrc ? false : true}
                >
                  Edited
                </button>
                <button
                  id="c"
                  onClick={(e) => onPreviewClicked(e)}
                  className={`${
                    compressedVideoSrc
                      ? "opacity-100 hover:bg-opacity-80"
                      : "opacity-50"
                  } ${
                    previewState == "c" ? "bg-indigo-600" : "bg-zinc-600"
                  }  text-zinc-100 px-4 py-2 mx-1 rounded-md uppercase`}
                  disabled={compressedVideoSrc ? false : true}
                >
                  Compressed
                </button>
              </div>
            )}

            <VideoContainer
              fileName={fileName}
              videoSrc={videoSrc}
              editedVideoSrc={editedVideoSrc}
              compressedVideoSrc={compressedVideoSrc}
              fileLocation={
                previewState == "o"
                  ? fileLocation
                  : previewState == "e"
                  ? editedfileLocation
                  : compressedfileLocation
              }
              videoRef={videoRef}
              setfileLocation={setfileLocation}
              setVideoSrc={setVideoSrc}
              setEditedVideoSrc={setEditedVideoSrc}
              setEditedfileLocation={setEditedfileLocation}
              setCompressedVideoSrc={setCompressedVideoSrc}
              setCompressedfileLocation={setCompressedfileLocation}
              previewState={previewState}
              fileSizes={fileSizes}
              setPreviewState={setPreviewState}
            />
          </div>
          {fileLocation.slice(fileLocation.search("Reco")) != fileName && (
            <div className="px-3 w-full border-2 border-indigo-600 rounded-md">
              <div className="text-center p-2">
                <p className="text-xl text-zinc-200 uppercase">Edit</p>
              </div>
              <SoundEdit
                onSoundClick={onSoundClick}
                muteState={editDetails.mute}
              />
              <SpeedEdit
                onSpeedClick={onSpeedClick}
                speedRate={editDetails.speed}
              />
              <TrimEdit
                onTrimTimeChanged={onTrimTimeChanged}
                enforceMinMax={enforceMinMax}
                trimTime={trimTime}
                duration={duration}
              />
              <CropEdit
                fakeCropArea={fakeCropArea}
                realCropArea={realCropArea}
                screenSize={screenSize}
                onRealValuesChange={onRealValuesChange}
                enforceMinMax={enforceMinMax}
                ratio={ratio}
              />
              <div
                className={`w-full flex border-2 mx-1 mt-7 mb-2 border-transparent rounded-lg transition-all h-12 justify-center`}
              >
                <button
                  disabled={
                    isSaving ||
                    editDetails.trim.startHr == "" ||
                    editDetails.trim.startMin == "" ||
                    editDetails.trim.startSec == "" ||
                    editDetails.crop.x == "" ||
                    editDetails.crop.y == "" ||
                    editDetails.crop.width == "" ||
                    editDetails.crop.height == ""
                  }
                  onClick={onSaveClicked}
                  className={`${
                    isSaving ||
                    editDetails.trim.startHr == "" ||
                    editDetails.trim.startMin == "" ||
                    editDetails.trim.startSec == "" ||
                    editDetails.crop.x == "" ||
                    editDetails.crop.y == "" ||
                    editDetails.crop.width == "" ||
                    editDetails.crop.height == ""
                      ? "opacity-50"
                      : "opacity-100 hover:bg-opacity-80"
                  } w-1/2 bg-indigo-600 text-zinc-100 h-full flex justify-center items-center rounded-md uppercase`}
                >
                  {isSaving
                    ? `${
                        currentEditProgress ? currentEditProgress : "trimming"
                      }...${progressRate}`
                    : "save edits"}
                </button>
              </div>
            </div>
          )}
        </div>
      )}
    </div>
  );
}

export default App;
