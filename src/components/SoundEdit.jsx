import React from "react";

const SoundEdit = ({ onSoundClick, muteState }) => {
  return (
    <div className="w-full flex gap-2 px-3 mb-4 mt-2 items-center">
      <p className="uppercase text-zinc-200 w-28">Sound</p>
      <button
        id=""
        onClick={(e) => onSoundClick(e)}
        className={`${
          muteState == "" ? "bg-indigo-600" : "bg-zinc-600"
        } hover:bg-opacity-80 text-zinc-100 px-4 py-2 mx-1 rounded-md uppercase`}
      >
        Unmute
      </button>
      <button
        id="-an"
        onClick={(e) => onSoundClick(e)}
        className={`${
          muteState == "-an" ? "bg-indigo-600" : "bg-zinc-600"
        } hover:bg-opacity-80 text-zinc-100 px-4 py-2 mx-1 rounded-md uppercase`}
      >
        Mute
      </button>
    </div>
  );
};

export default SoundEdit;
