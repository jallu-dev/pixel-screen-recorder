import { invoke } from "@tauri-apps/api/core";
import { BaseDirectory, exists, remove } from "@tauri-apps/plugin-fs";
import React, { useEffect } from "react";
import { FaRegFolderOpen, FaRegTrashAlt } from "react-icons/fa";

const VideoContainer = ({
  fileName,
  videoSrc,
  editedVideoSrc,
  compressedVideoSrc,
  fileLocation,
  videoRef,
  setfileLocation,
  setVideoSrc,
  previewState,
  setPreviewState,
  fileSizes,
  setEditedVideoSrc,
  setEditedfileLocation,
  setCompressedVideoSrc,
  setCompressedfileLocation,
}) => {
  const onOpenClicked = async () => {
    try {
      await invoke("open_default_video_directory");
      console.log("Default video directory opened successfully.");
    } catch (error) {
      console.error("Failed to open default video directory:", error);
    }
  };

  const onDeleteClicked = async (e) => {
    let file = e.target.id;
    console.log(file);

    try {
      let exist = await exists(file, {
        baseDir: BaseDirectory.Video,
      });

      if (exist) {
        await remove(file, {
          baseDir: BaseDirectory.Video,
        });

        if (file.startsWith("Reco")) {
          setfileLocation("");
          setVideoSrc("");
        } else if (file.startsWith("Edit")) {
          setEditedVideoSrc("");
          setEditedfileLocation("");
        } else {
          setCompressedVideoSrc("");
          setCompressedfileLocation("");
        }

        setPreviewState("o");
      }
    } catch (error) {
      console.log(error);
    }
  };

  useEffect(() => {}, [previewState]);

  return (
    <div>
      <video
        src={videoSrc}
        style={{
          width: "100%",
          aspectRatio: "16/9",
          display: `${previewState == "o" ? "inline-block" : "none"}`,
        }}
        controls
        // controlsList="nofullscreen"
        autoPlay
        playsInline
        ref={videoRef}
      />
      <video
        src={editedVideoSrc}
        style={{
          width: "100%",
          aspectRatio: "16/9",
          display: `${previewState == "e" ? "inline-block" : "none"}`,
        }}
        controls
        // controlsList="nofullscreen"
        autoPlay
        playsInline
      />
      <video
        src={compressedVideoSrc}
        style={{
          width: "100%",
          aspectRatio: "16/9",
          display: `${previewState == "c" ? "inline-block" : "none"}`,
        }}
        controls
        // controlsList="nofullscreen"
        autoPlay
        playsInline
      />
      {fileLocation.slice(fileLocation.search("Reco")) != fileName && (
        <div className="flex justify-between w-full items-end py-2 px-4 mt-5">
          <div className="flex flex-col">
            <p className="text-md text-zinc-300">
              {fileLocation.slice(
                fileLocation.search(
                  previewState == "o"
                    ? "Reco"
                    : previewState == "e"
                    ? "Edit"
                    : "Comp"
                )
              )}
            </p>
            <p className="text-sm text-zinc-400"> {fileLocation}</p>
          </div>
          <div className="flex flex-col w-[15%] lg:w-[12%] justify-between">
            <p className="text-sm text-zinc-400 mb-1">
              {previewState == "o"
                ? fileSizes.original
                : previewState == "e"
                ? fileSizes.edited
                : fileSizes.compressed}
            </p>
            <div className="flex  justify-between">
              <FaRegFolderOpen
                className="size-6 text-indigo-600 cursor-pointer "
                onClick={onOpenClicked}
              />
              <FaRegTrashAlt
                id={fileLocation.slice(
                  fileLocation.search(
                    previewState == "o"
                      ? "Reco"
                      : previewState == "e"
                      ? "Edit"
                      : "Comp"
                  )
                )}
                className="size-5 text-red-400 cursor-pointer"
                onClick={(e) => onDeleteClicked(e)}
              />
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

export default VideoContainer;
