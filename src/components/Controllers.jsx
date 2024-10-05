import React from "react";
import {
  FaRegPauseCircle,
  FaRegPlayCircle,
  FaRegStopCircle,
} from "react-icons/fa";

const Controllers = ({
  isPaused,
  isRecording,
  startRecording,
  pausedRecording,
  stopRecording,
  isControllersActive,
}) => {
  const msgs = [
    "Your video is baking in the oven. Just a few more minutes!",
    "Shhh... don't wake the video. It's sleeping soundly",
    "Your recording is getting a spa treatment. It'll look fabulous soon!",
    "Hang tight! Your video is on its way to stardom.",
    "We're stitching together your masterpiece. Almost there!",
    "Your video is being polished to perfection. Stay tuned!",
    "We're putting the finishing touches on your cinematic masterpiece.",
    "Your recording is taking a nap. It'll be ready to show off soon.",
    "Your video is getting a makeover. It'll look better than ever!",
    "Just a few more clicks... Your video is almost ready for its debut.",
  ];

  const colors = [
    "text-red-400",
    "text-orange-400",
    "text-green-400",
    "text-rose-400",
    "text-emerald-400",
    "text-pink-400",
    "text-fuchsia-400",
    "text-yellow-400",
    "text-sky-400",
    "text-teal-400",
  ];

  return (
    <div
      className={`flex flex-col border-2 border-indigo-600 rounded-xl mt-10 mb-2 w-2/5 h-16 mx-auto ${
        (!isControllersActive && !isRecording) ||
        (!isControllersActive && isRecording && isPaused)
          ? "opacity-50"
          : ""
      }`}
    >
      {!isControllersActive && isRecording && !isPaused ? (
        <div
          className={`${
            colors[Math.floor(Math.random() * 10)]
          } text-lg text-center h-full flex justify-center items-center px-2`}
        >
          {msgs[Math.floor(Math.random() * 10)]}
        </div>
      ) : (
        <div className="flex justify-around items-center h-full">
          <FaRegPlayCircle
            className={`text-3xl mx-6 ${
              !isPaused && isRecording
                ? "text-zinc-400 cursor-default"
                : "text-green-400 cursor-pointer"
            }`}
            onClick={
              !isPaused && isRecording
                ? null
                : isControllersActive
                ? startRecording
                : null
            }
          />
          <FaRegPauseCircle
            className={`text-3xl mx-6 ${
              !isRecording || (isRecording && isPaused)
                ? "text-zinc-400 cursor-default"
                : "text-yellow-400 cursor-pointer"
            }`}
            onClick={
              !isRecording || (isRecording && isPaused)
                ? null
                : isControllersActive
                ? pausedRecording
                : null
            }
          />
          <FaRegStopCircle
            className={`text-3xl mx-6 ${
              !isRecording
                ? "text-zinc-400 cursor-default"
                : "text-red-400 cursor-pointer"
            }`}
            onClick={
              !isRecording ? null : isControllersActive ? stopRecording : null
            }
          />
        </div>
      )}

      <div></div>
    </div>
  );
};

export default Controllers;
